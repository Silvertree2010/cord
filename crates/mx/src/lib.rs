pub mod map;

use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use cord_core::*;
use map::{Cx, Item, RoomInfo};
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::media::{MediaFormat, MediaRequestParameters};
use matrix_sdk::room::MessagesOptions;
use matrix_sdk::room::reply::{EnforceThread, Reply};
use matrix_sdk::ruma::api::client::filter::FilterDefinition;
use matrix_sdk::ruma::api::client::sync::sync_events::v3::Filter;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::room::EncryptedFile;
use matrix_sdk::ruma::events::room::message::{AddMentions, TextMessageEventContent};
use matrix_sdk::ruma::{EventId, OwnedEventId, OwnedMxcUri, RoomId, UserId, uint};
use matrix_sdk::sync::{State, SyncResponse};
use matrix_sdk::{Client, Room, RoomMemberships};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub use matrix_sdk;

#[derive(Serialize, Deserialize)]
struct Saved {
    hs: String,
    sess: MatrixSession,
}

#[derive(Default)]
struct Sh {
    msgs: HashMap<String, Msg>,
    ord: VecDeque<String>,
    rx: HashMap<String, (String, String, String)>,
    mine: HashMap<(String, String), String>,
    glds: HashSet<String>,
    chans: HashSet<String>,
}

impl Sh {
    fn put(&mut self, m: Msg) {
        let id = m.id.raw.to_string();
        if self.msgs.insert(id.clone(), m).is_none() {
            self.ord.push_back(id);
            while self.ord.len() > 3000 {
                if let Some(o) = self.ord.pop_front() {
                    self.msgs.remove(&o);
                }
            }
        }
    }
}

struct I {
    acc: u32,
    hs: String,
    cl: Client,
    sh: Mutex<Sh>,
    jh: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[derive(Clone)]
pub struct Mx {
    i: Arc<I>,
}

async fn build(hs: &str, dir: &Path) -> Result<Client> {
    Ok(Client::builder().homeserver_url(hs).sqlite_store(dir, None).build().await?)
}

fn mk(acc: u32, hs: String, cl: Client) -> Mx {
    Mx { i: Arc::new(I { acc, hs, cl, sh: Mutex::new(Sh::default()), jh: Mutex::new(None) }) }
}

impl Mx {
    pub async fn login(acc: u32, hs: &str, user: &str, pass: &str, dir: &Path) -> Result<Self> {
        let cl = build(hs, dir).await?;
        cl.matrix_auth().login_username(user, pass).initial_device_display_name("cord").await?;
        Ok(mk(acc, hs.to_string(), cl))
    }

    pub async fn restore(acc: u32, dir: &Path, session_json: &str) -> Result<Self> {
        let s: Saved = serde_json::from_str(session_json)?;
        let cl = build(&s.hs, dir).await?;
        cl.restore_session(s.sess).await?;
        Ok(mk(acc, s.hs, cl))
    }

    pub fn session_json(&self) -> Result<String> {
        let sess = self.i.cl.matrix_auth().session().ok_or_else(|| anyhow!("not logged in"))?;
        Ok(serde_json::to_string(&Saved { hs: self.i.hs.clone(), sess })?)
    }

    pub async fn recover(&self, key: &str) -> Result<()> {
        self.i.cl.encryption().recovery().recover(key).await?;
        Ok(())
    }

    pub async fn media(&self, url: &str) -> Result<Vec<u8>> {
        let source = if let Some(h) = url.strip_prefix("mxe:") {
            let b = map::unhex(h).ok_or_else(|| anyhow!("bad media url"))?;
            let f: EncryptedFile = serde_json::from_slice(&b)?;
            MediaSource::Encrypted(Box::new(f))
        } else if url.starts_with("mxc://") {
            MediaSource::Plain(OwnedMxcUri::from(url))
        } else {
            bail!("not a matrix media url")
        };
        let p = MediaRequestParameters { source, format: MediaFormat::File };
        Ok(self.i.cl.media().get_media_content(&p, true).await?)
    }

