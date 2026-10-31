use crate::{Rid, Svc};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Caps {
    pub call: bool,
    pub permalink: &'static str,
    #[serde(rename = "fakeEmoji")]
    pub fake_emoji: bool,
    pub threads: bool,
}

pub fn caps(sv: Svc) -> Caps {
    match sv {
        Svc::Dc => Caps { call: true, permalink: "https://discord.com/channels/", fake_emoji: true, threads: true },
        Svc::Mx => Caps { call: false, permalink: "https://matrix.to/#/", fake_emoji: false, threads: false },
        Svc::St => Caps { call: false, permalink: "https://stoat.chat/", fake_emoji: false, threads: false },
    }
}

pub fn link(chan: &Rid, gld: Option<&Rid>, msg: Option<&Rid>) -> String {
    let b = caps(chan.svc).permalink;
    match chan.svc {
        Svc::Dc => {
            let g = gld.map(|g| &*g.raw).unwrap_or("@me");
            match msg {
                Some(m) => format!("{b}{g}/{}/{}", chan.raw, m.raw),
                None => format!("{b}{g}/{}", chan.raw),
            }
        }
        Svc::Mx => match msg {
            Some(m) => format!("{b}{}/{}", chan.raw, m.raw),
            None => format!("{b}{}", chan.raw),
        },
        Svc::St => match (gld, msg) {
            (Some(g), Some(m)) => format!("{b}server/{}/channel/{}/{}", g.raw, chan.raw, m.raw),
            (Some(g), None) => format!("{b}server/{}/channel/{}", g.raw, chan.raw),
            (None, Some(m)) => format!("{b}channel/{}/{}", chan.raw, m.raw),
            (None, None) => format!("{b}channel/{}", chan.raw),
        },
    }
}

pub fn svc_list() -> Vec<serde_json::Value> {
    use serde_json::json;
    let f = |k: &str, label: &str, secret: bool, ph: &str| json!({"k": k, "label": label, "secret": secret, "ph": ph});
    vec![
        json!({"id":"dc","name":"Discord","col":"#5865f2","modes":["qr","tok"],"fields":[f("token","Token",true,"Account token")]}),
        json!({"id":"mx","name":"Matrix","col":"#0dbd8b","modes":["tok"],"fields":[
            f("hs","Homeserver",false,"matrix.org"),
            f("user","Username",false,"Username or @user:server"),
            f("pass","Password",true,"Password"),
            f("recovery","Recovery key",true,"Recovery key (optional)")]}),
        json!({"id":"st","name":"Stoat","col":"#ff6b4a","modes":["tok"],"fields":[
            f("base","Server",false,"https://api.stoat.chat"),
            f("email","Email",false,"Email"),
            f("pass","Password",true,"Password"),
            f("token","Session token",true,"Session token (instead of email and password)")]}),
    ]
}
