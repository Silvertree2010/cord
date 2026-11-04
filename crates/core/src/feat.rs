use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Feat {
    pub k: &'static str,
    pub on: bool,
    pub n: &'static str,
    pub d: &'static str,
}

pub const FEATS: &[Feat] = &[
    Feat { k: "msglog", on: true, n: "Message logger", d: "Keep deleted messages and edit history visible" },
    Feat { k: "hidden", on: true, n: "Show hidden channels", d: "List channels you cannot open, marked with a lock" },
    Feat { k: "fakenitro", on: true, n: "Emoji and sticker passthrough", d: "Send any emoji or sticker as a link when you cannot use it natively" },
    Feat { k: "streamq", on: true, n: "Stream quality unlock", d: "Allow screen share above 720p30; when off, streams are capped to 720p30" },
    Feat { k: "silenttyping", on: false, n: "Silent typing", d: "Do not send typing indicators" },
    Feat { k: "noread", on: false, n: "Do not send read receipts", d: "Read messages without marking them as read for others" },
    Feat { k: "platform", on: true, n: "Platform indicators", d: "Show desktop, mobile or web next to names" },
    Feat { k: "spoilers", on: false, n: "Reveal spoilers", d: "Show spoiler text without clicking" },
    Feat { k: "notrack", on: true, n: "Block tracking", d: "Refuse to send any Discord /science or /track analytics request" },
    Feat { k: "translate", on: true, n: "Translate", d: "Translate messages from the context menu" },
    Feat { k: "spotify", on: true, n: "Media controls", d: "Player bar for Spotify (MPRIS players on Linux)" },
    Feat { k: "relnotif", on: true, n: "Relationship notifier", d: "Tell you when someone removes you as a friend or a server removes you" },
    Feat { k: "typingtweaks", on: true, n: "Typing indicators everywhere", d: "Show who types in the channel list and server bar" },
    Feat { k: "callTimer", on: true, n: "Call timer", d: "Show how long you are in a call" },
    Feat { k: "copyraw", on: true, n: "Copy raw message", d: "Context menu entry to copy the unformatted message" },
    Feat { k: "readall", on: true, n: "Read all", d: "One click to mark every channel as read" },
    Feat { k: "pinDms", on: true, n: "Pin DMs", d: "Keep chosen DMs on top of the list" },
    Feat { k: "noreplyping", on: false, n: "No reply mention", d: "Replies do not ping the author (Discord)" },
    Feat { k: "anim", on: true, n: "Animations", d: "Interface motion" },
];

static ST: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();

fn st() -> &'static Mutex<HashMap<String, bool>> {
    ST.get_or_init(Default::default)
}

pub fn on(k: &str) -> bool {
    if let Some(v) = st().lock().ok().and_then(|g| g.get(k).copied()) {
        return v;
    }
    FEATS.iter().find(|f| f.k == k).map(|f| f.on).unwrap_or(false)
}

pub fn set(k: &str, v: bool) {
    if let Ok(mut g) = st().lock() {
        g.insert(k.to_string(), v);
    }
}

pub fn known(k: &str) -> bool {
    FEATS.iter().any(|f| f.k == k)
}
