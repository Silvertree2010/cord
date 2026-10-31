use crate::wire as w;
use cord_core::*;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Cx {
    pub acc: u32,
    pub me: String,
    pub api: String,
    pub cdn: String,
    pub prx: String,
}

pub struct Lk<'a> {
    pub user: &'a dyn Fn(&str) -> Option<w::User>,
    pub emoji: &'a dyn Fn(&str) -> Option<w::Emoji>,
}

impl Cx {
    pub fn rid(&self, raw: &str) -> Rid {
        Rid::new(Svc::St, self.acc, raw)
    }
}

pub fn enc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            o.push(b as char);
        } else {
            o.push_str(&format!("%{b:02X}"));
        }
    }
    o
}

pub fn ulid_ms(id: &str) -> Option<i64> {
    const A: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    if id.len() < 10 {
        return None;
    }
    let mut v: i64 = 0;
    for c in id.bytes().take(10) {
        let p = A.iter().position(|a| *a == c.to_ascii_uppercase())?;
        v = v.checked_mul(32)?.checked_add(p as i64)?;
    }
    Some(v)
}

pub fn is_ulid(s: &str) -> bool {
    s.len() == 26 && ulid_ms(s).is_some() && s.bytes().all(|c| c.is_ascii_alphanumeric())
}

fn days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn iso_ms(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let n = |a: usize, z: usize| s.get(a..z)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, se) = (n(0, 4)?, n(5, 7)?, n(8, 10)?, n(11, 13)?, n(14, 16)?, n(17, 19)?);
    let mut ms = 0;
    let mut i = 19;
    if b.get(19) == Some(&b'.') {
        let mut j = 20;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        let f = &s[20..j];
        let f3: String = f.chars().chain("000".chars()).take(3).collect();
        ms = f3.parse::<i64>().ok()?;
        i = j;
    }
    let mut off = 0;
    if let Some(t) = s.get(i..) {
        if (t.starts_with('+') || t.starts_with('-')) && t.len() >= 6 {
            let sg = if t.starts_with('-') { -1 } else { 1 };
            let oh = t.get(1..3)?.parse::<i64>().ok()?;
            let om = t.get(4..6)?.parse::<i64>().ok()?;
            off = sg * (oh * 3600 + om * 60);
        }
    }
    Some(((days(y, mo, d) * 86400 + h * 3600 + mi * 60 + se - off) * 1000) + ms)
}

pub fn col(s: &str) -> Option<u32> {
    let h = s.trim().strip_prefix('#')?;
    match h.len() {
        6 => u32::from_str_radix(h, 16).ok(),
        3 => {
            let v = u32::from_str_radix(h, 16).ok()?;
            let (r, g, b) = ((v >> 8) & 15, (v >> 4) & 15, v & 15);
            Some((r * 17) << 16 | (g * 17) << 8 | b * 17)
        }
        8 => u32::from_str_radix(&h[..6], 16).ok(),
        _ => None,
    }
}

pub fn furl(cx: &Cx, f: &w::File) -> String {
    let tag = if f.tag.is_empty() { "attachments" } else { &f.tag };
    if f.filename.is_empty() {
        format!("{}/{}/{}", cx.cdn, tag, f.id)
    } else {
        format!("{}/{}/{}/{}", cx.cdn, tag, f.id, enc(&f.filename))
    }
}

pub fn prx(cx: &Cx, u: &str) -> String {
    if cx.prx.is_empty() {
        u.to_string()
    } else {
        format!("{}/proxy?url={}", cx.prx, enc(u))
    }
}

pub fn avatar(cx: &Cx, u: &w::User) -> String {
    match &u.avatar {
        Some(f) => furl(cx, f),
        None => format!("{}/users/{}/default_avatar", cx.api, u.id),
    }
}

pub fn usr(cx: &Cx, u: &w::User) -> Usr {
    Usr {
        id: Some(cx.rid(&u.id)),
        name: u.username.clone(),
        disp: u.display_name.clone(),
        av: Some(avatar(cx, u)),
        bot: u.bot.is_some(),
        col: None,
    }
}

pub fn pres(u: &w::User) -> Pres {
    let (mut st, act) = match &u.status {
        Some(s) => (
            match s.presence {
                Some(w::Presence::Idle) => St::Idle,
                Some(w::Presence::Busy) | Some(w::Presence::Focus) => St::Dnd,
                Some(w::Presence::Invisible) => St::Off,
                _ => St::On,
            },
            s.text.clone().filter(|t| !t.is_empty()),
        ),
        None => (St::On, None),
    };
    if !u.online {
        st = St::Off;
    }
    Pres { st, act, ..Default::default() }
}

