use crate::map::{self, Cx, Lk};
use crate::wire as w;
use crate::{Cfg, I, NIL};
use anyhow::{Result, anyhow, bail};
use cord_core::{Emo, Mem, Msg, Out, Rid, St as Pr};
use reqwest::Method;
use serde_json::{Value, json};
use std::sync::atomic::Ordering;
use std::time::Duration;

pub async fn login(base: &str, email: &str, pass: &str) -> Result<String> {
    let http = reqwest::Client::new();
    let url = format!("{}/auth/session/login", base.trim_end_matches('/'));
    let r = http
        .post(url)
        .json(&json!({"email": email, "password": pass, "friendly_name": "cord"}))
        .send()
        .await?;
    let ok = r.status().is_success();
    let t = r.text().await?;
    let v: Value = serde_json::from_str(&t).unwrap_or(Value::Null);
    if !ok {
        let ty = v.get("type").and_then(|x| x.as_str()).unwrap_or("error");
        bail!("login failed: {ty}");
    }
    match v.get("result").and_then(|x| x.as_str()) {
        Some("Success") => v
            .get("token")
            .and_then(|x| x.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow!("login response without token")),
        Some("MFA") => bail!(
            "mfa required (allowed methods: {}); log in with a session token instead",
            v.get("allowed_methods").map(|m| m.to_string()).unwrap_or_default()
        ),
        Some("Disabled") => bail!("account disabled"),
        _ => bail!("unexpected login response"),
    }
}

pub fn ids(m: &w::Message) -> Vec<String> {
    use w::Sys::*;
    let mut o = vec![];
    if m.webhook.is_none() && m.author != NIL && !m.author.is_empty() {
        o.push(m.author.clone());
    }
    if let Some(s) = &m.system {
        match s {
            Added { id, by } | Remove { id, by } => o.extend([id.clone(), by.clone()]),
            Joined { id } | Left { id } | Kicked { id } | Banned { id } => o.push(id.clone()),
            Renamed { by, .. } | DescChanged { by } | IconChanged { by } | Call { by } => o.push(by.clone()),
            Owner { from, to } => o.extend([from.clone(), to.clone()]),
            Pinned { by, .. } | Unpinned { by, .. } => o.push(by.clone()),
            Text { .. } | Other => {}
        }
    }
    o
}

impl I {
    pub async fn cfg(&self) -> Result<&Cfg> {
        self.cfg
            .get_or_try_init(|| async {
                let v: Value = self.http.get(&self.base).send().await?.error_for_status()?.json().await?;
                let f = |p: &[&str]| -> Option<String> {
                    let mut c = &v;
                    for k in p {
                        c = c.get(*k)?;
                    }
                    c.as_str().map(String::from)
                };
                let ws = f(&["ws"]).ok_or_else(|| anyhow!("no ws url in root response"))?;
                let cdn = f(&["features", "autumn", "url"]).ok_or_else(|| anyhow!("no file server in root response"))?;
                Ok(Cfg {
                    ws,
                    api: self.base.clone(),
                    cdn: cdn.trim_end_matches('/').to_string(),
                    prx: f(&["features", "january", "url"]).unwrap_or_default().trim_end_matches('/').to_string(),
                })
            })
            .await
    }

    pub async fn cx(&self) -> Result<Cx> {
        let c = self.cfg().await?;
        let me = self.c.lock().me.clone();
        Ok(Cx { acc: self.acc, me, api: c.api.clone(), cdn: c.cdn.clone(), prx: c.prx.clone() })
    }

    pub fn view<R>(&self, f: impl FnOnce(&crate::Cache, &Lk) -> R) -> R {
        let c = self.c.lock();
        let uf = |id: &str| c.user(id);
        let ef = |id: &str| c.emoji(id);
        let lk = Lk { user: &uf, emoji: &ef };
        f(&c, &lk)
    }

    pub async fn req(&self, m: Method, path: &str, body: Option<Value>, hdr: &[(&str, String)]) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        for n in 0..4 {
            let mut r = self.http.request(m.clone(), &url).header("x-session-token", &self.tok);
            for (k, v) in hdr {
                r = r.header(*k, v);
            }
            if let Some(b) = &body {
                r = r.json(b);
            }
            let res = r.send().await?;
            let s = res.status();
            if s.as_u16() == 429 && n < 3 {
                let ms = res
                    .headers()
                    .get("x-ratelimit-reset-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.parse::<u64>().ok())
                    .unwrap_or(1000);
                tokio::time::sleep(Duration::from_millis(ms.min(30000) + 50)).await;
                continue;
            }
            let t = res.text().await?;
            if !s.is_success() {
                bail!("{} {} -> {}: {}", m, path, s.as_u16(), t.chars().take(200).collect::<String>());
            }
            return Ok(serde_json::from_str(&t).unwrap_or(Value::Null));
        }
        bail!("rate limited")
    }

