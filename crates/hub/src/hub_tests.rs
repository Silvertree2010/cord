use crate::*;
use cord_core::*;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

struct Stub {
    acks: AtomicU32,
    typs: AtomicU32,
}

#[async_trait::async_trait]
impl Bk for Stub {
    fn svc(&self) -> Svc {
        Svc::Dc
    }
    fn acc(&self) -> u32 {
        1
    }
    async fn start(&self, _tx: Tx) -> Res<()> {
        Ok(())
    }
    async fn hist(&self, _c: &Rid, _b: Option<&Rid>, _n: u32) -> Res<Vec<Msg>> {
        Ok(vec![])
    }
    async fn send(&self, _c: &Rid, _o: cord_core::Out) -> Res<()> {
        Ok(())
    }
    async fn edit(&self, _c: &Rid, _m: &Rid, _b: String) -> Res<()> {
        Ok(())
    }
    async fn del(&self, _c: &Rid, _m: &Rid) -> Res<()> {
        Ok(())
    }
    async fn react(&self, _c: &Rid, _m: &Rid, _e: &Emo, _on: bool) -> Res<()> {
        Ok(())
    }
    async fn typing(&self, _c: &Rid) -> Res<()> {
        self.typs.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn ack(&self, _c: &Rid, _m: &Rid) -> Res<()> {
        self.acks.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn noread_and_silenttyping_skip_backend() {
    let h = Hub::new(Arc::new(|_| {}), None);
    let b = Arc::new(Stub { acks: AtomicU32::new(0), typs: AtomicU32::new(0) });
    h.bks.write().await.insert(1, b.clone());
    let c = Rid::new(Svc::Dc, 1, "c");
    h.st.write().await.chans.insert(
        c.clone(),
        Chan { id: c.clone(), gld: None, par: None, k: ChK::Txt, name: "c".into(), topic: None, pos: 0, nsfw: false, hidden: false, last: Some(Rid::new(Svc::Dc, 1, "9")), rcp: vec![], icon: None },
    );
    feat::set("noread", false);
    feat::set("silenttyping", false);
    h.ack(&c).await.unwrap();
    h.typing(&c).await.unwrap();
    assert_eq!((b.acks.load(Ordering::SeqCst), b.typs.load(Ordering::SeqCst)), (1, 1));
    feat::set("noread", true);
    feat::set("silenttyping", true);
    h.ack(&c).await.unwrap();
    h.typing(&c).await.unwrap();
    assert_eq!((b.acks.load(Ordering::SeqCst), b.typs.load(Ordering::SeqCst)), (1, 1));
    feat::set("noread", false);
    feat::set("silenttyping", false);
}
