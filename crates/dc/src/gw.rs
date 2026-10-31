use crate::{Sh, hd, rest, zl::Inf};
use cord_core::{Ev, Res, Tx};
use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::time::{Instant, Interval, interval_at, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub const GW: &str = "wss://gateway.discord.gg/?v=9&encoding=json&compress=zlib-stream";

pub enum Fin {
    Re,
    Fatal(String),
}

pub fn idn(tok: &str) -> Value {
    json!({
        "op": 2,
        "d": {
            "token": tok,
            "capabilities": 30717,
            "properties": rest::props(),
            "presence": {"status": "online", "since": 0, "activities": [], "afk": false},
            "compress": false,
            "client_state": {
                "guild_versions": {},
                "highest_last_message_id": "0",
                "read_state_version": 0,
                "user_guild_settings_version": -1,
                "user_settings_version": -1,
                "private_channels_version": "0",
                "api_code_version": 0
            }
        }
    })
}

pub async fn run(sh: Arc<Sh>, tx: Tx) {
    let mut n = 0u32;
    loop {
        let t0 = std::time::Instant::now();
        let r = sess(&sh, &tx).await;
        *sh.gt.lock().unwrap() = None;
        let _ = tx.send(Ev::Conn { acc: sh.a, up: false });
        match r {
            Ok(Fin::Fatal(m)) => {
                let _ = tx.send(Ev::Err { acc: sh.a, msg: m });
                return;
            }
            Ok(Fin::Re) => {}
            Err(e) => {
                tracing::warn!("gw: {e}");
                let _ = tx.send(Ev::Err { acc: sh.a, msg: e.to_string() });
            }
        }
        if t0.elapsed() > Duration::from_secs(60) {
            n = 0;
        } else {
            n = (n + 1).min(5);
        }
        let d = (1u64 << n).min(30) as f64 + rand::random::<f64>();
        sleep(Duration::from_secs_f64(d)).await;
    }
}

async fn tick(iv: &mut Option<Interval>) {
    match iv {
        Some(i) => {
            i.tick().await;
        }
        None => std::future::pending().await,
    }
}

async fn sess(sh: &Sh, tx: &Tx) -> Res<Fin> {
    let (url, rs) = {
        let s = sh.ss.lock().unwrap();
        match (&s.id, &s.url) {
            (Some(_), Some(u)) => (format!("{u}/?v=9&encoding=json&compress=zlib-stream"), true),
            _ => (GW.to_string(), false),
        }
    };
    let _ = rs;
    let (ws, _) = connect_async(url).await?;
    let (mut wt, mut rd) = ws.split();
    let (otx, mut orx) = tokio::sync::mpsc::unbounded_channel::<String>();
    *sh.gt.lock().unwrap() = Some(otx);
    let mut inf = Inf::new();
    let mut iv: Option<Interval> = None;
    let mut ack = true;
    loop {
        tokio::select! {
            m = rd.next() => {
                let m = match m { Some(m) => m?, None => return Ok(Fin::Re) };
                match m {
                    Message::Binary(b) => {
                        let Some(t) = inf.push(&b)? else { continue };
                        let v: Value = serde_json::from_str(&t)?;
                        if let Some(s) = v["s"].as_u64() {
                            sh.ss.lock().unwrap().seq = Some(s);
                        }
                        match v["op"].as_u64() {
                            Some(0) => {
                                let t = v["t"].as_str().unwrap_or("").to_string();
                                hd::dis(sh, tx, &t, v["d"].clone());
                            }
                            Some(1) => {
                                let s = sh.ss.lock().unwrap().seq;
                                wt.send(Message::Text(json!({"op": 1, "d": s}).to_string().into())).await?;
                            }
                            Some(7) => return Ok(Fin::Re),
                            Some(9) => {
                                if v["d"].as_bool() != Some(true) {
                                    let mut s = sh.ss.lock().unwrap();
                                    s.id = None;
                                    s.seq = None;
                                    s.url = None;
                                }
                                sleep(Duration::from_millis(1000 + (rand::random::<f64>() * 4000.0) as u64)).await;
                                return Ok(Fin::Re);
                            }
                            Some(10) => {
                                let hi = v["d"]["heartbeat_interval"].as_u64().unwrap_or(41250);
                                let j = (hi as f64 * rand::random::<f64>()) as u64;
                                iv = Some(interval_at(Instant::now() + Duration::from_millis(j), Duration::from_millis(hi)));
                                ack = true;
                                let (id, seq) = { let s = sh.ss.lock().unwrap(); (s.id.clone(), s.seq) };
                                let p = match id {
                                    Some(id) => json!({"op": 6, "d": {"token": sh.tok, "session_id": id, "seq": seq}}),
                                    None => idn(&sh.tok),
                                };
                                wt.send(Message::Text(p.to_string().into())).await?;
                            }
                            Some(11) => ack = true,
                            _ => {}
                        }
                    }
                    Message::Close(cf) => {
                        let c = cf.map(|c| u16::from(c.code)).unwrap_or(0);
                        if matches!(c, 4004 | 4010 | 4011 | 4012 | 4013 | 4014) {
                            return Ok(Fin::Fatal(format!("gateway closed {c}")));
                        }
                        if c == 4007 || c == 4009 {
                            let mut s = sh.ss.lock().unwrap();
                            s.id = None;
                            s.seq = None;
                            s.url = None;
                        }
                        return Ok(Fin::Re);
                    }
                    _ => {}
                }
            }
            Some(s) = orx.recv() => {
                wt.send(Message::Text(s.into())).await?;
            }
            _ = tick(&mut iv) => {
                if !ack { return Ok(Fin::Re); }
                ack = false;
                let s = sh.ss.lock().unwrap().seq;
                wt.send(Message::Text(json!({"op": 1, "d": s}).to_string().into())).await?;
            }
        }
    }
}
