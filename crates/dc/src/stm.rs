use crate::Sh;
use cord_cap::{
    Au, Cod, Frame,
    aud::{self, Aenc},
    dec::Dec,
    enc::{self, Enc, Keyer, Src},
    scr,
};
use cord_core::*;
use cord_rtc::{
    dcv::{Cfg, Media, SEv, Sio, VEv, Vc},
    vrtp::{Codec, Q},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, unbounded_channel},
        watch,
    },
    task::JoinHandle,
};

type Fs = Arc<watch::Sender<Option<Arc<Frame>>>>;

#[derive(Default)]
struct Own {
    key: Option<String>,
    rtc: Option<String>,
    tok: Option<String>,
    ep: Option<String>,
    lq: Option<Lq>,
    _enc: Option<Enc>,
    keyer: Option<Keyer>,
    _aenc: Option<Aenc>,
    pre: Option<(UnboundedReceiver<Au>, Option<UnboundedReceiver<aud::Op>>)>,
    vc: Option<Vc>,
    tasks: Vec<JoinHandle<()>>,
}

#[derive(Default)]
struct Wat {
    key: String,
    rtc: Option<String>,
    tok: Option<String>,
    ep: Option<String>,
    vc: Option<Vc>,
    tasks: Vec<JoinHandle<()>>,
}

#[derive(Default)]
struct In {
    own: Option<Own>,
    wat: HashMap<String, Wat>,
    fr: HashMap<String, Fs>,
}

#[derive(Clone, Default)]
pub struct Stm(Arc<Mutex<In>>);

pub fn key_of(gld: Option<&str>, chan: &str, uid: &str) -> String {
    match gld {
        Some(g) => format!("guild:{g}:{chan}:{uid}"),
        None => format!("call:{chan}:{uid}"),
    }
}

pub fn key_uid(k: &str) -> Option<&str> {
    k.rsplit(':').next()
}

pub fn create_op(gld: Option<&str>, chan: &str) -> Value {
    json!({"op": 18, "d": {"type": if gld.is_some() { "guild" } else { "call" }, "guild_id": gld, "channel_id": chan, "preferred_region": null}})
}

pub fn watch_op(key: &str) -> Value {
    json!({"op": 20, "d": {"stream_key": key}})
}

pub fn delete_op(key: &str) -> Value {
    json!({"op": 19, "d": {"stream_key": key}})
}

pub fn pause_op(key: &str, p: bool) -> Value {
    json!({"op": 22, "d": {"stream_key": key, "paused": p}})
}

pub fn gs(sh: &Sh, v: Value) {
    if let Some(t) = &*sh.gt.lock().unwrap() {
        let _ = t.send(v.to_string());
    }
}

fn cod(c: Codec) -> Cod {
    match c {
        Codec::H264 => Cod::H264,
        Codec::Vp8 => Cod::Vp8,
        Codec::Av1 => Cod::Av1,
    }
}

impl Stm {
    fn fs(&self, uid: &str) -> Fs {
        self.0.lock().unwrap().fr.entry(uid.to_string()).or_insert_with(|| Arc::new(watch::channel(None).0)).clone()
    }

    pub fn frames(&self, uid: &str) -> watch::Receiver<Option<Arc<Frame>>> {
        self.fs(uid).subscribe()
    }

    pub fn stop_all(&self, sh: &Sh) {
        let (o, w) = {
            let mut i = self.0.lock().unwrap();
            (i.own.take(), std::mem::take(&mut i.wat))
        };
        if let Some(o) = o {
            for t in &o.tasks {
                t.abort();
            }
            if let Some(k) = &o.key {
                gs(sh, delete_op(k));
            }
            sh.vo.live(false);
        }
        for (_, w) in w {
            for t in &w.tasks {
                t.abort();
            }
            gs(sh, delete_op(&w.key));
        }
        for f in self.0.lock().unwrap().fr.values() {
            f.send_replace(None);
        }
    }

