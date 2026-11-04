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
        #[qsignal]
        #[cxx_name = "frameTick"]
        fn frame_tick(self: Pin<&mut Core>, k: QString, seq: f64);

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
        fn qr_cancel(self: &Core);
        #[qinvokable]
        fn qr_cap(self: &Core, t: &QString);
        #[qinvokable]
        #[cxx_name = "secretOf"]
        fn secret_of(self: &Core, a: i32) -> QString;
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
        fn bad(self: &Core, n: &QString) -> QString;
        #[qinvokable]
        fn cset(self: &Core, n: &QString, v: &QString);
        #[qinvokable]
        fn exists(self: &Core, p: &QString) -> bool;
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
    accounts, calls, cfg, feat_ctl, files, frames, media,
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
    want: feat_ctl::Want,
    _w: Option<notify::RecommendedWatcher>,
}

enum It {
    J(String),
    P(u64, u64, Vec<(String, Pres)>),
}

#[derive(Clone)]
pub struct Em {
    t: Arc<cxx_qt::CxxQtThread<q::Core>>,
    buf: Arc<Mutex<Vec<It>>>,
    sched: Arc<AtomicBool>,
}

pub const TICK_MS: u64 = 16;

fn stamp(seq: u64, from: Option<u64>, js: &str) -> String {
    if !js.starts_with('{') || js.len() < 3 {
        return js.to_string();
    }
    match from {
        Some(f) => format!("{{\"seq\":{seq},\"from\":{f},{}", &js[1..]),
        None => format!("{{\"seq\":{seq},{}", &js[1..]),
    }
}

impl Em {
    pub fn s(&self, v: &Out) {
        self.sq(0, v);
    }

    pub fn sq(&self, seq: u64, v: &Out) {
        let js = serde_json::to_string(v).unwrap_or_default();
        self.push(It::J(stamp(seq, None, &js)));
    }