    fn room(&self, r: &Rid) -> Result<Room> {
        self.i.room(r)
    }
}

fn eid(r: &Rid) -> Result<OwnedEventId> {
    Ok(EventId::parse(r.raw.as_ref())?)
}

fn jv(raw: &str) -> Option<Value> {
    serde_json::from_str(raw).ok()
}

impl I {
    fn room(&self, r: &Rid) -> Result<Room> {
        let id = RoomId::parse(r.raw.as_ref())?;
        self.cl.get_room(&id).ok_or_else(|| anyhow!("unknown room"))
    }

    fn cx(&self) -> Cx {
        Cx { acc: self.acc, me: self.cl.user_id().map(|u| u.to_string()).unwrap_or_default() }
    }

    async fn usr_of(&self, room: &Room, uid: &str) -> Usr {
        let cx = self.cx();
        if let Ok(u) = UserId::parse(uid) {
            if let Ok(Some(m)) = room.get_member_no_sync(&u).await {
                let av = m.avatar_url().map(|a| a.to_string());
                return map::usr(&cx, uid, m.display_name(), av.as_deref());
            }
        }
        map::usr(&cx, uid, None, None)
    }

    async fn info(&self, room: &Room, direct: bool) -> RoomInfo {
        let name = room.display_name().await.map(|n| n.to_string()).unwrap_or_else(|_| room.room_id().to_string());
        let mut kids = vec![];
        if room.is_space() {
            if let Ok(st) = room.get_state_events(matrix_sdk::ruma::events::StateEventType::SpaceChild).await {
                let vs: Vec<Value> = st
                    .iter()
                    .filter_map(|s| {
                        use matrix_sdk::deserialized_responses::RawAnySyncOrStrippedState as R;
                        match s {
                            R::Sync(r) => jv(r.json().get()),
                            R::Stripped(r) => jv(r.json().get()),
                        }
                    })
                    .collect();
                kids = map::children(&vs);
            }
        }
        let mut others = vec![];
        if direct {
            for t in room.direct_targets() {
                others.push(self.usr_of(room, &t.to_string()).await);
            }
        }
        RoomInfo {
            id: room.room_id().to_string(),
            name,
            topic: room.topic(),
            av: room.avatar_url().map(|a| a.to_string()),
            dm: direct,
            space: room.is_space(),
            kids,
            others,
            enc: room.encryption_state().is_encrypted(),
        }
    }

    async fn layout(&self, tx: &Tx) {
        let cx = self.cx();
        let mut infos = vec![];
        for r in self.cl.joined_rooms() {
            let d = r.is_direct().await.unwrap_or(false);
            infos.push(self.info(&r, d).await);
        }
        let (g, c) = map::layout(&cx, &infos);
        let gs: HashSet<String> = g.iter().map(|x| x.id.raw.to_string()).collect();
        let cs: HashSet<String> = c.iter().map(|x| x.id.raw.to_string()).collect();
        let (og, oc) = {
            let mut sh = self.sh.lock();
            (std::mem::replace(&mut sh.glds, gs.clone()), std::mem::replace(&mut sh.chans, cs.clone()))
        };
        for x in og.difference(&gs) {
            let _ = tx.send(Ev::GldDel(cx.rid(x)));
        }
        for x in oc.difference(&cs) {
            let _ = tx.send(Ev::ChanDel(cx.rid(x)));
        }
        for x in g {
            let _ = tx.send(Ev::Gld(x));
        }
        for x in c {
            let _ = tx.send(Ev::Chan(x));
        }
    }

    async fn me(&self) -> Usr {
        let cx = self.cx();
        let acct = self.cl.account();
        let dn = acct.get_display_name().await.ok().flatten();
        let av = acct.get_avatar_url().await.ok().flatten().map(|a| a.to_string());
        map::usr(&cx, &cx.me, dn.as_deref(), av.as_deref())
    }

    fn cached(&self, id: &str) -> Option<Msg> {
        self.sh.lock().msgs.get(id).cloned()
    }

    fn store(&self, m: Msg) {
        self.sh.lock().put(m);
    }

