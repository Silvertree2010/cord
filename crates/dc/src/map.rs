use crate::perm::pu;
use cord_core::*;
use serde_json::Value;
use std::collections::HashMap;

pub const CDN: &str = "https://cdn.discordapp.com";

pub fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(|x| x.as_str())
}

pub fn rid(a: u32, r: &str) -> Rid {
    Rid::new(Svc::Dc, a, r)
}

fn dn(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let e = y.div_euclid(400);
    let ye = y - e * 400;
    let mp = (m + 9) % 12;
    let dy = (153 * mp + 2) / 5 + d - 1;
    let de = ye * 365 + ye / 4 - ye / 100 + dy;
    e * 146097 + de - 719468
}

pub fn ts(t: &str) -> Option<i64> {
    let b = t.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let n = |a: usize, l: usize| -> Option<i64> { t.get(a..a + l)?.parse().ok() };
    let (y, mo, d, h, mi, se) = (n(0, 4)?, n(5, 2)?, n(8, 2)?, n(11, 2)?, n(14, 2)?, n(17, 2)?);
    let mut i = 19;
    let mut ms = 0i64;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let st = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        let f = &t[st..i];
        let f3: String = f.chars().chain("000".chars()).take(3).collect();
        ms = f3.parse().ok()?;
    }
    let mut off = 0i64;
    if let Some(&c) = b.get(i) {
        if c == b'+' || c == b'-' {
            let oh: i64 = t.get(i + 1..i + 3)?.parse().ok()?;
            let om: i64 = t.get(i + 4..i + 6).and_then(|x| x.parse().ok()).unwrap_or(0);
            off = (oh * 3600 + om * 60) * if c == b'-' { -1 } else { 1 };
        }
    }
    Some(((dn(y, mo, d) * 86400 + h * 3600 + mi * 60 + se) - off) * 1000 + ms)
}

pub fn av(id: &str, h: Option<&str>, disc: Option<&str>) -> String {
    match h {
        Some(h) if !h.is_empty() => {
            let e = if h.starts_with("a_") { "gif" } else { "png" };
            format!("{CDN}/avatars/{id}/{h}.{e}?size=128")
        }
        _ => {
            let d: u64 = disc.and_then(|x| x.parse().ok()).unwrap_or(0);
            let i = if d == 0 { (id.parse::<u64>().unwrap_or(0) >> 22) % 6 } else { d % 5 };
            format!("{CDN}/embed/avatars/{i}.png")
        }
    }
}

pub fn ic(k: &str, id: &str, h: Option<&str>) -> Option<String> {
    let h = h.filter(|h| !h.is_empty())?;
    let e = if h.starts_with("a_") { "gif" } else { "png" };
    Some(format!("{CDN}/{k}/{id}/{h}.{e}?size=256"))
}

pub fn usr(a: u32, v: &Value) -> Usr {
    let id = s(v, "id").unwrap_or("");
    Usr {
        id: if id.is_empty() { None } else { Some(rid(a, id)) },
        name: s(v, "username").unwrap_or("").to_string(),
        disp: s(v, "global_name").filter(|x| !x.is_empty()).map(str::to_string),
        av: Some(av(id, s(v, "avatar"), s(v, "discriminator"))),
        bot: v["bot"].as_bool().unwrap_or(false),
        col: None,
    }
}

const JOIN: &[&str] = &[
    "{} joined the party.",
    "{} is here.",
    "Welcome, {}. We hope you brought pizza.",
    "A wild {} appeared.",
    "{} just landed.",
    "{} just slid into the server.",
    "{} just showed up!",
    "Welcome {}. Say hi!",
    "{} hopped into the server.",
    "Everyone welcome {}!",
    "Glad you're here, {}.",
    "Good to see you, {}.",
    "Yay you made it, {}!",
];

fn shown(v: &Value) -> String {
    v.get("member")
        .and_then(|m| s(m, "nick"))
        .or_else(|| s(v, "global_name"))
        .or_else(|| s(v, "username"))
        .filter(|x| !x.is_empty())
        .unwrap_or("Someone")
        .to_string()
}

