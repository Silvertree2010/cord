use crate::acc::Ac;
use cord_core::*;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

static MX: OnceLock<Mutex<HashMap<u32, Arc<cord_mx::Mx>>>> = OnceLock::new();

fn mx() -> &'static Mutex<HashMap<u32, Arc<cord_mx::Mx>>> {
    MX.get_or_init(Default::default)
}

pub fn mx_of(a: u32) -> Option<Arc<cord_mx::Mx>> {
    mx().lock().ok()?.get(&a).cloned()
}

pub fn mx_any() -> Option<Arc<cord_mx::Mx>> {
    mx().lock().ok()?.values().next().cloned()
}

fn mdir(id: u32) -> std::path::PathBuf {
    let d = crate::cfg::data().join(format!("mx{id}"));
    let _ = std::fs::create_dir_all(&d);
    d
}

pub async fn mk(a: &Ac, sec: String) -> Option<Arc<dyn Bk>> {
    match a.svc {
        Svc::Dc => Some(Arc::new(cord_dc::Dc::new(a.id, sec))),
        Svc::St => {
            let v: Value = serde_json::from_str(&sec).unwrap_or(json!({"tok": sec}));
            let base = v["base"].as_str().map(String::from).unwrap_or_else(|| if a.base.is_empty() { "https://api.stoat.chat".into() } else { a.base.clone() });
            Some(Arc::new(cord_st::St::new(a.id, base, v["tok"].as_str().unwrap_or("").to_string())))
        }
        Svc::Mx => {
            let m = cord_mx::Mx::restore(a.id, &mdir(a.id), &sec).await.ok()?;
            let m = Arc::new(m);
            if let Ok(mut g) = mx().lock() {
                g.insert(a.id, m.clone());
            }
            Some(Arc::new(MxB(m)))
        }
    }
}

pub async fn login(sv: Svc, id: u32, v: &Value) -> Res<(String, String)> {
    let g = |k: &str| v[k].as_str().unwrap_or("").trim().to_string();
    match sv {
        Svc::Dc => {
            let t = g("token").trim_matches('"').to_string();
            anyhow::ensure!(!t.is_empty(), "token missing");
            Ok((String::new(), t))
        }
        Svc::St => {
            let base = Some(g("base")).filter(|s| !s.is_empty()).unwrap_or_else(|| "https://api.stoat.chat".into());
            let tok = if !g("token").is_empty() { g("token") } else { cord_st::login(&base, &g("email"), &g("pass")).await? };
            Ok((base.clone(), json!({"base": base, "tok": tok}).to_string()))
        }
        Svc::Mx => {
            let mut hs = g("hs");
            if hs.is_empty() {
                hs = "https://matrix.org".into();
            }
            if !hs.starts_with("http") {
                hs = format!("https://{hs}");
            }
            let m = cord_mx::Mx::login(id, &hs, &g("user"), &g("pass"), &mdir(id)).await?;
            if !g("recovery").is_empty() {
                let _ = m.recover(&g("recovery")).await;
            }
            Ok((hs, m.session_json()?))
        }
    }
}

struct MxB(Arc<cord_mx::Mx>);

#[async_trait::async_trait]
impl Bk for MxB {
    fn svc(&self) -> Svc {
        self.0.svc()
    }
    fn acc(&self) -> u32 {
        self.0.acc()
    }
    async fn start(&self, tx: Tx) -> Res<()> {
        self.0.start(tx).await
    }
    async fn hist(&self, c: &Rid, b: Option<&Rid>, n: u32) -> Res<Vec<Msg>> {
        self.0.hist(c, b, n).await
    }
    async fn send(&self, c: &Rid, o: Out) -> Res<()> {
        self.0.send(c, o).await
    }
    async fn edit(&self, c: &Rid, m: &Rid, b: String) -> Res<()> {
        self.0.edit(c, m, b).await
    }
    async fn del(&self, c: &Rid, m: &Rid) -> Res<()> {
        self.0.del(c, m).await
    }
    async fn react(&self, c: &Rid, m: &Rid, e: &Emo, on: bool) -> Res<()> {
        self.0.react(c, m, e, on).await
    }
    async fn typing(&self, c: &Rid) -> Res<()> {
        self.0.typing(c).await
    }
    async fn ack(&self, c: &Rid, m: &Rid) -> Res<()> {
        self.0.ack(c, m).await
    }
    async fn mems(&self, g: &Rid) -> Res<Vec<Mem>> {
        self.0.mems(g).await
    }
    async fn set_st(&self, s: St, a: Option<String>) -> Res<()> {
        self.0.set_st(s, a).await
    }
}
