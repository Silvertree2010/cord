use cord_core::*;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub const MCAP: usize = 400;
pub const HCAP: usize = 3000;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Unr {
    pub n: u32,
    pub ment: u32,
}

#[derive(Default)]
pub struct Cfg {
    pub log_del: bool,
    pub log_ed: bool,
    pub show_hid: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct GldV {
    #[serde(flatten)]
    pub g: Gld,
    pub rid: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChanV {
    #[serde(flatten)]
    pub c: Chan,
    pub rid: String,
    pub grid: String,
    pub prid: String,
    #[serde(rename = "lastTs")]
    pub last_ts: i64,
    pub caps: Caps,
    pub permalink: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MsgV {
    #[serde(flatten)]
    pub m: Msg,
    pub mid: String,
    pub permalink: String,
}

#[derive(Default)]
pub struct Sta {
    pub accs: BTreeMap<u32, Acc>,
    pub up: HashSet<u32>,
    pub glds: HashMap<Rid, Gld>,
    pub chans: HashMap<Rid, Chan>,
    pub msgs: HashMap<Rid, VecDeque<Msg>>,
    pub rels: HashMap<Rid, Rel>,
    pub pres: HashMap<Rid, Pres>,
    pub unr: HashMap<Rid, Unr>,
    pub mems: HashMap<Rid, HashMap<Rid, Mem>>,
    pub usrs: HashMap<Rid, Usr>,
    pub vs: HashMap<Rid, Vs>,
    pub lts: HashMap<Rid, i64>,
    pub hl: HashSet<Rid>,
    pub call: CallSt,
    pub cfg: Cfg,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "t", content = "d")]
pub enum Dl {
    Acc(Acc),
    Conn { acc: u32, up: bool },
    Gld(GldV),
    GldDel(String),
    Chan(ChanV),
    ChanDel(String),
    Msg { chan: String, m: MsgV, new: bool },
    MsgDel { chan: String, id: String, gone: bool },
    Typing { chan: String, usr: Usr },
    Pres { usr: String, p: Pres },
    Rels(Vec<Rel>),
    Unr { chan: String, u: Unr },
    Mems { gld: String, list: Vec<Mem> },
    Err { acc: u32, msg: String },
    Vs(Vs),
    Speak { chan: String, usr: String, on: bool },
    Call(CallSt),
}

pub fn flake_ts(r: &str) -> i64 {
    if r.len() >= 15 && r.bytes().all(|b| b.is_ascii_digit()) {
        return r.parse::<u64>().map(|n| (n >> 22) as i64 + 1420070400000).unwrap_or(0);
    }
    if r.len() == 26 && r.bytes().all(|b| b.is_ascii_alphanumeric()) {
        let a = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
        let mut t: u64 = 0;
        for b in r.bytes().take(10) {
            match a.iter().position(|x| *x == b.to_ascii_uppercase()) {
                Some(i) => t = t.wrapping_mul(32).wrapping_add(i as u64),
                None => return 0,
            }
        }
        return t as i64;
    }
    0
}

impl Sta {
    pub fn msg_v(&self, m: &Msg) -> MsgV {
        let g = self.chans.get(&m.chan).and_then(|c| c.gld.clone());
        MsgV { mid: m.id.to_string(), permalink: link(&m.chan, g.as_ref(), Some(&m.id)), m: m.clone() }
    }

    pub fn chan_v(&self, c: &Chan) -> ChanV {
        let lt = self.lts.get(&c.id).copied().unwrap_or_else(|| c.last.as_ref().map(|l| flake_ts(&l.raw)).unwrap_or(0));
        ChanV {
            rid: c.id.to_string(),
            grid: c.gld.as_ref().map(|g| g.to_string()).unwrap_or_default(),
            prid: c.par.as_ref().map(|g| g.to_string()).unwrap_or_default(),
            last_ts: lt,
            caps: caps(c.id.svc),
            permalink: link(&c.id, c.gld.as_ref(), None),
            c: c.clone(),
        }
    }

    pub fn gld_v(&self, g: &Gld) -> GldV {
        GldV { rid: g.id.to_string(), g: g.clone() }
    }

    pub fn vis_chans(&self) -> Vec<ChanV> {
        let show = self.cfg.show_hid;
        self.chans.values().filter(|c| show || !c.hidden).map(|c| self.chan_v(c)).collect()
    }

    pub fn msgs_v(&self, chan: &Rid) -> Vec<MsgV> {
        self.msgs.get(chan).map(|q| q.iter().map(|m| self.msg_v(m)).collect()).unwrap_or_default()
    }

    pub fn purge_acc(&mut self, a: u32) {
        self.accs.remove(&a);
        self.up.remove(&a);
        self.glds.retain(|k, _| k.acc != a);
        self.chans.retain(|k, _| k.acc != a);
        self.msgs.retain(|k, _| k.acc != a);
        self.rels.retain(|k, _| k.acc != a);
        self.pres.retain(|k, _| k.acc != a);
        self.unr.retain(|k, _| k.acc != a);
        self.mems.retain(|k, _| k.acc != a);
        self.usrs.retain(|k, _| k.acc != a);
        self.vs.retain(|k, _| k.acc != a);
        self.lts.retain(|k, _| k.acc != a);
        self.hl.retain(|k| k.acc != a);
        if self.call.chan.as_ref().is_some_and(|c| c.acc == a) {
            self.call = CallSt::default();
        }
    }

    pub fn apply(&mut self, e: Ev) -> Vec<Dl> {
        let mut o = vec![];
        match e {
            Ev::Ready(a) => {
                if let Some(id) = a.me.id.clone() {
                    self.usrs.insert(id, a.me.clone());
                }
                self.accs.insert(a.id, a.clone());
                o.push(Dl::Acc(a));
            }
            Ev::Conn { acc, up } => {
                if up {
                    self.up.insert(acc);
                } else {
                    self.up.remove(&acc);
                }
                o.push(Dl::Conn { acc, up });
            }
            Ev::Gld(g) => {
                let v = self.gld_v(&g);
                self.glds.insert(g.id.clone(), g);
                o.push(Dl::Gld(v));
            }
            Ev::GldDel(id) => {
                self.glds.remove(&id);
                let gone: Vec<Rid> = self.chans.iter().filter(|(_, c)| c.gld.as_ref() == Some(&id)).map(|(k, _)| k.clone()).collect();
                for k in gone {
                    self.chans.remove(&k);
                    self.msgs.remove(&k);
                    self.unr.remove(&k);
                    self.lts.remove(&k);
                    self.hl.remove(&k);
                }
                self.mems.remove(&id);
                o.push(Dl::GldDel(id.to_string()));
            }
            Ev::Chan(mut c) => {
                if let Some(o) = self.chans.get(&c.id) {
                    if newer(o.last.as_ref(), c.last.as_ref()) {
                        c.last = o.last.clone();
                    }
                }
                if c.hidden && !self.cfg.show_hid {
                    o.push(Dl::ChanDel(c.id.to_string()));
                } else {
                    let v = self.chan_v(&c);
                    o.push(Dl::Chan(v));
                }
                self.chans.insert(c.id.clone(), c);
            }
            Ev::ChanDel(id) => {
                self.chans.remove(&id);
                self.msgs.remove(&id);
                self.unr.remove(&id);
                self.lts.remove(&id);
                self.hl.remove(&id);
                o.push(Dl::ChanDel(id.to_string()));
            }
            Ev::Msg(m) => {
                if let Some(id) = m.auth.id.clone() {
                    self.usrs.entry(id).or_insert_with(|| m.auth.clone());
                }
                let ck = m.chan.to_string();
                if let Some(c) = self.chans.get_mut(&m.chan) {
                    if c.last.is_none() || newer(Some(&m.id), c.last.as_ref()) {
                        c.last = Some(m.id.clone());
                    }
                }
                let lt = self.lts.entry(m.chan.clone()).or_insert(0);
                if m.ts > *lt {
                    *lt = m.ts;
                }
                let cap = if self.hl.contains(&m.chan) { HCAP } else { MCAP };
                let q = self.msgs.entry(m.chan.clone()).or_default();
                let new = match q.iter_mut().find(|x| x.id == m.id) {
                    Some(x) => {
                        let mut n = m.clone();
                        n.hist = std::mem::take(&mut x.hist);
                        n.del = x.del;
                        *x = n;
                        false
                    }
                    None => {
                        q.push_back(m.clone());
                        while q.len() > cap {
                            q.pop_front();
                        }
                        true
                    }
                };
                let mv = self.msg_v(&m);
                o.push(Dl::Msg { chan: ck, m: mv, new });
            }
            Ev::MsgUpd(mut m) => {
                let ck = m.chan.to_string();
                let lg = self.cfg.log_ed;
                let part = m.ts == 0;
                let mut found = false;
                if let Some(q) = self.msgs.get_mut(&m.chan) {
                    if let Some(x) = q.iter_mut().find(|x| x.id == m.id) {
                        found = true;
                        if part {
                            m.ts = x.ts;
                            if m.body.is_empty() {
                                m.body = x.body.clone();
                            }
                            if m.atts.is_empty() {
                                m.atts = x.atts.clone();
                            }
                            if m.embs.is_empty() {
                                m.embs = x.embs.clone();
                            }
                            if m.stks.is_empty() {
                                m.stks = x.stks.clone();
                            }
                            if m.reacts.is_empty() {
                                m.reacts = x.reacts.clone();
                            }
                            if m.reply.is_none() {
                                m.reply = x.reply.clone();
                            }
                            if m.ed.is_none() {
                                m.ed = x.ed;
                            }
                            m.pin |= x.pin;
                            m.sys = x.sys;
                        }
                        if m.auth.id == x.auth.id {
                            if m.auth.col.is_none() {
                                m.auth.col = x.auth.col;
                            }
                            if x.auth.disp.is_some() {
                                m.auth.disp = x.auth.disp.clone();
                            }
                        }
                        if lg && x.body != m.body && !x.body.is_empty() {
                            let mut h = std::mem::take(&mut x.hist);
                            h.push(x.body.clone());
                            m.hist = h;
                        } else {
                            m.hist = std::mem::take(&mut x.hist);
                        }
                        if m.body.is_empty() && m.atts.is_empty() && m.embs.is_empty() {
                            m.body = x.body.clone();
                            m.atts = x.atts.clone();
                        }
                        if m.auth.name.is_empty() {
                            m.auth = x.auth.clone();
                        }
                        m.del = x.del;
                        *x = m.clone();
                    }
                }
                if found || !part {
                    let mv = self.msg_v(&m);
                    o.push(Dl::Msg { chan: ck, m: mv, new: false });
                }
            }
            Ev::MsgDel { chan, id } => {
                let lg = self.cfg.log_del;
                let mut gone = true;
                if let Some(q) = self.msgs.get_mut(&chan) {
                    if lg {
                        if let Some(x) = q.iter_mut().find(|x| x.id == id) {
                            x.del = true;
                            gone = false;
                        }
                    } else {
                        q.retain(|x| x.id != id);
                    }
                }
                o.push(Dl::MsgDel { chan: chan.to_string(), id: id.to_string(), gone });
            }
            Ev::Typing { chan, usr } => o.push(Dl::Typing { chan: chan.to_string(), usr }),
            Ev::React { chan, id, e, me, add } => {
                if let Some(m) = self.react(&chan, &id, e, me, add) {
                    let mv = self.msg_v(&m);
                    o.push(Dl::Msg { chan: chan.to_string(), m: mv, new: false });
                }
            }
            Ev::Pres { usr, p } => {
                self.pres.insert(usr.clone(), p.clone());
                o.push(Dl::Pres { usr: usr.to_string(), p });
            }
            Ev::Rel(r) => {
                if let Some(id) = r.usr.id.clone() {
                    self.usrs.insert(id.clone(), r.usr.clone());
                    self.rels.insert(id, r);
                    o.push(Dl::Rels(self.rels.values().cloned().collect()));
                }
            }
            Ev::RelDel(id) => {
                self.rels.remove(&id);
                o.push(Dl::Rels(self.rels.values().cloned().collect()));
            }
            Ev::Mem(m) => o.extend(self.apply(Ev::Mems(vec![m]))),
            Ev::Mems(v) => {
                let mut by: BTreeMap<Rid, Vec<Mem>> = BTreeMap::new();
                for m in v {
                    if let Some(id) = m.usr.id.clone() {
                        self.mems.entry(m.gld.clone()).or_default().insert(id, m.clone());
                        by.entry(m.gld.clone()).or_default().push(m);
                    }
                }
                for (g, list) in by {
                    o.push(Dl::Mems { gld: g.to_string(), list });
                }
            }
            Ev::Unread { chan, n, ment } => {
                let u = Unr { n, ment };
                self.unr.insert(chan.clone(), u.clone());
                o.push(Dl::Unr { chan: chan.to_string(), u });
            }
            Ev::Ack { chan } => {
                self.unr.remove(&chan);
                o.push(Dl::Unr { chan: chan.to_string(), u: Unr::default() });
            }
            Ev::Err { acc, msg } => o.push(Dl::Err { acc, msg }),
            Ev::Vs(v) => {
                if let Some(id) = v.usr.id.clone() {
                    if v.chan.is_some() {
                        self.vs.insert(id, v.clone());
                    } else {
                        self.vs.remove(&id);
                    }
                }
                o.push(Dl::Vs(v));
            }
            Ev::Speak { chan, usr, on } => o.push(Dl::Speak { chan: chan.to_string(), usr: usr.to_string(), on }),
            Ev::Call(c) => {
                self.call = c.clone();
                o.push(Dl::Call(c));
            }
        }
        o
    }

    pub fn ingest(&mut self, e: Ev) -> Vec<Dl> {
        let mut ex = None;
        if let Ev::Msg(m) = &e {
            let me = self.accs.get(&m.chan.acc).and_then(|a| a.me.id.clone());
            let dup = self.msgs.get(&m.chan).is_some_and(|q| q.iter().any(|x| x.id == m.id));
            if let Some(me) = me {
                if !dup && !m.sys && !m.del && m.auth.id.as_ref() != Some(&me) {
                    let men = is_ment(&me, &m.body);
                    let u = self.bump(&m.chan, men);
                    ex = Some(Dl::Unr { chan: m.chan.to_string(), u });
                }
            }
        }
        let mut o = self.apply(e);
        o.extend(ex);
        o
    }

    pub fn bump(&mut self, chan: &Rid, ment: bool) -> Unr {
        let u = self.unr.entry(chan.clone()).or_default();
        u.n += 1;
        if ment {
            u.ment += 1;
        }
        u.clone()
    }

    pub fn prepend(&mut self, chan: &Rid, mut v: Vec<Msg>) -> Vec<Msg> {
        v.sort_by_key(|m| m.ts);
        self.hl.insert(chan.clone());
        let q = self.msgs.entry(chan.clone()).or_default();
        let have: HashSet<Rid> = q.iter().map(|m| m.id.clone()).collect();
        let mut add: Vec<Msg> = v.into_iter().filter(|m| !have.contains(&m.id)).collect();
        let room = HCAP.saturating_sub(q.len());
        if add.len() > room {
            add.drain(..add.len() - room);
        }
        if !add.is_empty() {
            let mut all: Vec<Msg> = q.drain(..).collect();
            all.extend(add.iter().cloned());
            all.sort_by_key(|m| m.ts);
            q.extend(all);
        }
        if let (Some(f), Some(t)) = (q.back(), self.lts.get(chan)) {
            if f.ts > *t {
                self.lts.insert(chan.clone(), f.ts);
            }
        }
        add
    }

    pub fn react(&mut self, chan: &Rid, id: &Rid, e: Emo, me: bool, add: bool) -> Option<Msg> {
        let m = self.msgs.get_mut(chan)?.iter_mut().find(|x| &x.id == id)?;
        let same = |a: &Emo| match (&a.id, &e.id) {
            (Some(x), Some(y)) => x.raw == y.raw,
            (None, None) => a.name == e.name,
            _ => false,
        };
        match m.reacts.iter().position(|r| same(&r.emo)) {
            Some(i) => {
                let r = &mut m.reacts[i];
                if add {
                    if !(me && r.me) {
                        r.n += 1;
                    }
                    r.me |= me;
                } else {
                    r.n = r.n.saturating_sub(1);
                    if me {
                        r.me = false;
                    }
                    if r.n == 0 {
                        m.reacts.remove(i);
                    }
                }
            }
            None if add => m.reacts.push(React { emo: e, n: 1, me }),
            None => {}
        }
        Some(m.clone())
    }
}

pub fn is_ment(me: &Rid, body: &str) -> bool {
    let r = &*me.raw;
    if r.is_empty() {
        return false;
    }
    match me.svc {
        Svc::Dc => body.contains(&format!("<@{r}>")) || body.contains(&format!("<@!{r}>")) || body.contains("@everyone") || body.contains("@here"),
        Svc::St => body.contains(&format!("<@{r}>")),
        Svc::Mx => body.contains(r),
    }
}

fn newer(a: Option<&Rid>, b: Option<&Rid>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => (a.raw.len(), &*a.raw) > (b.raw.len(), &*b.raw),
        (Some(_), None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rid(r: &str) -> Rid {
        Rid::new(Svc::Dc, 1, r)
    }

    fn usr(id: &str) -> Usr {
        Usr { id: Some(rid(id)), name: id.into(), ..Default::default() }
    }

    fn msg(id: &str, au: &str, body: &str) -> Msg {
        Msg {
            id: rid(id),
            chan: rid("c"),
            auth: usr(au),
            body: body.into(),
            ts: 5,
            ed: None,
            atts: vec![],
            embs: vec![],
            reacts: vec![],
            reply: None,
            stks: vec![],
            pin: false,
            sys: false,
            del: false,
            hist: vec![],
        }
    }

    fn sta() -> Sta {
        let mut s = Sta::default();
        s.apply(Ev::Ready(Acc { id: 1, svc: Svc::Dc, me: usr("100") }));
        s
    }

    #[test]
    fn mention_forms() {
        let me = rid("100");
        assert!(is_ment(&me, "hi <@100>"));
        assert!(is_ment(&me, "hi <@!100>"));
        assert!(is_ment(&me, "@everyone"));
        assert!(!is_ment(&me, "call 1005 now"));
        assert!(!is_ment(&me, "<@1000>"));
        assert!(is_ment(&Rid::new(Svc::Mx, 1, "@a:x"), "hey @a:x"));
    }

    #[test]
    fn unread_dedupe_and_mentions() {
        let mut s = sta();
        s.ingest(Ev::Msg(msg("1", "200", "yo")));
        s.ingest(Ev::Msg(msg("1", "200", "yo")));
        s.ingest(Ev::Msg(msg("2", "200", "ping <@100>")));
        s.ingest(Ev::Msg(msg("3", "100", "mine")));
        s.ingest(Ev::Msg(msg("4", "200", "id 100 is not a mention")));
        let u = &s.unr[&rid("c")];
        assert_eq!((u.n, u.ment), (3, 1));
        assert_eq!(s.msgs[&rid("c")].len(), 4);
    }

    #[test]
    fn dup_is_update() {
        let mut s = sta();
        let a = s.apply(Ev::Msg(msg("1", "200", "x")));
        assert!(matches!(a[0], Dl::Msg { new: true, .. }));
        let b = s.apply(Ev::Msg(msg("1", "200", "x")));
        assert!(matches!(b[0], Dl::Msg { new: false, .. }));
    }

    #[test]
    fn partial_update_keeps_body() {
        let mut s = sta();
        s.apply(Ev::Msg(msg("1", "200", "link")));
        let mut p = msg("1", "", "");
        p.ts = 0;
        p.auth = Usr::default();
        p.embs.push(Emb::default());
        s.apply(Ev::MsgUpd(p));
        let m = &s.msgs[&rid("c")][0];
        assert_eq!((m.body.as_str(), m.ts, m.embs.len()), ("link", 5, 1));
        assert_eq!(m.auth.name, "200");
    }

    #[test]
    fn chan_keeps_last() {
        let mut s = sta();
        let c = |last: Option<&str>| Chan {
            id: rid("c"),
            gld: None,
            par: None,
            k: ChK::Txt,
            name: "c".into(),
            topic: None,
            pos: 0,
            nsfw: false,
            hidden: false,
            last: last.map(rid),
            rcp: vec![],
            icon: None,
        };
        s.apply(Ev::Chan(c(Some("90"))));
        s.apply(Ev::Msg(msg("100", "200", "x")));
        s.apply(Ev::Chan(c(Some("90"))));
        assert_eq!(&*s.chans[&rid("c")].last.as_ref().unwrap().raw, "100");
        s.apply(Ev::Chan(c(None)));
        assert_eq!(&*s.chans[&rid("c")].last.as_ref().unwrap().raw, "100");
    }

    #[test]
    fn live_react() {
        let mut s = Sta::default();
        s.apply(Ev::Msg(msg("1", "200", "x")));
        let ch = Rid::new(Svc::Dc, 1, "c");
        let e = Emo { id: None, name: "👍".into(), url: None, anim: false };
        let c = s.msgs.keys().next().unwrap().clone();
        let id = s.msgs[&c][0].id.clone();
        let _ = ch;
        s.apply(Ev::React { chan: c.clone(), id: id.clone(), e: e.clone(), me: true, add: true });
        s.apply(Ev::React { chan: c.clone(), id: id.clone(), e: e.clone(), me: false, add: true });
        assert_eq!(s.msgs[&c][0].reacts[0].n, 2);
        assert!(s.msgs[&c][0].reacts[0].me);
        s.apply(Ev::React { chan: c.clone(), id: id.clone(), e: e.clone(), me: true, add: false });
        s.apply(Ev::React { chan: c.clone(), id: id.clone(), e, me: false, add: false });
        assert!(s.msgs[&c][0].reacts.is_empty());
    }

    #[test]
    fn golden_chan_and_msg_entities() {
        let mut s = sta();
        let c = Chan { id: rid("c"), gld: Some(rid("g")), par: None, k: ChK::Txt, name: "gen".into(), topic: None, pos: 0, nsfw: false, hidden: false, last: None, rcp: vec![], icon: None };
        let d = s.apply(Ev::Chan(c));
        match &d[0] {
            Dl::Chan(v) => {
                assert_eq!(v.rid, "dc:1:c");
                assert_eq!(v.grid, "dc:1:g");
                assert_eq!(v.permalink, "https://discord.com/channels/g/c");
                assert!(v.caps.call);
            }
            _ => panic!(),
        }
        let d = s.apply(Ev::Msg(msg("1000000000000000", "200", "x")));
        match &d[0] {
            Dl::Msg { m, new, .. } => {
                assert!(*new);
                assert_eq!(m.permalink, "https://discord.com/channels/g/c/1000000000000000");
            }
            _ => panic!(),
        }
        assert_eq!(s.lts[&rid("c")], 5);
    }

    #[test]
    fn hidden_chan_emits_del() {
        let mut s = sta();
        let c = Chan { id: rid("h"), gld: None, par: None, k: ChK::Txt, name: "h".into(), topic: None, pos: 0, nsfw: false, hidden: true, last: None, rcp: vec![], icon: None };
        assert!(matches!(s.apply(Ev::Chan(c.clone()))[0], Dl::ChanDel(_)));
        s.cfg.show_hid = true;
        assert!(matches!(s.apply(Ev::Chan(c))[0], Dl::Chan(_)));
        assert_eq!(s.vis_chans().len(), 1);
    }

    #[test]
    fn mem_batch_single_event() {
        let mut s = sta();
        let mk = |i: &str| Mem { usr: usr(i), gld: rid("g"), nick: None, roles: vec![] };
        let d = s.apply(Ev::Mems(vec![mk("a"), mk("b"), mk("c")]));
        assert_eq!(d.len(), 1);
        assert!(matches!(&d[0], Dl::Mems { list, .. } if list.len() == 3));
    }

    #[test]
    fn history_survives_live_trim() {
        let mut s = sta();
        let h: Vec<Msg> = (0..450).map(|i| { let mut m = msg(&format!("{}", 100 + i), "200", "h"); m.ts = i; m }).collect();
        let added = s.prepend(&rid("c"), h);
        assert_eq!(added.len(), 450);
        for i in 0..20 {
            let mut m = msg(&format!("{}", 9000 + i), "200", "live");
            m.ts = 1000 + i;
            s.apply(Ev::Msg(m));
        }
        assert_eq!(s.msgs[&rid("c")].len(), 470);
        assert_eq!(&*s.msgs[&rid("c")][0].id.raw, "100");
    }

    #[test]
    fn live_trim_without_history() {
        let mut s = sta();
        for i in 0..(MCAP + 5) {
            s.apply(Ev::Msg(msg(&format!("{}", 100 + i), "200", "l")));
        }
        assert_eq!(s.msgs[&rid("c")].len(), MCAP);
    }

    #[test]
    fn purge_removes_acc_state() {
        let mut s = sta();
        s.apply(Ev::Msg(msg("1", "200", "x")));
        s.apply(Ev::Pres { usr: rid("200"), p: Pres::default() });
        s.purge_acc(1);
        assert!(s.msgs.is_empty() && s.pres.is_empty() && s.accs.is_empty() && s.unr.is_empty());
    }

    #[test]
    fn ulid_and_flake() {
        assert_eq!(flake_ts("4194304"), 0);
        assert_eq!(flake_ts("175928847299117063"), 1462015105796);
        assert!(flake_ts("01ARZ3NDEKTSV4RRFFQ69G5FAV") > 1_400_000_000_000);
    }
}
