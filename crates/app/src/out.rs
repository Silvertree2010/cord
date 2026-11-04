use cord_core::{Acc, CallSt, Mem, Pres, Rel, Usr, Vs};
use cord_hub::{ChanV, Dl, GldV, MsgV, Unr};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ErrV {
    pub acc: u32,
    pub msg: String,
    pub ctx: String,
}

impl ErrV {
    pub fn new(acc: u32, ctx: &str, msg: impl ToString) -> Self {
        Self { acc, msg: msg.to_string(), ctx: ctx.to_string() }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PresE {
    pub usr: String,
    pub p: Pres,
}

#[derive(Clone, Debug, Serialize)]
pub struct Mp {
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub art: String,
    pub url: String,
    pub pos: f64,
    pub len: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "t", content = "d")]
pub enum Out {
    Acc(Acc),
    Conn { acc: u32, up: bool },
    Gld(GldV),
    GldDel(String),
    Chan(ChanV),
    ChanDel(String),
    Chans(Vec<ChanV>),
    Msg { chan: String, m: MsgV, new: bool },
    MsgDel { chan: String, id: String, gone: bool },
    Msgs { chan: String, list: Vec<MsgV>, synced: bool },
    Typing { chan: String, usr: Usr },
    PresBatch(Vec<PresE>),
    Rels(Vec<Rel>),
    Unr { chan: String, u: Unr },
    Mems { gld: String, list: Vec<Mem> },
    Vs(Vs),
    Speak { chan: String, usr: String, on: bool },
    Call(CallSt),
    Err(ErrV),
    Hist { chan: String, n: usize, more: bool, ok: bool, list: Vec<MsgV>, err: String, noop: bool },
    Sent { chan: String, ok: bool, err: String },
    Qr(String),
    QrUsr { name: String, id: String },
    Cap { sk: String, rq: String },
    Folds(Vec<cord_core::Fold>),
    Accs,
    AccGone { acc: u32 },
    Fs(String),
    Mpris(Option<Mp>),
    Dm { usr: String, chan: String, ok: bool, err: String },
}

#[cfg(test)]
pub const TAGS: &[&str] = &[
    "Acc", "Conn", "Gld", "GldDel", "Chan", "ChanDel", "Chans", "Msg", "MsgDel", "Msgs", "Typing", "PresBatch", "Rels", "Unr", "Mems", "Vs", "Speak", "Call", "Err", "Hist", "Sent", "Qr", "QrUsr", "Cap", "Folds",
    "Accs", "AccGone", "Fs", "Mpris", "Dm",
];

#[cfg(test)]
pub fn tag(o: &Out) -> &'static str {
    match o {
        Out::Acc(_) => "Acc",
        Out::Conn { .. } => "Conn",
        Out::Gld(_) => "Gld",
        Out::GldDel(_) => "GldDel",
        Out::Chan(_) => "Chan",
        Out::ChanDel(_) => "ChanDel",
        Out::Chans(_) => "Chans",
        Out::Msg { .. } => "Msg",
        Out::MsgDel { .. } => "MsgDel",
        Out::Msgs { .. } => "Msgs",
        Out::Typing { .. } => "Typing",
        Out::PresBatch(_) => "PresBatch",
        Out::Rels(_) => "Rels",
        Out::Unr { .. } => "Unr",
        Out::Mems { .. } => "Mems",
        Out::Vs(_) => "Vs",
        Out::Speak { .. } => "Speak",
        Out::Call(_) => "Call",
        Out::Err(_) => "Err",
        Out::Hist { .. } => "Hist",
        Out::Sent { .. } => "Sent",
        Out::Qr(_) => "Qr",
        Out::QrUsr { .. } => "QrUsr",
        Out::Cap { .. } => "Cap",
        Out::Folds(_) => "Folds",
        Out::Accs => "Accs",
        Out::AccGone { .. } => "AccGone",
        Out::Fs(_) => "Fs",
        Out::Mpris(_) => "Mpris",
        Out::Dm { .. } => "Dm",
    }
}

