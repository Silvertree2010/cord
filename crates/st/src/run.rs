use crate::api::ids;
use crate::map::{self, Cx};
use crate::wire as w;
use crate::{Cache, I};
use anyhow::Result;
use cord_core::*;
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as Wm;

enum Fin {
    Stop,
    Retry,
}

fn snake(s: &str) -> String {
    let mut o = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                o.push('_');
            }
            o.extend(c.to_lowercase());
        } else {
            o.push(c);
        }
    }
    o
}

fn merge(base: &mut Value, data: &Value, clear: &[String]) {
    if let (Some(b), Some(d)) = (base.as_object_mut(), data.as_object()) {
        for (k, v) in d {
            b.insert(k.clone(), v.clone());
        }
        for c in clear {
            match c.as_str() {
                "StatusText" => {
                    if let Some(s) = b.get_mut("status").and_then(|s| s.as_object_mut()) {
                        s.remove("text");
                    }
                }
                "StatusPresence" => {
                    if let Some(s) = b.get_mut("status").and_then(|s| s.as_object_mut()) {
                        s.remove("presence");
                    }
                }
                _ => {
                    b.remove(&snake(c));
                }
            }
        }
    }
}

fn gid(v: &Value) -> String {
    v.get("_id").or_else(|| v.get("id")).and_then(|x| x.as_str()).unwrap_or_default().to_string()
}

fn flat(e: w::Event, out: &mut Vec<w::Event>) {
    match e {
        w::Event::Bulk { v } => v.into_iter().for_each(|x| flat(x, out)),
        o => out.push(o),
    }
}

impl I {
    pub async fn run(&self, tx: Tx) {
        let mut back = 1u64;
        loop {
            let mut ready = false;
            let r = self.session(&tx, &mut ready).await;
            self.wtx.lock().unwrap().take();
            let _ = tx.send(Ev::Conn { acc: self.acc, up: false });
            match r {
                Ok(Fin::Stop) => return,
                Ok(Fin::Retry) => {}
                Err(e) => {
                    tracing::warn!("st ws: {e}");
                    let _ = tx.send(Ev::Err { acc: self.acc, msg: e.to_string() });
                }
            }
            if ready {
                back = 1;
            }
            tokio::time::sleep(Duration::from_secs(back)).await;
            back = (back * 2).min(30);
        }
    }

    async fn session(&self, tx: &Tx, ready: &mut bool) -> Result<Fin> {
        let cfg = self.cfg().await?;
        let url = format!("{}/?version=1&format=json", cfg.ws.trim_end_matches('/'));
        let (ws, _) = tokio_tungstenite::connect_async(url.as_str()).await?;
        let (mut sink, mut st) = ws.split();
        sink.send(Wm::text(json!({"type": "Authenticate", "token": self.tok}).to_string())).await?;
        let (otx, mut orx) = mpsc::unbounded_channel::<String>();
        *self.wtx.lock().unwrap() = Some(otx);
        let mut tick = tokio::time::interval(Duration::from_secs(20));
        let mut last = Instant::now();
        loop {
            tokio::select! {
                m = st.next() => {
                    let Some(m) = m else { return Ok(Fin::Retry) };
                    last = Instant::now();
                    match m? {
                        Wm::Text(t) => {
                            let ev = match serde_json::from_str::<w::Event>(t.as_str()) {
                                Ok(e) => e,
                                Err(e) => {
                                    tracing::debug!("st parse: {e}");
                                    continue;
                                }
                            };
                            let mut list = vec![];
                            flat(ev, &mut list);
                            for e in list {
                                if matches!(e, w::Event::Ready { .. }) {
                                    *ready = true;
                                }
                                if self.handle(e, tx).await? {
                                    return Ok(Fin::Stop);
                                }
                            }
                        }
                        Wm::Close(_) => return Ok(Fin::Retry),
                        _ => {}
                    }
                }
                Some(o) = orx.recv() => sink.send(Wm::text(o)).await?,
                _ = tick.tick() => {
                    if last.elapsed() > Duration::from_secs(70) {
                        return Ok(Fin::Retry);
                    }
                    sink.send(Wm::text(json!({"type": "Ping", "data": 0}).to_string())).await?;
                }
            }
        }
    }