    pub async fn golive(&self, sh: &Sh, on: bool, q: Lq) -> Res<()> {
        if !on {
            let o = self.0.lock().unwrap().own.take();
            if let Some(o) = o {
                for t in &o.tasks {
                    t.abort();
                }
                if let Some(k) = &o.key {
                    gs(sh, delete_op(k));
                }
            }
            sh.vo.live(false);
            let me = sh.me.lock().unwrap().clone();
            self.fs(&me).send_replace(None);
            return Ok(());
        }
        let (chan, gld, _) = sh.vo.info().ok_or_else(|| anyhow::anyhow!("join a voice channel first"))?;
        if self.0.lock().unwrap().own.is_some() {
            anyhow::bail!("already streaming");
        }
        let pipe = scr::pick(false).await?;
        let c = enc::Cfg { w: q.w, h: q.h, fps: q.fps, kbps: q.kbps };
        let (e, au) = Enc::start(c, Src::Pw(pipe))?;
        let (aenc, arx) = if q.snd {
            match Aenc::start(aud::Cfg { kbps: 128, node: None, test: false }) {
                Ok((a, r)) => (Some(a), Some(r)),
                Err(e) => {
                    tracing::warn!("stream audio: {e}");
                    (None, None)
                }
            }
        } else {
            (None, None)
        };
        let o = Own { lq: Some(q), keyer: Some(e.keyer()), _enc: Some(e), _aenc: aenc, pre: Some((au, arx)), ..Default::default() };
        self.0.lock().unwrap().own = Some(o);
        gs(sh, create_op(gld.as_deref(), &chan));
        Ok(())
    }

    pub fn pause(&self, sh: &Sh, p: bool) -> Res<()> {
        let k = self.0.lock().unwrap().own.as_ref().and_then(|o| o.key.clone()).ok_or_else(|| anyhow::anyhow!("not streaming"))?;
        gs(sh, pause_op(&k, p));
        Ok(())
    }

    pub fn watch(&self, sh: &Sh, usr: &str, on: bool) -> Res<()> {
        if !on {
            let w = self.0.lock().unwrap().wat.remove(usr);
            if let Some(w) = w {
                for t in &w.tasks {
                    t.abort();
                }
                gs(sh, delete_op(&w.key));
            }
            self.fs(usr).send_replace(None);
            return Ok(());
        }
        let (chan, gld, _) = sh.vo.info().ok_or_else(|| anyhow::anyhow!("join a voice channel first"))?;
        let key = key_of(gld.as_deref(), &chan, usr);
        self.0.lock().unwrap().wat.insert(usr.to_string(), Wat { key: key.clone(), ..Default::default() });
        gs(sh, watch_op(&key));
        Ok(())
    }

    pub fn on(&self, sh: &Sh, t: &str, d: &Value) {
        let Some(key) = d["stream_key"].as_str() else { return };
        let me = sh.me.lock().unwrap().clone();
        let Some(uid) = key_uid(key).map(str::to_string) else { return };
        let mine = uid == me;
        match t {
            "STREAM_CREATE" => {
                let rtc = d["rtc_server_id"].as_str().map(str::to_string).or_else(|| d["rtc_server_id"].as_u64().map(|x| x.to_string()));
                let mut i = self.0.lock().unwrap();
                if mine {
                    if let Some(o) = &mut i.own {
                        o.key = Some(key.to_string());
                        o.rtc = rtc;
                    }
                    drop(i);
                    gs(sh, pause_op(key, false));
                } else if let Some(w) = i.wat.get_mut(&uid) {
                    w.rtc = rtc;
                }
            }
            "STREAM_SERVER_UPDATE" => {
                let tok = d["token"].as_str().map(str::to_string);
                let ep = d["endpoint"].as_str().map(str::to_string);
                {
                    let mut i = self.0.lock().unwrap();
                    if mine {
                        if let Some(o) = &mut i.own {
                            o.tok = tok;
                            o.ep = ep;
                        }
                    } else if let Some(w) = i.wat.get_mut(&uid) {
                        w.tok = tok;
                        w.ep = ep;
                    }
                }
                self.up(sh, &uid, mine);
            }
            "STREAM_DELETE" => {
                if mine {
                    let o = self.0.lock().unwrap().own.take();
                    if let Some(o) = o {
                        for t in &o.tasks {
                            t.abort();
                        }
                    }
                    sh.vo.live(false);
                    self.fs(&me).send_replace(None);
                } else {
                    let w = self.0.lock().unwrap().wat.remove(&uid);
                    if let Some(w) = w {
                        for t in &w.tasks {
                            t.abort();
                        }
                    }
                    self.fs(&uid).send_replace(None);
                }
            }
            _ => {}
        }
    }

