use crate::{Gs, Sh, map::*, perm};
use cord_core::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn sn(x: &str) -> u64 {
    x.parse().unwrap_or(0)
}

fn arr(v: &Value, k: &str) -> Vec<Value> {
    v.get(k).and_then(|x| x.as_array()).cloned().unwrap_or_default()
}

pub fn ents(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(a) => a.clone(),
        Value::Object(_) => arr(v, "entries"),
        _ => vec![],
    }
}

fn out(sh: &Sh, tx: &Tx, mut e: Ev) {
    if let Ev::Chan(c) = &mut e {
        {
            let mut lm = sh.lm.lock().unwrap();
            let k = c.id.raw.to_string();
            match (lm.get(&k), &c.last) {
                (Some(o), Some(l)) if sn(o) > sn(&l.raw) => c.last = Some(rid(sh.a, o)),
                (Some(o), None) => c.last = Some(rid(sh.a, o)),
                (_, Some(l)) => {
                    lm.insert(k, l.raw.to_string());
                }
                _ => {}
            }
        }
        if let Some(g) = &c.gld {
            sh.cg.lock().unwrap().insert(c.id.raw.to_string(), g.raw.to_string());
        }
    }
    let _ = tx.send(e);
}

fn is_th(c: &Value) -> bool {
    matches!(c["type"].as_u64(), Some(10..=12))
}

pub fn gs_evs(a: u32, gs: &Gs, uid: &str) -> Vec<Ev> {
    let gid = &*gs.g.id.raw;
    let mut vis: HashMap<String, bool> = HashMap::new();
    for c in gs.chs.iter().filter(|c| !is_th(c)) {
        vis.insert(s(c, "id").unwrap_or("").to_string(), perm::view(&gs.g, &gs.my, uid, c));
    }
    let eu = HashMap::new();
    let mut v = vec![Ev::Gld(gs.g.clone())];
    for c in &gs.chs {
        let vw = if is_th(c) { s(c, "parent_id").and_then(|p| vis.get(p)).copied().unwrap_or(true) } else { vis.get(s(c, "id").unwrap_or("")).copied().unwrap_or(true) };
        if let Some(c) = chan(a, Some(gid), c, !vw, &eu) {
            v.push(Ev::Chan(c));
        }
    }
    v
}

fn ldg(sh: &Sh, tx: &Tx, g: &Value, pos: i64, my: Option<Vec<String>>) {
    let a = sh.a;
    let gl = gld(a, g, pos);
    let gid = gl.id.raw.to_string();
    let gid_s = gid.clone();
    let uid = sh.me.lock().unwrap().clone();
    let mut chs = arr(g, "channels");
    chs.extend(arr(g, "threads"));
    let evs = {
        let mut m = sh.gs.lock().unwrap();
        let my = my
            .or_else(|| {
                arr(g, "members")
                    .iter()
                    .find(|x| x.get("user").and_then(|u| s(u, "id")) == Some(uid.as_str()))
                    .map(|x| arr(x, "roles").iter().filter_map(|r| r.as_str().map(str::to_string)).collect())
            })
            .or_else(|| m.get(&gid).map(|x| x.my.clone()))
            .unwrap_or_default();
        let gs = Gs { g: gl, my, chs };
        let e = gs_evs(a, &gs, &uid);
        m.insert(gid, gs);
        e
    };
    for e in evs {
        out(sh, tx, e);
    }
    crate::vc::guild_vs(sh, tx, &gid_s, g);
}

fn refresh(sh: &Sh, tx: &Tx, gid: &str) {
    let uid = sh.me.lock().unwrap().clone();
    let evs = sh.gs.lock().unwrap().get(gid).map(|g| gs_evs(sh.a, g, &uid));
    for e in evs.into_iter().flatten() {
        out(sh, tx, e);
    }
}

fn one(sh: &Sh, tx: &Tx, gid: Option<&str>, c: &Value, del: bool) {
    let a = sh.a;
    let id = s(c, "id").unwrap_or("").to_string();
    let Some(gid) = gid else {
        let us = sh.us.lock().unwrap().clone();
        if let Some(c) = chan(a, None, c, false, &us) {
            out(sh, tx, Ev::Chan(c));
        }
        return;
    };
    let hid = {
        let mut m = sh.gs.lock().unwrap();
        let uid = sh.me.lock().unwrap().clone();
        match m.get_mut(gid) {
            Some(gs) => {
                gs.chs.retain(|x| s(x, "id") != Some(id.as_str()));
                if !del {
                    gs.chs.push(c.clone());
                }
                let t = if is_th(c) { s(c, "parent_id").and_then(|p| gs.chs.iter().find(|x| s(x, "id") == Some(p))).unwrap_or(c) } else { c };
                !perm::view(&gs.g, &gs.my, &uid, t)
            }
            None => false,
        }
    };
    if del {
        out(sh, tx, Ev::ChanDel(rid(a, &id)));
    } else if let Some(c) = chan(a, Some(gid), c, hid, &HashMap::new()) {
        out(sh, tx, Ev::Chan(c));
    }
}