    async fn event_item(&self, room: &Room, ev: &Value) -> Item {
        let sender = ev.get("sender").and_then(|s| s.as_str()).unwrap_or_default();
        let u = self.usr_of(room, sender).await;
        map::item(&self.cx(), room.room_id().as_str(), ev, u)
    }

    async fn load(&self, room: &Room, id: &str) -> Option<Msg> {
        if let Some(m) = self.cached(id) {
            return Some(m);
        }
        let e = EventId::parse(id).ok()?;
        let te = room.event(&e, None).await.ok()?;
        let ev = jv(te.raw().json().get())?;
        match self.event_item(room, &ev).await {
            Item::New(m) => {
                self.store(m.clone());
                Some(m)
            }
            _ => None,
        }
    }

    fn note_react(&self, ev: &str, target: &str, key: &str, who: &str) {
        let me = self.cx().me;
        let mut sh = self.sh.lock();
        sh.rx.insert(ev.to_string(), (target.to_string(), key.to_string(), who.to_string()));
        if who == me {
            sh.mine.insert((target.to_string(), key.to_string()), ev.to_string());
        }
    }

    async fn live(&self, room: &Room, ev: Value, tx: &Tx) {
        let cx = self.cx();
        let rid = room.room_id().to_string();
        match self.event_item(room, &ev).await {
            Item::New(m) => {
                self.store(m.clone());
                let _ = tx.send(Ev::Msg(m));
            }
            Item::Edit { id, body, ts } => {
                let sender = ev.get("sender").and_then(|s| s.as_str()).unwrap_or_default();
                if let Some(mut m) = self.load(room, &id).await {
                    if m.auth.id.as_ref().map(|i| i.raw.as_ref()) != Some(sender) {
                        return;
                    }
                    map::apply_edit(&mut m, &body, ts);
                    self.store(m.clone());
                    let _ = tx.send(Ev::MsgUpd(m));
                }
            }
            Item::React { ev: rev, target, key, who } => {
                if self.sh.lock().rx.contains_key(&rev) {
                    return;
                }
                self.note_react(&rev, &target, &key, &who);
                if let Some(mut m) = self.load(room, &target).await {
                    map::add_react(&mut m, &key, who == cx.me);
                    self.store(m.clone());
                    let _ = tx.send(Ev::MsgUpd(m));
                }
            }
            Item::Del(t) => {
                let rx = self.sh.lock().rx.remove(&t);
                if let Some((target, key, who)) = rx {
                    self.sh.lock().mine.remove(&(target.clone(), key.clone()));
                    if let Some(mut m) = self.cached(&target) {
                        map::rm_react(&mut m, &key, who == cx.me);
                        self.store(m.clone());
                        let _ = tx.send(Ev::MsgUpd(m));
                    }
                } else {
                    let mut sh = self.sh.lock();
                    sh.msgs.remove(&t);
                    drop(sh);
                    let _ = tx.send(Ev::MsgDel { chan: cx.rid(&rid), id: cx.rid(&t) });
                }
            }
            Item::Skip => {}
        }
    }