    fn push(&self, it: It) {
        {
            let mut g = self.buf.lock().unwrap_or_else(|e| e.into_inner());
            match (g.last_mut(), it) {
                (Some(It::P(_, to, v)), It::P(f, t, m)) if *to + 1 == f => {
                    *to = t;
                    for (u, p) in m {
                        match v.iter_mut().find(|x| x.0 == u) {
                            Some(x) => x.1 = p,
                            None => v.push((u, p)),
                        }
                    }
                }
                (_, it) => g.push(it),
            }
        }
        if !self.sched.swap(true, Ordering::SeqCst) {
            let e = self.clone();
            rt().spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(TICK_MS)).await;
                e.sched.store(false, Ordering::SeqCst);
                e.flush();
            });
        }
    }

    fn flush(&self) {
        let items: Vec<It> = std::mem::take(&mut *self.buf.lock().unwrap_or_else(|e| e.into_inner()));
        if items.is_empty() {
            return;
        }
        let parts: Vec<String> = items
            .into_iter()
            .filter_map(|it| match it {
                It::J(s) => Some(s),
                It::P(f, t, m) => {
                    let o = out::pres_flush(m)?;
                    let js = serde_json::to_string(&o).unwrap_or_default();
                    Some(stamp(t, Some(f), &js))
                }
            })
            .collect();
        let s = format!("[{}]", parts.join(","));
        let _ = self.t.queue(move |o| o.dl(QString::from(&s)));
    }

    pub fn err(&self, acc: u32, ctx: &str, m: impl ToString) {
        self.s(&Out::Err(ErrV::new(acc, ctx, m)));
    }

    fn ticks(&self, v: Vec<(String, u64)>) {
        let _ = self.t.queue(move |mut o| {
            for (k, n) in v {
                o.as_mut().frame_tick(QString::from(&k), n as f64);
            }
        });
    }

    fn pres(&self, seq: u64, usr: String, p: Pres) {
        self.push(It::P(seq, seq, vec![(usr, p)]));
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

    fn go_ack<T, F, Fut, A>(&self, f: F, ack: A)
    where
        T: Send + 'static,
        F: FnOnce(Arc<Hub>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Res<T>> + Send + 'static,
        A: FnOnce(Res<T>) -> Out + Send + 'static,
    {
        if let Some((h, e)) = self.hb() {
            rt().spawn(async move {
                let r = f(h).await;
                e.s(&ack(r));
            });
        }
    }
}

fn sent(ck: String, r: Res<()>) -> Out {
    match r {
        Ok(()) => Out::Sent { chan: ck, ok: true, err: String::new() },
        Err(x) => Out::Sent { chan: ck, ok: false, err: x.to_string() },
    }
}

impl q::Core {
    pub fn boot(mut self: Pin<&mut Self>) {
        if self.h.is_some() {
            return;
        }
        let em = Em { t: Arc::new(self.as_mut().qt_thread()), buf: Default::default(), sched: Default::default() };
        let e2 = em.clone();
        let want = self.want.clone();
        let w2 = want.clone();
        let cell: Arc<OnceLock<Weak<Hub>>> = Default::default();
        let c2 = cell.clone();
        let store = match cord_store::Store::open(&cfg::data().join("cache.db")) {
            Ok(s) => {
                let s = Arc::new(s);
                crate::rt::keep_store(s.clone());
                Some(s)
            }
            Err(x) => {
                em.err(0, "store", format!("message cache disabled: {x}"));
                None
            }
        };
        if let Err(x) = feat_ctl::load(&want) {
            em.err(0, "cfg", x);
        }
        let hcfg = cord_hub::Cfg { log_del: feat::on("msglog"), log_ed: feat::on("msglog"), show_hid: feat::on("hidden") };
        let h = {
            let _g = rt().enter();
            Hub::with_cfg(
                Arc::new(move |d: cord_hub::Sd| match out::conv(d.d) {
                    Conv::Pres(u, p) => e2.pres(d.seq, u, p),
                    Conv::Now(o) => {
                        if let Out::Acc(a) = &o {
                            let acc = a.id;
                            let want = w2.lock().ok().and_then(|g| g.clone());
                            if let (Some((s, act)), Some(h)) = (want, c2.get().and_then(|w| w.upgrade())) {
                                rt().spawn(async move {
                                    if let Ok(b) = h.bk(acc).await {
                                        let _ = b.set_st(s, act).await;
                                    }
                                });
                            }
                        }
                        e2.sq(d.seq, &o)
                    }
                }),
                store,
                hcfg,
            )
        };
        let _ = cell.set(Arc::downgrade(&h));
        for (n, m) in files::bad_all() {
            em.err(0, "parse", format!("{n}: {m}"));
        }
        let te = em.clone();
        frames::set_tick(move |v| te.ticks(v));
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
        let mut v = snap::boot(&s);
        v["seq"] = json!(h.seq());
        js(&v)
    }

    pub fn load_msgs(&self, c: &QString) {
        let Some(c) = rid(c) else { return };
        self.go("load", c.acc, move |h, e| async move {
            let (list, synced) = h.msgs(&c).await;
            e.s(&Out::Msgs { chan: c.to_string(), list, synced });
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
        let more = b.is_some();
        let ck = c.to_string();
        self.go_ack(
            move |h| async move { h.hist(&c, b.as_ref()).await },
            move |r| match r {
                Ok(r) => Out::Hist { chan: ck, n: r.n, more, ok: true, list: r.added, err: String::new(), noop: r.noop },
                Err(x) => Out::Hist { chan: ck, n: 0, more, ok: false, list: vec![], err: x.to_string(), noop: false },
            },
        );
    }

    pub fn send(&self, c: &QString, body: &QString, reply: &QString, files: &QString, silent: bool) {
        let Some(c) = rid(c) else { return };
        let raw: Vec<String> = serde_json::from_str(&qs(files)).unwrap_or_default();
        let bad = raw.iter().any(|s| s.contains("://") && !s.starts_with("file://"));
        let fs: Vec<PathBuf> = raw.into_iter().map(|s| PathBuf::from(s.trim_start_matches("file://"))).collect();
        let o = cord_core::Out { body: qs(body), reply: rid(reply), files: fs, silent, stks: vec![] };
        let ck = c.to_string();
        self.go_ack(
            move |h| async move {
                if bad {
                    anyhow::bail!("only local files can be attached");
                }
                h.send(&c, o).await
            },
            move |r| sent(ck, r),
        );
    }

    pub fn edit(&self, c: &QString, m: &QString, body: &QString) {
        let (Some(c), Some(m)) = (rid(c), rid(m)) else { return };
        let b = qs(body);
        let ck = c.to_string();
        self.go_ack(move |h| async move { h.bk(c.acc).await?.edit(&c, &m, b).await }, move |r| sent(ck, r));
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
        self.go("join", c.acc, move |h, _| calls::join(h, c));
    }

    pub fn leave(&self) {
        self.go("leave", 0, move |h, _| calls::leave(h));
    }

    pub fn vset(&self, mute: bool, deaf: bool) {
        self.go("voice", 0, move |h, _| calls::vset(h, mute, deaf));
    }

    pub fn vol(&self, u: &QString, v: f32) {
        let Some(u) = rid(u) else { return };
        self.go("volume", u.acc, move |h, _| async move { h.bk(u.acc).await?.vol(&u, v).await });
    }

    pub fn golive(&self, on: bool, q: &QString) {
        let lq: Lq = feat_ctl::cap_lq(serde_json::from_str(&qs(q)).unwrap_or_default());
        self.go("stream", 0, move |h, _| calls::golive(h, on, lq));
    }

    pub fn watch(&self, u: &QString, on: bool) {
        let Some(u) = rid(u) else { return };
        self.go("watch", u.acc, move |h, _| calls::watch(h, u, on));
    }

    pub fn set_st(&self, st: &QString, act: &QString) {
        let s = feat_ctl::parse_st(&qs(st));
        let a = Some(qs(act)).filter(|s| !s.is_empty());
        if let Ok(mut g) = self.want.lock() {
            *g = Some((s, a.clone()));
        }
        self.go("status", 0, move |h, _| async move {
            feat_ctl::set_status(h, s, a).await;
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
        if let Some((h, e)) = self.hb() {
            accounts::qr(h, e);
        }
    }

    pub fn qr_cancel(&self) {
        accounts::qr_cancel();
    }

    pub fn qr_cap(&self, t: &QString) {
        accounts::qr_cap(qs(t));
    }

    pub fn rm_acc(&self, id: i32) {
        let id = id as u32;
        self.go("remove", id, move |h, e| accounts::remove(h, e, id));
    }

    pub fn open_dm(&self, u: &QString) {
        let Some(u) = rid(u) else { return };
        let us = u.to_string();
        self.go_ack(
            move |h| async move { h.bk(u.acc).await?.open_dm(&u).await },
            move |r| match r {
                Ok(c) => Out::Dm { usr: us, chan: c.to_string(), ok: true, err: String::new() },
                Err(x) => Out::Dm { usr: us, chan: String::new(), ok: false, err: x.to_string() },
            },
        );
    }

    pub fn feat(&self, k: &QString, on: bool) {
        let k = qs(k);
        if !feat::known(&k) {
            return;
        }
        feat::set(&k, on);
        if let (Err(x), Some(e)) = (feat_ctl::persist(&k, on), &self.em) {
            e.err(0, "cfg", x);
        }
        self.go("feature", 0, move |h, e| async move {
            if let Some(o) = feat_ctl::apply(&h, &k, on).await {
                e.s(&o);
            }
            Ok(())
        });
    }

    pub fn features(&self) -> QString {
        js(&feat_ctl::list())
    }

    pub fn services(&self) -> QString {
        js(&svc_list())
    }

    pub fn secret_of(&self, a: i32) -> QString {
        QString::from(crate::acc::secret_of(a as u32).as_str())
    }

    pub fn secret_backend(&self) -> QString {
        QString::from(crate::acc::backend())
    }

    pub fn frame_seq(&self, k: &QString) -> f64 {
        frames::seq(&qs(k)) as f64
    }

    pub fn cget(&self, n: &QString) -> QString {
        let n = qs(n);
        let (v, err) = files::cget(&n);
        if let (Some(m), Some(e)) = (err, &self.em) {
            e.err(0, "parse", format!("{n}: {m}"));
        }
        js(&v)
    }

    pub fn bad(&self, n: &QString) -> QString {
        QString::from(&files::bad(&qs(n)))
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

    pub fn exists(&self, p: &QString) -> bool {
        files::exists(&qs(p))
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