pub fn poll_txt(v: &Value) -> Option<String> {
    let p = v.get("poll")?;
    let q = s(&p["question"], "text").unwrap_or("Poll");
    let mut o = format!("**{q}**");
    for x in p.get("answers").and_then(|a| a.as_array()).into_iter().flatten() {
        let e = s(&x["poll_media"]["emoji"], "name").map(|e| format!("{e} ")).unwrap_or_default();
        o.push_str(&format!("\n- {e}{}", s(&x["poll_media"], "text").unwrap_or("")));
    }
    Some(o)
}

pub fn sys_txt(t: u64, v: &Value) -> Option<String> {
    let a = v.get("member").and_then(|m| s(m, "nick")).filter(|n| !n.is_empty()).map(str::to_string).unwrap_or_else(|| v.get("author").map(shown).unwrap_or_else(|| "Someone".into()));
    let c = s(v, "content").unwrap_or("");
    let m0 = v.get("mentions").and_then(|m| m.as_array()).and_then(|m| m.first()).map(shown).unwrap_or_else(|| "someone".into());
    let boost = |l: &str| {
        let n = c.parse::<u32>().unwrap_or(1);
        let x = if n > 1 { format!("{a} just boosted the server {n} times!") } else { format!("{a} just boosted the server!") };
        if l.is_empty() { x } else { format!("{x} The server has achieved {l}!") }
    };
    Some(match t {
        1 => format!("{a} added {m0} to the group."),
        2 if m0 == a => format!("{a} left the group."),
        2 => format!("{a} removed {m0} from the group."),
        3 => {
            let call = &v["call"];
            match (s(v, "timestamp").and_then(ts), s(call, "ended_timestamp").and_then(ts)) {
                (Some(b), Some(e)) => {
                    let min = ((e - b) / 60_000).max(1);
                    format!("{a} started a call that lasted {min} minute{}.", if min == 1 { "" } else { "s" })
                }
                _ => format!("{a} started a call."),
            }
        }
        4 => format!("{a} changed the channel name: {c}"),
        5 => format!("{a} changed the channel icon."),
        6 => format!("{a} pinned a message to this channel."),
        7 => {
            let id: u64 = s(v, "id").and_then(|i| i.parse().ok()).unwrap_or(0);
            let ms = (id >> 22) + 1_420_070_400_000;
            JOIN[(ms % JOIN.len() as u64) as usize].replace("{}", &a)
        }
        8 => boost(""),
        9 => boost("Level 1"),
        10 => boost("Level 2"),
        11 => boost("Level 3"),
        12 => format!("{a} has added {c} to this channel. Its most important updates will show up here."),
        18 => format!("{a} started a thread: {c}"),
        24 => "AutoMod has blocked a message.".into(),
        46 => "A poll has ended.".into(),
        _ => return None,
    })
}

pub fn mem_usr(a: u32, v: &Value) -> Usr {
    let mut u = usr(a, v);
    if let Some(n) = v.get("member").and_then(|m| s(m, "nick")).filter(|n| !n.is_empty()) {
        u.disp = Some(n.to_string());
    }
    u
}

pub fn emo(a: u32, v: &Value) -> Emo {
    let an = v["animated"].as_bool().unwrap_or(false);
    let id = s(v, "id");
    Emo {
        id: id.map(|i| rid(a, i)),
        name: s(v, "name").unwrap_or("").to_string(),
        url: id.map(|i| format!("{CDN}/emojis/{i}.{}?size=64", if an { "gif" } else { "png" })),
        anim: an,
    }
}

pub fn role(a: u32, r: &Value) -> Role {
    let c = r["color"].as_u64().unwrap_or(0) as u32;
    Role {
        id: rid(a, s(r, "id").unwrap_or("")),
        name: s(r, "name").unwrap_or("").to_string(),
        col: if c == 0 { None } else { Some(c) },
        pos: r["position"].as_i64().unwrap_or(0),
        perm: pu(&r["permissions"]),
        hoist: r["hoist"].as_bool().unwrap_or(false),
    }
}