    async fn process(&self, r: SyncResponse, first: bool, tx: &Tx) {
        let cx = self.cx();
        let mut relayout = first;
        for (rid, jr) in r.rooms.joined {
            let Some(room) = self.cl.get_room(&rid) else { continue };
            let rs = rid.to_string();
            match &jr.state {
                State::Before(v) => relayout |= !v.is_empty(),
                State::After(v) => relayout |= !v.is_empty(),
            }
            for te in jr.timeline.events {
                let Some(ev) = jv(te.raw().json().get()) else { continue };
                if ev.get("state_key").is_some() {
                    relayout = true;
                    continue;
                }
                self.live(&room, ev, tx).await;
            }
            for e in jr.ephemeral {
                let Some(ev) = jv(e.json().get()) else { continue };
                match ev.get("type").and_then(|t| t.as_str()) {
                    Some("m.typing") => {
                        let ids: Vec<String> = ev
                            .pointer("/content/user_ids")
                            .and_then(|u| u.as_array())
                            .into_iter()
                            .flatten()
                            .filter_map(|u| u.as_str().map(String::from))
                            .collect();
                        let mut us: HashMap<String, Usr> = HashMap::new();
                        for u in ids {
                            let x = self.usr_of(&room, &u).await;
                            us.insert(u, x);
                        }
                        let f = |u: &str| us.get(u).cloned().unwrap_or_default();
                        for e in map::typing(&cx, &rs, &ev, &f) {
                            let _ = tx.send(e);
                        }
                    }
                    Some("m.receipt") => {
                        if map::receipt_ack(&cx, &rs, &ev) {
                            let _ = tx.send(Ev::Ack { chan: cx.rid(&rs) });
                        }
                    }
                    _ => {}
                }
            }
            let un = &jr.unread_notifications;
            let _ = tx.send(Ev::Unread { chan: cx.rid(&rs), n: un.notification_count as u32, ment: un.highlight_count as u32 });
        }
        if !r.rooms.left.is_empty() || !r.rooms.invited.is_empty() {
            relayout = true;
        }
        for p in r.presence {
            if let Some(ev) = jv(p.json().get()) {
                if let Some(e) = map::pres(&cx, &ev) {
                    let _ = tx.send(e);
                }
            }
        }
        for a in r.account_data {
            if let Some(ev) = jv(a.json().get()) {
                if ev.get("type").and_then(|t| t.as_str()) == Some("m.direct") {
                    relayout = true;
                }
            }
        }
        if relayout {
            self.layout(tx).await;
        }
    }

    async fn run(&self, tx: Tx) {
        let mut first = true;
        let mut back = 1u64;
        loop {
            let s = SyncSettings::new()
                .timeout(Duration::from_secs(30))
                .filter(Filter::FilterDefinition(FilterDefinition::with_lazy_loading()));
            match self.cl.sync_once(s).await {
                Ok(r) => {
                    back = 1;
                    if first {
                        let _ = tx.send(Ev::Conn { acc: self.acc, up: true });
                        let me = self.me().await;
                        let _ = tx.send(Ev::Ready(Acc { id: self.acc, svc: Svc::Mx, me, demo: false }));
                    }
                    self.process(r, first, &tx).await;
                    first = false;
                }
                Err(e) => {
                    let m = e.to_string();
                    tracing::warn!("mx sync: {m}");
                    let _ = tx.send(Ev::Conn { acc: self.acc, up: false });
                    let _ = tx.send(Ev::Err { acc: self.acc, msg: m.clone() });
                    if m.contains("M_UNKNOWN_TOKEN") || format!("{e:?}").contains("M_UNKNOWN_TOKEN") {
                        return;
                    }
                    tokio::time::sleep(Duration::from_secs(back)).await;
                    back = (back * 2).min(60);
                    first = true;
                }
            }
        }
    }

    async fn hist(&self, chan: &Rid, before: Option<&Rid>, n: u32) -> Result<Vec<Msg>> {
        let room = self.room(chan)?;
        let cx = self.cx();
        let mut from: Option<String> = None;
        if let Some(b) = before {
            let c = room.event_with_context(&eid(b)?, false, uint!(0), None).await?;
            from = c.prev_batch_token;
            if from.is_none() {
                return Ok(vec![]);
            }
        }
        let mut o = MessagesOptions::backward().from(from.as_deref());
        o.limit = n.clamp(1, 200).into();
        let res = room.messages(o).await?;
        let mut out: Vec<Msg> = vec![];
        let mut edits = vec![];
        let mut reacts = vec![];
        for te in res.chunk.iter().rev() {
            let Some(ev) = jv(te.raw().json().get()) else { continue };
            match self.event_item(&room, &ev).await {
                Item::New(m) => out.push(m),
                Item::Edit { id, body, ts } => {
                    let sender = ev.get("sender").and_then(|s| s.as_str()).unwrap_or_default().to_string();
                    edits.push((id, body, ts, sender));
                }
                Item::React { ev: re, target, key, who } => {
                    self.note_react(&re, &target, &key, &who);
                    reacts.push((target, key, who));
                }
                _ => {}
            }
        }
        for (id, body, ts, sender) in edits {
            if let Some(m) = out.iter_mut().find(|m| m.id.raw.as_ref() == id) {
                if m.auth.id.as_ref().map(|i| i.raw.as_ref()) == Some(sender.as_str()) {
                    map::apply_edit(m, &body, ts);
                }
            }
        }
        for (t, k, w) in reacts {
            if let Some(m) = out.iter_mut().find(|m| m.id.raw.as_ref() == t) {
                map::add_react(m, &k, w == cx.me);
            }
        }
        for m in out.iter_mut() {
            if let Some(c) = self.cached(m.id.raw.as_ref()) {
                *m = c;
            } else {
                self.store(m.clone());
            }
        }
        Ok(out)
    }

