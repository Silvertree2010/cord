use crate::{
    dave::{Dv, MAXV, Out},
    eng::{Eng, Pkt},
    mix::FR,
    rtp::{self, Cry, Hdr, Mode},
    vrtp::{self, Codec, Fb, Q, Unp},
};
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    net::UdpSocket,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::{Interval, interval, sleep, timeout},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Clone, Debug)]
pub struct Cfg {
    pub endpoint: String,
    pub token: String,
    pub session: String,
    pub server: String,
    pub chan: String,
    pub user: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cs {
    pub up: bool,
    pub ping: Option<u32>,
    pub e2ee: bool,
    pub err: Option<String>,
}

#[derive(Clone, Debug)]
pub enum VEv {
    Call(Cs),
    Speak { ssrc: u32, uid: String, on: bool },
    Gone { uid: String },
}

#[derive(Clone, Debug)]
pub enum Media {
    Video { data: Vec<u8>, pts_us: u64 },
    Audio { data: Vec<u8>, pts_us: u64 },
}

#[derive(Clone, Debug)]
pub enum SEv {
    Key,
    Frame { uid: String, cod: Codec, data: Vec<u8>, key: bool },
    Pli,
}

pub struct Sio {
    pub media: UnboundedReceiver<Media>,
    pub ev: UnboundedSender<SEv>,
    pub q: Option<Q>,
    pub cod: Codec,
}

struct Sent {
    seq: u16,
    ts: u32,
    marker: bool,
    pl: Vec<u8>,
}

struct St {
    ev: UnboundedSender<SEv>,
    q: Option<Q>,
    cod: Codec,
    vss: u32,
    rss: u32,
    vseq: u16,
    xseq: u16,
    ring: VecDeque<Sent>,
    vusers: HashMap<u32, String>,
    unp: HashMap<String, Unp>,
    vmain: HashMap<String, u32>,
    pli: Option<Instant>,
}

pub struct Vc {
    stop: Option<oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl Vc {
    pub fn start(cfg: Cfg, eng: Eng, ev: UnboundedSender<VEv>) -> Vc {
        Vc::start_with(cfg, eng, ev, None)
    }

    pub fn start_stream(cfg: Cfg, eng: Eng, ev: UnboundedSender<VEv>, sio: Sio) -> Vc {
        Vc::start_with(cfg, eng, ev, Some(sio))
    }

    fn start_with(cfg: Cfg, eng: Eng, ev: UnboundedSender<VEv>, sio: Option<Sio>) -> Vc {
        let (st, sr) = oneshot::channel();
        let task = tokio::spawn(run(cfg, eng, ev, sio, sr));
        Vc { stop: Some(st), task: Some(task) }
    }

    pub async fn stop(mut self) {
        if let Some(s) = self.stop.take() {
            let _ = s.send(());
        }
        if let Some(t) = self.task.take() {
            let _ = timeout(Duration::from_secs(3), t).await;
        }
    }
}

impl Drop for Vc {
    fn drop(&mut self) {
        if let Some(s) = self.stop.take() {
            let _ = s.send(());
        }
    }
}

enum Fin {
    Stop,
    Re,
    Fresh,
    Fatal(String),
}

pub struct Conn {
    cfg: Cfg,
    eng: Eng,
    ev: UnboundedSender<VEv>,
    seq: i64,
    ssrc: u32,
    udp: Option<Arc<UdpSocket>>,
    cry: Option<Cry>,
    rseq: u16,
    ts: u32,
    users: HashMap<u32, String>,
    dv: Option<Dv>,
    speaking: bool,
    cs: Cs,
    ready: bool,
    acked: bool,
    resume: bool,
    fatal: Option<String>,
    sent: u64,
    stm: Option<St>,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn txt(v: Value) -> Message {
    Message::Text(v.to_string().into())
}

fn num(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

impl Conn {
    pub fn new(cfg: Cfg, eng: Eng, ev: UnboundedSender<VEv>) -> Conn {
        Conn::with(cfg, eng, ev, None)
    }

    fn with(cfg: Cfg, eng: Eng, ev: UnboundedSender<VEv>, sio: Option<(UnboundedSender<SEv>, Option<Q>, Codec)>) -> Conn {
        Conn {
            stm: sio.map(|(ev, q, cod)| St { ev, q, cod, vss: 0, rss: 0, vseq: rand::random(), xseq: rand::random(), ring: VecDeque::new(), vusers: HashMap::new(), unp: HashMap::new(), vmain: HashMap::new(), pli: None }),
            cfg,
            eng,
            ev,
            seq: -1,
            ssrc: 0,
            udp: None,
            cry: None,
            rseq: rand::random(),
            ts: rand::random(),
            users: HashMap::new(),
            dv: None,
            speaking: false,
            cs: Cs::default(),
            ready: false,
            acked: true,
            resume: false,
            fatal: None,
            sent: 0,
        }
    }

    fn emit(&mut self, f: impl FnOnce(&mut Cs)) {
        let mut n = self.cs.clone();
        f(&mut n);
        if n != self.cs {
            self.cs = n.clone();
            let _ = self.ev.send(VEv::Call(n));
        }
    }

    fn e2ee(&mut self) {
        let e = self.dv.as_ref().is_some_and(Dv::ready);
        self.emit(|c| c.e2ee = e);
    }

    fn me(&self) -> u64 {
        self.cfg.user.parse().unwrap_or(0)
    }

    fn outs(&self, v: Vec<Out>) -> Vec<Message> {
        v.into_iter()
            .map(|o| match o {
                Out::Bin(b) => Message::Binary(b.into()),
                Out::Json(j) => txt(j),
            })
            .collect()
    }

    fn hello(&mut self) -> Message {
        let c = &self.cfg;
        if self.resume {
            txt(json!({"op": 7, "d": {"server_id": c.server, "session_id": c.session, "token": c.token, "seq_ack": self.seq}}))
        } else {
            let mut d = json!({"server_id": c.server, "user_id": c.user, "session_id": c.session, "token": c.token, "max_dave_protocol_version": MAXV});
            if self.stm.is_some() {
                d["video"] = json!(true);
                d["streams"] = vrtp::ident_streams();
            }
            txt(json!({"op": 0, "d": d}))
        }
    }

    fn hb(&mut self) -> Message {
        self.acked = false;
        self.sent = now();
        txt(json!({"op": 3, "d": {"t": self.sent, "seq_ack": self.seq}}))
    }

    async fn disc(&mut self, ip: &str, port: u16) -> Option<(String, u16)> {
        let u = UdpSocket::bind("0.0.0.0:0").await.ok()?;
        u.connect((ip, port)).await.ok()?;
        let req = rtp::disc_req(self.ssrc);
        let mut b = [0u8; 128];
        for _ in 0..5 {
            u.send(&req).await.ok()?;
            if let Ok(Ok(n)) = timeout(Duration::from_secs(1), u.recv(&mut b)).await
                && let Some(r) = rtp::disc_res(&b[..n])
            {
                self.udp = Some(Arc::new(u));
                return Some(r);
            }
        }
        None
    }

    pub async fn text(&mut self, v: &Value) -> Vec<Message> {
        let mut o = Vec::new();
        if let Some(s) = v["seq"].as_i64() {
            self.seq = s;
        }
        let d = &v["d"];
        let op = v["op"].as_u64().unwrap_or(999);
        match op {
            2 => {
                self.ssrc = d["ssrc"].as_u64().unwrap_or(0) as u32;
                let ip = d["ip"].as_str().unwrap_or("").to_string();
                let port = d["port"].as_u64().unwrap_or(0) as u16;
                let modes: Vec<String> = d["modes"].as_array().map(|a| a.iter().filter_map(|m| m.as_str().map(String::from)).collect()).unwrap_or_default();
                let Some(m) = Mode::pick(&modes) else {
                    self.fatal = Some("no supported encryption mode".into());
                    return o;
                };
                let Some((addr, p)) = self.disc(&ip, port).await else {
                    self.fatal = Some("ip discovery failed".into());
                    return o;
                };
                if self.dv.is_none() {
                    self.dv = Dv::new(MAXV, self.me(), self.cfg.chan.parse().unwrap_or(0));
                }
                let mut sel = json!({"protocol": "udp", "data": {"address": addr, "port": p, "mode": m.name()}});
                if let Some(t) = &mut self.stm {
                    sel["codecs"] = vrtp::codecs();
                    let st = d["streams"].as_array().and_then(|a| a.first());
                    t.vss = st.and_then(|x| x["ssrc"].as_u64()).map(|x| x as u32).unwrap_or(self.ssrc.wrapping_add(1));
                    t.rss = st.and_then(|x| x["rtx_ssrc"].as_u64()).map(|x| x as u32).unwrap_or(self.ssrc.wrapping_add(2));
                }
                o.push(txt(json!({"op": 1, "d": sel})));
            }
            4 => {
                let key: Vec<u8> = d["secret_key"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64().map(|x| x as u8)).collect()).unwrap_or_default();
                let m = d["mode"].as_str().and_then(Mode::from_name);
                self.cry = m.and_then(|m| Cry::new(m, &key));
                if self.cry.is_none() {
                    self.fatal = Some("bad session description".into());
                    return o;
                }
                let dv = d["dave_protocol_version"].as_u64().unwrap_or(0) as u16;
                if dv == 0 {
                    self.dv = None;
                } else if self.dv.as_ref().map(Dv::ver) != Some(dv) {
                    self.dv = Dv::new(dv, self.me(), self.cfg.chan.parse().unwrap_or(0));
                }
                self.ready = true;
                if self.stm.is_none() {
                    self.eng.reset();
                }
                self.emit(|c| {
                    c.up = true;
                    c.err = None;
                });
                self.e2ee();
                if let Some(t) = &self.stm {
                    o.push(txt(vrtp::video_op(self.ssrc, t.vss, t.rss, t.q)));
                }
            }
            9 => {
                self.ready = true;
                self.emit(|c| {
                    c.up = true;
                    c.err = None;
                });
            }
            6 => {
                self.acked = true;
                let t = d["t"].as_u64().unwrap_or(self.sent);
                let p = now().saturating_sub(t).min(60000) as u32;
                self.emit(|c| c.ping = Some(p));
            }
            5 => {
                let (Some(s), Some(u)) = (d["ssrc"].as_u64(), num(&d["user_id"])) else { return o };
                let uid = u.to_string();
                self.users.insert(s as u32, uid.clone());
                let on = d["speaking"].as_u64().unwrap_or(0) != 0;
                let _ = self.ev.send(VEv::Speak { ssrc: s as u32, uid, on });
            }
            12 => {
                if let (Some(t), Some(u)) = (&mut self.stm, num(&d["user_id"])) {
                    for k in ["video_ssrc", "rtx_ssrc"] {
                        if let Some(s) = d[k].as_u64().filter(|&s| s != 0) {
                            t.vusers.insert(s as u32, u.to_string());
                            if k == "video_ssrc" {
                                t.vmain.insert(u.to_string(), s as u32);
                            }
                        }
                    }
                    if let Some(a) = d["streams"].as_array() {
                        for x in a {
                            for k in ["ssrc", "rtx_ssrc"] {
                                if let Some(s) = x[k].as_u64().filter(|&s| s != 0) {
                                    t.vusers.insert(s as u32, u.to_string());
                                    if k == "ssrc" {
                                        t.vmain.entry(u.to_string()).or_insert(s as u32);
                                    }
                                }
                            }
                        }
                    }
                }
                if let (Some(s), Some(u)) = (d["audio_ssrc"].as_u64(), num(&d["user_id"]))
                    && s != 0
                {
                    self.users.insert(s as u32, u.to_string());
                    let _ = self.ev.send(VEv::Speak { ssrc: s as u32, uid: u.to_string(), on: false });
                }
            }
            13 => {
                if let Some(u) = num(&d["user_id"]) {
                    let uid = u.to_string();
                    if let Some(t) = &mut self.stm {
                        t.vusers.retain(|_, x| *x != uid);
                        t.vmain.remove(&uid);
                        t.unp.remove(&uid);
                    }
                    let gone: Vec<u32> = self.users.iter().filter(|(_, x)| **x == uid).map(|(s, _)| *s).collect();
                    for s in gone {
                        self.users.remove(&s);
                        self.eng.drop_ssrc(s);
                    }
                    let _ = self.ev.send(VEv::Gone { uid });
                }
            }
            21 | 22 | 24 => {
                if let Some(dv) = &mut self.dv {
                    let r = dv.json(op, d);
                    o.extend(self.outs(r));
                    self.e2ee();
                }
            }
            _ => {}
        }
        o
    }

    pub fn bin(&mut self, b: &[u8]) -> Vec<Message> {
        if b.len() < 3 {
            return vec![];
        }
        self.seq = u16::from_be_bytes([b[0], b[1]]) as i64;
        let Some(dv) = &mut self.dv else { return vec![] };
        let r = dv.bin(b[2], &b[3..]);
        let o = self.outs(r);
        self.e2ee();
        o
    }

    pub fn inbound(&mut self, b: &[u8]) -> Vec<Vec<u8>> {
        let mut out = vec![];
        if b.len() < 12 {
            return out;
        }
        if (192..=223).contains(&b[1]) {
            self.rtcp(b, &mut out);
            return out;
        }
        let Some(c) = &self.cry else { return out };
        let Some((h, mut pl)) = c.open(b) else { return out };
        if h.pt == rtp::PT & 0x7f {
            if let Some(dv) = &mut self.dv {
                let Some(uid) = self.users.get(&h.ssrc).and_then(|u| u.parse::<u64>().ok()) else { return out };
                match dv.dec(uid, &pl) {
                    Some(p) => pl = p,
                    None => return out,
                }
            }
            self.eng.rx(h.ssrc, h.seq, pl);
            return out;
        }
        let Some(t) = &mut self.stm else { return out };
        let Some((cod, rtx)) = Codec::by_pt(h.pt) else { return out };
        let (mut seq, ssrc) = (h.seq, h.ssrc);
        if rtx {
            if pl.len() < 2 {
                return out;
            }
            seq = u16::from_be_bytes([pl[0], pl[1]]);
            pl.drain(..2);
        }
        let Some(uid) = t.vusers.get(&ssrc).cloned() else { return out };
        let media = t.vmain.get(&uid).copied().unwrap_or(ssrc);
        let marker = b[1] & 0x80 != 0;
        let r = t.unp.entry(uid.clone()).or_insert_with(|| Unp::new(cod)).push(seq, h.ts, marker, &pl);
        let mut want = r.lost;
        if let Some((au, k)) = r.au {
            let au = match (&mut self.dv, uid.parse::<u64>()) {
                (Some(dv), Ok(u)) => dv.decv(u, &au),
                _ => Some(au),
            };
            match au {
                Some(data) => {
                    let _ = t.ev.send(SEv::Frame { uid, cod, data, key: k });
                }
                None => want = true,
            }
        }
        if want && t.pli.is_none_or(|p| p.elapsed() > Duration::from_millis(700)) {
            t.pli = Some(Instant::now());
            let p = vrtp::pli(self.ssrc, media);
            if let Some(c) = &mut self.cry {
                out.push(c.seal_rtcp(&p));
            }
            let _ = t.ev.send(SEv::Pli);
        }
        out
    }

    fn rtcp(&mut self, b: &[u8], out: &mut Vec<Vec<u8>>) {
        let Some(c) = &self.cry else { return };
        let Some(p) = c.open_rtcp(b) else { return };
        let Some(t) = &mut self.stm else { return };
        for f in vrtp::parse_fb(&p) {
            match f {
                Fb::Pli(m) | Fb::Fir(m) if m == t.vss => {
                    let _ = t.ev.send(SEv::Key);
                }
                Fb::Nack(m, seqs) if m == t.vss => {
                    for s in seqs {
                        let Some(x) = t.ring.iter().find(|x| x.seq == s) else { continue };
                        let mut pl = s.to_be_bytes().to_vec();
                        pl.extend_from_slice(&x.pl);
                        t.xseq = t.xseq.wrapping_add(1);
                        let h = Hdr::pack_ex(t.cod.rtx(), x.marker, t.xseq, x.ts, t.rss);
                        if let Some(c) = &mut self.cry {
                            out.push(c.seal(&h, &pl));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn vwire(&mut self, au: &[u8], pts_us: u64) -> Vec<Vec<u8>> {
        let mut o = vec![];
        let (Some(t), Some(c)) = (&mut self.stm, &mut self.cry) else { return o };
        let ts = vrtp::ts90(pts_us);
        let au = match t.cod {
            Codec::H264 => vrtp::norm4(au),
            _ => au.to_vec(),
        };
        let au = match &mut self.dv {
            Some(d) => d.encv(t.cod, &au),
            None => au,
        };
        let ps = match t.cod {
            Codec::H264 => vrtp::pack_h264(&au),
            _ => return o,
        };
        let n = ps.len();
        for (i, pl) in ps.into_iter().enumerate() {
            t.vseq = t.vseq.wrapping_add(1);
            let marker = i + 1 == n;
            let h = Hdr::pack_ex(t.cod.pt(), marker, t.vseq, ts, t.vss);
            o.push(c.seal(&h, &pl));
            t.ring.push_back(Sent { seq: t.vseq, ts, marker, pl });
            if t.ring.len() > 1024 {
                t.ring.pop_front();
            }
        }
        o
    }

    pub fn wire(&mut self, p: &Pkt) -> (Option<Vec<u8>>, Vec<Message>) {
        let mut m = Vec::new();
        if self.cry.is_none() {
            return (None, m);
        }
        if !self.speaking {
            self.speaking = true;
            m.push(txt(json!({"op": 5, "d": {"speaking": 1, "delay": 0, "ssrc": self.ssrc}})));
            let _ = self.ev.send(VEv::Speak { ssrc: self.ssrc, uid: self.cfg.user.clone(), on: true });
        }
        self.rseq = self.rseq.wrapping_add(1);
        self.ts = self.ts.wrapping_add(FR as u32 * (p.skip + 1));
        let pl = match &mut self.dv {
            Some(d) => d.enc(&p.data),
            None => p.data.clone(),
        };
        let h = Hdr::pack(self.rseq, self.ts, self.ssrc);
        let pk = self.cry.as_mut().map(|c| c.seal(&h, &pl));
        if p.last {
            self.speaking = false;
            m.push(txt(json!({"op": 5, "d": {"speaking": 0, "delay": 0, "ssrc": self.ssrc}})));
            let _ = self.ev.send(VEv::Speak { ssrc: self.ssrc, uid: self.cfg.user.clone(), on: false });
        }
        (pk, m)
    }
}

async fn tick(iv: &mut Option<Interval>) {
    match iv {
        Some(i) => {
            i.tick().await;
        }
        None => std::future::pending().await,
    }
}

async fn urecv(u: &Option<Arc<UdpSocket>>, b: &mut [u8]) -> std::io::Result<usize> {
    match u {
        Some(u) => u.recv(b).await,
        None => std::future::pending().await,
    }
}

async fn mrecv(m: &mut Option<UnboundedReceiver<Media>>) -> Option<Media> {
    match m {
        Some(r) => r.recv().await,
        None => std::future::pending().await,
    }
}

async fn sess(c: &mut Conn, rx: &mut UnboundedReceiver<Pkt>, md: &mut Option<UnboundedReceiver<Media>>, stop: &mut oneshot::Receiver<()>) -> Fin {
    let ep = c.cfg.endpoint.trim_start_matches("wss://").trim_end_matches('/').to_string();
    let ws = match connect_async(format!("wss://{ep}/?v=8")).await {
        Ok((w, _)) => w,
        Err(e) => {
            tracing::warn!("voice ws: {e}");
            return Fin::Re;
        }
    };
    let (mut wt, mut rd) = ws.split();
    let mut iv: Option<Interval> = None;
    let mut ub = vec![0u8; 2048];
    c.acked = true;
    loop {
        let udp = c.udp.clone();
        let mut out: Vec<Message> = Vec::new();
        tokio::select! {
            _ = &mut *stop => {
                let _ = wt.close().await;
                return Fin::Stop;
            }
            m = rd.next() => {
                match m {
                    None | Some(Err(_)) => return Fin::Re,
                    Some(Ok(Message::Text(t))) => {
                        let Ok(v) = serde_json::from_str::<Value>(&t) else { continue };
                        if v["op"].as_u64() == Some(8) {
                            let hi = v["d"]["heartbeat_interval"].as_f64().unwrap_or(13750.0);
                            iv = Some(interval(Duration::from_millis((hi as u64).clamp(500, 120_000))));
                            out.push(c.hello());
                        } else {
                            out.extend(c.text(&v).await);
                        }
                    }
                    Some(Ok(Message::Binary(b))) => out.extend(c.bin(&b)),
                    Some(Ok(Message::Close(f))) => {
                        let code = f.map(|f| u16::from(f.code)).unwrap_or(0);
                        return match code {
                            4006 if c.resume => Fin::Fresh,
                            4004 | 4006 | 4009 | 4011 | 4012 | 4014 | 4016 | 4017 | 4020 | 4021 | 4022 => Fin::Fatal(format!("voice closed {code}")),
                            _ => Fin::Re,
                        };
                    }
                    _ => {}
                }
            }
            r = urecv(&udp, &mut ub) => {
                if let Ok(n) = r {
                    let o = c.inbound(&ub[..n]);
                    if let Some(u) = &udp {
                        for p in o {
                            let _ = u.send(&p).await;
                        }
                    }
                }
            }
            Some(m) = mrecv(md) => {
                let Some(u) = &udp else { continue };
                match m {
                    Media::Video { data, pts_us } => {
                        for p in c.vwire(&data, pts_us) {
                            let _ = u.send(&p).await;
                        }
                    }
                    Media::Audio { data, .. } => {
                        let (pk, m) = c.wire(&Pkt { data, skip: 0, last: false });
                        out.extend(m);
                        if let Some(pk) = pk {
                            let _ = u.send(&pk).await;
                        }
                    }
                }
            }
            Some(p) = rx.recv() => {
                let (pk, m) = c.wire(&p);
                out.extend(m);
                if let (Some(pk), Some(u)) = (pk, &udp) {
                    let _ = u.send(&pk).await;
                }
            }
            _ = tick(&mut iv) => {
                if !c.acked {
                    return Fin::Re;
                }
                out.push(c.hb());
            }
        }
        if let Some(f) = c.fatal.take() {
            return Fin::Fatal(f);
        }
        for m in out {
            if wt.send(m).await.is_err() {
                return Fin::Re;
            }
        }
    }
}

async fn run(cfg: Cfg, eng: Eng, ev: UnboundedSender<VEv>, sio: Option<Sio>, mut stop: oneshot::Receiver<()>) {
    let (_keep, dummy) = tokio::sync::mpsc::unbounded_channel::<Pkt>();
    let is_st = sio.is_some();
    let mut rx = if is_st {
        dummy
    } else {
        match eng.tx_frames() {
            Some(r) => r,
            None => {
                let _ = ev.send(VEv::Call(Cs { err: Some("audio frames already taken".into()), ..Default::default() }));
                return;
            }
        }
    };
    let (mut md, sx) = match sio {
        Some(s) => (Some(s.media), Some((s.ev, s.q, s.cod))),
        None => (None, None),
    };
    let mut c = Conn::with(cfg, eng.clone(), ev.clone(), sx);
    let _ = ev.send(VEv::Call(Cs::default()));
    let mut n = 0u32;
    loop {
        c.resume = c.ready;
        let t0 = Instant::now();
        let r = sess(&mut c, &mut rx, &mut md, &mut stop).await;
        if t0.elapsed() > Duration::from_secs(30) {
            n = 0;
        }
        match r {
            Fin::Stop => break,
            Fin::Fatal(m) => {
                c.emit(|s| {
                    s.up = false;
                    s.err = Some(m);
                });
                break;
            }
            Fin::Fresh => {
                c.ready = false;
                c.cry = None;
                c.udp = None;
                c.seq = -1;
            }
            Fin::Re => {
                n += 1;
                if n > 8 {
                    c.emit(|s| {
                        s.up = false;
                        s.err = Some("voice connection lost".into());
                    });
                    break;
                }
            }
        }
        c.emit(|s| {
            s.up = false;
            s.ping = None;
        });
        let d = Duration::from_millis((500u64 << n.min(4)).min(8000));
        tokio::select! {
            _ = sleep(d) => {}
            _ = &mut stop => break,
        }
    }
    if !is_st {
        eng.reset();
        eng.give_frames(rx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mix::FS;
    use tokio::sync::mpsc::unbounded_channel;

    fn pair() -> (Conn, Conn, Eng, Eng) {
        let cfg = Cfg { endpoint: String::new(), token: "t".into(), session: "s".into(), server: "1".into(), chan: "2".into(), user: "3".into() };
        let (a, b) = (Eng::new(), Eng::new());
        let mut ca = Conn::new(cfg.clone(), a.clone(), unbounded_channel().0);
        let mut cb = Conn::new(cfg, b.clone(), unbounded_channel().0);
        ca.ssrc = 77;
        ca.cry = Cry::new(Mode::Xcha, &[5; 32]);
        cb.cry = Cry::new(Mode::Xcha, &[5; 32]);
        cb.users.insert(77, "9".into());
        (ca, cb, a, b)
    }

    #[test]
    fn wire_roundtrip() {
        let (mut ca, mut cb, a, b) = pair();
        a.ns(false);
        let mut rx = a.tx_frames().unwrap();
        let mut cap = a.cap();
        let tone: Vec<f32> = (0..FR * 8).map(|i| (i as f32 * 0.04).sin() * 0.4).collect();
        cap.push(&tone, 1);
        let mut n = 0;
        let mut tss = vec![];
        while let Ok(p) = rx.try_recv() {
            let (pk, m) = ca.wire(&p);
            if n == 0 {
                assert_eq!(m.len(), 1);
            }
            let pk = pk.unwrap();
            let (h, _) = Hdr::parse(&pk).unwrap();
            tss.push(h.ts);
            cb.inbound(&pk);
            n += 1;
        }
        assert_eq!(n, 8);
        assert!(tss.windows(2).all(|w| w[1].wrapping_sub(w[0]) == FR as u32));
        let mut o = vec![0.0f32; FS];
        let mut peak = 0.0f32;
        for _ in 0..8 {
            b.pull(&mut o);
            peak = peak.max(o.iter().fold(0.0, |a, &x| a.max(x.abs())));
        }
        assert!(peak > 0.1);
    }

    #[tokio::test]
    async fn text_ops() {
        let (mut ca, _, _, _) = pair();
        let (tx, mut er) = unbounded_channel();
        ca.ev = tx;
        let o = ca.text(&json!({"op": 5, "d": {"ssrc": 5, "user_id": "42", "speaking": 1}, "seq": 3})).await;
        assert!(o.is_empty());
        assert_eq!(ca.seq, 3);
        assert_eq!(ca.users.get(&5).map(String::as_str), Some("42"));
        assert!(matches!(er.try_recv(), Ok(VEv::Speak { ssrc: 5, on: true, .. })));
        ca.text(&json!({"op": 13, "d": {"user_id": "42"}})).await;
        assert!(ca.users.is_empty());
        ca.text(&json!({"op": 6, "d": {"t": now()}})).await;
        assert!(ca.acked);
        let h = ca.hb();
        assert!(matches!(h, Message::Text(_)));
        assert!(!ca.acked);
        let m = ca.bin(&[0, 7, 25, 1]);
        assert!(m.is_empty());
        assert_eq!(ca.seq, 7);
    }

    fn vpair() -> (Conn, Conn, tokio::sync::mpsc::UnboundedReceiver<SEv>, tokio::sync::mpsc::UnboundedReceiver<SEv>) {
        let cfg = Cfg { endpoint: String::new(), token: "t".into(), session: "s".into(), server: "1".into(), chan: "2".into(), user: "3".into() };
        let q = Q { w: 1280, h: 720, fps: 30, kbps: 3000 };
        let (ea, ra) = unbounded_channel();
        let (eb, rb) = unbounded_channel();
        let mut a = Conn::with(cfg.clone(), Eng::new(), unbounded_channel().0, Some((ea, Some(q), Codec::H264)));
        let mut b = Conn::with(cfg, Eng::new(), unbounded_channel().0, Some((eb, None, Codec::H264)));
        a.ssrc = 70;
        a.stm.as_mut().unwrap().vss = 71;
        a.stm.as_mut().unwrap().rss = 72;
        b.ssrc = 80;
        a.cry = Cry::new(Mode::Xcha, &[5; 32]);
        b.cry = Cry::new(Mode::Xcha, &[5; 32]);
        b.stm.as_mut().unwrap().vusers.insert(71, "9".into());
        b.stm.as_mut().unwrap().vmain.insert("9".into(), 71);
        (a, b, ra, rb)
    }

    #[tokio::test]
    async fn video_roundtrip_nack_pli() {
        let (mut a, mut b, mut ra, mut rb) = vpair();
        let mut au = vec![];
        for (ty, n) in [(7u8, 20usize), (8, 6), (5, 4000)] {
            au.extend_from_slice(&[0, 0, 0, 1, 0x60 | ty]);
            au.extend((0..n).map(|k| (k % 200) as u8 + 1));
        }
        let pk = a.vwire(&au, 1_000_000);
        assert!(pk.len() > 3);
        let (h0, _) = Hdr::parse(&pk[0]).unwrap();
        assert_eq!((h0.pt, h0.ts, h0.ssrc), (101, 90_000, 71));
        assert_eq!(pk[0][1] & 0x80, 0);
        assert_eq!(pk.last().unwrap()[1] & 0x80, 0x80);
        for p in &pk {
            b.inbound(p);
        }
        match rb.try_recv() {
            Ok(SEv::Frame { uid, data, key, .. }) => {
                assert_eq!(uid, "9");
                assert!(key);
                assert_eq!(data, au);
            }
            o => panic!("{o:?}"),
        }
        let seq1 = Hdr::parse(&pk[1]).unwrap().0.seq;
        let nk = vrtp::nack(80, 71, &[seq1]);
        let sealed = b.cry.as_mut().unwrap().seal_rtcp(&nk);
        let out = a.inbound(&sealed);
        assert_eq!(out.len(), 1);
        let (h, pl) = b.cry.as_ref().unwrap().open(&out[0]).unwrap();
        assert_eq!((h.pt, h.ssrc), (102, 72));
        assert_eq!(u16::from_be_bytes([pl[0], pl[1]]), seq1);
        let sealed = b.cry.as_mut().unwrap().seal_rtcp(&vrtp::pli(80, 71));
        a.inbound(&sealed);
        assert!(matches!(ra.try_recv(), Ok(SEv::Key)));
        let sealed = b.cry.as_mut().unwrap().seal_rtcp(&vrtp::fir(80, 71, 1));
        a.inbound(&sealed);
        assert!(matches!(ra.try_recv(), Ok(SEv::Key)));
    }

    #[tokio::test]
    async fn video_loss_requests_pli() {
        let (mut a, mut b, _ra, mut rb) = vpair();
        let mut au = vec![0, 0, 0, 1, 0x65];
        au.extend((0..4000).map(|k| (k % 200) as u8 + 1));
        let pk = a.vwire(&au, 5);
        let mut plis = vec![];
        for (i, p) in pk.iter().enumerate() {
            if i == 2 {
                continue;
            }
            plis.extend(b.inbound(p));
        }
        assert_eq!(plis.len(), 1);
        assert!(matches!(rb.try_recv(), Ok(SEv::Pli)));
        let r = a.cry.as_ref().unwrap().open_rtcp(&plis[0]).unwrap();
        assert_eq!(vrtp::parse_fb(&r), vec![Fb::Pli(71)]);
    }

    #[test]
    fn ident_has_video() {
        let (a, _, _, _) = vpair();
        let mut a = a;
        let m = a.hello();
        let Message::Text(t) = m else { panic!() };
        let v: Value = serde_json::from_str(&t).unwrap();
        assert_eq!(v["d"]["video"], true);
        assert_eq!(v["d"]["streams"][0]["type"], "screen");
    }
}
