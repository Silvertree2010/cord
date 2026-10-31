use crate::*;
use cord_core::*;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, mpsc};

pub type Sink = Arc<dyn Fn(Dl) + Send + Sync>;

pub struct Hub {
    pub bks: RwLock<HashMap<u32, Arc<dyn Bk>>>,
    pub st: RwLock<Sta>,
    pub hks: RwLock<Hks>,
    tx: mpsc::UnboundedSender<Ev>,
}

impl Hub {
    pub fn new(sink: Sink) -> Arc<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Ev>();
        let h = Arc::new(Self {
            bks: RwLock::new(HashMap::new()),
            st: RwLock::new(Sta::default()),
            hks: RwLock::new(Hks::default()),
            tx,
        });
        let w = Arc::downgrade(&h);
        tokio::spawn(async move {
            while let Some(mut e) = rx.recv().await {
                let Some(h) = w.upgrade() else { break };
                if h.hks.read().await.ev(&mut e) == Flow::Stop {
                    continue;
                }
                let ds = h.st.write().await.ingest(e);
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

    pub async fn hist(&self, chan: &Rid, before: Option<&Rid>) -> Res<Vec<Msg>> {
        let b = self.bk(chan.acc).await?;
        let mut v = b.hist(chan, before, 50).await?;
        for m in v.iter_mut() {
            let mut e = Ev::Msg(m.clone());
            self.hks.read().await.ev(&mut e);
            if let Ev::Msg(x) = e {
                *m = x;
            }
        }
        self.st.write().await.prepend(chan, v.clone());
        Ok(v)
    }

    pub async fn send(&self, chan: &Rid, mut o: Out) -> Res<()> {
        if self.hks.read().await.out(chan, &mut o) == Flow::Stop {
            return Ok(());
        }
        self.bk(chan.acc).await?.send(chan, o).await
    }

    pub async fn ack(&self, chan: &Rid) -> Res<()> {
        let last = self.st.read().await.chans.get(chan).and_then(|c| c.last.clone());
        let _ = self.tx.send(Ev::Ack { chan: chan.clone() });
        if let Some(m) = last {
            self.bk(chan.acc).await?.ack(chan, &m).await?;
        }
        Ok(())
    }
}
