use crate::*;
use flate2::{Compress, Compression, FlushCompress};
use tokio::sync::mpsc::unbounded_channel;

fn sh() -> Sh {
    Arc::try_unwrap(Dc::new(1, "t".into()).0).ok().unwrap()
}

const RD: &str = r#"{
 "user":{"id":"100","username":"me","global_name":"Me","avatar":"a_abc","discriminator":"0"},
 "users":[{"id":"200","username":"bob","avatar":null,"discriminator":"0"}],
 "session_id":"s1","resume_gateway_url":"wss://x.discord.gg",
 "guilds":[{"id":"10","properties":{"name":"G","icon":"ic","owner_id":"999"},
   "roles":[{"id":"10","name":"@everyone","permissions":"1024","position":0,"color":0},
            {"id":"11","name":"mod","permissions":"0","position":1,"color":255}],
   "channels":[{"id":"20","type":0,"name":"gen","position":1,"last_message_id":"500","permission_overwrites":[]},
               {"id":"21","type":0,"name":"sec","position":2,"permission_overwrites":[{"id":"10","type":0,"allow":"0","deny":"1024"}]},
               {"id":"22","type":4,"name":"cat","position":0,"permission_overwrites":[]}],
   "emojis":[{"id":"30","name":"e","animated":true}]}],
 "merged_members":[[{"user_id":"100","roles":["11"]}]],
 "private_channels":[{"id":"40","type":1,"recipient_ids":["200"],"last_message_id":"900"}],
 "relationships":[{"id":"200","type":1,"user_id":"200"}],
 "read_state":{"entries":[{"id":"20","last_message_id":"400","mention_count":2},{"id":"40","last_message_id":"900","mention_count":0}]},
 "user_guild_settings":{"entries":[]}
}"#;

#[test]
fn ready() {
    let s = sh();
    let (tx, mut rx) = unbounded_channel();
    hd::dis(&s, &tx, "READY", serde_json::from_str(RD).unwrap());
    let mut v = vec![];
    while let Ok(e) = rx.try_recv() {
        v.push(e);
    }
    assert!(matches!(&v[0], Ev::Ready(a) if a.me.name == "me" && a.me.av.as_ref().unwrap().ends_with(".gif?size=128")));
    let ch: Vec<&Chan> = v.iter().filter_map(|e| if let Ev::Chan(c) = e { Some(c) } else { None }).collect();
    assert_eq!(ch.len(), 4);
    assert!(!ch.iter().find(|c| &*c.id.raw == "20").unwrap().hidden);
    assert!(ch.iter().find(|c| &*c.id.raw == "21").unwrap().hidden);
    let dm = ch.iter().find(|c| &*c.id.raw == "40").unwrap();
    assert_eq!(dm.rcp[0].name, "bob");
    assert_eq!(dm.name, "bob");
    assert!(v.iter().any(|e| matches!(e, Ev::Rel(r) if r.k == RelK::Friend)));
    assert!(v.iter().any(|e| matches!(e, Ev::Unread { chan, n: 1, ment: 2 } if &*chan.raw == "20")));
    assert!(!v.iter().any(|e| matches!(e, Ev::Unread { chan, .. } if &*chan.raw == "40")));
    let g = v.iter().find_map(|e| if let Ev::Gld(g) = e { Some(g) } else { None }).unwrap();
    assert_eq!(g.name, "G");
    assert_eq!(g.roles.len(), 2);
    assert!(g.emos[0].anim);
}

