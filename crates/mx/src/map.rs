use cord_core::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct Cx {
    pub acc: u32,
    pub me: String,
}

impl Cx {
    pub fn rid(&self, raw: &str) -> Rid {
        Rid::new(Svc::Mx, self.acc, raw)
    }
}

pub fn usr(cx: &Cx, id: &str, disp: Option<&str>, av: Option<&str>) -> Usr {
    Usr {
        id: Some(cx.rid(id)),
        name: id.to_string(),
        disp: disp.map(String::from).filter(|d| !d.is_empty()),
        av: av.map(String::from),
        bot: false,
        col: None,
    }
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

pub fn src(c: &Value) -> Option<String> {
    if let Some(u) = c.get("url").and_then(|u| u.as_str()) {
        return Some(u.to_string());
    }
    let f = c.get("file")?;
    f.get("url")?;
    Some(format!("mxe:{}", hex(f.to_string().as_bytes())))
}

pub fn strip_fallback(body: &str) -> String {
    if !body.starts_with("> ") {
        return body.to_string();
    }
    let mut it = body.lines().peekable();
    while it.peek().is_some_and(|l| l.starts_with('>')) {
        it.next();
    }
    if it.peek().is_some_and(|l| l.is_empty()) {
        it.next();
    }
    it.collect::<Vec<_>>().join("\n")
}

fn u32o(v: Option<&Value>) -> Option<u32> {
    v.and_then(|x| x.as_u64()).map(|x| x as u32)
}

fn att(c: &Value) -> Option<Att> {
    let url = src(c)?;
    let info = c.get("info");
    let name = c
        .get("filename")
        .and_then(|f| f.as_str())
        .or_else(|| c.get("body").and_then(|b| b.as_str()))
        .unwrap_or("file")
        .to_string();
    Some(Att {
        url,
        name,
        mime: info.and_then(|i| i.get("mimetype")).and_then(|m| m.as_str()).map(String::from),
        size: info.and_then(|i| i.get("size")).and_then(|s| s.as_u64()).unwrap_or(0),
        w: u32o(info.and_then(|i| i.get("w"))),
        h: u32o(info.and_then(|i| i.get("h"))),
    })
}

#[derive(Clone, Debug)]
pub enum Item {
    New(Msg),
    Edit { id: String, body: String, ts: i64 },
    Del(String),
    React { ev: String, target: String, key: String, who: String },
    Skip,
}

pub fn rel(c: &Value) -> Option<&Value> {
    c.get("m.relates_to")
}

pub fn item(cx: &Cx, room: &str, ev: &Value, sender: Usr) -> Item {
    let ty = ev.get("type").and_then(|t| t.as_str()).unwrap_or_default();
    let id = ev.get("event_id").and_then(|t| t.as_str()).unwrap_or_default();
    let ts = ev.get("origin_server_ts").and_then(|t| t.as_i64()).unwrap_or(0);
    let c = ev.get("content").cloned().unwrap_or(Value::Null);
    if id.is_empty() || ev.pointer("/unsigned/redacted_because").is_some() {
        return Item::Skip;
    }
    match ty {
        "m.room.redaction" => {
            let t = ev.get("redacts").or_else(|| c.get("redacts")).and_then(|r| r.as_str());
            t.map(|t| Item::Del(t.to_string())).unwrap_or(Item::Skip)
        }
        "m.reaction" => {
            let r = rel(&c);
            let rt = r.and_then(|r| r.get("rel_type")).and_then(|x| x.as_str());
            let t = r.and_then(|r| r.get("event_id")).and_then(|x| x.as_str());
            let k = r.and_then(|r| r.get("key")).and_then(|x| x.as_str());
            match (rt, t, k) {
                (Some("m.annotation"), Some(t), Some(k)) => Item::React {
                    ev: id.to_string(),
                    target: t.to_string(),
                    key: k.to_string(),
                    who: ev.get("sender").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
                },
                _ => Item::Skip,
            }
        }
        "m.sticker" => {
            let mut m = base(cx, room, id, ts, sender);
            m.body = c.get("body").and_then(|b| b.as_str()).unwrap_or_default().to_string();
            m.stks = att(&c).into_iter().collect();
            Item::New(m)
        }
        "m.room.encrypted" => {
            let mut m = base(cx, room, id, ts, sender);
            m.body = "Unable to decrypt this message".into();
            Item::New(m)
        }
        "m.room.message" => {
            let r = rel(&c);
            if r.and_then(|r| r.get("rel_type")).and_then(|x| x.as_str()) == Some("m.replace") {
                let t = r.and_then(|r| r.get("event_id")).and_then(|x| x.as_str());
                let nc = c.get("m.new_content");
                let b = nc.and_then(|n| n.get("body")).and_then(|b| b.as_str());
                return match (t, b) {
                    (Some(t), Some(b)) => Item::Edit { id: t.to_string(), body: b.to_string(), ts },
                    _ => Item::Skip,
                };
            }
            let Some(mt) = c.get("msgtype").and_then(|m| m.as_str()) else { return Item::Skip };
            let mut m = base(cx, room, id, ts, sender);
            let raw = c.get("body").and_then(|b| b.as_str()).unwrap_or_default();
            let reply = r
                .and_then(|r| r.get("m.in_reply_to"))
                .filter(|i| i.get("is_falling_back").and_then(|f| f.as_bool()) != Some(true))
                .and_then(|i| i.get("event_id"))
                .and_then(|e| e.as_str());
            if let Some(re) = reply {
                m.reply = Some(cx.rid(re));
            }
            let text = if reply.is_some() { strip_fallback(raw) } else { raw.to_string() };
            match mt {
                "m.image" | "m.video" | "m.audio" | "m.file" => {
                    if let Some(a) = att(&c) {
                        m.atts.push(a);
                    }
                    let fname = c.get("filename").and_then(|f| f.as_str());
                    m.body = if fname.is_some() && fname != Some(raw) { text } else { String::new() };
                }
                "m.emote" => m.body = format!("* {}", text),
                _ => m.body = text,
            }
            Item::New(m)
        }
        _ => Item::Skip,
    }
}

fn base(cx: &Cx, room: &str, id: &str, ts: i64, auth: Usr) -> Msg {
    Msg {
        id: cx.rid(id),
        chan: cx.rid(room),
        auth,
        body: String::new(),
        ts,
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
        ments: vec![],
        aroles: vec![],
        fwd: false,
        rq: None,
        rqb: String::new(),
    }
}

pub fn emo(key: &str) -> Emo {
    Emo { id: None, name: key.to_string(), url: key.starts_with("mxc://").then(|| key.to_string()), anim: false }
}

pub fn add_react(m: &mut Msg, key: &str, mine: bool) {
    let e = emo(key);
    match m.reacts.iter_mut().find(|r| r.emo == e) {
        Some(r) => {
            r.n += 1;
            r.me |= mine;
        }
        None => m.reacts.push(React { emo: e, n: 1, me: mine }),
    }
}

pub fn rm_react(m: &mut Msg, key: &str, mine: bool) {
    let e = emo(key);
    if let Some(i) = m.reacts.iter().position(|r| r.emo == e) {
        let r = &mut m.reacts[i];
        r.n = r.n.saturating_sub(1);
        if mine {
            r.me = false;
        }
        if r.n == 0 {
            m.reacts.remove(i);
        }
    }
}

pub fn apply_edit(m: &mut Msg, body: &str, ts: i64) {
    m.body = body.to_string();
    m.ed = Some(ts);
}

pub fn typing(cx: &Cx, room: &str, ev: &Value, who: &dyn Fn(&str) -> Usr) -> Vec<Ev> {
    ev.pointer("/content/user_ids")
        .and_then(|u| u.as_array())
        .into_iter()
        .flatten()
        .filter_map(|u| u.as_str())
        .filter(|u| *u != cx.me)
        .map(|u| Ev::Typing { chan: cx.rid(room), usr: who(u) })
        .collect()
}

pub fn receipt_ack(cx: &Cx, room: &str, ev: &Value) -> bool {
    let Some(o) = ev.get("content").and_then(|c| c.as_object()) else { return false };
    o.values().any(|per| {
        ["m.read", "m.read.private"]
            .iter()
            .any(|k| per.get(k).and_then(|r| r.get(&cx.me)).is_some())
    }) && !room.is_empty()
}

pub fn pres(cx: &Cx, ev: &Value) -> Option<Ev> {
    let who = ev.get("sender").and_then(|s| s.as_str())?;
    let c = ev.get("content")?;
    let st = match c.get("presence").and_then(|p| p.as_str())? {
        "online" => St::On,
        "unavailable" => St::Idle,
        _ => St::Off,
    };
    let act = c.get("status_msg").and_then(|s| s.as_str()).filter(|s| !s.is_empty()).map(String::from);
    Some(Ev::Pres { usr: cx.rid(who), p: Pres { st, act, ..Default::default() } })
}

pub fn direct_ids(ev: &Value) -> HashSet<String> {
    let mut s = HashSet::new();
    if let Some(o) = ev.get("content").and_then(|c| c.as_object()) {
        for rooms in o.values() {
            for r in rooms.as_array().into_iter().flatten().filter_map(|r| r.as_str()) {
                s.insert(r.to_string());
            }
        }
    }
    s
}

pub fn children(states: &[Value]) -> Vec<String> {
    states
        .iter()
        .filter(|s| s.get("type").and_then(|t| t.as_str()) == Some("m.space.child"))
        .filter(|s| s.pointer("/content/via").and_then(|v| v.as_array()).is_some_and(|v| !v.is_empty()))
        .filter_map(|s| s.get("state_key").and_then(|k| k.as_str()).map(String::from))
        .collect()
}

#[derive(Clone, Debug, Default)]
pub struct RoomInfo {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
    pub av: Option<String>,
    pub dm: bool,
    pub space: bool,
    pub kids: Vec<String>,
    pub others: Vec<Usr>,
    pub enc: bool,
}

pub fn layout(cx: &Cx, rooms: &[RoomInfo]) -> (Vec<Gld>, Vec<Chan>) {
    let idx: HashMap<&str, &RoomInfo> = rooms.iter().map(|r| (r.id.as_str(), r)).collect();
    let mut sub: HashSet<&str> = HashSet::new();
    for r in rooms.iter().filter(|r| r.space) {
        for k in &r.kids {
            if idx.get(k.as_str()).is_some_and(|c| c.space) {
                sub.insert(k.as_str());
            }
        }
    }
    let mut seen: HashSet<String> = HashSet::new();
    let mut glds = vec![];
    let mut chans = vec![];
    let mut order: Vec<&RoomInfo> = rooms.iter().filter(|r| r.space && !sub.contains(r.id.as_str())).collect();
    let walk_all = |top: &RoomInfo, glds: &mut Vec<Gld>, chans: &mut Vec<Chan>, seen: &mut HashSet<String>| {
        if !seen.insert(top.id.clone()) {
            return;
        }
        glds.push(Gld {
            id: cx.rid(&top.id),
            name: top.name.clone(),
            icon: top.av.clone(),
            ban: None,
            owner: None,
            pos: glds.len() as i64,
            roles: vec![],
            emos: vec![],
        });
        let mut stack: Vec<(&RoomInfo, Option<Rid>)> = vec![(top, None)];
        let mut pos = 0i64;
        while let Some((sp, par)) = stack.pop() {
            let mut next = vec![];
            for k in &sp.kids {
                let Some(r) = idx.get(k.as_str()) else { continue };
                if r.dm || !seen.insert(r.id.clone()) {
                    continue;
                }
                let mut c = chan(cx, r, if r.space { ChK::Cat } else { ChK::Txt });
                c.gld = Some(cx.rid(&top.id));
                c.par = par.clone();
                c.pos = pos;
                pos += 1;
                chans.push(c);
                if r.space {
                    next.push((*r, Some(cx.rid(&r.id))));
                }
            }
            next.reverse();
            stack.extend(next);
        }
    };
    for t in order.drain(..) {
        walk_all(t, &mut glds, &mut chans, &mut seen);
    }
    let rest: Vec<&RoomInfo> = rooms.iter().filter(|r| r.space && !seen.contains(&r.id)).collect();
    for r in rest {
        walk_all(r, &mut glds, &mut chans, &mut seen);
    }
    let mut pos = 0;
    for r in rooms.iter().filter(|r| !r.space && !seen.contains(&r.id) || r.dm) {
        let mut c = chan(cx, r, if r.dm { ChK::Dm } else { ChK::Grp });
        c.pos = pos;
        pos += 1;
        chans.push(c);
    }
    (glds, chans)
}

fn chan(cx: &Cx, r: &RoomInfo, k: ChK) -> Chan {
    Chan {
        id: cx.rid(&r.id),
        gld: None,
        par: None,
        k,
        name: r.name.clone(),
        topic: r.topic.clone(),
        pos: 0,
        nsfw: false,
        hidden: false,
        last: None,
        rcp: r.others.clone(),
        icon: r.av.clone(),
        enc: r.enc,
        mute: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cx() -> Cx {
        Cx { acc: 2, me: "@me:x".into() }
    }

    fn who() -> Usr {
        usr(&cx(), "@a:x", Some("Ann"), Some("mxc://x/av"))
    }

    #[test]
    fn text_and_reply() {
        let ev = json!({
            "type": "m.room.message", "event_id": "$1", "sender": "@a:x", "origin_server_ts": 1700000000000i64,
            "content": {"msgtype": "m.text", "body": "> <@b:x> hi\n\nanswer",
                "m.relates_to": {"m.in_reply_to": {"event_id": "$0"}}}
        });
        let Item::New(m) = item(&cx(), "!r:x", &ev, who()) else { panic!() };
        assert_eq!(m.body, "answer");
        assert_eq!(m.reply.unwrap().raw.as_ref(), "$0");
        assert_eq!(m.ts, 1700000000000);
        assert_eq!(m.auth.shown(), "Ann");
        assert_eq!(m.chan.raw.as_ref(), "!r:x");
    }

    #[test]
    fn image_plain_and_encrypted() {
        let ev = json!({
            "type": "m.room.message", "event_id": "$2", "sender": "@a:x", "origin_server_ts": 5,
            "content": {"msgtype": "m.image", "body": "look", "filename": "cat.png", "url": "mxc://x/abc",
                "info": {"w": 10, "h": 20, "mimetype": "image/png", "size": 77}}
        });
        let Item::New(m) = item(&cx(), "!r:x", &ev, who()) else { panic!() };
        assert_eq!(m.body, "look");
        assert_eq!(m.atts[0].url, "mxc://x/abc");
        assert_eq!(m.atts[0].name, "cat.png");
        assert_eq!((m.atts[0].w, m.atts[0].h, m.atts[0].size), (Some(10), Some(20), 77));
        let ev = json!({
            "type": "m.room.message", "event_id": "$3", "sender": "@a:x", "origin_server_ts": 5,
            "content": {"msgtype": "m.file", "body": "doc.pdf", "file": {"url": "mxc://x/enc", "v": "v2"}}
        });
        let Item::New(m) = item(&cx(), "!r:x", &ev, who()) else { panic!() };
        assert_eq!(m.body, "");
        let u = &m.atts[0].url;
        let j: Value = serde_json::from_slice(&unhex(u.strip_prefix("mxe:").unwrap()).unwrap()).unwrap();
        assert_eq!(j["url"], "mxc://x/enc");
    }

    #[test]
    fn edit_redact_react() {
        let ev = json!({
            "type": "m.room.message", "event_id": "$4", "sender": "@a:x", "origin_server_ts": 9,
            "content": {"msgtype": "m.text", "body": "* new", "m.new_content": {"msgtype": "m.text", "body": "new"},
                "m.relates_to": {"rel_type": "m.replace", "event_id": "$1"}}
        });
        match item(&cx(), "!r:x", &ev, who()) {
            Item::Edit { id, body, ts } => assert_eq!((id.as_str(), body.as_str(), ts), ("$1", "new", 9)),
            _ => panic!(),
        }
        let ev = json!({"type": "m.room.redaction", "event_id": "$5", "sender": "@a:x", "origin_server_ts": 1, "redacts": "$1", "content": {}});
        assert!(matches!(item(&cx(), "!r:x", &ev, who()), Item::Del(d) if d == "$1"));
        let ev = json!({"type": "m.room.redaction", "event_id": "$5", "sender": "@a:x", "origin_server_ts": 1, "content": {"redacts": "$2"}});
        assert!(matches!(item(&cx(), "!r:x", &ev, who()), Item::Del(d) if d == "$2"));
        let ev = json!({
            "type": "m.reaction", "event_id": "$6", "sender": "@me:x", "origin_server_ts": 1,
            "content": {"m.relates_to": {"rel_type": "m.annotation", "event_id": "$1", "key": "\u{1F44D}"}}
        });
        let Item::React { target, key, who: w, .. } = item(&cx(), "!r:x", &ev, who()) else { panic!() };
        let mut m = base(&cx(), "!r:x", "$1", 1, who());
        add_react(&mut m, &key, w == cx().me);
        add_react(&mut m, &key, false);
        assert_eq!(target, "$1");
        assert!(m.reacts[0].me && m.reacts[0].n == 2);
        rm_react(&mut m, &key, true);
        assert!(!m.reacts[0].me && m.reacts[0].n == 1);
        rm_react(&mut m, &key, false);
        assert!(m.reacts.is_empty());
        apply_edit(&mut m, "z", 3);
        assert_eq!((m.body.as_str(), m.ed), ("z", Some(3)));
    }

    #[test]
    fn redacted_skipped() {
        let ev = json!({
            "type": "m.room.message", "event_id": "$7", "sender": "@a:x", "origin_server_ts": 1,
            "content": {}, "unsigned": {"redacted_because": {"type": "m.room.redaction"}}
        });
        assert!(matches!(item(&cx(), "!r:x", &ev, who()), Item::Skip));
    }

    #[test]
    fn ephemeral() {
        let ev = json!({"type": "m.typing", "content": {"user_ids": ["@a:x", "@me:x"]}});
        let w = |u: &str| usr(&cx(), u, None, None);
        let o = typing(&cx(), "!r:x", &ev, &w);
        assert_eq!(o.len(), 1);
        let ev = json!({"type": "m.receipt", "content": {"$1": {"m.read": {"@me:x": {"ts": 1}}}}});
        assert!(receipt_ack(&cx(), "!r:x", &ev));
        let ev = json!({"type": "m.receipt", "content": {"$1": {"m.read": {"@a:x": {"ts": 1}}}}});
        assert!(!receipt_ack(&cx(), "!r:x", &ev));
        let ev = json!({"type": "m.presence", "sender": "@a:x", "content": {"presence": "unavailable", "status_msg": "brb"}});
        let Some(Ev::Pres { p, .. }) = pres(&cx(), &ev) else { panic!() };
        assert_eq!(p.st, St::Idle);
        assert_eq!(p.act.as_deref(), Some("brb"));
        let ev = json!({"type": "m.direct", "content": {"@a:x": ["!d:x"]}});
        assert!(direct_ids(&ev).contains("!d:x"));
    }

    #[test]
    fn spaces_layout() {
        let st = vec![
            json!({"type": "m.space.child", "state_key": "!sub:x", "content": {"via": ["x"]}}),
            json!({"type": "m.space.child", "state_key": "!gone:x", "content": {}}),
            json!({"type": "m.space.child", "state_key": "!a:x", "content": {"via": ["x"]}}),
        ];
        let kids = children(&st);
        assert_eq!(kids, vec!["!sub:x", "!a:x"]);
        let ri = |id: &str, space: bool, dm: bool, kids: Vec<&str>| RoomInfo {
            id: id.into(),
            name: id.into(),
            space,
            dm,
            kids: kids.into_iter().map(String::from).collect(),
            ..Default::default()
        };
        let rooms = vec![
            ri("!top:x", true, false, vec!["!sub:x", "!a:x", "!d:x"]),
            ri("!sub:x", true, false, vec!["!b:x"]),
            ri("!a:x", false, false, vec![]),
            ri("!b:x", false, false, vec![]),
            ri("!d:x", false, true, vec![]),
            ri("!g:x", false, false, vec![]),
        ];
        let (g, c) = layout(&cx(), &rooms);
        assert_eq!(g.len(), 1);
        let get = |id: &str| c.iter().find(|x| x.id.raw.as_ref() == id).unwrap();
        assert_eq!(get("!sub:x").k, ChK::Cat);
        assert_eq!(get("!b:x").par.as_ref().unwrap().raw.as_ref(), "!sub:x");
        assert_eq!(get("!b:x").gld.as_ref().unwrap().raw.as_ref(), "!top:x");
        assert!(get("!a:x").par.is_none());
        assert_eq!(get("!d:x").k, ChK::Dm);
        assert!(get("!d:x").gld.is_none());
        assert_eq!(get("!g:x").k, ChK::Grp);
        assert_eq!(c.len(), 5);
    }
}
