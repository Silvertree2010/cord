use crate::{cfg, out::Out};
use cord_core::*;
use cord_hub::Hub;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub type Want = Arc<Mutex<Option<(St, Option<String>)>>>;

pub fn parse_st(s: &str) -> St {
    match s.to_lowercase().as_str() {
        "on" | "online" => St::On,
        "idle" => St::Idle,
        "dnd" => St::Dnd,
        _ => St::Off,
    }
}

pub fn cap_lq(mut q: Lq) -> Lq {
    if !feat::on("streamq") {
        q.w = q.w.min(1280);
        q.h = q.h.min(720);
        q.fps = q.fps.min(30);
        q.kbps = q.kbps.min(4000);
    }
    q
}

pub fn load(want: &Want) -> Result<(), String> {
    match cfg::rd_in(&cfg::dir(), "cfg.json") {
        Ok(fc) => {
            if let Some(o) = fc.unwrap_or(json!({}))["feat"].as_object() {
                for (k, v) in o {
                    if let Some(b) = v.as_bool() {
                        feat::set(k, b);
                    }
                }
            }
        }
        Err(x) => return Err(x.to_string()),
    }
    if let Some(s) = cfg::rd("ui.json").and_then(|u| u["st"].as_str().map(parse_st)) {
        if let Ok(mut g) = want.lock() {
            *g = Some((s, None));
        }
    }
    Ok(())
}

pub fn persist(k: &str, on: bool) -> anyhow::Result<()> {
    cfg::update("cfg.json", |c| {
        if !c["feat"].is_object() {
            c["feat"] = json!({});
        }
        c["feat"][k] = json!(on);
    })
}

pub async fn apply(h: &Hub, k: &str, on: bool) -> Option<Out> {
    let mut s = h.st.write().await;
    match k {
        "msglog" => {
            s.cfg.log_del = on;
            s.cfg.log_ed = on;
            None
        }
        "hidden" => {
            s.cfg.show_hid = on;
            Some(Out::Chans(s.vis_chans()))
        }
        _ => None,
    }
}

pub async fn set_status(h: Arc<Hub>, s: St, a: Option<String>) {
    let bs: Vec<_> = h.bks.read().await.values().cloned().collect();
    for b in bs {
        let _ = b.set_st(s, a.clone()).await;
    }
}

pub fn list() -> Value {
    Value::Array(feat::FEATS.iter().map(|f| json!({"k": f.k, "n": f.n, "d": f.d, "on": f.on})).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_words() {
        assert_eq!(parse_st("Online"), St::On);
        assert_eq!(parse_st("dnd"), St::Dnd);
        assert_eq!(parse_st("x"), St::Off);
    }

    #[test]
    fn stream_quality_cap() {
        feat::set("streamq", false);
        let q = cap_lq(Lq::default());
        assert_eq!((q.w, q.h, q.fps, q.kbps), (1280, 720, 30, 4000));
        feat::set("streamq", true);
        assert_eq!(cap_lq(Lq::default()).h, 1080);
    }
}