fn rcol(sh: &Sh, gid: &str, mb: &Value) -> Option<u32> {
    let m = sh.gs.lock().unwrap();
    let g = m.get(gid)?;
    let rs = arr(mb, "roles");
    g.g.roles
        .iter()
        .filter(|r| r.col.is_some() && rs.iter().any(|x| x.as_str() == Some(&*r.id.raw)))
        .max_by_key(|r| r.pos)
        .and_then(|r| r.col)
}

fn usr_of(sh: &Sh, d: &Value) -> Usr {
    if let Some(u) = d.get("member").and_then(|m| m.get("user")) {
        return usr(sh.a, u);
    }
    let id = s(d, "user_id").unwrap_or("");
    sh.us.lock().unwrap().get(id).cloned().unwrap_or(Usr { id: Some(rid(sh.a, id)), ..Default::default() })
}

fn ready(sh: &Sh, tx: &Tx, d: &Value) {
    let a = sh.a;
    {
        let mut u = sh.us.lock().unwrap();
        for x in arr(d, "users") {
            if let Some(i) = s(&x, "id") {
                u.insert(i.to_string(), usr(a, &x));
            }
        }
    }
    let me = usr(a, &d["user"]);
    let uid = s(&d["user"], "id").unwrap_or("").to_string();
    *sh.me.lock().unwrap() = uid.clone();
    sh.us.lock().unwrap().insert(uid.clone(), me.clone());
    {
        let mut s_ = sh.ss.lock().unwrap();
        s_.id = s(d, "session_id").map(str::to_string);
        s_.url = s(d, "resume_gateway_url").map(str::to_string);
    }
    let _ = tx.send(Ev::Ready(Acc { id: a, svc: Svc::Dc, me }));
    let _ = tx.send(Ev::Conn { acc: a, up: true });
    {
        let mut mu = sh.mu.lock().unwrap();
        for e in ents(&d["user_guild_settings"]) {
            if e["muted"].as_bool() == Some(true) {
                if let Some(g) = s(&e, "guild_id") {
                    mu.insert(g.to_string());
                }
            }
            for o in arr(&e, "channel_overrides") {
                if o["muted"].as_bool() == Some(true) {
                    if let Some(c) = s(&o, "channel_id") {
                        mu.insert(c.to_string());
                    }
                }
            }
        }
    }
    let mm = arr(d, "merged_members");
    for (i, g) in arr(d, "guilds").iter().enumerate() {
        if g["unavailable"].as_bool() == Some(true) {
            continue;
        }
        let my = mm.get(i).and_then(|l| {
            let l = l.as_array()?;
            let m = l.iter().find(|x| s(x, "user_id") == Some(uid.as_str())).or_else(|| l.first())?;
            Some(arr(m, "roles").iter().filter_map(|r| r.as_str().map(str::to_string)).collect::<Vec<_>>())
        });
        ldg(sh, tx, g, i as i64, Some(my.unwrap_or_default()));
    }
    {
        let us = sh.us.lock().unwrap().clone();
        for c in arr(d, "private_channels") {
            if let Some(c) = chan(a, None, &c, false, &us) {
                out(sh, tx, Ev::Chan(c));
            }
        }
        for r in arr(d, "relationships") {
            let k = match r["type"].as_u64() {
                Some(1) => RelK::Friend,
                Some(2) => RelK::Block,
                Some(3) => RelK::In,
                Some(4) => RelK::Out,
                _ => continue,
            };
            let u = match r.get("user") {
                Some(u) => usr(a, u),
                None => {
                    let i = s(&r, "user_id").or_else(|| s(&r, "id")).unwrap_or("");
                    us.get(i).cloned().unwrap_or(Usr { id: Some(rid(a, i)), ..Default::default() })
                }
            };
            let _ = tx.send(Ev::Rel(Rel { usr: u, k }));
        }
    }
    for e in ents(&d["read_state"]) {
        let (Some(id), la) = (s(&e, "id"), s(&e, "last_message_id").unwrap_or("0")) else { continue };
        if e["read_state_type"].as_u64().is_some_and(|t| t != 0) {
            continue;
        }
        let ment = e["mention_count"].as_u64().unwrap_or(0) as u32;
        sh.ak.lock().unwrap().insert(id.to_string(), (la.to_string(), ment));
        let lm = sh.lm.lock().unwrap().get(id).cloned().unwrap_or_default();
        let g = sh.cg.lock().unwrap().get(id).cloned();
        let mt = {
            let mu = sh.mu.lock().unwrap();
            mu.contains(id) || g.map(|g| mu.contains(&g)).unwrap_or(false)
        };
        let n = if !mt && sn(&lm) > sn(la) { 1 } else { 0 };
        if n > 0 || ment > 0 {
            let _ = tx.send(Ev::Unread { chan: rid(a, id), n, ment });
        }
    }
}

