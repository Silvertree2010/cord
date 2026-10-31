use async_trait::async_trait;
use cord_core::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Demo {
    a: u32,
    sv: Svc,
    n: AtomicU64,
    tx: tokio::sync::OnceCell<Tx>,
}

impl Demo {
    pub fn new(a: u32, sv: Svc) -> Self {
        Self { a, sv, n: AtomicU64::new(1000), tx: Default::default() }
    }
    fn r(&self, s: impl Into<String>) -> Rid {
        Rid::new(self.sv, self.a, s.into())
    }
    fn u(&self, i: u32) -> Usr {
        let ns = ["mira", "jonas", "kai", "lea", "noah", "zoe", "finn", "ida"];
        let n = ns[i as usize % ns.len()];
        Usr {
            id: Some(self.r(format!("u{i}"))),
            name: n.into(),
            disp: Some(format!("{}{}", &n[..1].to_uppercase(), &n[1..])),
            av: None,
            bot: false,
            col: [None, Some(0x5865f2), Some(0x3ba55c), Some(0xeb459e)][i as usize % 4],
        }
    }
    fn now() -> i64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
    }
    fn m(&self, c: &Rid, i: u64, ts: i64) -> Msg {
        let bs = [
            "anyone up for a round later?",
            "pushed the fix, check **main**",
            "lol",
            "the new layout looks way cleaner",
            "`cargo build` takes forever on this machine",
            "> quote test\nreply under it",
            "https://example.org",
            "brb",
        ];
        Msg {
            id: self.r(format!("m{i}")),
            chan: c.clone(),
            auth: self.u((i % 5) as u32),
            body: bs[i as usize % bs.len()].into(),
            ts,
            ed: None,
            atts: vec![],
            embs: vec![],
            reacts: if i % 4 == 0 {
                vec![React { emo: Emo { id: None, name: "👍".into(), url: None, anim: false }, n: 2, me: false }]
            } else {
                vec![]
            },
            reply: None,
            stks: vec![],
            pin: false,
            sys: false,
            del: false,
            hist: vec![],
        }
    }
}

