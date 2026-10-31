use anyhow::{Result as Res, bail};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use reqwest::{Client, Method, RequestBuilder, multipart};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

pub const BN: u32 = 485000;
pub const API: &str = "https://discord.com/api/v9";
pub const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.80 Chrome/134.0.6998.205 Electron/35.3.0 Safari/537.36";

pub fn props() -> Value {
    json!({
        "os": "Linux",
        "browser": "Discord Client",
        "release_channel": "stable",
        "client_version": "0.0.80",
        "os_version": "6.8.0-generic",
        "os_arch": "x64",
        "app_arch": "x64",
        "system_locale": "en-US",
        "has_client_mods": false,
        "browser_user_agent": UA,
        "browser_version": "35.3.0",
        "window_manager": "KDE,unknown",
        "client_build_number": BN,
        "native_build_number": null,
        "client_event_source": null
    })
}

pub fn rkey(m: &Method, p: &str) -> String {
    let p = p.split('?').next().unwrap_or(p);
    let mut o = String::new();
    o.push_str(m.as_str());
    let mut maj = false;
    let mut seen = false;
    for s in p.split('/').filter(|s| !s.is_empty()) {
        o.push('/');
        if maj && !seen {
            o.push_str(s);
            seen = true;
            maj = false;
        } else if s.bytes().all(|b| b.is_ascii_digit()) && !s.is_empty() {
            o.push_str(":id");
        } else {
            o.push_str(s);
            if matches!(s, "channels" | "guilds" | "webhooks") && !seen {
                maj = true;
            }
        }
    }
    o
}

pub struct Rest {
    c: Client,
    tok: String,
    sp: String,
    bk: Mutex<HashMap<String, Instant>>,
}

impl Rest {
    pub fn new(tok: String) -> Self {
        let c = Client::builder().user_agent(UA).build().expect("http client");
        let sp = B64.encode(props().to_string());
        Self { c, tok, sp, bk: Mutex::new(HashMap::new()) }
    }

    async fn wt(&self, k: &str) {
        let d = {
            let b = self.bk.lock().unwrap();
            let now = Instant::now();
            [k, "global"]
                .iter()
                .filter_map(|k| b.get(*k))
                .filter(|t| **t > now)
                .map(|t| *t - now)
                .max()
        };
        if let Some(d) = d {
            tokio::time::sleep(d).await;
        }
    }

    pub async fn call(&self, k: &str, f: impl Fn(&Client) -> RequestBuilder) -> Res<Value> {
        for _ in 0..6 {
            self.wt(k).await;
            let r = f(&self.c)
                .header("Authorization", &self.tok)
                .header("X-Super-Properties", &self.sp)
                .header("X-Discord-Locale", "en-US")
                .header("Origin", "https://discord.com")
                .header("Referer", "https://discord.com/channels/@me")
                .send()
                .await?;
            let st = r.status();
            let hd = r.headers().clone();
            let hv = |n: &str| hd.get(n).and_then(|v| v.to_str().ok()).map(str::to_owned);
            if hv("x-ratelimit-remaining").as_deref() == Some("0") {
                let ra = hv("x-ratelimit-reset-after").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let now = Instant::now();
                let mut b = self.bk.lock().unwrap();
                b.retain(|_, t| *t > now);
                b.insert(k.to_string(), now + Duration::from_secs_f64(ra.clamp(0.0, 3600.0)));
            }
            if st.as_u16() == 429 {
                let v: Value = r.json().await.unwrap_or(Value::Null);
                let ra = v["retry_after"].as_f64().or_else(|| hv("retry-after").and_then(|s| s.parse().ok())).unwrap_or(1.0);
                let g = v["global"].as_bool().unwrap_or(false) || hv("x-ratelimit-global").is_some();
                let t = Instant::now() + Duration::from_secs_f64(ra.clamp(0.0, 3600.0) + 0.05);
                let mut b = self.bk.lock().unwrap();
                b.insert(if g { "global".into() } else { k.to_string() }, t);
                continue;
            }
            let tx = r.text().await?;
            if !st.is_success() {
                bail!("{} {}", st, tx);
            }
            return Ok(if tx.trim().is_empty() { Value::Null } else { serde_json::from_str(&tx)? });
        }
        bail!("rate limited: {k}")
    }

    fn blocked(p: &str) -> bool {
        if !cord_core::feat::on("notrack") {
            return false;
        }
        let path = p.split('?').next().unwrap_or(p);
        path.split('/').any(|s| s == "science" || s == "track")
    }

    pub async fn js(&self, m: Method, p: &str, b: Option<Value>) -> Res<Value> {
        if Self::blocked(p) {
            bail!("tracking request blocked: {p}");
        }
        let k = rkey(&m, p);
        let u = format!("{API}{p}");
        self.call(&k, |c| {
            let r = c.request(m.clone(), &u);
            match &b {
                Some(b) => r.json(b),
                None => r,
            }
        })
        .await
    }

    pub async fn mp(&self, p: &str, j: Value, fs: Vec<(String, Vec<u8>, String)>) -> Res<Value> {
        let m = Method::POST;
        let k = rkey(&m, p);
        let u = format!("{API}{p}");
        let js = j.to_string();
        self.call(&k, |c| {
            let mut f = multipart::Form::new().part(
                "payload_json",
                multipart::Part::text(js.clone()).mime_str("application/json").unwrap(),
            );
            for (i, (n, d, mt)) in fs.iter().enumerate() {
                let pt = multipart::Part::bytes(d.clone()).file_name(n.clone()).mime_str(mt).unwrap();
                f = f.part(format!("files[{i}]"), pt);
            }
            c.post(&u).multipart(f)
        })
        .await
    }
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn rk() {
        assert_eq!(rkey(&Method::GET, "/channels/123/messages/456?x=1"), "GET/channels/123/messages/:id");
        assert_eq!(rkey(&Method::PUT, "/channels/9/messages/8/reactions/a%3A1/@me"), "PUT/channels/9/messages/:id/reactions/a%3A1/@me");
    }
}