    pub async fn fetch_user(&self, id: &str) -> Option<w::User> {
        if let Some(u) = self.c.lock().user(id) {
            return Some(u);
        }
        let v = self.req(Method::GET, &format!("/users/{id}"), None, &[]).await.ok()?;
        let u = serde_json::from_value::<w::User>(v.clone()).ok()?;
        self.c.lock().users.insert(id.to_string(), v);
        Some(u)
    }

    pub async fn ensure(&self, ids: Vec<String>) {
        let miss: Vec<String> = {
            let c = self.c.lock();
            let mut m: Vec<String> = ids.into_iter().filter(|i| !c.users.contains_key(i) && i != NIL).collect();
            m.sort();
            m.dedup();
            m
        };
        futures::future::join_all(miss.iter().map(|i| self.fetch_user(i))).await;
    }

    pub fn absorb(&self, m: &w::Message) {
        let mut c = self.c.lock();
        if let Some(u) = &m.user {
            if !u.id.is_empty() && !c.users.contains_key(&u.id) {
                let mut v = json!({"_id": u.id, "username": u.username, "display_name": u.display_name, "bot": u.bot});
                if let Some(a) = &u.avatar {
                    v["avatar"] = json!({"_id": a.id, "tag": a.tag, "filename": a.filename});
                }
                c.users.insert(u.id.clone(), v);
            }
        }
        if let Some(mem) = &m.member {
            let mut v = json!({"_id": {"server": mem.id.server, "user": mem.id.user}, "nickname": mem.nickname, "roles": mem.roles});
            if let Some(a) = &mem.avatar {
                v["avatar"] = json!({"_id": a.id, "tag": a.tag, "filename": a.filename});
            }
            c.mems.insert((mem.id.server.clone(), mem.id.user.clone()), v);
        }
    }

    pub fn map_msg(&self, cx: &Cx, m: &w::Message) -> Msg {
        let u = self
            .c
            .lock()
            .user(&m.author)
            .or_else(|| m.user.clone())
            .unwrap_or_else(|| w::User {
                id: m.author.clone(),
                username: m.webhook.as_ref().map(|h| h.name.clone()).unwrap_or_else(|| m.author.clone()),
                ..Default::default()
            });
        self.view(|_, lk| map::msg(cx, lk, m, &u))
    }

    pub async fn hist(&self, chan: &Rid, before: Option<&Rid>, n: u32) -> Result<Vec<Msg>> {
        let mut p = format!("/channels/{}/messages?limit={}&include_users=true", chan.raw, n.clamp(1, 100));
        if let Some(b) = before {
            p.push_str(&format!("&before={}", b.raw));
        }
        let v = self.req(Method::GET, &p, None, &[]).await?;
        let b: w::Bulk = serde_json::from_value(v)?;
        let msgs = match b {
            w::Bulk::Plain(m) => m,
            w::Bulk::Full { messages, users, members } => {
                {
                    let mut c = self.c.lock();
                    for u in users {
                        if let Some(id) = u.get("_id").and_then(|i| i.as_str()) {
                            c.users.entry(id.to_string()).or_insert(u.clone());
                        }
                    }
                    for m in members {
                        if let Some(k) = m.get("_id") {
                            let s = k.get("server").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                            let u = k.get("user").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                            c.mems.insert((s, u), m);
                        }
                    }
                }
                messages
            }
        };
        for m in &msgs {
            self.absorb(m);
        }
        self.ensure(msgs.iter().flat_map(ids).collect()).await;
        let cx = self.cx().await?;
        let mut o: Vec<Msg> = msgs.iter().map(|m| self.map_msg(&cx, m)).collect();
        o.sort_by(|a, b| a.id.raw.cmp(&b.id.raw));
        Ok(o)
    }

    pub async fn fetch_msg(&self, chan: &str, id: &str) -> Result<w::Message> {
        let v = self.req(Method::GET, &format!("/channels/{chan}/messages/{id}"), None, &[]).await?;
        Ok(serde_json::from_value(v)?)
    }

    fn nonce(&self) -> String {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("cord{:x}{:x}", t, self.seq.fetch_add(1, Ordering::Relaxed))
    }