    fn up(&self, sh: &Sh, uid: &str, mine: bool) {
        let Some((chan, _, sess)) = sh.vo.info() else { return };
        let me = sh.me.lock().unwrap().clone();
        let mut i = self.0.lock().unwrap();
        let (rtc, tok, ep, has) = if mine {
            let Some(o) = &i.own else { return };
            (o.rtc.clone(), o.tok.clone(), o.ep.clone(), o.vc.is_some())
        } else {
            let Some(w) = i.wat.get(uid) else { return };
            (w.rtc.clone(), w.tok.clone(), w.ep.clone(), w.vc.is_some())
        };
        let (Some(rtc), Some(tok), Some(ep), Some(sess)) = (rtc, tok, ep, sess) else { return };
        let _ = chan;
        let cfg = Cfg { endpoint: ep, token: tok, session: sess, server: rtc.clone(), chan: rtc, user: me.clone() };
        let (vt, vr) = unbounded_channel::<VEv>();
        let (st, sr) = unbounded_channel::<SEv>();
        let (mt, mr) = unbounded_channel::<Media>();
        let mut tasks = vec![];
        let fs = if mine { self.fs_in(&mut i, &me) } else { self.fs_in(&mut i, uid) };
        let vc;
        if mine {
            if has {
                return;
            }
            let o = i.own.as_mut().unwrap();
            let Some(lq) = o.lq else { return };
            let Some((au, ar)) = o.pre.take() else { return };
            let e = o.keyer.clone().unwrap();
            e.key();
            let q = Q { w: lq.w & !1, h: lq.h & !1, fps: lq.fps, kbps: lq.kbps };
            vc = Vc::start_stream(cfg, sh.vo.eng.clone(), vt, Sio { media: mr, ev: st, q: Some(q), cod: Codec::H264 });
            tasks.push(tokio::spawn(fwd(au, ar, mt, fs, e.clone())));
            tasks.push(tokio::spawn(own_ev(sr, e)));
        } else {
            if has {
                return;
            }
            let q = Sio { media: mr, ev: st, q: None, cod: Codec::H264 };
            vc = Vc::start_stream(cfg, sh.vo.eng.clone(), vt, q);
            tasks.push(tokio::spawn(view_ev(sr, fs, mt)));
        }
        tasks.push(tokio::spawn(vc_ev(vr, sh.vo.live_handle(), mine)));
        if mine {
            let o = i.own.as_mut().unwrap();
            o.vc = Some(vc);
            o.tasks.extend(tasks);
        } else if let Some(w) = i.wat.get_mut(uid) {
            w.vc = Some(vc);
            w.tasks.extend(tasks);
        }
    }

    fn fs_in(&self, i: &mut In, uid: &str) -> Fs {
        i.fr.entry(uid.to_string()).or_insert_with(|| Arc::new(watch::channel(None).0)).clone()
    }
}

async fn vc_ev(mut r: UnboundedReceiver<VEv>, live: crate::vc::Live, mine: bool) {
    while let Some(e) = r.recv().await {
        if let VEv::Call(c) = e
            && mine
        {
            live.set(c.up);
        }
    }
}

async fn fwd(mut au: UnboundedReceiver<Au>, mut ar: Option<UnboundedReceiver<aud::Op>>, mt: tokio::sync::mpsc::UnboundedSender<Media>, fs: Fs, kr: Keyer) {
    let prev = {
        let f = fs.clone();
        Dec::new(Cod::H264, false, move |fr| {
            f.send_replace(Some(Arc::new(fr)));
        })
        .ok()
    };
    let mut got = false;
    let mut pg = false;
    loop {
        tokio::select! {
            a = au.recv() => {
                let Some(a) = a else { break };
                got |= a.key;
                if !got {
                    continue;
                }
                if let Some(p) = &prev {
                    if fs.receiver_count() == 0 {
                        pg = false;
                    } else if pg || a.key {
                        pg = true;
                        p.push(a.data.clone(), Some(a.pts_us));
                    } else {
                        kr.key();
                    }
                }
                if mt.send(Media::Video { data: a.data, pts_us: a.pts_us }).is_err() {
                    break;
                }
            }
            o = async { match &mut ar { Some(r) => r.recv().await, None => std::future::pending().await } } => {
                let Some(o) = o else { ar = None; continue };
                if got && mt.send(Media::Audio { data: o.data, pts_us: o.pts_us }).is_err() {
                    break;
                }
            }
        }
    }
}

async fn own_ev(mut r: UnboundedReceiver<SEv>, e: Keyer) {
    while let Some(ev) = r.recv().await {
        if let SEv::Key = ev {
            e.key();
        }
    }
}

