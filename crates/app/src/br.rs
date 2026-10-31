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
        fn load_msgs(self: &Core, c: &QString);
        #[qinvokable]
        fn load_mems(self: &Core, g: &QString);
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
        fn open_dm(self: &Core, u: &QString);
        #[qinvokable]
        fn feat(self: &Core, k: &QString, on: bool);
        #[qinvokable]
        fn features(self: &Core) -> QString;
        #[qinvokable]
        fn services(self: &Core) -> QString;
        #[qinvokable]
        #[cxx_name = "secretBackend"]
        fn secret_backend(self: &Core) -> QString;
        #[qinvokable]
        #[cxx_name = "frameSeq"]
        fn frame_seq(self: &Core, k: &QString) -> f64;
        #[qinvokable]
        fn cget(self: &Core, n: &QString) -> QString;
        #[qinvokable]
        fn cset(self: &Core, n: &QString, v: &QString);
        #[qinvokable]
        fn ls(self: &Core, sub: &QString, ext: &QString) -> QString;
        #[qinvokable]
        fn rdf(self: &Core, p: &QString) -> QString;
        #[qinvokable]
        fn scheme(self: &Core) -> QString;
        #[qinvokable]
        fn home(self: &Core) -> QString;
        #[qinvokable]
        fn copy(self: &Core, t: &QString);
        #[qinvokable]
        fn qr(self: &Core, t: &QString) -> QString;
        #[qinvokable]
        fn mpris(self: &Core);
        #[qinvokable]
        fn notify(self: &Core, t: &QString, b: &QString);
        #[qinvokable]
        fn ptt(self: &Core, on: bool);
        #[qinvokable]
        fn media(self: &Core, cmd: &QString);
        #[qinvokable]
        fn rel(self: &Core, u: &QString, act: &QString);
        #[qinvokable]
        fn qml(self: &Core) -> QString;
    }
    impl cxx_qt::Threading for Core {}
}

use crate::{
    accounts, cfg, files, media,
    out::{self, Conv, ErrV, Out},
    rt::rt,
    snap,
};
use cord_core::*;
use cord_hub::Hub;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Default)]
pub struct CoreR {
    h: Option<Arc<Hub>>,
    em: Option<Em>,
    _w: Option<notify::RecommendedWatcher>,
}

#[derive(Clone)]
pub struct Em {
    t: Arc<cxx_qt::CxxQtThread<q::Core>>,
    pend: Arc<Mutex<HashMap<String, Pres>>>,
    sched: Arc<AtomicBool>,
}

impl Em {
    pub fn s(&self, v: &Out) {
        let s = serde_json::to_string(v).unwrap_or_default();
        let _ = self.t.queue(move |o| o.dl(QString::from(&s)));
    }

    pub fn err(&self, acc: u32, ctx: &str, m: impl ToString) {
        self.s(&Out::Err(ErrV::new(acc, ctx, m)));
    }

    fn pres(&self, usr: String, p: Pres) {
        if let Ok(mut g) = self.pend.lock() {
            g.insert(usr, p);
        }
        if !self.sched.swap(true, Ordering::SeqCst) {
            let e = self.clone();
            rt().spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                e.sched.store(false, Ordering::SeqCst);
                let m: Vec<(String, Pres)> = e.pend.lock().map(|mut g| g.drain().collect()).unwrap_or_default();
                if let Some(o) = out::pres_flush(m) {
                    e.s(&o);
                }
            });
        }
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

static WANT: Mutex<Option<(St, Option<String>)>> = Mutex::new(None);

fn parse_st(s: &str) -> St {
    match s.to_lowercase().as_str() {
        "on" | "online" => St::On,
        "idle" => St::Idle,
        "dnd" => St::Dnd,
        _ => St::Off,
    }
}

fn cap_lq(mut q: Lq) -> Lq {
    if !feat::on("streamq") {
        q.w = q.w.min(1280);
        q.h = q.h.min(720);
        q.fps = q.fps.min(30);
        q.kbps = q.kbps.min(4000);
    }
    q
}

