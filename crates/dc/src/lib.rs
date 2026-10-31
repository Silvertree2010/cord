pub mod gw;
pub mod hd;
pub mod map;
pub mod perm;
pub mod ra;
pub mod rest;
pub mod stm;
pub mod vc;
pub mod zl;

use async_trait::async_trait;
use cord_core::*;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::Method;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct Sess {
    pub id: Option<String>,
    pub seq: Option<u64>,
    pub url: Option<String>,
}

pub struct Gs {
    pub g: Gld,
    pub my: Vec<String>,
    pub chs: Vec<Value>,
}

pub struct Sh {
    pub a: u32,
    pub tok: String,
    pub rest: rest::Rest,
    pub me: Mutex<String>,
    pub gs: Mutex<HashMap<String, Gs>>,
    pub us: Mutex<HashMap<String, Usr>>,
    pub lm: Mutex<HashMap<String, String>>,
    pub cg: Mutex<HashMap<String, String>>,
    pub ak: Mutex<HashMap<String, (String, u32)>>,
    pub mu: Mutex<HashSet<String>>,
    pub ss: Mutex<Sess>,
    pub gt: Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>>,
    pub vo: vc::Vo,
    pub st: stm::Stm,
    pub jh: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

pub struct Dc(Arc<Sh>);

impl Dc {
    pub fn new(acc: u32, tok: String) -> Dc {
        Dc(Arc::new(Sh {
            a: acc,
            rest: rest::Rest::new(tok.clone()),
            tok,
            me: Default::default(),
            gs: Default::default(),
            us: Default::default(),
            lm: Default::default(),
            cg: Default::default(),
            ak: Default::default(),
            mu: Default::default(),
            ss: Default::default(),
            gt: Default::default(),
            vo: Default::default(),
            st: Default::default(),
            jh: Default::default(),
        }))
    }

    pub fn eng(&self) -> cord_rtc::Eng {
        self.0.vo.eng.clone()
    }

    pub fn frames(&self, usr: &Rid) -> tokio::sync::watch::Receiver<Option<Arc<cord_cap::Frame>>> {
        self.0.st.frames(&usr.raw)
    }

    pub fn pause(&self, on: bool) -> Res<()> {
        self.0.st.pause(&self.0, on)
    }

    fn gsend(&self, v: Value) -> Res<()> {
        let g = self.0.gt.lock().unwrap();
        match &*g {
            Some(t) => t.send(v.to_string()).map_err(|e| anyhow::anyhow!("{e}")),
            None => anyhow::bail!("gateway down"),
        }
    }
}

fn nonce() -> String {
    let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
    ((ms.saturating_sub(1420070400000)) << 22 | (rand::random::<u64>() & 0x3fffff)).to_string()
}

fn mime(n: &str) -> &'static str {
    match n.rsplit('.').next().map(str::to_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
}

#[async_trait]
impl Bk for Dc {
    fn svc(&self) -> Svc {
        Svc::Dc
    }

    fn acc(&self) -> u32 {
        self.0.a
    }

    async fn start(&self, tx: Tx) -> Res<()> {
        self.0.vo.set_tx(tx.clone());
        let j = tokio::spawn(gw::run(self.0.clone(), tx));
        if let Some(o) = self.0.jh.lock().unwrap().replace(j) {
            o.abort();
        }
        Ok(())
    }

    async fn stop(&self) {
        if let Some(j) = self.0.jh.lock().unwrap().take() {
            j.abort();
        }
        self.0.st.stop_all(&self.0);
        let _ = self.0.vo.end();
        *self.0.gt.lock().unwrap() = None;
    }

    async fn open_dm(&self, usr: &Rid) -> Res<Rid> {
        let v = self.0.rest.js(Method::POST, "/users/@me/channels", Some(json!({"recipient_id": &*usr.raw}))).await?;
        let id = v["id"].as_str().ok_or_else(|| anyhow::anyhow!("no channel id in reply"))?;
        Ok(Rid::new(Svc::Dc, self.0.a, id))
    }

    async fn hist(&self, chan: &Rid, before: Option<&Rid>, n: u32) -> Res<Vec<Msg>> {
        let mut p = format!("/channels/{}/messages?limit={}", chan.raw, n.clamp(1, 100));
        if let Some(b) = before {
            p.push_str(&format!("&before={}", b.raw));
        }
        let v = self.0.rest.js(Method::GET, &p, None).await?;
        let mut o: Vec<Msg> = v.as_array().map(|a| a.iter().filter_map(|m| map::msg(self.0.a, m)).collect()).unwrap_or_default();
        o.reverse();
        Ok(o)
    }

    async fn send(&self, chan: &Rid, o: Out) -> Res<()> {
        let mut j = json!({"content": o.body, "nonce": nonce(), "tts": false});
        if o.silent {
            j["flags"] = json!(4096);
        }
        if let Some(r) = &o.reply {
            let mut mr = json!({"message_id": &*r.raw, "channel_id": &*chan.raw, "fail_if_not_exists": false});
            if let Some(g) = self.0.cg.lock().unwrap().get(&*chan.raw) {
                mr["guild_id"] = json!(g);
            }
            j["message_reference"] = mr;
            if feat::on("noreplyping") {
                j["allowed_mentions"] = json!({"parse": ["users", "roles", "everyone"], "replied_user": false});
            }
        }
        if !o.stks.is_empty() {
            j["sticker_ids"] = json!(o.stks.iter().map(|s| s.raw.to_string()).collect::<Vec<_>>());
        }
        let p = format!("/channels/{}/messages", chan.raw);
        if o.files.is_empty() {
            self.0.rest.js(Method::POST, &p, Some(j)).await?;
        } else {
            let mut fs = vec![];
            let mut at = vec![];
            for (i, f) in o.files.iter().enumerate() {
                let n = f.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(|| format!("file{i}"));
                let d = tokio::fs::read(f).await?;
                at.push(json!({"id": i, "filename": n}));
                let m = mime(&n).to_string();
                fs.push((n, d, m));
            }
            j["attachments"] = Value::Array(at);
            self.0.rest.mp(&p, j, fs).await?;
        }
        Ok(())
    }

