#[cxx_qt::bridge]
pub mod q {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Core = super::CoreR;

        #[qsignal]
        fn dl(self: Pin<&mut Core>, d: QString);

        #[qinvokable]
        fn boot(self: Pin<&mut Core>);
        #[qinvokable]
        fn snap(self: &Core) -> QString;
        #[qinvokable]
        fn msgs(self: &Core, c: &QString) -> QString;
        #[qinvokable]
        fn mems(self: &Core, g: &QString) -> QString;
        #[qinvokable]
        fn hist(self: &Core, c: &QString, b: &QString);
        #[qinvokable]
        fn send(self: &Core, c: &QString, body: &QString, reply: &QString, files: &QString, silent: bool);
        #[qinvokable]
        fn edit(self: &Core, c: &QString, m: &QString, body: &QString);
        #[qinvokable]
        fn del(self: &Core, c: &QString, m: &QString);
        #[qinvokable]
        fn react(self: &Core, c: &QString, m: &QString, e: &QString, on: bool);
        #[qinvokable]
        fn typing(self: &Core, c: &QString);
        #[qinvokable]
        fn ack(self: &Core, c: &QString);
        #[qinvokable]
        fn join(self: &Core, c: &QString);
        #[qinvokable]
        fn leave(self: &Core);
        #[qinvokable]
        fn vset(self: &Core, mute: bool, deaf: bool);
        #[qinvokable]
        fn vol(self: &Core, u: &QString, v: f32);
        #[qinvokable]
        fn golive(self: &Core, on: bool, q: &QString);
        #[qinvokable]
        fn watch(self: &Core, u: &QString, on: bool);
        #[qinvokable]
        fn set_st(self: &Core, st: &QString, act: &QString);
        #[qinvokable]
        fn accs(self: &Core) -> QString;
        #[qinvokable]
        fn add(self: &Core, svc: &QString, a: &QString);
        #[qinvokable]
        fn dc_qr(self: &Core);
        #[qinvokable]
        fn rm_acc(self: &Core, id: i32);
        #[qinvokable]
        fn feat(self: &Core, k: &QString, on: bool);
        #[qinvokable]
        fn cget(self: &Core, n: &QString) -> QString;
        #[qinvokable]
        fn cset(self: &Core, n: &QString, v: &QString);
        #[qinvokable]
        fn ls(self: &Core, sub: &QString, ext: &QString) -> QString;
        #[qinvokable]
        fn rdf(self: &Core, p: &QString) -> QString;
        #[qinvokable]
        fn home(self: &Core) -> QString;
        #[qinvokable]
        fn copy(self: &Core, t: &QString);
        #[qinvokable]
        fn qml(self: &Core) -> QString;
    }
    impl cxx_qt::Threading for Core {}
}

use crate::{acc, cfg, rt::rt};
use cord_core::*;
use cord_hub::{Dl, Hub};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use serde_json::{Value, json};
use std::{path::PathBuf, str::FromStr, sync::Arc};

#[derive(Default)]
pub struct CoreR {
    h: Option<Arc<Hub>>,
    em: Option<Em>,
    _w: Option<notify::RecommendedWatcher>,
}

#[derive(Clone)]
pub struct Em(Arc<cxx_qt::CxxQtThread<q::Core>>);

impl Em {
    pub fn s(&self, v: Value) {
        let s = v.to_string();
        let _ = self.0.queue(move |o| o.dl(QString::from(&s)));
    }
}

fn qs(s: &QString) -> String {
    s.to_string()
}

fn rid(s: &QString) -> Option<Rid> {
    Rid::from_str(&s.to_string()).ok()
}

fn js<T: serde::Serialize>(v: &T) -> QString {
    QString::from(&serde_json::to_string(v).unwrap_or_default())
}

pub async fn mk(a: &acc::Ac, sec: Option<String>) -> Option<Arc<dyn Bk>> {
    if a.base == "demo" {
        return Some(Arc::new(cord_hub::demo::Demo::new(a.id, a.svc)));
    }
    crate::bks::mk(a, sec?).await
}

impl CoreR {
    fn hb(&self) -> Option<(Arc<Hub>, Em)> {
        Some((self.h.clone()?, self.em.clone()?))
    }
    fn go<F, Fut>(&self, f: F)
    where
        F: FnOnce(Arc<Hub>, Em) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Res<()>> + Send + 'static,
    {
        if let Some((h, e)) = self.hb() {
            rt().spawn(async move {
                let e2 = e.clone();
                if let Err(x) = f(h, e).await {
                    e2.s(json!({"t":"Err","d":{"acc":0,"msg":x.to_string()}}));
                }
            });
        }
    }
}