#[async_trait]
impl Bk for Demo {
    fn svc(&self) -> Svc {
        self.sv
    }
    fn acc(&self) -> u32 {
        self.a
    }
    async fn start(&self, tx: Tx) -> Res<()> {
        let _ = self.tx.set(tx.clone());
        let me = self.u(7);
        tx.send(Ev::Ready(Acc { id: self.a, svc: self.sv, me }))?;
        tx.send(Ev::Conn { acc: self.a, up: true })?;
        let gs: &[(&str, &[&str])] = match self.sv {
            Svc::Dc => &[("Rust Lounge", &["general", "help", "showcase"]), ("Gaming", &["general", "clips", "lfg"])],
            Svc::Mx => &[("Linux Space", &["arch", "hyprland"])],
            Svc::St => &[("Stoat Devs", &["general", "feedback"])],
        };
        for (gi, (gn, cs)) in gs.iter().enumerate() {
            let gid = self.r(format!("g{gi}"));
            tx.send(Ev::Gld(Gld { id: gid.clone(), name: gn.to_string(), icon: None, ban: None, owner: None, pos: gi as i64, roles: vec![], emos: vec![] }))?;
            let cat = self.r(format!("g{gi}cat"));
            tx.send(Ev::Chan(Chan { id: cat.clone(), gld: Some(gid.clone()), par: None, k: ChK::Cat, name: "text channels".into(), topic: None, pos: 0, nsfw: false, hidden: false, last: None, rcp: vec![], icon: None }))?;
            for (ci, cn) in cs.iter().enumerate() {
                tx.send(Ev::Chan(Chan { id: self.r(format!("g{gi}c{ci}")), gld: Some(gid.clone()), par: Some(cat.clone()), k: ChK::Txt, name: cn.to_string(), topic: Some(format!("{cn} talk")), pos: ci as i64, nsfw: false, hidden: false, last: None, rcp: vec![], icon: None }))?;
            }
            tx.send(Ev::Chan(Chan { id: self.r(format!("g{gi}v")), gld: Some(gid.clone()), par: None, k: ChK::Voice, name: "Voice".into(), topic: None, pos: 9, nsfw: false, hidden: false, last: None, rcp: vec![], icon: None }))?;
            if gi == 0 {
                tx.send(Ev::Chan(Chan { id: self.r(format!("g{gi}h")), gld: Some(gid.clone()), par: Some(cat.clone()), k: ChK::Txt, name: "mods-only".into(), topic: None, pos: 8, nsfw: false, hidden: true, last: None, rcp: vec![], icon: None }))?;
                tx.send(Ev::Unread { chan: self.r(format!("g{gi}c1")), n: 3, ment: 1 })?;
            }
        }
        for i in 0..3u32 {
            let u = self.u(i + (self.a % 3));
            tx.send(Ev::Rel(Rel { usr: u.clone(), k: if i == 2 { RelK::In } else { RelK::Friend } }))?;
            tx.send(Ev::Pres { usr: u.id.clone().unwrap(), p: Pres { st: [St::On, St::Idle, St::Dnd][i as usize], act: if i == 0 { Some("Playing Counter-Strike 2".into()) } else { None }, ..Default::default() } })?;
            if i == 0 {
                tx.send(Ev::Chan(Chan { id: self.r(format!("dm{i}")), gld: None, par: None, k: ChK::Dm, name: u.shown().into(), topic: None, pos: 0, nsfw: false, hidden: false, last: None, rcp: vec![u], icon: None }))?;
            }
        }
        let c = self.r("g0c0");
        let s = Self { a: self.a, sv: self.sv, n: AtomicU64::new(0), tx: Default::default() };
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(9 + s.a as u64 * 4)).await;
                let i = s.n.fetch_add(1, Ordering::Relaxed);
                if tx.send(Ev::Typing { chan: c.clone(), usr: s.u(1) }).is_err() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                if tx.send(Ev::Msg(s.m(&c, 5000 + i, Self::now()))).is_err() {
                    break;
                }
            }
        });
        Ok(())
    }
    async fn hist(&self, chan: &Rid, before: Option<&Rid>, n: u32) -> Res<Vec<Msg>> {
        let b: u64 = before.and_then(|r| r.raw.trim_start_matches('m').parse().ok()).unwrap_or(200);
        let t = Self::now() - 60_000;
        Ok((b.saturating_sub(n as u64)..b).map(|i| self.m(chan, i, t - (b - i) as i64 * 97_000)).collect())
    }
    async fn send(&self, chan: &Rid, o: Out) -> Res<()> {
        let i = self.n.fetch_add(1, Ordering::Relaxed);
        let mut m = self.m(chan, 9000 + i, Self::now());
        m.auth = self.u(7);
        m.body = o.body;
        m.reply = o.reply;
        if let Some(tx) = self.tx.get() {
            tx.send(Ev::Msg(m))?;
        }
        Ok(())
    }
    async fn edit(&self, chan: &Rid, msg: &Rid, body: String) -> Res<()> {
        let mut m = self.m(chan, 0, Self::now());
        m.id = msg.clone();
        m.auth = self.u(7);
        m.body = body;
        m.ed = Some(Self::now());
        if let Some(tx) = self.tx.get() {
            tx.send(Ev::MsgUpd(m))?;
        }
        Ok(())
    }
    async fn del(&self, chan: &Rid, msg: &Rid) -> Res<()> {
        if let Some(tx) = self.tx.get() {
            tx.send(Ev::MsgDel { chan: chan.clone(), id: msg.clone() })?;
        }
        Ok(())
    }
    async fn react(&self, _c: &Rid, _m: &Rid, _e: &Emo, _on: bool) -> Res<()> {
        Ok(())
    }
    async fn typing(&self, _c: &Rid) -> Res<()> {
        Ok(())
    }
    async fn ack(&self, _c: &Rid, _m: &Rid) -> Res<()> {
        Ok(())
    }
}
