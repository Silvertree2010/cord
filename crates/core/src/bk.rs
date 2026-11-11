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
    async fn mems_at(&self, _gld: &Rid, _at: u32) -> Res<()> {
        Ok(())
    }
    async fn join(&self, _chan: &Rid) -> Res<()> {
        anyhow::bail!("voice unsupported")
    }
    async fn leave(&self) -> Res<()> {
        Ok(())
    }
    async fn vset(&self, _mute: bool, _deaf: bool) -> Res<()> {
        Ok(())
    }
    async fn vol(&self, _usr: &Rid, _v: f32) -> Res<()> {
        Ok(())
    }
    async fn golive(&self, _on: bool, _q: Lq) -> Res<()> {
        anyhow::bail!("streaming unsupported")
    }
    async fn watch(&self, _usr: &Rid, _on: bool) -> Res<()> {
        anyhow::bail!("streaming unsupported")
    }
    async fn rel(&self, _usr: &Rid, _on: bool) -> Res<()> {
        anyhow::bail!("not supported on this network")
    }
    async fn set_st(&self, _st: St, _act: Option<String>) -> Res<()> {
        Ok(())
    }
    async fn api(&self, op: &str, _args: &serde_json::Value) -> Res<serde_json::Value> {
        anyhow::bail!("{op} is not supported on this network")
    }
    async fn profile(&self, _usr: &Rid, _gld: Option<&Rid>) -> Res<serde_json::Value> {
        anyhow::bail!("profiles unsupported on this network")
    }
    async fn open_dm(&self, _usr: &Rid) -> Res<Rid> {
        anyhow::bail!("direct messages unsupported on this network")
    }
    async fn stop(&self) {}
    fn demo(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Lq {
    pub w: u32,
    pub h: u32,
    pub fps: u32,
    pub kbps: u32,
    pub snd: bool,
}

impl Default for Lq {
    fn default() -> Self {
        Self { w: 1920, h: 1080, fps: 60, kbps: 8000, snd: true }
    }
}