    async fn send(&self, chan: &Rid, o: Out, me: &Mx) -> Result<()> {
        let room = me.room(chan)?;
        if o.files.is_empty() {
            if o.body.trim().is_empty() {
                bail!("empty message");
            }
            let mut c = json!({"msgtype": "m.text", "body": o.body});
            if let Some(r) = &o.reply {
                c["m.relates_to"] = json!({"m.in_reply_to": {"event_id": r.raw}});
            }
            room.send_raw("m.room.message", c).await?;
            return Ok(());
        }
        for (i, f) in o.files.iter().enumerate() {
            let data = tokio::fs::read(f).await?;
            let name = f.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
            let mime = mime_guess::from_path(f).first_or_octet_stream();
            let mut cfg = matrix_sdk::attachment::AttachmentConfig::new();
            if i == 0 {
                if !o.body.is_empty() {
                    cfg = cfg.caption(Some(TextMessageEventContent::plain(o.body.clone())));
                }
                if let Some(r) = &o.reply {
                    cfg = cfg.reply(Some(Reply {
                        event_id: eid(r)?,
                        enforce_thread: EnforceThread::Unthreaded,
                        add_mentions: AddMentions::No,
                    }));
                }
            }
            room.send_attachment(name, &mime, data, cfg).await?;
        }
        Ok(())
    }

    async fn edit(&self, me: &Mx, chan: &Rid, msg: &Rid, body: String) -> Result<()> {
        let room = me.room(chan)?;
        let c = json!({
            "msgtype": "m.text",
            "body": format!("* {body}"),
            "m.new_content": {"msgtype": "m.text", "body": body},
            "m.relates_to": {"rel_type": "m.replace", "event_id": msg.raw},
        });
        room.send_raw("m.room.message", c).await?;
        Ok(())
    }

