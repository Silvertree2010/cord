use anyhow::{Result as Res, anyhow, bail};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD as B64, URL_SAFE_NO_PAD as B64U},
};
use futures::{SinkExt, StreamExt};
use rsa::{Oaep, RsaPrivateKey, pkcs8::EncodePublicKey};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Instant, interval_at};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::HeaderValue},
};

pub const URL: &str = "wss://remote-auth-gateway.discord.gg/?v=2";

#[derive(Clone, Debug)]
pub struct RaUsr {
    pub id: String,
    pub disc: String,
    pub av: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum Ra {
    Qr(String),
    Usr(RaUsr),
}

pub fn pay(s: &str) -> RaUsr {
    let mut p = s.splitn(4, ':');
    let mut n = || p.next().unwrap_or("").to_string();
    let (id, disc, av, name) = (n(), n(), n(), n());
    RaUsr { id, disc, av, name }
}

pub fn proof(n: &[u8]) -> String {
    B64U.encode(Sha256::digest(n))
}

pub async fn login(tx: UnboundedSender<Ra>) -> Res<String> {
    let k = tokio::task::spawn_blocking(|| RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048)).await??;
    let der = k.to_public_key().to_public_key_der().map_err(|e| anyhow!("{e}"))?;
    let pk = B64.encode(der.as_bytes());
    let dec = |s: &str| -> Res<Vec<u8>> { Ok(k.decrypt(Oaep::new::<Sha256>(), &B64.decode(s)?)?) };
    let mut rq = URL.into_client_request()?;
    rq.headers_mut().insert("Origin", HeaderValue::from_static("https://discord.com"));
    rq.headers_mut().insert("User-Agent", HeaderValue::from_static(crate::rest::UA));
    let (ws, _) = connect_async(rq).await?;
    let (mut wt, mut rd) = ws.split();
    let mut iv: Option<tokio::time::Interval> = None;
    loop {
        let m = tokio::select! {
            m = rd.next() => m,
            _ = async { match iv.as_mut() { Some(i) => { i.tick().await; } None => std::future::pending().await } } => {
                wt.send(Message::Text(json!({"op": "heartbeat"}).to_string().into())).await?;
                continue;
            }
        };
        let m = match m {
            Some(m) => m?,
            None => bail!("ra closed"),
        };
        let Message::Text(t) = m else {
            if let Message::Close(_) = m {
                bail!("ra closed");
            }
            continue;
        };
        let v: Value = serde_json::from_str(t.as_str())?;
        match v["op"].as_str().unwrap_or("") {
            "hello" => {
                let hi = v["heartbeat_interval"].as_u64().unwrap_or(41250);
                iv = Some(interval_at(Instant::now() + Duration::from_millis(hi), Duration::from_millis(hi)));
                wt.send(Message::Text(json!({"op": "init", "encoded_public_key": pk}).to_string().into())).await?;
            }
            "nonce_proof" => {
                let n = dec(v["encrypted_nonce"].as_str().unwrap_or(""))?;
                wt.send(Message::Text(json!({"op": "nonce_proof", "nonce": proof(&n)}).to_string().into())).await?;
            }
            "pending_remote_init" => {
                let f = v["fingerprint"].as_str().unwrap_or("");
                let _ = tx.send(Ra::Qr(format!("https://discord.com/ra/{f}")));
            }
            "pending_ticket" => {
                let p = dec(v["encrypted_user_payload"].as_str().unwrap_or(""))?;
                let _ = tx.send(Ra::Usr(pay(&String::from_utf8_lossy(&p))));
            }
            "pending_login" => {
                let tk = v["ticket"].as_str().unwrap_or("");
                let c = reqwest::Client::builder().user_agent(crate::rest::UA).build()?;
                let r: Value = c
                    .post(format!("{}/users/@me/remote-auth/login", crate::rest::API))
                    .header("Origin", "https://discord.com")
                    .json(&json!({"ticket": tk}))
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                let e = r["encrypted_token"].as_str().ok_or_else(|| anyhow!("no encrypted_token"))?;
                return Ok(String::from_utf8(dec(e)?)?);
            }
            "cancel" => bail!("login cancelled"),
            _ => {}
        }
    }
}