    fn put(&self, f: impl FnOnce(&mut Cache)) {
        f(&mut self.c.lock().unwrap());
    }

    fn srv_evs(&self, cx: &Cx, sid: &str, with_gld: bool) -> Vec<Ev> {
        self.view(|c, lk| {
            let Some(s) = c.server(sid) else { return vec![] };
            let mut o = vec![];
            if with_gld {
                let pos = c.spos.get(sid).copied().unwrap_or(0);
                o.push(Ev::Gld(map::gld(cx, &s, &c.emojis_of(sid), pos)));
            }
            o.extend(map::srv_chans(cx, lk, &s, &c.chans_of(sid)).into_iter().map(Ev::Chan));
            o
        })
    }

    fn chan_evs(&self, cx: &Cx, id: &str) -> Vec<Ev> {
        let sid = self.c.lock().unwrap().chan(id).and_then(|c| c.server().map(String::from));
        if let Some(s) = sid {
            return self.srv_evs(cx, &s, false);
        }
        self.view(|c, lk| c.chan(id).map(|ch| vec![Ev::Chan(map::chan(cx, lk, &ch))]).unwrap_or_default())
    }

    async fn ready(&self, e: w::Event, tx: &Tx) -> Result<()> {
        let w::Event::Ready { users, servers, channels, members, emojis } = e else { return Ok(()) };
        self.put(|c| {
            let me = users
                .iter()
                .find(|u| u.get("relationship").and_then(|r| r.as_str()) == Some("User"))
                .map(gid)
                .unwrap_or_default();
            *c = Cache { me, ..Default::default() };
            for u in &users {
                c.users.insert(gid(u), u.clone());
            }
            for (i, s) in servers.iter().enumerate() {
                c.spos.insert(gid(s), i as i64);
                c.servers.insert(gid(s), s.clone());
            }
            for ch in &channels {
                c.chans.insert(gid(ch), ch.clone());
            }
            for e in &emojis {
                c.emojis.insert(gid(e), e.clone());
            }
            for m in &members {
                let k = m.get("_id").cloned().unwrap_or_default();
                let s = k.get("server").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                let u = k.get("user").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                c.mems.insert((s, u), m.clone());
            }
        });
        let cx = self.cx().await?;
        let _ = tx.send(Ev::Conn { acc: self.acc, up: true });
        let (me, mut evs) = self.view(|c, _| {
            let me = c.user(&cx.me).map(|u| map::usr(&cx, &u)).unwrap_or_default();
            let mut o = vec![];
            for v in c.users.values() {
                let Ok(u) = serde_json::from_value::<w::User>(v.clone()) else { continue };
                if u.id == cx.me {
                    continue;
                }
                if let Some(r) = map::rel(&cx, &u) {
                    o.push(Ev::Rel(r));
                }
                if u.online || u.status.is_some() {
                    o.push(Ev::Pres { usr: cx.rid(&u.id), p: map::pres(&u) });
                }
            }
            (me, o)
        });
        let _ = tx.send(Ev::Ready(Acc { id: self.acc, svc: Svc::St, me }));
        let sids: Vec<String> = self.c.lock().unwrap().servers.keys().cloned().collect();
        for s in sids {
            evs.extend(self.srv_evs(&cx, &s, true));
        }
        let dms: Vec<String> = {
            let c = self.c.lock().unwrap();
            c.chans.iter().filter(|(_, v)| v.get("server").is_none()).map(|(k, _)| k.clone()).collect()
        };
        for d in dms {
            evs.extend(self.chan_evs(&cx, &d));
        }
        for e in evs {
            let _ = tx.send(e);
        }
        if let Ok(un) = self.unreads().await {
            for u in un {
                let ch = u.id.channel;
                let last = self.c.lock().unwrap().chan(&ch).and_then(|c| match c {
                    w::Channel::DirectMessage { last_message_id, .. }
                    | w::Channel::Group { last_message_id, .. }
                    | w::Channel::TextChannel { last_message_id, .. } => last_message_id,
                    _ => None,
                });
                let n = match (&last, &u.last_id) {
                    (Some(l), Some(r)) if l == r => 0,
                    (Some(_), _) => 1,
                    _ => 0,
                };
                let ment = u.mentions.len() as u32;
                if n > 0 || ment > 0 {
                    self.put(|c| {
                        c.unread.insert(ch.clone(), (n, ment));
                    });
                    let _ = tx.send(Ev::Unread { chan: cx.rid(&ch), n, ment });
                }
            }
        }
        Ok(())
    }

