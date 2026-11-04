use crate::*;
use cord_core::*;
use cord_store::{Log, Row, Store};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::{RwLock, mpsc};

pub struct Sd {
    pub seq: u64,
    pub d: Dl,
}

pub type Sink = Arc<dyn Fn(Sd) + Send + Sync>;

pub const WIN: usize = 50;

pub struct Hub {
    pub bks: RwLock<HashMap<u32, Arc<dyn Bk>>>,
    pub st: RwLock<Sta>,
    pub store: Option<Arc<Store>>,
    gone: Mutex<HashSet<u32>>,
    locks: Mutex<HashMap<Rid, Arc<tokio::sync::Mutex<()>>>>,
    acked: Mutex<HashMap<Rid, Rid>>,
    seq: AtomicU64,
    tx: mpsc::UnboundedSender<Ev>,
}

pub struct HistR {
    pub n: usize,
    pub added: Vec<MsgV>,
    pub noop: bool,
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

pub fn cache_ok(st: &Sta, chan: &Rid) -> bool {
    if st.accs.get(&chan.acc).is_some_and(|a| a.demo) || st.demo.contains(&chan.acc) {
        return false;
    }
    match st.chans.get(chan) {
        Some(c) => !c.enc,
        None => chan.svc != Svc::Mx,
    }
}

fn ev_acc(e: &Ev) -> Option<u32> {
    match e {
        Ev::Ready(a) => Some(a.id),
        Ev::Gld(g) => Some(g.id.acc),
        Ev::GldDel(r) | Ev::ChanDel(r) | Ev::RelDel(r) => Some(r.acc),
        Ev::Chan(c) => Some(c.id.acc),
        Ev::Msg(m) | Ev::MsgUpd(m) => Some(m.chan.acc),
        Ev::MsgDel { chan, .. } | Ev::React { chan, .. } | Ev::Typing { chan, .. } | Ev::Unread { chan, .. } | Ev::Ack { chan } | Ev::Speak { chan, .. } => Some(chan.acc),
        Ev::Pres { usr, .. } => Some(usr.acc),
        Ev::Rel(r) => r.usr.id.as_ref().map(|i| i.acc),
        Ev::Mem(m) => Some(m.gld.acc),
        Ev::Mems(_) => None,
        Ev::Vs(v) => v.usr.id.as_ref().map(|i| i.acc),
        Ev::Call(c) => c.chan.as_ref().map(|i| i.acc),
        Ev::Conn { acc, .. } | Ev::Err { acc, .. } | Ev::Folds { acc, .. } => Some(*acc),
    }
}

impl Hub {
    pub fn new(sink: Sink, store: Option<Arc<Store>>) -> Arc<Self> {
        Self::with_cfg(sink, store, Cfg::default())
    }

    pub fn seq(&self) -> u64 {
        self.seq.load(Ordering::SeqCst)
    }