fn supp(sh: &Sh, tx: &Tx, d: &Value) {
    let mp = &d["merged_presences"];
    let mut l = arr(mp, "friends");
    for g in arr(mp, "guilds") {
        if let Some(x) = g.as_array() {
            l.extend(x.iter().cloned());
        }
    }
    for p in l {
        if let Some(u) = s(&p, "user_id") {
            let _ = tx.send(Ev::Pres { usr: rid(sh.a, u), p: pres(&p) });
        }
    }
}

pub fn dis(sh: &Sh, tx: &Tx, t: &str, d: Value) {
    let a = sh.a;
    match t {
        "READY" => ready(sh, tx, &d),
        "READY_SUPPLEMENTAL" => supp(sh, tx, &d),
        "VOICE_STATE_UPDATE" => crate::vc::vsu(sh, tx, &d),
        "VOICE_SERVER_UPDATE" => crate::vc::vsv(sh, &d),
        "STREAM_CREATE" | "STREAM_SERVER_UPDATE" | "STREAM_DELETE" => sh.st.on(sh, t, &d),
        "CALL_CREATE" | "CALL_UPDATE" => crate::vc::call(sh, tx, &d),
        "RESUMED" => {
            let _ = tx.send(Ev::Conn { acc: a, up: true });
        }
        "MESSAGE_CREATE" => {
            if let Some(mut m) = msg(a, &d) {
                sh.lm.lock().unwrap().insert(m.chan.raw.to_string(), m.id.raw.to_string());
                if let Some(mb) = d.get("member") {
                    if let Some(n) = s(mb, "nick") {
                        m.auth.disp = Some(n.to_string());
                    }
                    if let Some(g) = s(&d, "guild_id") {
                        m.auth.col = rcol(sh, g, mb);
                    }
                }
                if let Some(i) = &m.auth.id {
                    sh.us.lock().unwrap().insert(i.raw.to_string(), m.auth.clone());
                }
                let _ = tx.send(Ev::Msg(m));
            }
        }
        "MESSAGE_UPDATE" => {
            if let Some(m) = msg_p(a, &d, false) {
                let _ = tx.send(Ev::MsgUpd(m));
            }
        }
        "MESSAGE_DELETE" => {
            if let (Some(c), Some(i)) = (s(&d, "channel_id"), s(&d, "id")) {
                let _ = tx.send(Ev::MsgDel { chan: rid(a, c), id: rid(a, i) });
            }
        }
        "MESSAGE_DELETE_BULK" => {
            if let Some(c) = s(&d, "channel_id") {
                for i in arr(&d, "ids") {
                    if let Some(i) = i.as_str() {
                        let _ = tx.send(Ev::MsgDel { chan: rid(a, c), id: rid(a, i) });
                    }
                }
            }
        }
        "TYPING_START" => {
            if let Some(c) = s(&d, "channel_id") {
                let u = usr_of(sh, &d);
                if u.id.as_ref().map(|i| *i.raw == *sh.me.lock().unwrap()).unwrap_or(false) {
                    return;
                }
                let _ = tx.send(Ev::Typing { chan: rid(a, c), usr: u });
            }
        }
        "PRESENCE_UPDATE" => {
            if let Some(u) = d.get("user").and_then(|u| s(u, "id")) {
                let _ = tx.send(Ev::Pres { usr: rid(a, u), p: pres(&d) });
            }
        }
        "GUILD_CREATE" => {
            if d["unavailable"].as_bool() != Some(true) {
                let p = sh.gs.lock().unwrap().len() as i64;
                ldg(sh, tx, &d, p, None);
            }
        }
        "GUILD_UPDATE" => {
            let id = s(&d, "id").unwrap_or("").to_string();
            let uid = sh.me.lock().unwrap().clone();
            let evs = {
                let mut m = sh.gs.lock().unwrap();
                m.get_mut(&id).map(|gs| {
                    let n = gld(a, &d, gs.g.pos);
                    gs.g.name = n.name;
                    gs.g.icon = n.icon;
                    gs.g.ban = n.ban;
                    gs.g.owner = n.owner;
                    if d.get("roles").is_some() {
                        gs.g.roles = n.roles;
                    }
                    if d.get("emojis").is_some() {
                        gs.g.emos = n.emos;
                    }
                    gs_evs(a, gs, &uid)
                })
            };
            for e in evs.into_iter().flatten() {
                out(sh, tx, e);
            }
        }
        "GUILD_DELETE" => {
            if d["unavailable"].as_bool() != Some(true) {
                if let Some(i) = s(&d, "id") {
                    sh.gs.lock().unwrap().remove(i);
                    let _ = tx.send(Ev::GldDel(rid(a, i)));
                }
            }
        }
        "GUILD_EMOJIS_UPDATE" => {
            let Some(g) = s(&d, "guild_id") else { return };
            let ev = sh.gs.lock().unwrap().get_mut(g).map(|gs| {
                gs.g.emos = arr(&d, "emojis").iter().map(|e| emo(a, e)).collect();
                Ev::Gld(gs.g.clone())
            });
            if let Some(e) = ev {
                let _ = tx.send(e);
            }
        }
        "GUILD_MEMBER_UPDATE" => {
            let (Some(g), Some(u)) = (s(&d, "guild_id"), d.get("user").and_then(|u| s(u, "id"))) else { return };
            if u != *sh.me.lock().unwrap() {
                return;
            }
            if let Some(gs) = sh.gs.lock().unwrap().get_mut(g) {
                gs.my = arr(&d, "roles").iter().filter_map(|r| r.as_str().map(str::to_string)).collect();
            }
            refresh(sh, tx, g);
        }
        "GUILD_ROLE_CREATE" | "GUILD_ROLE_UPDATE" => {
            let Some(g) = s(&d, "guild_id") else { return };
            let r = role(a, &d["role"]);
            if let Some(gs) = sh.gs.lock().unwrap().get_mut(g) {
                gs.g.roles.retain(|x| x.id != r.id);
                gs.g.roles.push(r);
            }
            refresh(sh, tx, g);
        }
        "GUILD_ROLE_DELETE" => {
            let (Some(g), Some(r)) = (s(&d, "guild_id"), s(&d, "role_id")) else { return };
            if let Some(gs) = sh.gs.lock().unwrap().get_mut(g) {
                gs.g.roles.retain(|x| *x.id.raw != *r);
            }
            refresh(sh, tx, g);
        }
        "CHANNEL_CREATE" | "CHANNEL_UPDATE" | "THREAD_CREATE" | "THREAD_UPDATE" => one(sh, tx, s(&d, "guild_id"), &d, false),
        "CHANNEL_DELETE" | "THREAD_DELETE" => one(sh, tx, s(&d, "guild_id"), &d, true),
        "RELATIONSHIP_ADD" => {
            let k = match d["type"].as_u64() {
                Some(1) => RelK::Friend,
                Some(2) => RelK::Block,
                Some(3) => RelK::In,
                Some(4) => RelK::Out,
                _ => return,
            };
            if let Some(u) = d.get("user") {
                let _ = tx.send(Ev::Rel(Rel { usr: usr(a, u), k }));
            }
        }
        "RELATIONSHIP_REMOVE" => {
            if let Some(i) = s(&d, "id") {
                let _ = tx.send(Ev::RelDel(rid(a, i)));
            }
        }
        "MESSAGE_ACK" => {
            if let Some(c) = s(&d, "channel_id") {
                let ment = d["mention_count"].as_u64().unwrap_or(0) as u32;
                if let Some(m) = s(&d, "message_id") {
                    sh.ak.lock().unwrap().insert(c.to_string(), (m.to_string(), ment));
                }
                let _ = tx.send(Ev::Ack { chan: rid(a, c) });
                let _ = tx.send(Ev::Unread { chan: rid(a, c), n: 0, ment });
            }
        }
        "GUILD_MEMBER_LIST_UPDATE" => {
            let Some(g) = s(&d, "guild_id") else { return };
            for op in arr(&d, "ops") {
                let mut its = arr(&op, "items");
                if let Some(i) = op.get("item") {
                    its.push(i.clone());
                }
                for it in its {
                    let Some(m) = it.get("member") else { continue };
                    if let Some(x) = mem(a, g, m) {
                        if let Some(p) = m.get("presence") {
                            if let Some(i) = &x.usr.id {
                                let _ = tx.send(Ev::Pres { usr: i.clone(), p: pres(p) });
                            }
                        }
                        let _ = tx.send(Ev::Mem(x));
                    }
                }
            }
        }
        _ => {}
    }
}