pub fn rel(cx: &Cx, u: &w::User) -> Option<Rel> {
    let k = match u.relationship {
        w::Relation::Friend => RelK::Friend,
        w::Relation::Incoming => RelK::In,
        w::Relation::Outgoing => RelK::Out,
        w::Relation::Blocked => RelK::Block,
        _ => return None,
    };
    Some(Rel { usr: usr(cx, u), k })
}

pub fn emo(cx: &Cx, lk: &Lk, key: &str) -> Emo {
    if is_ulid(key) {
        let e = (lk.emoji)(key);
        Emo {
            id: Some(cx.rid(key)),
            name: e.as_ref().map(|e| e.name.clone()).unwrap_or_else(|| key.to_string()),
            url: Some(format!("{}/emojis/{}", cx.cdn, key)),
            anim: e.map(|e| e.animated).unwrap_or(false),
        }
    } else {
        Emo { id: None, name: key.to_string(), url: None, anim: false }
    }
}

pub fn emoji_key(e: &Emo) -> String {
    match &e.id {
        Some(i) => i.raw.to_string(),
        None => e.name.clone(),
    }
}

fn att(cx: &Cx, f: &w::File) -> Att {
    let (w, h) = match f.metadata {
        w::Meta::Image { width, height } | w::Meta::Video { width, height } => (Some(width), Some(height)),
        _ => (None, None),
    };
    Att {
        url: furl(cx, f),
        name: f.filename.clone(),
        mime: Some(f.content_type.clone()).filter(|m| !m.is_empty()),
        size: f.size.max(0) as u64,
        w,
        h,
    }
}

fn emb(cx: &Cx, e: &w::Embed) -> Option<Emb> {
    match e {
        w::Embed::Website(s) => Some(Emb {
            title: s.title.clone(),
            desc: s.description.clone(),
            url: s.url.clone(),
            col: s.colour.as_deref().and_then(col),
            img: s.image.as_ref().map(|i| prx(cx, &i.url)),
            vid: s.video.as_ref().map(|v| prx(cx, &v.url)),
            prov: s.site_name.clone(),
            ..Default::default()
        }),
        w::Embed::Image(i) => Some(Emb { img: Some(prx(cx, &i.url)), url: Some(i.url.clone()), ..Default::default() }),
        w::Embed::Video(v) => Some(Emb { vid: Some(prx(cx, &v.url)), url: Some(v.url.clone()), ..Default::default() }),
        w::Embed::Text(t) => Some(Emb {
            title: t.title.clone(),
            desc: t.description.clone(),
            url: t.url.clone(),
            col: t.colour.as_deref().and_then(col),
            img: t.media.as_ref().map(|m| furl(cx, m)),
            ..Default::default()
        }),
        w::Embed::Nope => None,
    }
}

fn nm(lk: &Lk, id: &str) -> String {
    (lk.user)(id)
        .map(|u| u.display_name.unwrap_or(u.username))
        .unwrap_or_else(|| id.to_string())
}

pub fn sys(lk: &Lk, s: &w::Sys) -> String {
    use w::Sys::*;
    match s {
        Text { content } => content.clone(),
        Added { id, by } => format!("{} added {}", nm(lk, by), nm(lk, id)),
        Remove { id, by } => format!("{} removed {}", nm(lk, by), nm(lk, id)),
        Joined { id } => format!("{} joined", nm(lk, id)),
        Left { id } => format!("{} left", nm(lk, id)),
        Kicked { id } => format!("{} was kicked", nm(lk, id)),
        Banned { id } => format!("{} was banned", nm(lk, id)),
        Renamed { name, by } => format!("{} renamed the channel to {}", nm(lk, by), name),
        DescChanged { by } => format!("{} changed the description", nm(lk, by)),
        IconChanged { by } => format!("{} changed the icon", nm(lk, by)),
        Owner { from, to } => format!("{} transferred ownership to {}", nm(lk, from), nm(lk, to)),
        Pinned { by, .. } => format!("{} pinned a message", nm(lk, by)),
        Unpinned { by, .. } => format!("{} unpinned a message", nm(lk, by)),
        Call { by } => format!("{} started a call", nm(lk, by)),
        Other => String::new(),
    }
}

