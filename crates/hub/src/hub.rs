use crate::*;
use cord_core::*;
use cord_store::{Log, Row, Store};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, mpsc};

pub type Sink = Arc<dyn Fn(Dl) + Send + Sync>;

pub struct Hub {
    pub bks: RwLock<HashMap<u32, Arc<dyn Bk>>>,
    pub st: RwLock<Sta>,
    pub hks: RwLock<Hks>,
    pub store: Option<Arc<Store>>,
    tx: mpsc::UnboundedSender<Ev>,
}

pub struct HistR {
    pub n: usize,
    pub added: Vec<MsgV>,
}

fn row(m: &Msg) -> Row {
    Row {
        chan: m.chan.to_string(),
        id: m.id.to_string(),
        json: serde_json::to_string(m).unwrap_or_default(),
        ts: m.ts,
        deleted: m.del,
        edited: m.ed.is_some() || !m.hist.is_empty(),
    }
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

impl Hub {
    pub fn new(sink: Sink, store: Option<Arc<Store>>) -> Arc<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Ev>();
        let h = Arc::new(Self {
            bks: RwLock::new(HashMap::new()),
            st: RwLock::new(Sta::default()),
            hks: RwLock::new(Hks::default()),
            store,
            tx,
        });
        let w = Arc::downgrade(&h);
        tokio::spawn(async move {
            let mut held: Option<Ev> = None;
            loop {
                let mut e = match held.take() {
                    Some(e) => e,
                    None => match rx.recv().await {
                        Some(e) => e,
                        None => break,
                    },
                };
                let Some(h) = w.upgrade() else { break };
                if let Ev::Mem(m) = e {
                    let mut v = vec![m];
                    loop {
                        match rx.try_recv() {
                            Ok(Ev::Mem(m)) => v.push(m),
                            Ok(o) => {
                                held = Some(o);
                                break;
                            }
                            Err(_) => break,
                        }
                    }
                    e = Ev::Mems(v);
                }
                if h.hks.read().await.ev(&mut e) == Flow::Stop {
                    continue;
                }
                let key = match &e {
                    Ev::Msg(m) | Ev::MsgUpd(m) => Some((m.chan.clone(), m.id.clone())),
                    Ev::MsgDel { chan, id } | Ev::React { chan, id, .. } => Some((chan.clone(), id.clone())),
                    _ => None,
                };
                let is_del = matches!(e, Ev::MsgDel { .. });
                let is_upd = matches!(e, Ev::MsgUpd(_));
                let mut st = h.st.write().await;
                let before = key.as_ref().and_then(|(c, i)| st.msgs.get(c).and_then(|q| q.iter().find(|x| &x.id == i)).map(|m| m.hist.len()));
                let ds = st.ingest(e);
                if let (Some(s), Some((c, i))) = (&h.store, key) {
                    if let Some(m) = st.msgs.get(&c).and_then(|q| q.iter().find(|x| x.id == i)) {
                        s.put(row(m));
                        if is_del && m.del {
                            s.log(Log { chan: c.to_string(), id: i.to_string(), kind: "del", body: m.body.clone(), ts: now() });
                        }
                        if is_upd && before.is_some_and(|b| m.hist.len() > b) {
                            if let Some(old) = m.hist.last() {
                                s.log(Log { chan: c.to_string(), id: i.to_string(), kind: "edit", body: old.clone(), ts: now() });
                            }
                        }
                    }
                }
                drop(st);
                for d in ds {
                    sink(d);
                }
            }
        });
        h
    }

    pub fn tx(&self) -> mpsc::UnboundedSender<Ev> {
        self.tx.clone()
    }

    pub async fn add(&self, b: Arc<dyn Bk>) -> Res<()> {
        let a = b.acc();
        self.bks.write().await.insert(a, b.clone());
        b.start(self.tx.clone()).await
    }

    pub async fn bk(&self, acc: u32) -> Res<Arc<dyn Bk>> {
        self.bks.read().await.get(&acc).cloned().ok_or_else(|| anyhow::anyhow!("no acc {acc}"))
    }

    pub async fn remove(&self, acc: u32) {
        let b = self.bks.write().await.remove(&acc);
        if let Some(b) = b {
            b.stop().await;
        }
        self.st.write().await.purge_acc(acc);
        if let Some(s) = &self.store {
            let svs = [Svc::Dc, Svc::Mx, Svc::St];
            for sv in svs {
                s.purge_prefix(&format!("{}:{}:", sv.tag(), acc));
            }
        }
    }

    pub async fn hydrate(&self, chan: &Rid) {
        let Some(s) = &self.store else { return };
        if self.st.read().await.msgs.get(chan).is_some_and(|q| !q.is_empty()) {
            return;
        }
        let rows = s.last(&chan.to_string(), 50);
        let v: Vec<Msg> = rows.iter().filter_map(|j| serde_json::from_str(j).ok()).collect();
        if !v.is_empty() {
            let mut st = self.st.write().await;
            if st.msgs.get(chan).is_none_or(|q| q.is_empty()) {
                st.prepend(chan, v);
            }
        }
    }

    pub async fn msgs(&self, chan: &Rid) -> Vec<MsgV> {
        self.hydrate(chan).await;
        self.st.read().await.msgs_v(chan)
    }

    pub async fn hist(&self, chan: &Rid, before: Option<&Rid>) -> Res<HistR> {
        if before.is_none() {
            self.hydrate(chan).await;
        }
        let b = self.bk(chan.acc).await?;
        let mut v = b.hist(chan, before, 50).await?;
        let n = v.len();
        for m in v.iter_mut() {
            let mut e = Ev::Msg(m.clone());
            self.hks.read().await.ev(&mut e);
            if let Ev::Msg(x) = e {
                *m = x;
            }
        }
        let mut st = self.st.write().await;
        let added = st.prepend(chan, v);
        if let Some(s) = &self.store {
            for m in &added {
                s.put(row(m));
            }
        }
        let added = added.iter().map(|m| st.msg_v(m)).collect();
        Ok(HistR { n, added })
    }

    pub async fn send(&self, chan: &Rid, mut o: Out) -> Res<()> {
        if self.hks.read().await.out(chan, &mut o) == Flow::Stop {
            return Ok(());
        }
        self.bk(chan.acc).await?.send(chan, o).await
    }

    pub async fn typing(&self, chan: &Rid) -> Res<()> {
        if feat::on("silenttyping") {
            return Ok(());
        }
        self.bk(chan.acc).await?.typing(chan).await
    }

    pub async fn ack(&self, chan: &Rid) -> Res<()> {
        let last = self.st.read().await.chans.get(chan).and_then(|c| c.last.clone());
        let _ = self.tx.send(Ev::Ack { chan: chan.clone() });
        if feat::on("noread") {
            return Ok(());
        }
        if let Some(m) = last {
            self.bk(chan.acc).await?.ack(chan, &m).await?;
        }
        Ok(())
    }
}
