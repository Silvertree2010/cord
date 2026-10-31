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
    pub call: CallSt,
    pub cfg: Cfg,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "t", content = "d")]
pub enum Dl {
    Acc(Acc),
    Conn { acc: u32, up: bool },
    Gld(String),
    GldDel(String),
    Chan(String),
    ChanDel(String),
    Msg { chan: String, m: Msg, new: bool },
    MsgDel { chan: String, id: String, gone: bool },
    Typing { chan: String, usr: Usr },
    Pres { usr: String, p: Pres },
    Rels,
    Unr { chan: String, u: Unr },
    Mems(String),
    Err { acc: u32, msg: String },
    Vs(Vs),
    Speak { chan: String, usr: String, on: bool },
    Call(CallSt),
}

impl Sta {
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
                let k = g.id.to_string();
                self.glds.insert(g.id.clone(), g);
                o.push(Dl::Gld(k));
            }
            Ev::GldDel(id) => {
                self.glds.remove(&id);
                let gone: Vec<Rid> = self.chans.iter().filter(|(_, c)| c.gld.as_ref() == Some(&id)).map(|(k, _)| k.clone()).collect();
                for k in gone {
                    self.chans.remove(&k);
                    self.msgs.remove(&k);
                    self.unr.remove(&k);
                }
                self.mems.remove(&id);
                o.push(Dl::GldDel(id.to_string()));
            }
            Ev::Chan(mut c) => {
                let k = c.id.to_string();
                if let Some(o) = self.chans.get(&c.id) {
                    if newer(o.last.as_ref(), c.last.as_ref()) {
                        c.last = o.last.clone();
                    }
                }
                self.chans.insert(c.id.clone(), c);
                o.push(Dl::Chan(k));
            }
            Ev::ChanDel(id) => {
                self.chans.remove(&id);
                self.msgs.remove(&id);
                self.unr.remove(&id);
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
                        while q.len() > MCAP {
                            q.pop_front();
                        }
                        true
                    }
                };
                o.push(Dl::Msg { chan: ck, m, new });
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
                    o.push(Dl::Msg { chan: ck, m, new: false });
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
                    o.push(Dl::Msg { chan: chan.to_string(), m, new: false });
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
                    o.push(Dl::Rels);
                }
            }
            Ev::RelDel(id) => {
                self.rels.remove(&id);
                o.push(Dl::Rels);
            }
            Ev::Mem(m) => {
                let gk = m.gld.to_string();
                if let Some(id) = m.usr.id.clone() {
                    self.mems.entry(m.gld.clone()).or_default().insert(id, m);
                }
                o.push(Dl::Mems(gk));
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

    pub fn prepend(&mut self, chan: &Rid, mut v: Vec<Msg>) {
        v.sort_by_key(|m| m.ts);
        let q = self.msgs.entry(chan.clone()).or_default();
        let have: HashSet<Rid> = q.iter().map(|m| m.id.clone()).collect();
        for m in v.into_iter().rev() {
            if !have.contains(&m.id) {
                q.push_front(m);
            }
        }
        while q.len() > HCAP {
            q.pop_back();
        }
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
}