pub fn author(cx: &Cx, m: &w::Message, u: &w::User) -> Usr {
    let mut a = usr(cx, u);
    if let Some(h) = &m.webhook {
        a.name = h.name.clone();
        a.bot = true;
        if let Some(av) = &h.avatar {
            a.av = Some(format!("{}/avatars/{}", cx.cdn, av));
        }
    }
    if let Some(mem) = &m.member {
        if let Some(n) = &mem.nickname {
            a.disp = Some(n.clone());
        }
        if let Some(f) = &mem.avatar {
            a.av = Some(furl(cx, f));
        }
    }
    if let Some(q) = &m.masquerade {
        if let Some(n) = &q.name {
            a.disp = Some(n.clone());
        }
        if let Some(av) = &q.avatar {
            a.av = Some(av.clone());
        }
        if let Some(c) = q.colour.as_deref().and_then(col) {
            a.col = Some(c);
        }
    }
    a
}

pub fn msg(cx: &Cx, lk: &Lk, m: &w::Message, u: &w::User) -> Msg {
    let body = match &m.system {
        Some(s) => sys(lk, s),
        None => m.content.clone().unwrap_or_default(),
    };
    Msg {
        id: cx.rid(&m.id),
        chan: cx.rid(&m.channel),
        auth: author(cx, m, u),
        body,
        ts: ulid_ms(&m.id).unwrap_or(0),
        ed: m.edited.as_deref().and_then(iso_ms),
        atts: m.attachments.iter().flatten().map(|f| att(cx, f)).collect(),
        embs: m.embeds.iter().flatten().filter_map(|e| emb(cx, e)).collect(),
        reacts: m
            .reactions
            .iter()
            .map(|(k, v)| React { emo: emo(cx, lk, k), n: v.len() as u32, me: v.iter().any(|x| *x == cx.me) })
            .collect(),
        reply: m.replies.as_ref().and_then(|r| r.first()).map(|r| cx.rid(r)),
        stks: vec![],
        pin: m.pinned.unwrap_or(false),
        sys: m.system.is_some(),
        del: false,
        hist: vec![],
    }
}

pub fn gld(cx: &Cx, s: &w::Server, emojis: &[w::Emoji], pos: i64) -> Gld {
    let mut roles: Vec<Role> = s
        .roles
        .iter()
        .map(|(id, r)| Role {
            id: cx.rid(id),
            name: r.name.clone(),
            col: r.colour.as_deref().and_then(col),
            pos: r.rank,
            perm: r.permissions.a as u64,
            hoist: r.hoist,
        })
        .collect();
    roles.sort_by_key(|r| r.pos);
    Gld {
        id: cx.rid(&s.id),
        name: s.name.clone(),
        icon: s.icon.as_ref().map(|f| furl(cx, f)),
        ban: s.banner.as_ref().map(|f| furl(cx, f)),
        owner: Some(cx.rid(&s.owner)),
        pos,
        roles,
        emos: emojis
            .iter()
            .filter(|e| matches!(&e.parent, w::Parent::Server { id } if *id == s.id))
            .map(|e| Emo {
                id: Some(cx.rid(&e.id)),
                name: e.name.clone(),
                url: Some(format!("{}/emojis/{}", cx.cdn, e.id)),
                anim: e.animated,
            })
            .collect(),
    }
}

pub fn cat_id(sid: &str, cid: &str) -> String {
    format!("{sid}/{cid}")
}

fn base(cx: &Cx, id: &str, k: ChK, name: &str) -> Chan {
    Chan {
        id: cx.rid(id),
        gld: None,
        par: None,
        k,
        name: name.to_string(),
        topic: None,
        pos: 0,
        nsfw: false,
        hidden: false,
        last: None,
        rcp: vec![],
        icon: None,
    }
}