pub fn gld(a: u32, g: &Value, pos: i64) -> Gld {
    let p = g.get("properties").unwrap_or(g);
    let id = s(g, "id").or_else(|| s(p, "id")).unwrap_or("");
    Gld {
        id: rid(a, id),
        name: s(p, "name").unwrap_or("").to_string(),
        icon: ic("icons", id, s(p, "icon")),
        ban: s(p, "banner").filter(|x| !x.is_empty()).map(|h| {
            format!("{CDN}/banners/{id}/{h}.{}?size=512", if h.starts_with("a_") { "gif" } else { "png" })
        }),
        owner: s(p, "owner_id").map(|o| rid(a, o)),
        pos,
        roles: g.get("roles").and_then(|x| x.as_array()).map(|r| r.iter().map(|r| role(a, r)).collect()).unwrap_or_default(),
        emos: g.get("emojis").and_then(|x| x.as_array()).map(|r| r.iter().map(|r| emo(a, r)).collect()).unwrap_or_default(),
    }
}

pub fn chk(t: u64) -> Option<ChK> {
    Some(match t {
        0 => ChK::Txt,
        1 => ChK::Dm,
        2 => ChK::Voice,
        3 => ChK::Grp,
        4 => ChK::Cat,
        5 => ChK::Ann,
        10..=12 => ChK::Thread,
        13 => ChK::Stage,
        15 | 16 => ChK::Forum,
        _ => return None,
    })
}

pub fn chan(a: u32, gid: Option<&str>, c: &Value, hid: bool, us: &HashMap<String, Usr>) -> Option<Chan> {
    let k = chk(c["type"].as_u64()?)?;
    let id = s(c, "id")?;
    let mut rcp: Vec<Usr> = vec![];
    if gid.is_none() {
        if let Some(r) = c.get("recipients").and_then(|x| x.as_array()) {
            rcp = r.iter().map(|u| usr(a, u)).collect();
        } else if let Some(r) = c.get("recipient_ids").and_then(|x| x.as_array()) {
            rcp = r
                .iter()
                .filter_map(|i| i.as_str())
                .map(|i| us.get(i).cloned().unwrap_or_else(|| Usr { id: Some(rid(a, i)), ..Default::default() }))
                .collect();
        }
    }
    let mut name = s(c, "name").unwrap_or("").to_string();
    if gid.is_none() && name.is_empty() {
        name = rcp.iter().map(|u| u.shown().to_string()).collect::<Vec<_>>().join(", ");
    }
    Some(Chan {
        id: rid(a, id),
        gld: gid.map(|g| rid(a, g)),
        par: s(c, "parent_id").map(|p| rid(a, p)),
        k,
        name,
        topic: s(c, "topic").filter(|x| !x.is_empty()).map(str::to_string),
        pos: c["position"].as_i64().unwrap_or(0),
        nsfw: c["nsfw"].as_bool().unwrap_or(false),
        hidden: hid,
        last: s(c, "last_message_id").map(|m| rid(a, m)),
        rcp,
        icon: if k == ChK::Grp { ic("channel-icons", id, s(c, "icon")) } else { None },
        enc: false,
        mute: false,
    })
}

pub fn att(a: &Value) -> Att {
    Att {
        url: s(a, "url").unwrap_or("").to_string(),
        name: s(a, "filename").unwrap_or("").to_string(),
        mime: s(a, "content_type").map(str::to_string),
        size: a["size"].as_u64().unwrap_or(0),
        w: a["width"].as_u64().map(|x| x as u32),
        h: a["height"].as_u64().map(|x| x as u32),
    }
}