    async fn react(&self, me: &Mx, chan: &Rid, msg: &Rid, e: &Emo, on: bool) -> Result<()> {
        let room = me.room(chan)?;
        let key = e.url.clone().unwrap_or_else(|| e.name.clone());
        if on {
            let c = json!({"m.relates_to": {"rel_type": "m.annotation", "event_id": msg.raw, "key": key}});
            let r = room.send_raw("m.reaction", c).await?;
            self.note_react(r.response.event_id.as_str(), msg.raw.as_ref(), &key, &self.cx().me);
            return Ok(());
        }
        let ev = self.sh.lock().mine.get(&(msg.raw.to_string(), key.clone())).cloned();
        let ev = ev.ok_or_else(|| anyhow!("own reaction not known"))?;
        room.redact(&EventId::parse(&ev)?, None, None).await?;
        Ok(())
    }
}

#[async_trait]
impl Bk for Mx {
    fn svc(&self) -> Svc {
        Svc::Mx
    }
    fn acc(&self) -> u32 {
        self.i.acc
    }
    async fn start(&self, tx: Tx) -> Res<()> {
        let i = self.i.clone();
        let j = tokio::spawn(async move { i.run(tx).await });
        if let Some(o) = self.i.jh.lock().replace(j) {
            o.abort();
        }
        Ok(())
    }
    async fn stop(&self) {
        if let Some(j) = self.i.jh.lock().take() {
            j.abort();
        }
    }
    async fn hist(&self, chan: &Rid, before: Option<&Rid>, n: u32) -> Res<Vec<Msg>> {
        self.i.hist(chan, before, n).await
    }
    async fn send(&self, chan: &Rid, o: Out) -> Res<()> {
        self.i.send(chan, o, self).await
    }
    async fn edit(&self, chan: &Rid, msg: &Rid, body: String) -> Res<()> {
        self.i.edit(self, chan, msg, body).await
    }
    async fn del(&self, chan: &Rid, msg: &Rid) -> Res<()> {
        self.room(chan)?.redact(&eid(msg)?, None, None).await?;
        Ok(())
    }
    async fn react(&self, chan: &Rid, msg: &Rid, e: &Emo, on: bool) -> Res<()> {
        self.i.react(self, chan, msg, e, on).await
    }
    async fn typing(&self, chan: &Rid) -> Res<()> {
        self.room(chan)?.typing_notice(true).await?;
        Ok(())
    }
    async fn ack(&self, chan: &Rid, msg: &Rid) -> Res<()> {
        use matrix_sdk::ruma::api::client::receipt::create_receipt::v3::ReceiptType;
        use matrix_sdk::ruma::events::receipt::ReceiptThread;
        let room = self.room(chan)?;
        let id = eid(msg)?;
        room.send_single_receipt(ReceiptType::Read, ReceiptThread::Unthreaded, id.clone()).await?;
        let _ = room.send_single_receipt(ReceiptType::FullyRead, ReceiptThread::Unthreaded, id).await;
        Ok(())
    }
    async fn mems(&self, gld: &Rid) -> Res<Vec<Mem>> {
        let room = self.room(gld)?;
        let cx = self.i.cx();
        let ms = room.members(RoomMemberships::JOIN).await?;
        Ok(ms
            .iter()
            .map(|m| {
                let av = m.avatar_url().map(|a| a.to_string());
                Mem {
                    usr: map::usr(&cx, m.user_id().as_str(), m.display_name(), av.as_deref()),
                    gld: gld.clone(),
                    nick: m.display_name().map(String::from),
                    roles: vec![],
                }
            })
            .collect())
    }
    async fn api(&self, op: &str, a: &serde_json::Value) -> Res<serde_json::Value> {
        use serde_json::json;
        let st = |k: &str| a[k].as_str().unwrap_or("").to_string();
        let cl = &self.i.cl;
        match op {
            "acct" => {
                let name = cl.account().get_display_name().await?.unwrap_or_default();
                let uid = cl.user_id().map(|u| u.to_string()).unwrap_or_default();
                Ok(json!({"name": name, "user": uid, "email": null, "bio": "", "caps": ["name", "rename"]}))
            }
            "set_name" => {
                cl.account().set_display_name(Some(&st("name"))).await?;
                Ok(json!({}))
            }
            "sessions" => {
                let cur = cl.device_id().map(|d| d.to_string()).unwrap_or_default();
                let ds = cl.devices().await?;
                let list: Vec<serde_json::Value> = ds
                    .devices
                    .iter()
                    .map(|d| json!({"id": d.device_id.to_string(), "name": d.display_name.clone().unwrap_or_else(|| d.device_id.to_string()), "kind": "desktop", "last": d.last_seen_ts.map(|t| u64::from(t.0) / 1000).unwrap_or(0), "loc": d.last_seen_ip}))
                    .collect();
                Ok(json!({"cur": cur, "list": list}))
            }
            "session_name" => {
                let id: matrix_sdk::ruma::OwnedDeviceId = st("id").into();
                cl.rename_device(&id, &st("name")).await?;
                Ok(json!({}))
            }
            _ => anyhow::bail!("{op} is not supported on this network"),
        }
    }
    async fn set_st(&self, st: St, act: Option<String>) -> Res<()> {
        use matrix_sdk::ruma::api::client::presence::set_presence::v3::Request;
        use matrix_sdk::ruma::presence::PresenceState;
        let uid = self.i.cl.user_id().ok_or_else(|| anyhow!("not logged in"))?.to_owned();
        let p = match st {
            St::On => PresenceState::Online,
            St::Idle => PresenceState::Unavailable,
            St::Dnd | St::Off => PresenceState::Offline,
        };
        let mut r = Request::new(uid, p);
        r.status_msg = act;
        self.i.cl.send(r).await?;
        Ok(())
    }
}