impl q::Core {
    pub fn boot(mut self: Pin<&mut Self>) {
        if self.h.is_some() {
            return;
        }
        let em = Em(Arc::new(self.as_mut().qt_thread()));
        let e2 = em.clone();
        let h = {
            let _g = rt().enter();
            Hub::new(Arc::new(move |d: Dl| e2.s(serde_json::to_value(&d).unwrap_or(Value::Null))))
        };
        let fc = cfg::rd("cfg.json").unwrap_or(json!({}));
        {
            let mut s = h.st.blocking_write();
            let f = |k: &str, d: bool| fc.pointer(&format!("/feat/{k}")).and_then(|v| v.as_bool()).unwrap_or(d);
            s.cfg.log_del = f("msglog", true);
            s.cfg.log_ed = f("msglog", true);
            s.cfg.show_hid = f("hidden", true);
        }
        crate::feat::install(&h, &fc);
        let e3 = em.clone();
        let w = notify::recommended_watcher(move |r: notify::Result<notify::Event>| {
            if let Ok(ev) = r {
                for p in ev.paths {
                    e3.s(json!({"t":"Fs","d":p.to_string_lossy()}));
                }
            }
        })
        .ok()
        .map(|mut w| {
            use notify::Watcher;
            let _ = w.watch(&cfg::dir(), notify::RecursiveMode::Recursive);
            w
        });
        let accs = acc::load();
        let hh = h.clone();
        let e4 = em.clone();
        rt().spawn(async move {
            for a in accs.into_iter().filter(|a| a.on) {
                let s = acc::get(a.id);
                if let Some(b) = mk(&a, s).await {
                    if let Err(x) = hh.add(b).await {
                        e4.s(json!({"t":"Err","d":{"acc":a.id,"msg":x.to_string()}}));
                    }
                }
            }
        });
        let mut r = self.as_mut().rust_mut();
        r.h = Some(h);
        r.em = Some(em);
        r._w = w;
    }