pub fn pres_flush(m: Vec<(String, Pres)>) -> Option<Out> {
    if m.is_empty() {
        return None;
    }
    Some(Out::PresBatch(m.into_iter().map(|(usr, p)| PresE { usr, p }).collect()))
}

pub enum Conv {
    Now(Out),
    Pres(String, Pres),
}

pub fn conv(d: Dl) -> Conv {
    Conv::Now(match d {
        Dl::Pres { usr, p } => return Conv::Pres(usr, p),
        Dl::Acc(a) => Out::Acc(a),
        Dl::Conn { acc, up } => Out::Conn { acc, up },
        Dl::Gld(g) => Out::Gld(g),
        Dl::GldDel(g) => Out::GldDel(g),
        Dl::Chan(c) => Out::Chan(c),
        Dl::ChanDel(c) => Out::ChanDel(c),
        Dl::Msg { chan, m, new } => Out::Msg { chan, m, new },
        Dl::MsgDel { chan, id, gone } => Out::MsgDel { chan, id, gone },
        Dl::Typing { chan, usr } => Out::Typing { chan, usr },
        Dl::Rels(r) => Out::Rels(r),
        Dl::Unr { chan, u } => Out::Unr { chan, u },
        Dl::Mems { gld, list } => Out::Mems { gld, list },
        Dl::Err { acc, msg } => Out::Err(ErrV::new(acc, "backend", msg)),
        Dl::Vs(v) => Out::Vs(v),
        Dl::Speak { chan, usr, on } => Out::Speak { chan, usr, on },
        Dl::Call(c) => Out::Call(c),
        Dl::Folds(f) => Out::Folds(f),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn err_shape_is_acc_msg_ctx() {
        let v = serde_json::to_value(Out::Err(ErrV::new(3, "send", "boom"))).unwrap();
        assert_eq!(v, serde_json::json!({"t": "Err", "d": {"acc": 3, "msg": "boom", "ctx": "send"}}));
    }

    #[test]
    fn sent_and_hist_shape() {
        let v = serde_json::to_value(Out::Sent { chan: "dc:1:c".into(), ok: false, err: "x".into() }).unwrap();
        assert_eq!(v["t"], "Sent");
        assert_eq!(v["d"]["ok"], false);
        let h = serde_json::to_value(Out::Hist { chan: "c".into(), n: 0, more: false, ok: false, list: vec![], err: "x".into(), noop: false }).unwrap();
        assert_eq!(h["d"]["ok"], false);
    }

    #[test]
    fn every_out_tag_has_a_case_in_s_qml() {
        let q = include_str!("../qml/Cord/S.qml");
        let on = &q[q.find("function on(d)").unwrap()..];
        let on = &on[..on.find("property var speaking").unwrap()];
        for t in TAGS {
            assert!(on.contains(&format!("case \"{t}\"")), "S.on has no case for {t}");
        }
        let samples = [
            Out::Accs,
            Out::AccGone { acc: 1 },
            Out::Qr(String::new()),
            Out::Fs(String::new()),
            Out::Mpris(None),
            Out::Conn { acc: 1, up: true },
            Out::Sent { chan: String::new(), ok: true, err: String::new() },
            Out::Msgs { chan: String::new(), list: vec![], synced: true },
            Out::Hist { chan: String::new(), n: 0, more: false, ok: true, list: vec![], err: String::new(), noop: false },
            Out::PresBatch(vec![]),
            Out::Rels(vec![]),
            Out::Chans(vec![]),
            Out::GldDel(String::new()),
            Out::ChanDel(String::new()),
            Out::Err(ErrV::new(0, "c", "m")),
        ];
        for o in &samples {
            assert_eq!(serde_json::to_value(o).unwrap()["t"], tag(o));
            assert!(TAGS.contains(&tag(o)));
        }
        let mut u = TAGS.to_vec();
        u.sort();
        u.dedup();
        assert_eq!(u.len(), TAGS.len());
    }

    #[test]
    fn pres_batches() {
        assert!(pres_flush(vec![]).is_none());
        let o = pres_flush(vec![("dc:1:u".into(), Pres::default())]).unwrap();
        assert_eq!(serde_json::to_value(o).unwrap()["t"], "PresBatch");
    }
}