    async fn edit(&self, chan: &Rid, msg: &Rid, body: String) -> Res<()> {
        let p = format!("/channels/{}/messages/{}", chan.raw, msg.raw);
        self.0.rest.js(Method::PATCH, &p, Some(json!({"content": body}))).await?;
        Ok(())
    }

    async fn del(&self, chan: &Rid, msg: &Rid) -> Res<()> {
        let p = format!("/channels/{}/messages/{}", chan.raw, msg.raw);
        self.0.rest.js(Method::DELETE, &p, None).await?;
        Ok(())
    }

    async fn rel(&self, usr: &Rid, on: bool) -> Res<()> {
        let p = format!("/users/@me/relationships/{}", usr.raw);
        if on {
            self.0.rest.js(Method::PUT, &p, Some(serde_json::json!({}))).await?;
        } else {
            self.0.rest.js(Method::DELETE, &p, None).await?;
        }
        Ok(())
    }

    async fn react(&self, chan: &Rid, msg: &Rid, e: &Emo, on: bool) -> Res<()> {
        let raw = match &e.id {
            Some(i) => format!("{}:{}", e.name, i.raw),
            None => e.name.clone(),
        };
        let en = utf8_percent_encode(&raw, NON_ALPHANUMERIC);
        let p = format!("/channels/{}/messages/{}/reactions/{}/@me", chan.raw, msg.raw, en);
        if on {
            self.0.rest.js(Method::PUT, &format!("{p}?location=Message&type=0"), None).await?;
        } else {
            self.0.rest.js(Method::DELETE, &format!("{p}?location=Message&type=0"), None).await?;
        }
        Ok(())
    }

    async fn typing(&self, chan: &Rid) -> Res<()> {
        self.0.rest.js(Method::POST, &format!("/channels/{}/typing", chan.raw), None).await?;
        Ok(())
    }

    async fn ack(&self, chan: &Rid, msg: &Rid) -> Res<()> {
        let p = format!("/channels/{}/messages/{}/ack", chan.raw, msg.raw);
        self.0.rest.js(Method::POST, &p, Some(json!({"token": null}))).await?;
        Ok(())
    }

    async fn mems(&self, gld: &Rid) -> Res<Vec<Mem>> {
        let c = {
            let uid = self.0.me.lock().unwrap().clone();
            let m = self.0.gs.lock().unwrap();
            let g = m.get(&*gld.raw).ok_or_else(|| anyhow::anyhow!("unknown guild"))?;
            g.chs
                .iter()
                .find(|c| c["type"].as_u64() == Some(0) && perm::view(&g.g, &g.my, &uid, c))
                .and_then(|c| map::s(c, "id").map(str::to_string))
        };
        let Some(c) = c else { return Ok(vec![]) };
        self.gsend(json!({"op": 14, "d": {"guild_id": &*gld.raw, "typing": true, "activities": true, "threads": true, "channels": {c: [[0, 99]]}}}))?;
        Ok(vec![])
    }

    async fn join(&self, chan: &Rid) -> Res<()> {
        let g = self.0.cg.lock().unwrap().get(&*chan.raw).cloned();
        let (g, m, d) = self.0.vo.begin(chan, g)?;
        self.gsend(vc::op4(Some(&chan.raw), g.as_deref(), m, d))
    }

    async fn leave(&self) -> Res<()> {
        self.0.st.stop_all(&self.0);
        let g = self.0.vo.gld();
        let (m, d) = self.0.vo.end();
        self.gsend(vc::op4(None, g.as_deref(), m, d))
    }

    async fn vset(&self, mute: bool, deaf: bool) -> Res<()> {
        match self.0.vo.flags(mute, deaf) {
            Some(c) => {
                let g = self.0.vo.gld();
                self.gsend(vc::op4(Some(&c), g.as_deref(), mute, deaf))
            }
            None => Ok(()),
        }
    }

    async fn vol(&self, usr: &Rid, v: f32) -> Res<()> {
        self.0.vo.vol(&usr.raw, v);
        Ok(())
    }

    async fn golive(&self, on: bool, q: Lq) -> Res<()> {
        self.0.st.golive(&self.0, on, q).await
    }

    async fn watch(&self, usr: &Rid, on: bool) -> Res<()> {
        self.0.st.watch(&self.0, &usr.raw, on)
    }

    async fn set_st(&self, st: St, act: Option<String>) -> Res<()> {
        let s = match st {
            St::On => "online",
            St::Idle => "idle",
            St::Dnd => "dnd",
            St::Off => "invisible",
        };
        let acts: Vec<Value> = act.map(|a| vec![json!({"type": 4, "name": "Custom Status", "state": a})]).unwrap_or_default();
        self.gsend(json!({"op": 3, "d": {"status": s, "since": 0, "activities": acts, "afk": false}}))
    }
}

#[cfg(test)]
mod tests;