    pub fn with_cfg(sink: Sink, store: Option<Arc<Store>>, cfg: Cfg) -> Arc<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Ev>();
        let h = Arc::new(Self {
            bks: RwLock::new(HashMap::new()),
            st: RwLock::new(Sta { cfg, ..Default::default() }),
            store,
            gone: Default::default(),
            locks: Default::default(),
            acked: Default::default(),
            seq: AtomicU64::new(0),
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
                let out: Vec<Sd> = {
                    let mut st = h.st.write().await;
                    {
                        let bks = h.bks.read().await;
                        let live = |a: u32| a == 0 || bks.contains_key(&a);
                        if let Ev::Mems(v) = &mut e {
                            v.retain(|m| live(m.gld.acc));
                            if v.is_empty() {
                                continue;
                            }
                        } else if let Some(a) = ev_acc(&e) {
                            if !live(a) {
                                continue;
                            }
                        }
                    }
                    let ds = h.apply(&mut st, e);
                    ds.into_iter().map(|d| Sd { seq: h.seq.fetch_add(1, Ordering::SeqCst) + 1, d }).collect()
                };
                for d in out {
                    sink(d);
                }
            }
        });
        h
    }

    fn apply(&self, st: &mut Sta, e: Ev) -> Vec<Dl> {
        let key = match &e {
            Ev::Msg(m) | Ev::MsgUpd(m) => Some((m.chan.clone(), m.id.clone())),
            Ev::MsgDel { chan, id } | Ev::React { chan, id, .. } => Some((chan.clone(), id.clone())),
            _ => None,
        };
        let is_del = matches!(e, Ev::MsgDel { .. });
        let is_upd = matches!(e, Ev::MsgUpd(_));
        let upd_copy = match &e {
            Ev::MsgUpd(m) if m.ts != 0 => Some(m.clone()),
            _ => None,
        };
        let chan_enc = match &e {
            Ev::Chan(c) if c.enc && st.chans.get(&c.id).is_none_or(|o| !o.enc) => Some(c.id.to_string()),
            _ => None,
        };
        let del_chans: Vec<String> = match &e {
            Ev::ChanDel(id) => vec![id.to_string()],
            Ev::GldDel(g) => st.chans.values().filter(|c| c.gld.as_ref() == Some(g)).map(|c| c.id.to_string()).collect(),
            _ => vec![],
        };
        let log_del = st.cfg.log_del;
        let before = key.as_ref().and_then(|(c, i)| st.msgs.get(c).and_then(|q| q.iter().find(|x| &x.id == i)).map(|m| m.hist.len()));
        let ds = st.ingest(e);
        if let Some(s) = &self.store {
            if let Some(c) = chan_enc {
                s.purge_chan(&c);
            }
            for c in del_chans {
                s.purge_chan(&c);
            }
            if let Some((c, i)) = key {
                if cache_ok(st, &c) {
                    let found = st.msgs.get(&c).and_then(|q| q.iter().find(|x| x.id == i));
                    match found {
                        Some(m) => {
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
                        None if is_del => s.del(&c.to_string(), &i.to_string(), log_del, now()),
                        None => {
                            if let Some(m) = upd_copy {
                                s.put(row(&m));
                            }
                        }
                    }
                }
            }
        }
        ds
    }

    pub fn tx(&self) -> mpsc::UnboundedSender<Ev> {
        self.tx.clone()
    }

    pub async fn add(&self, b: Arc<dyn Bk>) -> Res<()> {
        let a = b.acc();
        if let Ok(mut g) = self.gone.lock() {
            g.remove(&a);
        }
        if b.demo() {
            self.st.write().await.demo.insert(a);
            if let Some(s) = &self.store {
                for sv in [Svc::Dc, Svc::Mx, Svc::St] {
                    s.purge_prefix(&format!("{}:{}:", sv.tag(), a));
                }
            }
        }
        self.bks.write().await.insert(a, b.clone());
        if let Err(x) = b.start(self.tx.clone()).await {
            self.remove(a).await;
            return Err(x);
        }
        Ok(())
    }

    pub async fn bk(&self, acc: u32) -> Res<Arc<dyn Bk>> {
        self.bks.read().await.get(&acc).cloned().ok_or_else(|| anyhow::anyhow!("no acc {acc}"))
    }

    pub async fn remove(&self, acc: u32) {
        if let Ok(mut g) = self.gone.lock() {
            g.insert(acc);
        }
        let b = self.bks.write().await.remove(&acc);
        if let Some(b) = b {
            b.stop().await;
        }
        self.st.write().await.purge_acc(acc);
        if let Ok(mut g) = self.locks.lock() {
            g.retain(|k, _| k.acc != acc);
        }
        if let Ok(mut g) = self.acked.lock() {
            g.retain(|k, _| k.acc != acc);
        }
        if let Some(s) = &self.store {
            for sv in [Svc::Dc, Svc::Mx, Svc::St] {
                s.purge_prefix(&format!("{}:{}:", sv.tag(), acc));
            }
        }
    }

    fn removed(&self, acc: u32) -> bool {
        self.gone.lock().map(|g| g.contains(&acc)).unwrap_or(false)
    }

    pub async fn hydrate(&self, chan: &Rid) {
        let Some(s) = &self.store else { return };
        {
            let st = self.st.read().await;
            if st.msgs.get(chan).is_some_and(|q| !q.is_empty()) || !cache_ok(&st, chan) {
                return;
            }
        }
        let ck = chan.to_string();
        let s = s.clone();
        let v = tokio::task::spawn_blocking(move || {
            let mut v: Vec<Msg> = vec![];
            for j in s.last(&ck, WIN as u32) {
                match serde_json::from_str::<Msg>(&j) {
                    Ok(m) => v.push(m),
                    Err(e) => tracing::warn!("cache row in {ck} unreadable: {e}"),
                }
            }
            if v.is_empty() {
                return v;
            }
            for (id, kind, body, _) in s.logs_chan(&ck) {
                if let Some(m) = v.iter_mut().find(|m| m.id.to_string() == id) {
                    match kind.as_str() {
                        "edit" if !m.hist.contains(&body) => m.hist.push(body),
                        "del" => m.del = true,
                        _ => {}
                    }
                }
            }
            v
        })
        .await
        .unwrap_or_default();
        if v.is_empty() {
            return;
        }
        let mut st = self.st.write().await;
        if st.msgs.get(chan).is_none_or(|q| q.is_empty()) {
            st.hydrate(chan, v);
        }
    }

    pub async fn msgs(&self, chan: &Rid) -> (Vec<MsgV>, bool) {
        self.hydrate(chan).await;
        let st = self.st.read().await;
        (st.window(chan, WIN).iter().map(|m| st.msg_v(m)).collect(), st.synced.contains(chan))
    }

    fn lock_for(&self, chan: &Rid) -> Arc<tokio::sync::Mutex<()>> {
        let Ok(mut g) = self.locks.lock() else { return Default::default() };
        g.entry(chan.clone()).or_default().clone()
    }

    pub async fn hist(&self, chan: &Rid, before: Option<&Rid>) -> Res<HistR> {
        match before {
            None => self.hist_head(chan).await,
            Some(b) => self.hist_old(chan, b).await,
        }
    }

    async fn hist_head(&self, chan: &Rid) -> Res<HistR> {
        let l = self.lock_for(chan);
        let _g = l.lock().await;
        if self.st.read().await.synced.contains(chan) {
            return Ok(HistR { n: 0, added: vec![], noop: true });
        }
        self.hydrate(chan).await;
        let v = self.bk(chan.acc).await?.hist(chan, None, WIN as u32).await?;
        let n = v.len();
        anyhow::ensure!(!self.removed(chan.acc), "account removed");
        let mut st = self.st.write().await;
        let rec = st.reconcile(chan, v);
        if let (Some(s), true) = (&self.store, cache_ok(&st, chan)) {
            for m in rec.added.iter().chain(rec.changed.iter()) {
                s.put(row(m));
            }
            for id in &rec.removed {
                s.del(&chan.to_string(), &id.to_string(), false, now());
            }
        }
        let added = if rec.dirty() { st.window(chan, WIN).iter().map(|m| st.msg_v(m)).collect() } else { vec![] };
        Ok(HistR { n, added, noop: false })
    }

    async fn hist_old(&self, chan: &Rid, before: &Rid) -> Res<HistR> {
        {
            let st = self.st.read().await;
            let o = st.older(chan, before, WIN);
            if o.len() >= WIN {
                return Ok(HistR { n: WIN, added: o.iter().map(|m| st.msg_v(m)).collect(), noop: false });
            }
        }
        let v = self.bk(chan.acc).await?.hist(chan, Some(before), WIN as u32).await?;
        let n = v.len();
        anyhow::ensure!(!self.removed(chan.acc), "account removed");
        let mut st = self.st.write().await;
        let add = st.prepend(chan, v);
        if let (Some(s), true) = (&self.store, cache_ok(&st, chan)) {
            for m in &add {
                s.put(row(m));
            }
        }
        let o = st.older(chan, before, WIN);
        Ok(HistR { n, added: o.iter().map(|m| st.msg_v(m)).collect(), noop: false })
    }

    pub async fn send(&self, chan: &Rid, o: Out) -> Res<()> {
        self.bk(chan.acc).await?.send(chan, o).await
    }

    pub async fn typing(&self, chan: &Rid) -> Res<()> {
        if feat::on("silenttyping") {
            return Ok(());
        }
        self.bk(chan.acc).await?.typing(chan).await
    }

    pub async fn ack(&self, chan: &Rid) -> Res<()> {
        let (last, unread) = {
            let st = self.st.read().await;
            (st.newest(chan), st.unr.contains_key(chan))
        };
        if !unread {
            return Ok(());
        }
        let local = || {
            let _ = self.tx.send(Ev::Ack { chan: chan.clone() });
        };
        if feat::on("noread") {
            local();
            return Ok(());
        }
        let Some(m) = last else { return Ok(()) };
        if self.acked.lock().is_ok_and(|g| g.get(chan) == Some(&m)) {
            local();
            return Ok(());
        }
        self.bk(chan.acc).await?.ack(chan, &m).await?;
        if let Ok(mut g) = self.acked.lock() {
            g.insert(chan.clone(), m);
        }
        local();
        Ok(())
    }
}
