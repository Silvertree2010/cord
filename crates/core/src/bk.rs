use crate::*;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;

pub type Tx = UnboundedSender<Ev>;

#[async_trait]
pub trait Bk: Send + Sync {
    fn svc(&self) -> Svc;
    fn acc(&self) -> u32;
    async fn start(&self, tx: Tx) -> Res<()>;
    async fn hist(&self, chan: &Rid, before: Option<&Rid>, n: u32) -> Res<Vec<Msg>>;
    async fn send(&self, chan: &Rid, o: Out) -> Res<()>;
    async fn edit(&self, chan: &Rid, msg: &Rid, body: String) -> Res<()>;
    async fn del(&self, chan: &Rid, msg: &Rid) -> Res<()>;
    async fn react(&self, chan: &Rid, msg: &Rid, e: &Emo, on: bool) -> Res<()>;
    async fn typing(&self, chan: &Rid) -> Res<()>;
    async fn ack(&self, chan: &Rid, msg: &Rid) -> Res<()>;
    async fn mems(&self, _gld: &Rid) -> Res<Vec<Mem>> {
        Ok(vec![])
    }
    async fn set_st(&self, _st: St, _act: Option<String>) -> Res<()> {
        Ok(())
    }
}
