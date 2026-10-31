use parking_lot::Mutex;
use crate::{Sh, map::{rid, s, usr}};
use cord_core::*;
use cord_rtc::{
    Eng,
    dcv::{Cfg, VEv, Vc},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::mpsc::unbounded_channel;

#[derive(Default)]
pub struct In {
    chan: Option<String>,
    gld: Option<String>,
    sess: Option<String>,
    tok: Option<String>,
    ep: Option<String>,
    mute: bool,
    deaf: bool,
    ssrc: HashMap<String, u32>,
    vols: HashMap<String, f32>,
    cs: CallSt,
    tx: Option<Tx>,
    vc: Option<Vc>,
    gen_: u64,
}

pub struct Vo {
    pub eng: Eng,
    pub i: Arc<Mutex<In>>,
}

impl Default for Vo {
    fn default() -> Self {
        Vo { eng: Eng::new(), i: Default::default() }
    }
}

#[derive(Clone)]
pub struct Live(Arc<Mutex<In>>);

impl Live {
    pub fn set(&self, on: bool) {
        let mut i = self.0.lock();
        if i.cs.live != on {
            i.cs.live = on;
            if let Some(t) = &i.tx {
                let _ = t.send(Ev::Call(i.cs.clone()));
            }
        }
    }
}

impl Vo {
    pub fn live_handle(&self) -> Live {
        Live(self.i.clone())
    }

    pub fn live(&self, on: bool) {
        self.live_handle().set(on);
    }

    pub fn info(&self) -> Option<(String, Option<String>, Option<String>)> {
        let i = self.i.lock();
        Some((i.chan.clone()?, i.gld.clone(), i.sess.clone()))
    }

    pub fn set_tx(&self, tx: Tx) {
        self.i.lock().tx = Some(tx);
    }

    pub fn begin(&self, chan: &Rid, gld: Option<String>) -> Res<(Option<String>, bool, bool)> {
        self.eng.start()?;
        let mut i = self.i.lock();
        i.vc = None;
        i.chan = Some(chan.raw.to_string());
        i.gld = gld.clone();
        i.sess = None;
        i.tok = None;
        i.ep = None;
        i.ssrc.clear();
        i.gen_ += 1;
        i.cs = CallSt { chan: Some(chan.clone()), mute: i.mute, deaf: i.deaf, ..Default::default() };
        if let Some(t) = &i.tx {
            let _ = t.send(Ev::Call(i.cs.clone()));
        }
        Ok((gld, i.mute, i.deaf))
    }

    pub fn end(&self) -> (bool, bool) {
        let mut i = self.i.lock();
        i.vc = None;
        i.chan = None;
        i.gld = None;
        i.sess = None;
        i.tok = None;
        i.ep = None;
        i.ssrc.clear();
        i.gen_ += 1;
        self.eng.stop();
        i.cs = CallSt { mute: i.mute, deaf: i.deaf, ..Default::default() };
        if let Some(t) = &i.tx {
            let _ = t.send(Ev::Call(i.cs.clone()));
        }
        (i.mute, i.deaf)
    }

    pub fn flags(&self, mute: bool, deaf: bool) -> Option<String> {
        let mut i = self.i.lock();
        i.mute = mute;
        i.deaf = deaf;
        self.eng.mute(mute || deaf);
        self.eng.deaf(deaf);
        i.cs.mute = mute;
        i.cs.deaf = deaf;
        if let Some(t) = &i.tx {
            let _ = t.send(Ev::Call(i.cs.clone()));
        }
        i.chan.clone()
    }

    pub fn gld(&self) -> Option<String> {
        self.i.lock().gld.clone()
    }

    pub fn vol(&self, uid: &str, v: f32) {
        let mut i = self.i.lock();
        i.vols.insert(uid.to_string(), v);
        if let Some(&s) = i.ssrc.get(uid) {
            self.eng.set_vol(s, v);
        }
    }
}

pub fn op4(chan: Option<&str>, gld: Option<&str>, mute: bool, deaf: bool) -> Value {
    json!({"op": 4, "d": {"guild_id": gld, "channel_id": chan, "self_mute": mute, "self_deaf": deaf, "self_video": false}})
}

fn vs_of(sh: &Sh, d: &Value, gld: Option<&str>) -> Vs {
    let a = sh.a;
    let u = if let Some(u) = d.get("member").and_then(|m| m.get("user")) {
        usr(a, u)
    } else {
        let id = s(d, "user_id").unwrap_or("");
        sh.us.lock().get(id).cloned().unwrap_or(Usr { id: Some(rid(a, id)), ..Default::default() })
    };
    let g = gld.or_else(|| s(d, "guild_id"));
    let b = |k: &str| d[k].as_bool().unwrap_or(false);
    Vs {
        usr: u,
        chan: s(d, "channel_id").map(|c| rid(a, c)),
        gld: g.map(|g| rid(a, g)),
        mute: b("self_mute"),
        deaf: b("self_deaf"),
        smute: b("mute"),
        sdeaf: b("deaf"),
        stream: b("self_stream"),
        vid: b("self_video"),
    }
}

pub fn guild_vs(sh: &Sh, tx: &Tx, gid: &str, g: &Value) {
    if let Some(a) = g["voice_states"].as_array() {
        for d in a {
            let _ = tx.send(Ev::Vs(vs_of(sh, d, Some(gid))));
        }
    }
}

pub fn call(sh: &Sh, tx: &Tx, d: &Value) {
    let ch = s(d, "channel_id");
    if let Some(a) = d["voice_states"].as_array() {
        for v in a {
            let mut v = v.clone();
            if v.get("channel_id").is_none() {
                v["channel_id"] = json!(ch);
            }
            let _ = tx.send(Ev::Vs(vs_of(sh, &v, None)));
        }
    }
}


pub fn vsu(sh: &Sh, tx: &Tx, d: &Value) {
    let _ = tx.send(Ev::Vs(vs_of(sh, d, None)));
    let me = sh.me.lock().clone();
    if s(d, "user_id") != Some(me.as_str()) {
        return;
    }
    let mut i = sh.vo.i.lock();
    let Some(want) = i.chan.clone() else { return };
    match s(d, "channel_id") {
        None => {
            drop(i);
            sh.st.stop_all(sh);
            sh.vo.end();
        }
        Some(c) => {
            if c != want {
                i.chan = Some(c.to_string());
                i.cs.chan = Some(rid(sh.a, c));
            }
            i.sess = s(d, "session_id").map(str::to_string);
            up(sh, &mut i, &me);
        }
    }
}

pub fn vsv(sh: &Sh, d: &Value) {
    let me = sh.me.lock().clone();
    let mut i = sh.vo.i.lock();
    if i.chan.is_none() {
        return;
    }
    let tok = s(d, "token").map(str::to_string);
    let ep = s(d, "endpoint").map(str::to_string);
    if ep.is_none() {
        i.vc = None;
        return;
    }
    if i.tok != tok || i.ep != ep {
        i.vc = None;
    }
    i.tok = tok;
    i.ep = ep;
    up(sh, &mut i, &me);
}

fn up(sh: &Sh, i: &mut In, me: &str) {
    if i.vc.is_some() {
        return;
    }
    let (Some(ch), Some(se), Some(tk), Some(ep)) = (i.chan.clone(), i.sess.clone(), i.tok.clone(), i.ep.clone()) else { return };
    let Some(tx) = i.tx.clone() else { return };
    let cfg = Cfg { endpoint: ep, token: tk, session: se, server: i.gld.clone().unwrap_or_else(|| ch.clone()), chan: ch.clone(), user: me.to_string() };
    let (vt, mut vr) = unbounded_channel::<VEv>();
    i.vc = Some(Vc::start(cfg, sh.vo.eng.clone(), vt));
    let st = sh.vo.i.clone();
    let eng = sh.vo.eng.clone();
    let a = sh.a;
    let g = i.gen_;
    tokio::spawn(async move {
        while let Some(e) = vr.recv().await {
            let mut i = st.lock();
            if i.gen_ != g {
                return;
            }
            match e {
                VEv::Call(c) => {
                    i.cs.up = c.up;
                    i.cs.ping = c.ping;
                    i.cs.e2ee = c.e2ee;
                    i.cs.err = c.err;
                    let _ = tx.send(Ev::Call(i.cs.clone()));
                }
                VEv::Speak { ssrc, uid, on } => {
                    if i.ssrc.insert(uid.clone(), ssrc) != Some(ssrc)
                        && let Some(&v) = i.vols.get(&uid)
                    {
                        eng.set_vol(ssrc, v);
                    }
                    if let Some(ch) = i.chan.clone() {
                        let _ = tx.send(Ev::Speak { chan: rid(a, &ch), usr: rid(a, &uid), on });
                    }
                }
                VEv::Gone { uid } => {
                    i.ssrc.remove(&uid);
                }
            }
        }
    });
}