    pub fn snap(&self) -> QString {
        let Some(h) = &self.h else { return QString::from("{}") };
        let s = h.st.blocking_read();
        let show = s.cfg.show_hid;
        let chans: Vec<&Chan> = s.chans.values().filter(|c| show || !c.hidden).collect();
        let pres: serde_json::Map<String, Value> = s.pres.iter().map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap_or(Value::Null))).collect();
        let unr: serde_json::Map<String, Value> = s.unr.iter().map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap_or(Value::Null))).collect();
        let v = json!({
            "accs": s.accs.values().collect::<Vec<_>>(),
            "up": s.up.iter().collect::<Vec<_>>(),
            "glds": s.glds.values().collect::<Vec<_>>(),
            "chans": chans,
            "rels": s.rels.values().collect::<Vec<_>>(),
            "pres": pres,
            "unr": unr,
            "vs": s.vs.values().collect::<Vec<_>>(),
            "call": s.call,
        });
        js(&v)
    }

    pub fn msgs(&self, c: &QString) -> QString {
        let (Some(h), Some(c)) = (&self.h, rid(c)) else { return QString::from("[]") };
        let s = h.st.blocking_read();
        js(&s.msgs.get(&c).map(|q| q.iter().collect::<Vec<_>>()).unwrap_or_default())
    }

    pub fn mems(&self, g: &QString) -> QString {
        let (Some(h), Some(g)) = (&self.h, rid(g)) else { return QString::from("[]") };
        let s = h.st.blocking_read();
        let mut v: Vec<Value> = s
            .mems
            .get(&g)
            .map(|m| {
                m.values()
                    .map(|m| {
                        let p = m.usr.id.as_ref().and_then(|i| s.pres.get(i)).cloned().unwrap_or_default();
                        json!({"m": m, "p": p})
                    })
                    .collect()
            })
            .unwrap_or_default();
        v.sort_by(|a, b| a["m"]["usr"]["name"].as_str().cmp(&b["m"]["usr"]["name"].as_str()));
        js(&v)
    }

    pub fn hist(&self, c: &QString, b: &QString) {
        let Some(c) = rid(c) else { return };
        let b = rid(b);
        self.go(move |h, e| async move {
            let v = h.hist(&c, b.as_ref()).await?;
            e.s(json!({"t":"Hist","d":{"chan":c.to_string(),"n":v.len(),"more":b.is_some()}}));
            Ok(())
        });
    }

    pub fn send(&self, c: &QString, body: &QString, reply: &QString, files: &QString, silent: bool) {
        let Some(c) = rid(c) else { return };
        let fs: Vec<PathBuf> = serde_json::from_str::<Vec<String>>(&qs(files)).unwrap_or_default().into_iter().map(|s| PathBuf::from(s.trim_start_matches("file://"))).collect();
        let o = Out { body: qs(body), reply: rid(reply), files: fs, silent, stks: vec![] };
        self.go(move |h, _| async move { h.send(&c, o).await });
    }

    pub fn edit(&self, c: &QString, m: &QString, body: &QString) {
        let (Some(c), Some(m)) = (rid(c), rid(m)) else { return };
        let b = qs(body);
        self.go(move |h, _| async move { h.bk(c.acc).await?.edit(&c, &m, b).await });
    }

    pub fn del(&self, c: &QString, m: &QString) {
        let (Some(c), Some(m)) = (rid(c), rid(m)) else { return };
        self.go(move |h, _| async move { h.bk(c.acc).await?.del(&c, &m).await });
    }

    pub fn react(&self, c: &QString, m: &QString, e: &QString, on: bool) {
        let (Some(c), Some(m)) = (rid(c), rid(m)) else { return };
        let Ok(em) = serde_json::from_str::<Emo>(&qs(e)) else { return };
        self.go(move |h, _| async move { h.bk(c.acc).await?.react(&c, &m, &em, on).await });
    }

    pub fn typing(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go(move |h, _| async move { h.bk(c.acc).await?.typing(&c).await });
    }

    pub fn ack(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go(move |h, _| async move { h.ack(&c).await });
    }

    pub fn join(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go(move |h, _| async move {
            let cur = h.st.read().await.call.chan.clone();
            if let Some(o) = cur {
                if o.acc != c.acc {
                    h.bk(o.acc).await?.leave().await?;
                }
            }
            h.bk(c.acc).await?.join(&c).await
        });
    }

    pub fn leave(&self) {
        self.go(move |h, _| async move {
            let cur = h.st.read().await.call.chan.clone();
            if let Some(o) = cur {
                h.bk(o.acc).await?.leave().await?;
            }
            Ok(())
        });
    }

    pub fn vset(&self, mute: bool, deaf: bool) {
        self.go(move |h, _| async move {
            let cur = h.st.read().await.call.chan.clone();
            if let Some(o) = cur {
                h.bk(o.acc).await?.vset(mute, deaf).await?;
            }
            Ok(())
        });
    }

    pub fn vol(&self, u: &QString, v: f32) {
        let Some(u) = rid(u) else { return };
        self.go(move |h, _| async move { h.bk(u.acc).await?.vol(&u, v).await });
    }

    pub fn golive(&self, on: bool, q: &QString) {
        let lq: Lq = serde_json::from_str(&qs(q)).unwrap_or_default();
        self.go(move |h, _| async move {
            let cur = h.st.read().await.call.chan.clone();
            match cur {
                Some(o) => h.bk(o.acc).await?.golive(on, lq).await,
                None => anyhow::bail!("not in a call"),
            }
        });
    }

    pub fn watch(&self, u: &QString, on: bool) {
        let Some(u) = rid(u) else { return };
        self.go(move |h, _| async move { h.bk(u.acc).await?.watch(&u, on).await });
    }

    pub fn set_st(&self, st: &QString, act: &QString) {
        let s = match qs(st).as_str() {
            "on" => St::On,
            "idle" => St::Idle,
            "dnd" => St::Dnd,
            _ => St::Off,
        };
        let a = Some(qs(act)).filter(|s| !s.is_empty());
        self.go(move |h, _| async move {
            let bs: Vec<_> = h.bks.read().await.values().cloned().collect();
            for b in bs {
                let _ = b.set_st(s, a.clone()).await;
            }
            Ok(())
        });
    }

    pub fn accs(&self) -> QString {
        js(&acc::load())
    }

    pub fn add(&self, svc: &QString, a: &QString) {
        let Some(sv) = Svc::from_tag(&qs(svc)) else { return };
        let v: Value = serde_json::from_str(&qs(a)).unwrap_or(json!({}));
        self.go(move |h, e| async move {
            let mut l = acc::load();
            let id = acc::next(&l);
            let demo = v["demo"].as_bool().unwrap_or(false);
            let (base, sec) = if demo { ("demo".to_string(), String::new()) } else { crate::bks::login(sv, id, &v).await? };
            let ac = acc::Ac { id, svc: sv, lbl: v["lbl"].as_str().unwrap_or("").into(), base, on: true };
            if !demo {
                acc::put(id, &sec)?;
            }
            l.push(ac.clone());
            acc::save(&l);
            if let Some(b) = mk(&ac, Some(sec)).await {
                h.add(b).await?;
            }
            e.s(json!({"t":"Accs"}));
            Ok(())
        });
    }

    pub fn dc_qr(&self) {
        self.go(move |h, e| async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let e2 = e.clone();
            tokio::spawn(async move {
                while let Some(r) = rx.recv().await {
                    match r {
                        cord_dc::ra::Ra::Qr(u) => e2.s(json!({"t":"Qr","d":u})),
                        cord_dc::ra::Ra::Usr(u) => e2.s(json!({"t":"QrUsr","d":{"name":u.name,"id":u.id}})),
                    }
                }
            });
            let tok = cord_dc::ra::login(tx).await?;
            let mut l = acc::load();
            let id = acc::next(&l);
            acc::put(id, &tok)?;
            let ac = acc::Ac { id, svc: Svc::Dc, lbl: String::new(), base: String::new(), on: true };
            l.push(ac.clone());
            acc::save(&l);
            if let Some(b) = mk(&ac, Some(tok)).await {
                h.add(b).await?;
            }
            e.s(json!({"t":"Accs"}));
            Ok(())
        });
    }

    pub fn rm_acc(&self, id: i32) {
        let id = id as u32;
        let mut l = acc::load();
        l.retain(|a| a.id != id);
        acc::save(&l);
        acc::drop_sec(id);
        self.go(move |h, e| async move {
            h.bks.write().await.remove(&id);
            let mut s = h.st.write().await;
            s.accs.remove(&id);
            s.glds.retain(|k, _| k.acc != id);
            s.chans.retain(|k, _| k.acc != id);
            s.rels.retain(|k, _| k.acc != id);
            e.s(json!({"t":"Accs"}));
            Ok(())
        });
    }

    pub fn feat(&self, k: &QString, on: bool) {
        let k = qs(k);
        let mut c = cfg::rd("cfg.json").unwrap_or(json!({}));
        if !c["feat"].is_object() {
            c["feat"] = json!({});
        }
        c["feat"][&k] = json!(on);
        let _ = cfg::wr("cfg.json", &c);
        self.go(move |h, _| async move {
            {
                let mut s = h.st.write().await;
                match k.as_str() {
                    "msglog" => {
                        s.cfg.log_del = on;
                        s.cfg.log_ed = on;
                    }
                    "hidden" => s.cfg.show_hid = on,
                    _ => {}
                }
            }
            crate::feat::set(&h, &k, on).await;
            Ok(())
        });
    }

    pub fn cget(&self, n: &QString) -> QString {
        let n = qs(n);
        if n.contains('/') || n.contains("..") {
            return QString::from("null");
        }
        js(&cfg::rd(&n).unwrap_or(Value::Null))
    }

    pub fn cset(&self, n: &QString, v: &QString) {
        let n = qs(n);
        if n.contains('/') || n.contains("..") {
            return;
        }
        if let Ok(v) = serde_json::from_str::<Value>(&qs(v)) {
            let _ = cfg::wr(&n, &v);
        }
    }

    pub fn ls(&self, sub: &QString, ext: &QString) -> QString {
        let s = qs(sub);
        if s.contains("..") {
            return QString::from("[]");
        }
        js(&cfg::ls(&s, &qs(ext)).into_iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>())
    }

    pub fn rdf(&self, p: &QString) -> QString {
        let p = PathBuf::from(qs(p).trim_start_matches("file://"));
        let sch = dirs::state_dir().map(|d| d.join("caelestia/scheme.json")).and_then(|p| p.canonicalize().ok());
        let ok = p.canonicalize().map(|c| c.starts_with(cfg::dir().canonicalize().unwrap_or_default()) || c.starts_with(cfg::qml_dir().canonicalize().unwrap_or_default()) || Some(&c) == sch.as_ref()).unwrap_or(false);
        if !ok {
            return QString::default();
        }
        QString::from(&std::fs::read_to_string(p).unwrap_or_default())
    }

    pub fn copy(&self, t: &QString) {
        use std::io::Write;
        let t = qs(t);
        for (c, a) in [("wl-copy", vec![]), ("xclip", vec!["-selection", "clipboard"])] {
            if let Ok(mut ch) = std::process::Command::new(c).args(&a).stdin(std::process::Stdio::piped()).spawn() {
                if let Some(mut i) = ch.stdin.take() {
                    let _ = i.write_all(t.as_bytes());
                }
                let _ = ch.wait();
                return;
            }
        }
    }

    pub fn home(&self) -> QString {
        QString::from(cfg::dir().to_string_lossy().as_ref())
    }

    pub fn qml(&self) -> QString {
        QString::from(cfg::qml_dir().to_string_lossy().as_ref())
    }
}