async fn view_ev(mut r: UnboundedReceiver<SEv>, fs: Fs, _keep: tokio::sync::mpsc::UnboundedSender<Media>) {
    let mut dec: Option<(Codec, Dec)> = None;
    let mut need = true;
    while let Some(ev) = r.recv().await {
        let SEv::Frame { cod: c, data, key, .. } = ev else { continue };
        if dec.as_ref().is_none_or(|(k, _)| *k != c) {
            let f = fs.clone();
            match Dec::new(cod(c), false, move |fr| {
                f.send_replace(Some(Arc::new(fr)));
            }) {
                Ok(d) => dec = Some((c, d)),
                Err(e) => {
                    tracing::warn!("decoder: {e}");
                    continue;
                }
            }
            need = true;
        }
        if need && !key {
            continue;
        }
        need = false;
        if let Some((_, d)) = &dec {
            d.push(data, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_ops() {
        assert_eq!(key_of(Some("1"), "2", "3"), "guild:1:2:3");
        assert_eq!(key_of(None, "2", "3"), "call:2:3");
        assert_eq!(key_uid("guild:1:2:3"), Some("3"));
        let v = create_op(Some("1"), "2");
        assert_eq!(v["op"], 18);
        assert_eq!(v["d"]["type"], "guild");
        let v = create_op(None, "2");
        assert_eq!(v["d"]["type"], "call");
        assert!(v["d"]["guild_id"].is_null());
        assert_eq!(pause_op("k", false)["op"], 22);
        assert_eq!(delete_op("k")["op"], 19);
        assert_eq!(watch_op("k")["op"], 20);
    }

    fn mk() -> (Arc<Sh>, tokio::sync::mpsc::UnboundedReceiver<String>) {
        let dc = crate::Dc::new(1, "t".into());
        let sh = dc.0.clone();
        *sh.me.lock().unwrap() = "100".into();
        let (t, r) = unbounded_channel();
        *sh.gt.lock().unwrap() = Some(t);
        (sh, r)
    }

    #[test]
    fn create_unpauses_and_watch_flow() {
        let (sh, mut r) = mk();
        sh.st.0.lock().unwrap().own = Some(Own::default());
        sh.st.on(&sh, "STREAM_CREATE", &json!({"stream_key": "guild:1:2:100", "rtc_server_id": "555", "paused": true}));
        let m: Value = serde_json::from_str(&r.try_recv().unwrap()).unwrap();
        assert_eq!(m["op"], 22);
        assert_eq!(m["d"]["paused"], false);
        let i = sh.st.0.lock().unwrap();
        let o = i.own.as_ref().unwrap();
        assert_eq!(o.key.as_deref(), Some("guild:1:2:100"));
        assert_eq!(o.rtc.as_deref(), Some("555"));
        drop(i);
        sh.st.on(&sh, "STREAM_SERVER_UPDATE", &json!({"stream_key": "guild:1:2:100", "token": "tk", "endpoint": "x.discord.media:443"}));
        assert_eq!(sh.st.0.lock().unwrap().own.as_ref().unwrap().tok.as_deref(), Some("tk"));
        sh.st.0.lock().unwrap().wat.insert("200".into(), Wat { key: "guild:1:2:200".into(), ..Default::default() });
        sh.st.on(&sh, "STREAM_CREATE", &json!({"stream_key": "guild:1:2:200", "rtc_server_id": 777}));
        assert_eq!(sh.st.0.lock().unwrap().wat["200"].rtc.as_deref(), Some("777"));
        let rx = sh.st.frames("200");
        sh.st.on(&sh, "STREAM_DELETE", &json!({"stream_key": "guild:1:2:200"}));
        assert!(sh.st.0.lock().unwrap().wat.is_empty());
        assert!(rx.borrow().is_none());
        sh.st.on(&sh, "STREAM_DELETE", &json!({"stream_key": "guild:1:2:100"}));
        assert!(sh.st.0.lock().unwrap().own.is_none());
    }

    #[tokio::test]
    async fn watch_off_sends_delete() {
        let (sh, mut r) = mk();
        sh.st.0.lock().unwrap().wat.insert("200".into(), Wat { key: "call:2:200".into(), ..Default::default() });
        sh.st.watch(&sh, "200", false).unwrap();
        let m: Value = serde_json::from_str(&r.try_recv().unwrap()).unwrap();
        assert_eq!(m["op"], 19);
        assert_eq!(m["d"]["stream_key"], "call:2:200");
        assert!(sh.st.watch(&sh, "200", true).is_err());
    }
}