pub fn chan(cx: &Cx, lk: &Lk, c: &w::Channel) -> Chan {
    let rcp = |ids: &[String]| -> Vec<Usr> {
        ids.iter()
            .map(|i| match (lk.user)(i) {
                Some(u) => usr(cx, &u),
                None => Usr { id: Some(cx.rid(i)), name: i.clone(), ..Default::default() },
            })
            .collect()
    };
    match c {
        w::Channel::SavedMessages { id, user } => {
            let mut o = base(cx, id, ChK::Dm, "Saved Messages");
            o.rcp = rcp(std::slice::from_ref(user));
            o
        }
        w::Channel::DirectMessage { id, active, recipients, last_message_id } => {
            let others: Vec<String> = recipients.iter().filter(|r| **r != cx.me).cloned().collect();
            let list = rcp(&others);
            let name = list.first().map(|u| u.shown().to_string()).unwrap_or_else(|| "Unknown".into());
            let mut o = base(cx, id, ChK::Dm, &name);
            o.rcp = list;
            o.hidden = !active;
            o.last = last_message_id.as_deref().map(|l| cx.rid(l));
            o
        }
        w::Channel::Group { id, name, description, recipients, icon, last_message_id, nsfw } => {
            let mut o = base(cx, id, ChK::Grp, name);
            o.topic = description.clone();
            o.rcp = rcp(recipients);
            o.icon = icon.as_ref().map(|f| furl(cx, f));
            o.last = last_message_id.as_deref().map(|l| cx.rid(l));
            o.nsfw = *nsfw;
            o
        }
        w::Channel::TextChannel { id, server, name, description, icon, last_message_id, nsfw, voice } => {
            let k = if voice.is_some() { ChK::Voice } else { ChK::Txt };
            let mut o = base(cx, id, k, name);
            o.gld = Some(cx.rid(server));
            o.topic = description.clone();
            o.icon = icon.as_ref().map(|f| furl(cx, f));
            o.last = last_message_id.as_deref().map(|l| cx.rid(l));
            o.nsfw = *nsfw;
            o
        }
        w::Channel::VoiceChannel { id, server, name, description, icon, nsfw } => {
            let mut o = base(cx, id, ChK::Voice, name);
            o.gld = Some(cx.rid(server));
            o.topic = description.clone();
            o.icon = icon.as_ref().map(|f| furl(cx, f));
            o.nsfw = *nsfw;
            o
        }
    }
}

pub fn srv_chans(cx: &Cx, lk: &Lk, s: &w::Server, all: &HashMap<String, w::Channel>) -> Vec<Chan> {
    let mut out = Vec::new();
    let mut parent: HashMap<&str, (String, usize)> = HashMap::new();
    let cats = s.categories.as_deref().unwrap_or(&[]);
    for (ci, c) in cats.iter().enumerate() {
        for (i, ch) in c.channels.iter().enumerate() {
            parent.insert(ch.as_str(), (cat_id(&s.id, &c.id), i));
        }
        let mut cc = base(cx, &cat_id(&s.id, &c.id), ChK::Cat, &c.title);
        cc.gld = Some(cx.rid(&s.id));
        cc.pos = 1000 + ci as i64;
        out.push(cc);
    }
    for (i, id) in s.channels.iter().enumerate() {
        let Some(ch) = all.get(id) else { continue };
        let mut c = chan(cx, lk, ch);
        match parent.get(id.as_str()) {
            Some((p, pi)) => {
                c.par = Some(cx.rid(p));
                c.pos = *pi as i64;
            }
            None => c.pos = i as i64,
        }
        out.push(c);
    }
    out
}

