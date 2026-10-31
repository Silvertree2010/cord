use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct File {
    #[serde(rename = "_id", alias = "id", default)]
    pub id: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub metadata: Meta,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub size: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "type")]
pub enum Meta {
    Image {
        width: u32,
        height: u32,
    },
    Video {
        width: u32,
        height: u32,
    },
    #[serde(other)]
    #[default]
    File,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum Relation {
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
    #[serde(other)]
    #[default]
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Presence {
    Online,
    Idle,
    Focus,
    Busy,
    Invisible,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Status {
    pub text: Option<String>,
    pub presence: Option<Presence>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct User {
    #[serde(rename = "_id", alias = "id", default)]
    pub id: String,
    #[serde(default)]
    pub username: String,
    pub display_name: Option<String>,
    pub avatar: Option<File>,
    pub status: Option<Status>,
    pub bot: Option<Value>,
    #[serde(default)]
    pub relationship: Relation,
    #[serde(default)]
    pub online: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Key {
    pub server: String,
    pub user: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Member {
    #[serde(rename = "_id", alias = "id")]
    pub id: Key,
    pub nickname: Option<String>,
    pub avatar: Option<File>,
    #[serde(default)]
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Perm {
    #[serde(default)]
    pub a: i64,
    #[serde(default)]
    pub d: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Role {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub permissions: Perm,
    pub colour: Option<String>,
    #[serde(default)]
    pub hoist: bool,
    #[serde(default)]
    pub rank: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Cat {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub channels: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Server {
    #[serde(rename = "_id", alias = "id")]
    pub id: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub channels: Vec<String>,
    pub categories: Option<Vec<Cat>>,
    #[serde(default)]
    pub roles: HashMap<String, Role>,
    pub icon: Option<File>,
    pub banner: Option<File>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "channel_type")]
pub enum Channel {
    SavedMessages {
        #[serde(rename = "_id", alias = "id")]
        id: String,
        #[serde(default)]
        user: String,
    },
    DirectMessage {
        #[serde(rename = "_id", alias = "id")]
        id: String,
        #[serde(default = "yes")]
        active: bool,
        #[serde(default)]
        recipients: Vec<String>,
        last_message_id: Option<String>,
    },
    Group {
        #[serde(rename = "_id", alias = "id")]
        id: String,
        #[serde(default)]
        name: String,
        description: Option<String>,
        #[serde(default)]
        recipients: Vec<String>,
        icon: Option<File>,
        last_message_id: Option<String>,
        #[serde(default)]
        nsfw: bool,
    },
    TextChannel {
        #[serde(rename = "_id", alias = "id")]
        id: String,
        server: String,
        #[serde(default)]
        name: String,
        description: Option<String>,
        icon: Option<File>,
        last_message_id: Option<String>,
        #[serde(default)]
        nsfw: bool,
        voice: Option<Value>,
    },
    VoiceChannel {
        #[serde(rename = "_id", alias = "id")]
        id: String,
        server: String,
        #[serde(default)]
        name: String,
        description: Option<String>,
        icon: Option<File>,
        #[serde(default)]
        nsfw: bool,
    },
}

impl Channel {
    pub fn id(&self) -> &str {
        match self {
            Channel::SavedMessages { id, .. }
            | Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. }
            | Channel::TextChannel { id, .. }
            | Channel::VoiceChannel { id, .. } => id,
        }
    }
    pub fn server(&self) -> Option<&str> {
        match self {
            Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => Some(server),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Parent {
    Server { id: String },
    #[serde(other)]
    Detached,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Emoji {
    #[serde(rename = "_id", alias = "id")]
    pub id: String,
    pub parent: Parent,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub animated: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Img {
    pub url: String,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Web {
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<Img>,
    pub video: Option<Img>,
    pub site_name: Option<String>,
    pub icon_url: Option<String>,
    pub colour: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Txt {
    pub icon_url: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<File>,
    pub colour: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Embed {
    Website(Web),
    Image(Img),
    Video(Img),
    Text(Txt),
    #[serde(other)]
    Nope,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Masq {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub colour: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Hook {
    #[serde(default)]
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Sys {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "user_added")]
    Added { id: String, by: String },
    #[serde(rename = "user_remove")]
    Remove { id: String, by: String },
    #[serde(rename = "user_joined")]
    Joined { id: String },
    #[serde(rename = "user_left")]
    Left { id: String },
    #[serde(rename = "user_kicked")]
    Kicked { id: String },
    #[serde(rename = "user_banned")]
    Banned { id: String },
    #[serde(rename = "channel_renamed")]
    Renamed { name: String, by: String },
    #[serde(rename = "channel_description_changed")]
    DescChanged { by: String },
    #[serde(rename = "channel_icon_changed")]
    IconChanged { by: String },
    #[serde(rename = "channel_ownership_changed")]
    Owner { from: String, to: String },
    #[serde(rename = "message_pinned")]
    Pinned { id: String, by: String },
    #[serde(rename = "message_unpinned")]
    Unpinned { id: String, by: String },
    #[serde(rename = "call_started")]
    Call { by: String },
    #[serde(other)]
    Other,
}

fn ordered<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<(String, Vec<String>)>, D::Error> {
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Vec<(String, Vec<String>)>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("map")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut m: A) -> Result<Self::Value, A::Error> {
            let mut o = Vec::new();
            while let Some((k, v)) = m.next_entry::<String, Vec<String>>()? {
                o.push((k, v));
            }
            Ok(o)
        }
    }
    d.deserialize_map(V)
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Message {
    #[serde(rename = "_id", alias = "id")]
    pub id: String,
    pub channel: String,
    #[serde(default)]
    pub author: String,
    pub user: Option<User>,
    pub member: Option<Member>,
    pub webhook: Option<Hook>,
    pub content: Option<String>,
    pub system: Option<Sys>,
    pub attachments: Option<Vec<File>>,
    pub edited: Option<String>,
    pub embeds: Option<Vec<Embed>>,
    pub mentions: Option<Vec<String>>,
    pub replies: Option<Vec<String>>,
    #[serde(default, deserialize_with = "ordered")]
    pub reactions: Vec<(String, Vec<String>)>,
    pub masquerade: Option<Masq>,
    pub pinned: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Bulk {
    Full {
        messages: Vec<Message>,
        #[serde(default)]
        users: Vec<Value>,
        #[serde(default)]
        members: Vec<Value>,
    },
    Plain(Vec<Message>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Unread {
    #[serde(rename = "_id")]
    pub id: UnreadKey,
    pub last_id: Option<String>,
    #[serde(default)]
    pub mentions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UnreadKey {
    pub channel: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Bulk {
        v: Vec<Event>,
    },
    Error {
        data: Value,
    },
    Authenticated,
    Logout,
    Ready {
        #[serde(default)]
        users: Vec<Value>,
        #[serde(default)]
        servers: Vec<Value>,
        #[serde(default)]
        channels: Vec<Value>,
        #[serde(default)]
        members: Vec<Value>,
        #[serde(default)]
        emojis: Vec<Value>,
    },
    Pong,
    Message(Message),
    MessageUpdate {
        id: String,
        channel: String,
    },
    MessageAppend {
        id: String,
        channel: String,
    },
    MessageDelete {
        id: String,
        channel: String,
    },
    MessageReact {
        id: String,
        channel_id: String,
    },
    MessageUnreact {
        id: String,
        channel_id: String,
    },
    MessageRemoveReaction {
        id: String,
        channel_id: String,
    },
    BulkMessageDelete {
        channel: String,
        ids: Vec<String>,
    },
    ServerCreate {
        server: Value,
        #[serde(default)]
        channels: Vec<Value>,
        #[serde(default)]
        emojis: Vec<Value>,
    },
    ServerUpdate {
        id: String,
        data: Value,
        #[serde(default)]
        clear: Vec<String>,
    },
    ServerDelete {
        id: String,
    },
    ServerMemberUpdate {
        id: Key,
        data: Value,
        #[serde(default)]
        clear: Vec<String>,
    },
    ServerMemberJoin {
        member: Value,
    },
    ServerMemberLeave {
        id: String,
        user: String,
    },
    ServerRoleUpdate {
        id: String,
        role_id: String,
        data: Value,
        #[serde(default)]
        clear: Vec<String>,
    },
    ServerRoleDelete {
        id: String,
        role_id: String,
    },
    UserUpdate {
        id: String,
        data: Value,
        #[serde(default)]
        clear: Vec<String>,
    },
    UserRelationship {
        user: Value,
    },
    EmojiCreate(Value),
    EmojiDelete {
        id: String,
    },
    ChannelCreate(Value),
    ChannelUpdate {
        id: String,
        data: Value,
        #[serde(default)]
        clear: Vec<String>,
    },
    ChannelDelete {
        id: String,
    },
    ChannelGroupJoin {
        id: String,
        user: String,
    },
    ChannelGroupLeave {
        id: String,
        user: String,
    },
    ChannelStartTyping {
        id: String,
        user: String,
    },
    ChannelAck {
        id: String,
        user: String,
        message_id: String,
    },
    #[serde(other)]
    Unknown,
}