    async fn on_msg(&self, m: w::Message, tx: &Tx, upd: bool) -> Result<()> {
        self.absorb(&m);
        self.ensure(ids(&m)).await;
        let cx = self.cx().await?;
        let out = self.map_msg(&cx, &m);
        let _ = tx.send(if upd { Ev::MsgUpd(out) } else { Ev::Msg(out) });
        if !upd && m.author != cx.me {
            let ment = m.mentions.as_ref().map(|l| l.contains(&cx.me)).unwrap_or(false) as u32;
            let (n, mm) = {
                let mut c = self.c.lock().unwrap();
                let e = c.unread.entry(m.channel.clone()).or_insert((0, 0));
                e.0 += 1;
                e.1 += ment;
                *e
            };
            let _ = tx.send(Ev::Unread { chan: cx.rid(&m.channel), n, ment: mm });
        }
        Ok(())
    }

    async fn refresh(&self, chan: &str, id: &str, tx: &Tx) -> Result<()> {
        let m = self.fetch_msg(chan, id).await?;
        self.on_msg(m, tx, true).await
    }

    async fn handle(&self, e: w::Event, tx: &Tx) -> Result<bool> {
        use w::Event as E;
        let send = |e: Ev| {
            let _ = tx.send(e);
        };
        match e {
            E::Logout => {
                send(Ev::Err { acc: self.acc, msg: "logged out".into() });
                return Ok(true);
            }
            E::Error { data } => {
                let ty = data.get("type").and_then(|t| t.as_str()).unwrap_or("error").to_string();
                send(Ev::Err { acc: self.acc, msg: ty.clone() });
                if ty == "InvalidSession" || ty == "OnboardingNotFinished" {
                    return Ok(true);
                }
            }
            e @ E::Ready { .. } => self.ready(e, tx).await?,
            E::Message(m) => self.on_msg(m, tx, false).await?,
            E::MessageUpdate { id, channel }
            | E::MessageAppend { id, channel }
            | E::MessageReact { id, channel_id: channel }
            | E::MessageUnreact { id, channel_id: channel }
            | E::MessageRemoveReaction { id, channel_id: channel } => {
                if let Err(e) = self.refresh(&channel, &id, tx).await {
                    tracing::debug!("st refresh: {e}");
                }
            }
            E::MessageDelete { id, channel } => {
                let cx = self.cx().await?;
                send(Ev::MsgDel { chan: cx.rid(&channel), id: cx.rid(&id) });
            }
            E::BulkMessageDelete { channel, ids } => {
                let cx = self.cx().await?;
                for id in ids {
                    send(Ev::MsgDel { chan: cx.rid(&channel), id: cx.rid(&id) });
                }
            }
            E::ServerCreate { server, channels, emojis } => {
                let sid = gid(&server);
                self.put(|c| {
                    let n = c.spos.len() as i64;
                    c.spos.insert(sid.clone(), n);
                    c.servers.insert(sid.clone(), server);
                    for ch in channels {
                        c.chans.insert(gid(&ch), ch);
                    }
                    for e in emojis {
                        c.emojis.insert(gid(&e), e);
                    }
                });
                let cx = self.cx().await?;
                self.srv_evs(&cx, &sid, true).into_iter().for_each(send);
            }
            E::ServerUpdate { id, data, clear } => {
                self.put(|c| {
                    if let Some(s) = c.servers.get_mut(&id) {
                        merge(s, &data, &clear);
                    }
                });
                let cx = self.cx().await?;
                self.srv_evs(&cx, &id, true).into_iter().for_each(send);
            }
            E::ServerDelete { id } => self.drop_server(&id, tx).await?,
            E::ServerMemberUpdate { id, data, clear } => {
                let key = (id.server.clone(), id.user.clone());
                self.put(|c| {
                    let m = c.mems.entry(key.clone()).or_insert_with(|| json!({"_id": {"server": key.0, "user": key.1}}));
                    merge(m, &data, &clear);
                });
                self.emit_mem(&key, tx).await?;
            }
            E::ServerMemberJoin { member } => {
                if let Ok(m) = serde_json::from_value::<w::Member>(member.clone()) {
                    let key = (m.id.server.clone(), m.id.user.clone());
                    self.ensure(vec![key.1.clone()]).await;
                    self.put(|c| {
                        c.mems.insert(key.clone(), member);
                    });
                    self.emit_mem(&key, tx).await?;
                }
            }
            E::ServerMemberLeave { id, user } => {
                let me = self.c.lock().unwrap().me.clone();
                if user == me {
                    self.drop_server(&id, tx).await?;
                } else {
                    self.put(|c| {
                        c.mems.remove(&(id, user));
                    });
                }
            }
            E::ServerRoleUpdate { id, role_id, data, clear } => {
                self.put(|c| {
                    if let Some(s) = c.servers.get_mut(&id) {
                        if let Some(o) = s.as_object_mut() {
                            let roles = o.entry("roles").or_insert_with(|| json!({}));
                            if let Some(r) = roles.as_object_mut() {
                                let slot = r.entry(role_id.clone()).or_insert_with(|| json!({}));
                                merge(slot, &data, &clear);
                            }
                        }
                    }
                });
                let cx = self.cx().await?;
                self.srv_evs(&cx, &id, true).into_iter().take(1).for_each(send);
            }
            E::ServerRoleDelete { id, role_id } => {
                self.put(|c| {
                    if let Some(r) = c.servers.get_mut(&id).and_then(|s| s.get_mut("roles")).and_then(|r| r.as_object_mut()) {
                        r.remove(&role_id);
                    }
                });
                let cx = self.cx().await?;
                self.srv_evs(&cx, &id, true).into_iter().take(1).for_each(send);
            }
            E::UserUpdate { id, data, clear } => {
                self.put(|c| {
                    let u = c.users.entry(id.clone()).or_insert_with(|| json!({"_id": id}));
                    merge(u, &data, &clear);
                });
                let cx = self.cx().await?;
                if let Some(u) = self.c.lock().unwrap().user(&id) {
                    if id != cx.me {
                        if data.get("online").is_some() || data.get("status").is_some() || !clear.is_empty() {
                            send(Ev::Pres { usr: cx.rid(&id), p: map::pres(&u) });
                        }
                        if let Some(r) = map::rel(&cx, &u) {
                            send(Ev::Rel(r));
                        }
                    }
                }
            }
            E::UserRelationship { user } => {
                let id = gid(&user);
                self.put(|c| {
                    c.users.insert(id.clone(), user.clone());
                });
                let cx = self.cx().await?;
                match serde_json::from_value::<w::User>(user).ok().and_then(|u| map::rel(&cx, &u)) {
                    Some(r) => send(Ev::Rel(r)),
                    None => send(Ev::RelDel(cx.rid(&id))),
                }
            }
            E::EmojiCreate(v) => {
                let id = gid(&v);
                let sid = serde_json::from_value::<w::Emoji>(v.clone()).ok().and_then(|e| match e.parent {
                    w::Parent::Server { id } => Some(id),
                    _ => None,
                });
                self.put(|c| {
                    c.emojis.insert(id, v);
                });
                if let Some(s) = sid {
                    let cx = self.cx().await?;
                    self.srv_evs(&cx, &s, true).into_iter().take(1).for_each(send);
                }
            }
            E::EmojiDelete { id } => {
                let old = self.c.lock().unwrap().emoji(&id);
                self.put(|c| {
                    c.emojis.remove(&id);
                });
                if let Some(w::Emoji { parent: w::Parent::Server { id: s }, .. }) = old {
                    let cx = self.cx().await?;
                    self.srv_evs(&cx, &s, true).into_iter().take(1).for_each(send);
                }
            }
            E::ChannelCreate(v) => {
                let id = gid(&v);
                let rcp: Vec<String> = v
                    .get("recipients")
                    .and_then(|r| r.as_array())
                    .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                self.ensure(rcp).await;
                let sid = v.get("server").and_then(|s| s.as_str()).map(String::from);
                self.put(|c| {
                    c.chans.insert(id.clone(), v);
                    if let Some(s) = sid.as_ref().and_then(|s| c.servers.get_mut(s)) {
                        if let Some(l) = s.get_mut("channels").and_then(|l| l.as_array_mut()) {
                            if !l.iter().any(|x| x.as_str() == Some(&id)) {
                                l.push(json!(id));
                            }
                        }
                    }
                });
                let cx = self.cx().await?;
                self.chan_evs(&cx, &id).into_iter().for_each(send);
            }
            E::ChannelUpdate { id, data, clear } => {
                self.put(|c| {
                    if let Some(ch) = c.chans.get_mut(&id) {
                        merge(ch, &data, &clear);
                    }
                });
                let cx = self.cx().await?;
                self.chan_evs(&cx, &id).into_iter().for_each(send);
            }
            E::ChannelDelete { id } => {
                self.put(|c| {
                    c.chans.remove(&id);
                    c.unread.remove(&id);
                });
                let cx = self.cx().await?;
                send(Ev::ChanDel(cx.rid(&id)));
            }
            E::ChannelGroupJoin { id, user } => {
                self.ensure(vec![user.clone()]).await;
                self.put(|c| {
                    if let Some(l) = c.chans.get_mut(&id).and_then(|c| c.get_mut("recipients")).and_then(|l| l.as_array_mut()) {
                        if !l.iter().any(|x| x.as_str() == Some(&user)) {
                            l.push(json!(user));
                        }
                    }
                });
                let cx = self.cx().await?;
                self.chan_evs(&cx, &id).into_iter().for_each(send);
            }
            E::ChannelGroupLeave { id, user } => {
                self.put(|c| {
                    if let Some(l) = c.chans.get_mut(&id).and_then(|c| c.get_mut("recipients")).and_then(|l| l.as_array_mut()) {
                        l.retain(|x| x.as_str() != Some(&user));
                    }
                });
                let cx = self.cx().await?;
                self.chan_evs(&cx, &id).into_iter().for_each(send);
            }
            E::ChannelStartTyping { id, user } => {
                let cx = self.cx().await?;
                if user != cx.me {
                    self.ensure(vec![user.clone()]).await;
                    let u = self.c.lock().unwrap().user(&user);
                    let usr = u.map(|u| map::usr(&cx, &u)).unwrap_or_else(|| Usr {
                        id: Some(cx.rid(&user)),
                        name: user.clone(),
                        ..Default::default()
                    });
                    send(Ev::Typing { chan: cx.rid(&id), usr });
                }
            }
            E::ChannelAck { id, user, .. } => {
                let cx = self.cx().await?;
                if user == cx.me {
                    self.put(|c| {
                        c.unread.remove(&id);
                    });
                    send(Ev::Ack { chan: cx.rid(&id) });
                }
            }
            E::Bulk { .. } | E::Authenticated | E::Pong | E::Unknown => {}
        }
        Ok(false)
    }

    async fn drop_server(&self, id: &str, tx: &Tx) -> Result<()> {
        self.put(|c| {
            c.servers.remove(id);
            c.chans.retain(|_, v| v.get("server").and_then(|s| s.as_str()) != Some(id));
        });
        let cx = self.cx().await?;
        let _ = tx.send(Ev::GldDel(cx.rid(id)));
        Ok(())
    }

    async fn emit_mem(&self, key: &(String, String), tx: &Tx) -> Result<()> {
        let cx = self.cx().await?;
        let m = self.c.lock().unwrap().mems.get(key).and_then(|v| serde_json::from_value::<w::Member>(v.clone()).ok());
        if let Some(m) = m {
            let o = self.view(|_, lk| map::mem(&cx, lk, &m));
            let _ = tx.send(Ev::Mem(o));
        }
        Ok(())
    }
}