pub fn stk(s_: &Value) -> Att {
    let id = s(s_, "id").unwrap_or("");
    let (u, m) = match s_["format_type"].as_u64().unwrap_or(1) {
        4 => (format!("https://media.discordapp.net/stickers/{id}.gif"), "image/gif"),
        3 => (format!("{CDN}/stickers/{id}.json"), "application/json"),
        2 => (format!("{CDN}/stickers/{id}.png"), "image/apng"),
        _ => (format!("{CDN}/stickers/{id}.png"), "image/png"),
    };
    Att { url: u, name: s(s_, "name").unwrap_or("").to_string(), mime: Some(m.into()), size: 0, w: Some(160), h: Some(160) }
}

pub fn gal(l: Vec<Emb>) -> Vec<Emb> {
    let mut o: Vec<Emb> = vec![];
    for e in l {
        let side = e.title.is_none() && e.desc.is_none() && e.img.is_some() && e.url.is_some();
        if side {
            if let Some(p) = o.iter_mut().rev().find(|p| p.url == e.url && p.img.is_some()) {
                if let Some(i) = e.img {
                    p.more.push(i);
                }
                continue;
            }
        }
        o.push(e);
    }
    o
}

pub fn emb(e: &Value) -> Emb {
    let u = |k: &str| e.get(k).and_then(|x| s(x, "proxy_url").or_else(|| s(x, "url"))).map(str::to_string);
    let n = |k: &str, f: &str| e.get(k).and_then(|x| s(x, f)).map(str::to_string);
    Emb {
        title: s(e, "title").map(str::to_string),
        desc: s(e, "description").map(str::to_string),
        url: s(e, "url").map(str::to_string),
        col: e["color"].as_u64().map(|c| c as u32),
        img: u("image"),
        thumb: u("thumbnail"),
        vid: u("video"),
        kind: s(e, "type").map(str::to_string),
        auth_ic: e.get("author").and_then(|x| s(x, "proxy_icon_url").or_else(|| s(x, "icon_url"))).map(str::to_string),
        auth_url: n("author", "url"),
        foot_ic: e.get("footer").and_then(|x| s(x, "proxy_icon_url").or_else(|| s(x, "icon_url"))).map(str::to_string),
        ts: s(e, "timestamp").and_then(ts),
        iw: ["image", "video", "thumbnail"].iter().find_map(|k| e[*k]["width"].as_u64()).unwrap_or(0) as u32,
        ih: ["image", "video", "thumbnail"].iter().find_map(|k| e[*k]["height"].as_u64()).unwrap_or(0) as u32,
        more: vec![],
        auth: n("author", "name"),
        foot: n("footer", "text"),
        prov: n("provider", "name"),
        fields: e
            .get("fields")
            .and_then(|f| f.as_array())
            .map(|f| {
                f.iter()
                    .map(|x| (s(x, "name").unwrap_or("").to_string(), s(x, "value").unwrap_or("").to_string(), x["inline"].as_bool().unwrap_or(false)))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

pub fn msg(a: u32, v: &Value) -> Option<Msg> {
    msg_p(a, v, true)
}

pub fn msg_p(a: u32, v: &Value, need: bool) -> Option<Msg> {
    let au = match v.get("author") {
        Some(a) => a.clone(),
        None if !need => Value::Null,
        None => return None,
    };
    let au = &au;
    let arr = |k: &str| v.get(k).and_then(|x| x.as_array()).cloned().unwrap_or_default();
    let t = v["type"].as_u64().unwrap_or(0);
    let fwd = v["message_reference"]["type"].as_u64() == Some(1);
    let snap = v.get("message_snapshots").and_then(|x| x.as_array()).and_then(|x| x.first()).and_then(|x| x.get("message")).filter(|_| fwd).unwrap_or(v);
    let sarr = |k: &str| snap.get(k).and_then(|x| x.as_array()).cloned().unwrap_or_default();
    Some(Msg {
        id: rid(a, s(v, "id")?),
        chan: rid(a, s(v, "channel_id")?),
        auth: if au.is_null() {
            Usr::default()
        } else {
            let mut u = usr(a, au);
            if let Some(n) = v.get("member").and_then(|m| s(m, "nick")).filter(|n| !n.is_empty()) {
                u.disp = Some(n.to_string());
            }
            u
        },
        body: sys_txt(t, v).unwrap_or_else(|| {
            let b = s(snap, "content").unwrap_or("");
            if b.is_empty() { poll_txt(v).unwrap_or_default() } else { b.to_string() }
        }),
        ts: s(v, "timestamp").and_then(ts).unwrap_or(0),
        ed: s(v, "edited_timestamp").and_then(ts),
        atts: sarr("attachments").iter().map(att).collect(),
        embs: if v["flags"].as_u64().unwrap_or(0) & 4 != 0 { vec![] } else { gal(sarr("embeds").iter().map(emb).collect()) },
        reacts: arr("reactions")
            .iter()
            .map(|r| React { emo: emo(a, &r["emoji"]), n: r["count"].as_u64().unwrap_or(0) as u32, me: r["me"].as_bool().unwrap_or(false) })
            .collect(),
        reply: if fwd { None } else { v.get("message_reference").and_then(|r| s(r, "message_id")).map(|m| rid(a, m)) },
        stks: arr("sticker_items").iter().map(stk).collect(),
        pin: v["pinned"].as_bool().unwrap_or(false),
        sys: !matches!(t, 0 | 19 | 20 | 21 | 23),
        del: false,
        hist: vec![],
        ments: arr("mentions").iter().map(|x| mem_usr(a, x)).collect(),
        fwd,
        rq: v.get("referenced_message").and_then(|r| r.get("author")).filter(|_| !fwd).map(|u| usr(a, u)),
        rqb: v.get("referenced_message").and_then(|r| s(r, "content")).filter(|_| !fwd).unwrap_or("").to_string(),
        aroles: v.get("member").and_then(|m| m.get("roles")).and_then(|r| r.as_array()).map(|r| r.iter().filter_map(|x| x.as_str()).map(|x| rid(a, x)).collect()).unwrap_or_default(),
    })
}

pub fn pres(v: &Value) -> Pres {
    let st = match s(v, "status") {
        Some("online") => St::On,
        Some("idle") => St::Idle,
        Some("dnd") => St::Dnd,
        _ => St::Off,
    };
    let cs = v.get("client_status");
    let has = |k: &str| cs.map(|c| c.get(k).is_some()).unwrap_or(false);
    let acts = v.get("activities").and_then(|x| x.as_array());
    let act = acts.and_then(|l| {
        let cust = l.iter().find(|x| x["type"].as_u64() == Some(4)).and_then(|x| {
            let st = s(x, "state").filter(|t| !t.is_empty());
            let em = x.get("emoji").filter(|e| e.get("id").map(|i| i.is_null()).unwrap_or(true)).and_then(|e| s(e, "name"));
            match (em, st) {
                (Some(e), Some(t)) => Some(format!("{e} {t}")),
                (None, Some(t)) => Some(t.to_string()),
                (Some(e), None) => Some(e.to_string()),
                _ => None,
            }
        });
        cust.or_else(|| {
            l.iter().filter(|x| x["type"].as_u64() != Some(4)).find_map(|x| {
                let n = s(x, "name")?;
                Some(match x["type"].as_u64().unwrap_or(0) {
                    1 => format!("Streaming {}", s(x, "details").unwrap_or(n)),
                    2 => format!("Listening to {n}"),
                    3 => format!("Watching {n}"),
                    5 => format!("Competing in {n}"),
                    _ => format!("Playing {n}"),
                })
            })
        })
    });
    Pres { st, act, mob: has("mobile"), web: has("web"), desk: has("desktop") }
}

pub fn mem(a: u32, gid: &str, m: &Value) -> Option<Mem> {
    let u = m.get("user")?;
    Some(Mem {
        usr: usr(a, u),
        gld: rid(a, gid),
        nick: s(m, "nick").map(str::to_string),
        roles: m.get("roles").and_then(|r| r.as_array()).map(|r| r.iter().filter_map(|x| x.as_str()).map(|x| rid(a, x)).collect()).unwrap_or_default(),
    })
}
