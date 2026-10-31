use cord_core::*;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub const MCAP: usize = 400;

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
                self.chans.retain(|_, c| c.gld.as_ref() != Some(&id));
                o.push(Dl::GldDel(id.to_string()));
            }
            Ev::Chan(c) => {
                let k = c.id.to_string();
                self.chans.insert(c.id.clone(), c);
                o.push(Dl::Chan(k));
            }
            Ev::ChanDel(id) => {
                self.chans.remove(&id);
                self.msgs.remove(&id);
                o.push(Dl::ChanDel(id.to_string()));
            }
            Ev::Msg(m) => {
                if let Some(id) = m.auth.id.clone() {
                    self.usrs.entry(id).or_insert_with(|| m.auth.clone());
                }
                let ck = m.chan.to_string();
                if let Some(c) = self.chans.get_mut(&m.chan) {
                    c.last = Some(m.id.clone());
                }
                let q = self.msgs.entry(m.chan.clone()).or_default();
                if !q.iter().any(|x| x.id == m.id) {
                    q.push_back(m.clone());
                    while q.len() > MCAP {
                        q.pop_front();
                    }
                }
                o.push(Dl::Msg { chan: ck, m, new: true });
            }
            Ev::MsgUpd(mut m) => {
                let ck = m.chan.to_string();
                let lg = self.cfg.log_ed;
                if let Some(q) = self.msgs.get_mut(&m.chan) {
                    if let Some(x) = q.iter_mut().find(|x| x.id == m.id) {
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
                        *x = m.clone();
                    }
                }
                o.push(Dl::Msg { chan: ck, m, new: false });
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
    }
}
