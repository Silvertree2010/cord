use crate::Rid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Usr {
    pub id: Option<Rid>,
    pub name: String,
    pub disp: Option<String>,
    pub av: Option<String>,
    pub bot: bool,
    pub col: Option<u32>,
}

impl Usr {
    pub fn shown(&self) -> &str {
        self.disp.as_deref().unwrap_or(&self.name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Acc {
    pub id: u32,
    pub svc: crate::Svc,
    pub me: Usr,
    #[serde(default)]
    pub demo: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gld {
    pub id: Rid,
    pub name: String,
    pub icon: Option<String>,
    pub ban: Option<String>,
    pub owner: Option<Rid>,
    pub pos: i64,
    pub roles: Vec<Role>,
    pub emos: Vec<Emo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Role {
    pub id: Rid,
    pub name: String,
    pub col: Option<u32>,
    pub pos: i64,
    pub perm: u64,
    pub hoist: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChK {
    Txt,
    Voice,
    Cat,
    Dm,
    Grp,
    Thread,
    Forum,
    Stage,
    Ann,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chan {
    pub id: Rid,
    pub gld: Option<Rid>,
    pub par: Option<Rid>,
    pub k: ChK,
    pub name: String,
    pub topic: Option<String>,
    pub pos: i64,
    pub nsfw: bool,
    pub hidden: bool,
    pub last: Option<Rid>,
    pub rcp: Vec<Usr>,
    pub icon: Option<String>,
    #[serde(default)]
    pub enc: bool,
    #[serde(default)]
    pub mute: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Att {
    pub url: String,
    pub name: String,
    pub mime: Option<String>,
    pub size: u64,
    pub w: Option<u32>,
    pub h: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MRow {
    G { id: String, n: u32 },
    M(Rid),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Fold {
    pub id: Option<String>,
    pub name: Option<String>,
    pub col: Option<u32>,
    pub glds: Vec<Rid>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Emb {
    pub title: Option<String>,
    pub desc: Option<String>,
    pub url: Option<String>,
    pub col: Option<u32>,
    pub img: Option<String>,
    pub thumb: Option<String>,
    pub vid: Option<String>,
    pub auth: Option<String>,
    pub foot: Option<String>,
    pub prov: Option<String>,
    pub fields: Vec<(String, String, bool)>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub auth_ic: Option<String>,
    #[serde(default)]
    pub auth_url: Option<String>,
    #[serde(default)]
    pub foot_ic: Option<String>,
    #[serde(default)]
    pub ts: Option<i64>,
    #[serde(default)]
    pub iw: u32,
    #[serde(default)]
    pub ih: u32,
    #[serde(default)]
    pub more: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emo {
    pub id: Option<Rid>,
    pub name: String,
    pub url: Option<String>,
    pub anim: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct React {
    pub emo: Emo,
    pub n: u32,
    pub me: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub id: Rid,
    pub chan: Rid,
    pub auth: Usr,
    pub body: String,
    pub ts: i64,
    pub ed: Option<i64>,
    pub atts: Vec<Att>,
    pub embs: Vec<Emb>,
    pub reacts: Vec<React>,
    pub reply: Option<Rid>,
    pub stks: Vec<Att>,
    pub pin: bool,
    pub sys: bool,
    pub del: bool,
    pub hist: Vec<String>,
    #[serde(default)]
    pub ments: Vec<Usr>,
    #[serde(default)]
    pub aroles: Vec<Rid>,
    #[serde(default)]
    pub fwd: bool,
    #[serde(default)]
    pub rq: Option<Usr>,
    #[serde(default)]
    pub rqb: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelK {
    Friend,
    In,
    Out,
    Block,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rel {
    pub usr: Usr,
    pub k: RelK,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum St {
    On,
    Idle,
    Dnd,
    #[default]
    Off,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pres {
    pub st: St,
    pub act: Option<String>,
    pub mob: bool,
    pub web: bool,
    pub desk: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mem {
    pub usr: Usr,
    pub gld: Rid,
    pub nick: Option<String>,
    pub roles: Vec<Rid>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Out {
    pub body: String,
    pub reply: Option<Rid>,
    pub files: Vec<std::path::PathBuf>,
    pub silent: bool,
    pub stks: Vec<Rid>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Vs {
    pub usr: Usr,
    pub chan: Option<Rid>,
    pub gld: Option<Rid>,
    pub mute: bool,
    pub deaf: bool,
    pub smute: bool,
    pub sdeaf: bool,
    pub stream: bool,
    pub vid: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CallSt {
    pub chan: Option<Rid>,
    pub up: bool,
    pub mute: bool,
    pub deaf: bool,
    pub ping: Option<u32>,
    pub e2ee: bool,
    pub live: bool,
    pub err: Option<String>,
}