#[test]
fn msgmap() {
    let j = serde_json::json!({
        "id":"5","channel_id":"20","content":"hi","timestamp":"2024-02-29T12:30:45.123000+00:00",
        "edited_timestamp":"2024-02-29T14:30:45+02:00","type":19,"pinned":true,
        "author":{"id":"200","username":"bob","discriminator":"0"},
        "attachments":[{"url":"http://a/b.png","filename":"b.png","content_type":"image/png","size":10,"width":3,"height":4}],
        "embeds":[{"title":"t","color":5,"image":{"url":"http://i"},"fields":[{"name":"n","value":"v","inline":true}]}],
        "reactions":[{"count":2,"me":true,"emoji":{"id":"9","name":"x","animated":false}}],
        "message_reference":{"message_id":"4"},
        "sticker_items":[{"id":"7","name":"s","format_type":4}]
    });
    let m = map::msg(1, &j).unwrap();
    assert_eq!(m.ts, 1709209845123);
    assert_eq!(m.ed, Some(1709209845000));
    assert!(!m.sys && m.pin);
    assert_eq!(m.atts[0].w, Some(3));
    assert_eq!(m.embs[0].fields[0].2, true);
    assert_eq!(m.reacts[0].n, 2);
    assert_eq!(m.reply.unwrap().raw.as_ref(), "4");
    assert!(m.stks[0].url.ends_with("7.gif"));
    let mut j2 = j.clone();
    j2["type"] = 7.into();
    assert!(map::msg(1, &j2).unwrap().sys);
}

#[test]
fn perms() {
    let g = map::gld(
        1,
        &serde_json::json!({"id":"10","properties":{"name":"g","owner_id":"1"},"roles":[
            {"id":"10","permissions":"1024"},{"id":"11","permissions":"0"},{"id":"12","permissions":"8"}]}),
        0,
    );
    let ow = |o: &str| serde_json::json!({"permission_overwrites": serde_json::from_str::<Value>(o).unwrap()});
    let my = vec!["11".to_string()];
    assert!(perm::view(&g, &my, "5", &ow("[]")));
    assert!(!perm::view(&g, &my, "5", &ow(r#"[{"id":"10","type":0,"allow":"0","deny":"1024"}]"#)));
    assert!(perm::view(&g, &my, "5", &ow(r#"[{"id":"10","type":0,"allow":"0","deny":"1024"},{"id":"11","type":0,"allow":"1024","deny":"0"}]"#)));
    assert!(!perm::view(&g, &my, "5", &ow(r#"[{"id":"11","type":0,"allow":"0","deny":"1024"},{"id":"5","type":1,"allow":"0","deny":"1024"}]"#)));
    assert!(perm::view(&g, &my, "5", &ow(r#"[{"id":"10","type":0,"allow":"0","deny":"1024"},{"id":"5","type":1,"allow":"1024","deny":"0"}]"#)));
    let adm = vec!["12".to_string()];
    assert!(perm::view(&g, &adm, "5", &ow(r#"[{"id":"10","type":0,"allow":"0","deny":"1024"}]"#)));
    assert!(perm::view(&g, &[], "1", &ow(r#"[{"id":"10","type":0,"allow":"0","deny":"1024"}]"#)));
}

#[test]
fn inflate() {
    let mut c = Compress::new(Compression::default(), true);
    let mut enc = |s: &str| {
        let mut o = Vec::with_capacity(256);
        c.compress_vec(s.as_bytes(), &mut o, FlushCompress::Sync).unwrap();
        o
    };
    let (a, b) = (enc(r#"{"op":10,"d":{"heartbeat_interval":41250}}"#), enc(r#"{"op":11}"#));
    assert!(a.ends_with(&[0, 0, 0xff, 0xff]));
    let mut i = zl::Inf::new();
    let mid = a.len() / 2;
    assert!(i.push(&a[..mid]).unwrap().is_none());
    assert_eq!(i.push(&a[mid..]).unwrap().unwrap(), r#"{"op":10,"d":{"heartbeat_interval":41250}}"#);
    assert_eq!(i.push(&b).unwrap().unwrap(), r#"{"op":11}"#);
}

#[test]
fn ra() {
    let u = ra::pay("123:0:abcd:some:name");
    assert_eq!((u.id.as_str(), u.name.as_str()), ("123", "some:name"));
    assert_eq!(ra::proof(b"x").len(), 43);
}
