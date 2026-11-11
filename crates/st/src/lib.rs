mod api;
pub mod map;
mod run;
pub mod wire;

use anyhow::bail;
use async_trait::async_trait;
use cord_core::*;
use serde_json::Value;
use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tokio::sync::mpsc::UnboundedSender;

pub use api::login;

pub(crate) const NIL: &str = "00000000000000000000000000";

#[derive(Clone, Debug)]
pub(crate) struct Cfg {
    pub ws: String,
    pub api: String,
    pub cdn: String,
    pub prx: String,
}

#[derive(Default)]
pub(crate) struct Cache {
    pub me: String,
    pub users: HashMap<String, Value>,
    pub servers: HashMap<String, Value>,
    pub spos: HashMap<String, i64>,
    pub chans: HashMap<String, Value>,
    pub mems: HashMap<(String, String), Value>,
    pub emojis: HashMap<String, Value>,
    pub unread: HashMap<String, (u32, u32)>,
}

impl Cache {
    pub fn user(&self, id: &str) -> Option<wire::User> {
        serde_json::from_value(self.users.get(id)?.clone()).ok()
    }
    pub fn emoji(&self, id: &str) -> Option<wire::Emoji> {
        serde_json::from_value(self.emojis.get(id)?.clone()).ok()
    }
    pub fn emojis_of(&self, sid: &str) -> Vec<wire::Emoji> {
        self.emojis
            .values()
            .filter_map(|v| serde_json::from_value::<wire::Emoji>(v.clone()).ok())
            .filter(|e| matches!(&e.parent, wire::Parent::Server { id } if id == sid))
            .collect()
    }
    pub fn server(&self, id: &str) -> Option<wire::Server> {
        serde_json::from_value(self.servers.get(id)?.clone()).ok()
    }
    pub fn chan(&self, id: &str) -> Option<wire::Channel> {
        serde_json::from_value(self.chans.get(id)?.clone()).ok()
    }
    pub fn chans_of(&self, sid: &str) -> HashMap<String, wire::Channel> {
        self.chans
            .values()
            .filter_map(|v| serde_json::from_value::<wire::Channel>(v.clone()).ok())
            .filter(|c| c.server() == Some(sid))
            .map(|c| (c.id().to_string(), c))
            .collect()
    }
}

pub(crate) struct I {
    pub acc: u32,
    pub base: String,
    pub tok: String,
    pub http: reqwest::Client,
    pub cfg: OnceCell<Cfg>,
    pub c: Mutex<Cache>,
    pub wtx: Mutex<Option<UnboundedSender<String>>>,
    pub seq: AtomicU64,
    pub jh: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[derive(Clone)]
pub struct St {
    pub(crate) i: Arc<I>,
}

impl St {
    pub fn new(acc: u32, base: String, tok: String) -> Self {
        let http = reqwest::Client::builder().user_agent("cord-st/0.1").build().unwrap_or_default();
        Self {
            i: Arc::new(I {
                acc,
                base: base.trim_end_matches('/').to_string(),
                tok,
                http,
                cfg: OnceCell::new(),
                c: Mutex::new(Cache::default()),
                wtx: Mutex::new(None),
                seq: AtomicU64::new(0),
                jh: Mutex::new(None),
            }),
        }
    }
}

#[async_trait]
impl Bk for St {
    fn svc(&self) -> Svc {
        Svc::St
    }
    fn acc(&self) -> u32 {
        self.i.acc
    }
    async fn start(&self, tx: Tx) -> Res<()> {
        self.i.cfg().await?;
        let i = self.i.clone();
        let j = tokio::spawn(async move { i.run(tx).await });
        if let Some(o) = self.i.jh.lock().replace(j) {
            o.abort();
        }
        Ok(())
    }
    async fn stop(&self) {
        if let Some(j) = self.i.jh.lock().take() {
            j.abort();
        }
        *self.i.wtx.lock() = None;
    }
    async fn hist(&self, chan: &Rid, before: Option<&Rid>, n: u32) -> Res<Vec<Msg>> {
        self.i.hist(chan, before, n).await
    }
    async fn send(&self, chan: &Rid, o: Out) -> Res<()> {
        self.i.send(chan, o).await
    }
    async fn edit(&self, chan: &Rid, msg: &Rid, body: String) -> Res<()> {
        self.i.edit(chan, msg, body).await
    }
    async fn del(&self, chan: &Rid, msg: &Rid) -> Res<()> {
        self.i.del(chan, msg).await
    }
    async fn react(&self, chan: &Rid, msg: &Rid, e: &Emo, on: bool) -> Res<()> {
        self.i.react(chan, msg, e, on).await
    }
    async fn typing(&self, chan: &Rid) -> Res<()> {
        let g = self.i.wtx.lock();
        match g.as_ref() {
            Some(t) => {
                t.send(serde_json::json!({"type": "BeginTyping", "channel": chan.raw}).to_string())
                    .map_err(|_| anyhow::anyhow!("socket closed"))?;
                Ok(())
            }
            None => bail!("not connected"),
        }
    }
    async fn ack(&self, chan: &Rid, msg: &Rid) -> Res<()> {
        self.i.ack(chan, msg).await
    }
    async fn mems(&self, gld: &Rid) -> Res<Vec<Mem>> {
        self.i.mems(gld).await
    }
    async fn set_st(&self, st: cord_core::St, act: Option<String>) -> Res<()> {
        self.i.set_st(st, act).await
    }
    async fn api(&self, op: &str, a: &Value) -> Res<Value> {
        use reqwest::Method;
        use serde_json::json;
        let st = |k: &str| a[k].as_str().unwrap_or("").to_string();
        let i = &self.i;
        match op {
            "acct" => {
                let m = i.req(Method::GET, "/users/@me", None, &[]).await?;
                Ok(json!({"name": m["display_name"].as_str().unwrap_or(m["username"].as_str().unwrap_or("")), "user": m["username"], "email": Value::Null, "bio": "", "caps": ["name", "end", "rename"]}))
            }
            "set_name" => i.req(Method::PATCH, "/users/@me", Some(json!({"display_name": st("name")})), &[]).await.map(|_| json!({})),
            "sessions" => {
                let v = i.req(Method::GET, "/auth/session/all", None, &[]).await?;
                let list: Vec<Value> = v.as_array().map(|a| a.iter().map(|s| json!({"id": s["_id"], "name": s["name"], "kind": "desktop", "last": 0, "loc": Value::Null})).collect()).unwrap_or_default();
                Ok(json!({"cur": "", "list": list}))
            }
            "session_end" => i.req(Method::DELETE, &format!("/auth/session/{}", st("id")), None, &[]).await.map(|_| json!({})),
            "session_name" => i.req(Method::PATCH, &format!("/auth/session/{}", st("id")), Some(json!({"friendly_name": st("name")})), &[]).await.map(|_| json!({})),
            "sessions_end_other" => i.req(Method::DELETE, "/auth/session/all?revoke_self=false", None, &[]).await.map(|_| json!({})),
            _ => bail!("{op} is not supported on this network"),
        }
    }
}