    pub async fn upload(&self, p: &std::path::Path) -> Result<String> {
        let cfg = self.cfg().await?;
        let bytes = tokio::fs::read(p).await?;
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
        let part = reqwest::multipart::Part::bytes(bytes).file_name(name);
        let form = reqwest::multipart::Form::new().part("file", part);
        let r = self
            .http
            .post(format!("{}/attachments", cfg.cdn))
            .header("x-session-token", &self.tok)
            .multipart(form)
            .send()
            .await?;
        let s = r.status();
        let t = r.text().await?;
        if !s.is_success() {
            bail!("upload -> {}: {}", s.as_u16(), t.chars().take(200).collect::<String>());
        }
        let v: Value = serde_json::from_str(&t)?;
        v.get("id").and_then(|x| x.as_str()).map(String::from).ok_or_else(|| anyhow!("upload without id"))
    }

    pub async fn send(&self, chan: &Rid, o: Out) -> Result<()> {
        let mut att = vec![];
        for f in &o.files {
            att.push(self.upload(f).await?);
        }
        if o.body.trim().is_empty() && att.is_empty() {
            bail!("empty message");
        }
        let n = self.nonce();
        let mut b = json!({"nonce": n});
        if !o.body.is_empty() {
            b["content"] = json!(o.body);
        }
        if !att.is_empty() {
            b["attachments"] = json!(att);
        }
        if let Some(r) = &o.reply {
            b["replies"] = json!([{"id": r.raw, "mention": true}]);
        }
        if o.silent {
            b["flags"] = json!(1);
        }
        self.req(Method::POST, &format!("/channels/{}/messages", chan.raw), Some(b), &[("Idempotency-Key", n)])
            .await?;
        Ok(())
    }

    pub async fn edit(&self, chan: &Rid, msg: &Rid, body: String) -> Result<()> {
        self.req(
            Method::PATCH,
            &format!("/channels/{}/messages/{}", chan.raw, msg.raw),
            Some(json!({"content": body})),
            &[],
        )
        .await?;
        Ok(())
    }

    pub async fn del(&self, chan: &Rid, msg: &Rid) -> Result<()> {
        self.req(Method::DELETE, &format!("/channels/{}/messages/{}", chan.raw, msg.raw), None, &[]).await?;
        Ok(())
    }

    pub async fn react(&self, chan: &Rid, msg: &Rid, e: &Emo, on: bool) -> Result<()> {
        let p = format!("/channels/{}/messages/{}/reactions/{}", chan.raw, msg.raw, map::enc(&map::emoji_key(e)));
        self.req(if on { Method::PUT } else { Method::DELETE }, &p, None, &[]).await?;
        Ok(())
    }

    pub async fn ack(&self, chan: &Rid, msg: &Rid) -> Result<()> {
        self.req(Method::PUT, &format!("/channels/{}/ack/{}", chan.raw, msg.raw), None, &[]).await?;
        Ok(())
    }

    pub async fn mems(&self, gld: &Rid) -> Result<Vec<Mem>> {
        let v = self.req(Method::GET, &format!("/servers/{}/members", gld.raw), None, &[]).await?;
        let users = v.get("users").and_then(|u| u.as_array()).cloned().unwrap_or_default();
        let members = v.get("members").and_then(|u| u.as_array()).cloned().unwrap_or_default();
        {
            let mut c = self.c.lock();
            for u in users {
                if let Some(id) = u.get("_id").and_then(|i| i.as_str()) {
                    c.users.insert(id.to_string(), u.clone());
                }
            }
        }
        let cx = self.cx().await?;
        let ms: Vec<w::Member> = members.into_iter().filter_map(|m| serde_json::from_value(m).ok()).collect();
        Ok(self.view(|_, lk| ms.iter().map(|m| map::mem(&cx, lk, m)).collect()))
    }

    pub async fn set_st(&self, st: Pr, act: Option<String>) -> Result<()> {
        let p = match st {
            Pr::On => "Online",
            Pr::Idle => "Idle",
            Pr::Dnd => "Busy",
            Pr::Off => "Invisible",
        };
        let mut s = json!({"presence": p});
        s["text"] = json!(act.unwrap_or_default());
        self.req(Method::PATCH, "/users/@me", Some(json!({"status": s})), &[]).await?;
        Ok(())
    }

    pub async fn unreads(&self) -> Result<Vec<w::Unread>> {
        let v = self.req(Method::GET, "/sync/unreads", None, &[]).await?;
        Ok(serde_json::from_value(v)?)
    }
}