impl CoreR {
    fn hb(&self) -> Option<(Arc<Hub>, Em)> {
        Some((self.h.clone()?, self.em.clone()?))
    }
    fn go<F, Fut>(&self, ctx: &'static str, acc: u32, f: F)
    where
        F: FnOnce(Arc<Hub>, Em) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Res<()>> + Send + 'static,
    {
        if let Some((h, e)) = self.hb() {
            rt().spawn(async move {
                let e2 = e.clone();
                if let Err(x) = f(h, e).await {
                    e2.err(acc, ctx, x);
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
        let em = Em { t: Arc::new(self.as_mut().qt_thread()), pend: Default::default(), sched: Default::default() };
        let e2 = em.clone();
        let cell: Arc<OnceLock<Weak<Hub>>> = Default::default();
        let c2 = cell.clone();
        let store = match cord_store::Store::open(&cfg::data().join("cache.db")) {
            Ok(s) => Some(Arc::new(s)),
            Err(x) => {
                em.err(0, "store", format!("message cache disabled: {x}"));
                None
            }
        };
        let h = {
            let _g = rt().enter();
            Hub::new(
                Arc::new(move |d| match out::conv(d) {
                    Conv::Pres(u, p) => e2.pres(u, p),
                    Conv::Now(o) => {
                        if let Out::Acc(a) = &o {
                            let acc = a.id;
                            let want = WANT.lock().ok().and_then(|g| g.clone());
                            if let (Some((s, act)), Some(h)) = (want, c2.get().and_then(|w| w.upgrade())) {
                                rt().spawn(async move {
                                    if let Ok(b) = h.bk(acc).await {
                                        let _ = b.set_st(s, act).await;
                                    }
                                });
                            }
                        }
                        e2.s(&o)
                    }
                }),
                store,
            )
        };
        let _ = cell.set(Arc::downgrade(&h));
        match cfg::rd_in(&cfg::dir(), "cfg.json") {
            Ok(fc) => {
                let fc = fc.unwrap_or(json!({}));
                if let Some(o) = fc["feat"].as_object() {
                    for (k, v) in o {
                        if let Some(b) = v.as_bool() {
                            feat::set(k, b);
                        }
                    }
                }
            }
            Err(x) => em.err(0, "cfg", x),
        }
        if let Some(u) = cfg::rd("ui.json") {
            if let Some(s) = u["st"].as_str() {
                if let Ok(mut g) = WANT.lock() {
                    *g = Some((parse_st(s), None));
                }
            }
        }
        if let Ok(mut s) = h.st.try_write() {
            s.cfg.log_del = feat::on("msglog");
            s.cfg.log_ed = feat::on("msglog");
            s.cfg.show_hid = feat::on("hidden");
        }
        let e3 = em.clone();
        let w = notify::recommended_watcher(move |r: notify::Result<notify::Event>| {
            if let Ok(ev) = r {
                for p in ev.paths {
                    if files::watch_ignored(&p) {
                        continue;
                    }
                    e3.s(&Out::Fs(p.to_string_lossy().to_string()));
                }
            }
        })
        .ok()
        .map(|mut w| {
            use notify::Watcher;
            let _ = w.watch(&cfg::dir(), notify::RecursiveMode::Recursive);
            w
        });
        let hh = h.clone();
        let e4 = em.clone();
        rt().spawn(accounts::start_all(hh, e4));
        let mut r = self.as_mut().rust_mut();
        r.h = Some(h);
        r.em = Some(em);
        r._w = w;
    }

    pub fn snap(&self) -> QString {
        let Some(h) = &self.h else { return QString::from("{}") };
        let s = match h.st.try_read() {
            Ok(g) => g,
            Err(_) => h.st.blocking_read(),
        };
        js(&snap::boot(&s))
    }

    pub fn load_msgs(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go("load", c.acc, move |h, e| async move {
            let list = h.msgs(&c).await;
            e.s(&Out::Msgs { chan: c.to_string(), list });
            Ok(())
        });
    }

    pub fn load_mems(&self, g: &QString) {
        let Some(g) = rid(g) else { return };
        self.go("members", g.acc, move |h, e| async move {
            let have: Vec<Mem> = h.st.read().await.mems.get(&g).map(|m| m.values().cloned().collect()).unwrap_or_default();
            if !have.is_empty() {
                e.s(&Out::Mems { gld: g.to_string(), list: have });
            }
            let v = h.bk(g.acc).await?.mems(&g).await?;
            if !v.is_empty() {
                let _ = h.tx().send(Ev::Mems(v));
            }
            Ok(())
        });
    }

    pub fn hist(&self, c: &QString, b: &QString) {
        let Some(c) = rid(c) else { return };
        let b = rid(b);
        self.go("history", c.acc, move |h, e| async move {
            let ck = c.to_string();
            match h.hist(&c, b.as_ref()).await {
                Ok(r) => e.s(&Out::Hist { chan: ck, n: r.n, more: b.is_some(), ok: true, list: r.added }),
                Err(x) => {
                    e.s(&Out::Hist { chan: ck, n: 0, more: b.is_some(), ok: false, list: vec![] });
                    return Err(x);
                }
            }
            Ok(())
        });
    }

    pub fn send(&self, c: &QString, body: &QString, reply: &QString, files: &QString, silent: bool) {
        let Some(c) = rid(c) else { return };
        let raw: Vec<String> = serde_json::from_str(&qs(files)).unwrap_or_default();
        let bad = raw.iter().any(|s| s.contains("://") && !s.starts_with("file://"));
        let fs: Vec<PathBuf> = raw.into_iter().map(|s| PathBuf::from(s.trim_start_matches("file://"))).collect();
        let o = cord_core::Out { body: qs(body), reply: rid(reply), files: fs, silent, stks: vec![] };
        self.go("send", c.acc, move |h, e| async move {
            let ck = c.to_string();
            let r = if bad { Err(anyhow::anyhow!("only local files can be attached")) } else { h.send(&c, o).await };
            match r {
                Ok(()) => e.s(&Out::Sent { chan: ck, ok: true, err: String::new() }),
                Err(x) => {
                    e.s(&Out::Sent { chan: ck, ok: false, err: x.to_string() });
                    return Err(x);
                }
            }
            Ok(())
        });
    }

    pub fn edit(&self, c: &QString, m: &QString, body: &QString) {
        let (Some(c), Some(m)) = (rid(c), rid(m)) else { return };
        let b = qs(body);
        self.go("edit", c.acc, move |h, e| async move {
            let ck = c.to_string();
            match h.bk(c.acc).await?.edit(&c, &m, b).await {
                Ok(()) => e.s(&Out::Sent { chan: ck, ok: true, err: String::new() }),
                Err(x) => {
                    e.s(&Out::Sent { chan: ck, ok: false, err: x.to_string() });
                    return Err(x);
                }
            }
            Ok(())
        });
    }

    pub fn del(&self, c: &QString, m: &QString) {
        let (Some(c), Some(m)) = (rid(c), rid(m)) else { return };
        self.go("delete", c.acc, move |h, _| async move { h.bk(c.acc).await?.del(&c, &m).await });
    }

    pub fn react(&self, c: &QString, m: &QString, e: &QString, on: bool) {
        let (Some(c), Some(m)) = (rid(c), rid(m)) else { return };
        let Ok(em) = serde_json::from_str::<Emo>(&qs(e)) else { return };
        self.go("react", c.acc, move |h, _| async move { h.bk(c.acc).await?.react(&c, &m, &em, on).await });
    }

    pub fn typing(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go("typing", c.acc, move |h, _| async move { h.typing(&c).await });
    }

    pub fn ack(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go("ack", c.acc, move |h, _| async move { h.ack(&c).await });
    }

    pub fn join(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go("join", c.acc, move |h, _| async move {
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
        self.go("leave", 0, move |h, _| async move {
            let cur = h.st.read().await.call.chan.clone();
            if let Some(o) = cur {
                h.bk(o.acc).await?.leave().await?;
            }
            Ok(())
        });
    }

    pub fn vset(&self, mute: bool, deaf: bool) {
        self.go("voice", 0, move |h, _| async move {
            let cur = h.st.read().await.call.chan.clone();
            if let Some(o) = cur {
                h.bk(o.acc).await?.vset(mute, deaf).await?;
            }
            Ok(())
        });
    }

    pub fn vol(&self, u: &QString, v: f32) {
        let Some(u) = rid(u) else { return };
        self.go("volume", u.acc, move |h, _| async move { h.bk(u.acc).await?.vol(&u, v).await });
    }

    pub fn golive(&self, on: bool, q: &QString) {
        let lq: Lq = cap_lq(serde_json::from_str(&qs(q)).unwrap_or_default());
        self.go("stream", 0, move |h, _| async move {
            let cur = h.st.read().await.call.chan.clone();
            let Some(o) = cur else { anyhow::bail!("not in a call") };
            h.bk(o.acc).await?.golive(on, lq).await?;
            if let (true, Some(d), Some(me)) = (on, crate::bks::dc_of(o.acc), h.st.read().await.accs.get(&o.acc).and_then(|a| a.me.id.clone())) {
                let mut rx = d.frames(&me);
                tokio::spawn(async move {
                    while rx.changed().await.is_ok() {
                        let f = rx.borrow().clone();
                        match f {
                            Some(f) => crate::frames::put("me", f),
                            None => break,
                        }
                    }
                });
            } else if !on {
                crate::frames::drop_k("me");
            }
            Ok(())
        });
    }

    pub fn watch(&self, u: &QString, on: bool) {
        let Some(u) = rid(u) else { return };
        self.go("watch", u.acc, move |h, _| async move {
            h.bk(u.acc).await?.watch(&u, on).await?;
            let k = u.to_string();
            if !on {
                crate::frames::drop_k(&k);
                return Ok(());
            }
            if let Some(d) = crate::bks::dc_of(u.acc) {
                let mut rx = d.frames(&u);
                tokio::spawn(async move {
                    while rx.changed().await.is_ok() {
                        let f = rx.borrow().clone();
                        match f {
                            Some(f) => crate::frames::put(&k, f),
                            None => break,
                        }
                    }
                });
            }
            Ok(())
        });
    }

    pub fn set_st(&self, st: &QString, act: &QString) {
        let s = parse_st(&qs(st));
        let a = Some(qs(act)).filter(|s| !s.is_empty());
        if let Ok(mut g) = WANT.lock() {
            *g = Some((s, a.clone()));
        }
        self.go("status", 0, move |h, _| async move {
            let bs: Vec<_> = h.bks.read().await.values().cloned().collect();
            for b in bs {
                let _ = b.set_st(s, a.clone()).await;
            }
            Ok(())
        });
    }

    pub fn accs(&self) -> QString {
        js(&crate::acc::load())
    }

    pub fn add(&self, svc: &QString, a: &QString) {
        let Some(sv) = Svc::from_tag(&qs(svc)) else { return };
        let v: Value = serde_json::from_str(&qs(a)).unwrap_or(json!({}));
        self.go("add", 0, move |h, e| accounts::add(h, e, sv, v));
    }

    pub fn dc_qr(&self) {
        self.go("login", 0, move |h, e| accounts::qr(h, e));
    }

    pub fn rm_acc(&self, id: i32) {
        let id = id as u32;
        self.go("remove", id, move |h, e| accounts::remove(h, e, id));
    }

    pub fn open_dm(&self, u: &QString) {
        let Some(u) = rid(u) else { return };
        self.go("dm", u.acc, move |h, e| async move {
            let us = u.to_string();
            match h.bk(u.acc).await?.open_dm(&u).await {
                Ok(c) => e.s(&Out::Dm { usr: us, chan: c.to_string(), ok: true, err: String::new() }),
                Err(x) => {
                    e.s(&Out::Dm { usr: us, chan: String::new(), ok: false, err: x.to_string() });
                    return Err(x);
                }
            }
            Ok(())
        });
    }

    pub fn feat(&self, k: &QString, on: bool) {
        let k = qs(k);
        if !feat::known(&k) {
            return;
        }
        feat::set(&k, on);
        let w = (|| -> anyhow::Result<()> {
            let mut c = cfg::rd_in(&cfg::dir(), "cfg.json")?.unwrap_or(json!({}));
            if !c["feat"].is_object() {
                c["feat"] = json!({});
            }
            c["feat"][&k] = json!(on);
            cfg::wr("cfg.json", &c)
        })();
        if let (Err(x), Some(e)) = (w, &self.em) {
            e.err(0, "cfg", x);
        }
        self.go("feature", 0, move |h, e| async move {
            let vis = {
                let mut s = h.st.write().await;
                match k.as_str() {
                    "msglog" => {
                        s.cfg.log_del = on;
                        s.cfg.log_ed = on;
                        None
                    }
                    "hidden" => {
                        s.cfg.show_hid = on;
                        Some(s.vis_chans())
                    }
                    _ => None,
                }
            };
            if let Some(v) = vis {
                e.s(&Out::Chans(v));
            }
            Ok(())
        });
    }

    pub fn features(&self) -> QString {
        let v: Vec<Value> = feat::FEATS.iter().map(|f| json!({"k": f.k, "n": f.n, "d": f.d, "on": f.on})).collect();
        js(&v)
    }

    pub fn services(&self) -> QString {
        js(&svc_list())
    }

    pub fn secret_backend(&self) -> QString {
        QString::from(crate::acc::backend())
    }

    pub fn frame_seq(&self, k: &QString) -> f64 {
        crate::frames::seq(&qs(k)) as f64
    }

    pub fn cget(&self, n: &QString) -> QString {
        js(&files::cget(&qs(n)))
    }

    pub fn cset(&self, n: &QString, v: &QString) {
        let n = qs(n);
        let r = serde_json::from_str::<Value>(&qs(v)).map_err(anyhow::Error::from).and_then(|v| files::cset(&n, &v));
        if let (Err(x), Some(e)) = (r, &self.em) {
            e.err(0, "cset", x);
        }
        if n == "voice.json" {
            crate::bks::vapply_all();
        }
    }

    pub fn ls(&self, sub: &QString, ext: &QString) -> QString {
        js(&files::ls(&qs(sub), &qs(ext)))
    }

    pub fn rdf(&self, p: &QString) -> QString {
        QString::from(&files::rdf(&qs(p)))
    }

    pub fn scheme(&self) -> QString {
        QString::from(&files::scheme())
    }

    pub fn copy(&self, t: &QString) {
        media::copy(qs(t));
    }

    pub fn qr(&self, t: &QString) -> QString {
        let Ok(c) = qrcode::QrCode::new(qs(t).as_bytes()) else { return QString::default() };
        let svg = c.render::<qrcode::render::svg::Color>().min_dimensions(400, 400).quiet_zone(false).build();
        QString::from(&format!("data:image/svg+xml;utf8,{}", svg.replace('#', "%23")))
    }

    pub fn ptt(&self, on: bool) {
        crate::bks::ptt(on);
    }

    pub fn notify(&self, t: &QString, b: &QString) {
        media::notify(qs(t), qs(b));
    }

    pub fn mpris(&self) {
        let Some(e) = self.em.clone() else { return };
        media::mpris(move |m| e.s(&Out::Mpris(m)));
    }

    pub fn media(&self, cmd: &QString) {
        media::media_cmd(&qs(cmd));
    }

    pub fn rel(&self, u: &QString, act: &QString) {
        let Some(u) = rid(u) else { return };
        let on = qs(act) == "accept";
        self.go("relation", u.acc, move |h, _| async move { h.bk(u.acc).await?.rel(&u, on).await });
    }

    pub fn home(&self) -> QString {
        QString::from(cfg::dir().to_string_lossy().as_ref())
    }

    pub fn qml(&self) -> QString {
        QString::from(cfg::qml_dir().to_string_lossy().as_ref())
    }
}