pub fn mem(cx: &Cx, lk: &Lk, m: &w::Member) -> Mem {
    let u = (lk.user)(&m.id.user).unwrap_or(w::User { id: m.id.user.clone(), username: m.id.user.clone(), ..Default::default() });
    let mut us = usr(cx, &u);
    if let Some(f) = &m.avatar {
        us.av = Some(furl(cx, f));
    }
    Mem {
        usr: us,
        gld: cx.rid(&m.id.server),
        nick: m.nickname.clone(),
        roles: m.roles.iter().map(|r| cx.rid(r)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cx() -> Cx {
        Cx {
            acc: 1,
            me: "ME".into(),
            api: "https://api.test".into(),
            cdn: "https://cdn.test".into(),
            prx: "https://prx.test".into(),
        }
    }

    fn user() -> w::User {
        serde_json::from_value(json!({
            "_id": "01HZZZZZZZZZZZZZZZZZZZZZZA", "username": "ann", "discriminator": "0001",
            "display_name": "Ann", "avatar": {"_id": "AV1", "tag": "avatars", "filename": "a b.png", "metadata": {"type": "Image", "width": 8, "height": 8}, "content_type": "image/png", "size": 12},
            "relationship": "Friend", "online": true, "status": {"text": "hi", "presence": "Busy"}
        }))
        .unwrap()
    }

    #[test]
    fn time() {
        assert_eq!(iso_ms("1970-01-01T00:00:01.500Z"), Some(1500));
        assert_eq!(iso_ms("2024-02-29T12:00:00Z"), Some(1709208000000));
        assert_eq!(iso_ms("2024-02-29T14:00:00.000+02:00"), Some(1709208000000));
        assert_eq!(ulid_ms("01ARZ3NDEKTSV4RRFFQ69G5FAV"), Some(1469922850259));
    }

    #[test]
    fn colors() {
        assert_eq!(col("#ff8800"), Some(0xff8800));
        assert_eq!(col("#f80"), Some(0xff8800));
        assert_eq!(col("red"), None);
    }

    #[test]
    fn user_map() {
        let c = cx();
        let u = user();
        let o = usr(&c, &u);
        assert_eq!(o.shown(), "Ann");
        assert_eq!(o.av.as_deref(), Some("https://cdn.test/avatars/AV1/a%20b.png"));
        let p = pres(&u);
        assert_eq!(p.st, St::Dnd);
        assert_eq!(p.act.as_deref(), Some("hi"));
        assert_eq!(rel(&c, &u).unwrap().k, RelK::Friend);
    }

    #[test]
    fn message_map() {
        let c = cx();
        let u = user();
        let m: w::Message = serde_json::from_value(json!({
            "_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV", "channel": "CH1", "author": "01HZZZZZZZZZZZZZZZZZZZZZZA",
            "content": "hello", "edited": "2024-02-29T12:00:00Z", "replies": ["R1"],
            "attachments": [{"_id": "F1", "tag": "attachments", "filename": "x.png", "metadata": {"type": "Image", "width": 4, "height": 3}, "content_type": "image/png", "size": 99}],
            "embeds": [{"type": "Website", "url": "https://e.x", "title": "T", "image": {"url": "https://e.x/i.png", "width": 1, "height": 1, "size": "Large"}, "site_name": "E", "colour": "#00ff00"}, {"type": "Whatever"}],
            "reactions": {"\u{1F44D}": ["ME", "u2"], "01ARZ3NDEKTSV4RRFFQ69G5FAW": ["u3"]},
            "masquerade": {"name": "Bot", "avatar": "https://m/a.png", "colour": "#102030"},
            "pinned": true
        }))
        .unwrap();
        let uf = |_: &str| None;
        let ef = |id: &str| {
            Some(w::Emoji { id: id.into(), parent: w::Parent::Detached, name: "party".into(), animated: true })
        };
        let lk = Lk { user: &uf, emoji: &ef };
        let o = msg(&c, &lk, &m, &u);
        assert_eq!(o.body, "hello");
        assert_eq!(o.ts, 1469922850259);
        assert_eq!(o.ed, Some(1709208000000));
        assert_eq!(o.atts[0].url, "https://cdn.test/attachments/F1/x.png");
        assert_eq!(o.atts[0].w, Some(4));
        assert_eq!(o.embs.len(), 1);
        assert_eq!(o.embs[0].img.as_deref(), Some("https://prx.test/proxy?url=https%3A%2F%2Fe.x%2Fi.png"));
        assert_eq!(o.embs[0].col, Some(0x00ff00));
        assert_eq!(o.reacts.len(), 2);
        assert!(o.reacts[1].me && o.reacts[1].n == 2 && o.reacts[1].emo.id.is_none());
        assert_eq!(o.reacts[0].emo.name, "party");
        assert!(o.reacts[0].emo.anim);
        assert_eq!(o.auth.disp.as_deref(), Some("Bot"));
        assert_eq!(o.auth.av.as_deref(), Some("https://m/a.png"));
        assert_eq!(o.auth.col, Some(0x102030));
        assert_eq!(o.reply.unwrap().raw.as_ref(), "R1");
        assert!(o.pin);
    }

    #[test]
    fn system_map() {
        let c = cx();
        let u = user();
        let m: w::Message = serde_json::from_value(json!({
            "_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV", "channel": "CH1", "author": "00000000000000000000000000",
            "system": {"type": "user_joined", "id": "01HZZZZZZZZZZZZZZZZZZZZZZA"}
        }))
        .unwrap();
        let uf = |_: &str| Some(user());
        let ef = |_: &str| None;
        let lk = Lk { user: &uf, emoji: &ef };
        let o = msg(&c, &lk, &m, &u);
        assert!(o.sys);
        assert_eq!(o.body, "Ann joined");
    }

    #[test]
    fn server_map() {
        let c = cx();
        let s: w::Server = serde_json::from_value(json!({
            "_id": "S1", "owner": "O1", "name": "Srv", "channels": ["C1", "C2", "C3"],
            "categories": [{"id": "cat", "title": "Cat", "channels": ["C2"]}],
            "roles": {"R1": {"name": "Mod", "permissions": {"a": 5, "d": 0}, "colour": "#ff0000", "hoist": true, "rank": 1}},
            "icon": {"_id": "I1", "tag": "icons", "filename": "i.png"}
        }))
        .unwrap();
        let ch: HashMap<String, w::Channel> = [
            json!({"_id": "C1", "channel_type": "TextChannel", "server": "S1", "name": "gen"}),
            json!({"_id": "C2", "channel_type": "TextChannel", "server": "S1", "name": "vc", "voice": {}}),
            json!({"_id": "C3", "channel_type": "VoiceChannel", "server": "S1", "name": "old"}),
        ]
        .into_iter()
        .map(|v| {
            let c: w::Channel = serde_json::from_value(v).unwrap();
            (c.id().to_string(), c)
        })
        .collect();
        let uf = |_: &str| None;
        let ef = |_: &str| None;
        let lk = Lk { user: &uf, emoji: &ef };
        let em = vec![w::Emoji { id: "E1".into(), parent: w::Parent::Server { id: "S1".into() }, name: "e".into(), animated: false }];
        let g = gld(&c, &s, &em, 3);
        assert_eq!(g.roles[0].col, Some(0xff0000));
        assert_eq!(g.roles[0].perm, 5);
        assert_eq!(g.emos.len(), 1);
        assert_eq!(g.icon.as_deref(), Some("https://cdn.test/icons/I1/i.png"));
        let cs = srv_chans(&c, &lk, &s, &ch);
        assert_eq!(cs.len(), 4);
        let cat = cs.iter().find(|x| x.k == ChK::Cat).unwrap();
        let vc = cs.iter().find(|x| x.name == "vc").unwrap();
        assert_eq!(vc.par.as_ref(), Some(&cat.id));
        assert_eq!(vc.k, ChK::Voice);
        assert!(cs.iter().find(|x| x.name == "gen").unwrap().par.is_none());
        assert_eq!(cs.iter().find(|x| x.name == "old").unwrap().k, ChK::Voice);
    }

    #[test]
    fn dm_map() {
        let c = Cx { me: "01HZZZZZZZZZZZZZZZZZZZZZZB".into(), ..cx() };
        let ch: w::Channel = serde_json::from_value(json!({
            "_id": "D1", "channel_type": "DirectMessage", "active": false,
            "recipients": ["01HZZZZZZZZZZZZZZZZZZZZZZA", "01HZZZZZZZZZZZZZZZZZZZZZZB"], "last_message_id": "M9"
        }))
        .unwrap();
        let uf = |_: &str| Some(user());
        let ef = |_: &str| None;
        let lk = Lk { user: &uf, emoji: &ef };
        let o = chan(&c, &lk, &ch);
        assert_eq!(o.k, ChK::Dm);
        assert!(o.hidden);
        assert_eq!(o.name, "Ann");
        assert_eq!(o.last.unwrap().raw.as_ref(), "M9");
    }

    #[test]
    fn events_parse() {
        let e: w::Event = serde_json::from_value(json!({"type": "ChannelStartTyping", "id": "C", "user": "U"})).unwrap();
        assert!(matches!(e, w::Event::ChannelStartTyping { .. }));
        let e: w::Event = serde_json::from_value(json!({"type": "Nonsense", "x": 1})).unwrap();
        assert!(matches!(e, w::Event::Unknown));
        let e: w::Event = serde_json::from_value(json!({"type": "Bulk", "v": [{"type": "Pong", "data": 0}]})).unwrap();
        assert!(matches!(e, w::Event::Bulk { .. }));
        let e: w::Event = serde_json::from_value(json!({"type": "ChannelCreate", "_id": "C", "channel_type": "Group", "name": "g", "recipients": []})).unwrap();
        assert!(matches!(e, w::Event::ChannelCreate(_)));
    }
}
