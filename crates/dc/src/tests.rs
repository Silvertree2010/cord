use crate::*;
use flate2::{Compress, Compression, FlushCompress};
use tokio::sync::mpsc::unbounded_channel;

fn sh() -> Sh {
    Arc::try_unwrap(Dc::new(1, "t".into()).unwrap().0).ok().unwrap()
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

#[test]
fn ready_sparse() {
    let s = sh();
    let (tx, mut rx) = unbounded_channel();
    hd::dis(&s, &tx, "READY", serde_json::json!({}));
    hd::dis(&s, &tx, "READY", serde_json::json!({"user":{}, "guilds":[{"id":"1","unavailable":true},{}], "read_state":[{}, {"id":"7","mention_count":3}], "private_channels":[{"type":1}, 5]}));
    let mut n = 0;
    while let Ok(e) = rx.try_recv() {
        if matches!(e, Ev::Gld(_)) {
            n += 1;
        }
        if let Ev::Unread { chan, ment, .. } = &e {
            assert_eq!((&*chan.raw, *ment), ("7", 3));
        }
    }
    assert_eq!(n, 1);
}

#[test]
fn chan_last_not_rolled_back() {
    let s = sh();
    let (tx, mut rx) = unbounded_channel();
    hd::dis(&s, &tx, "READY", serde_json::from_str(RD).unwrap());
    hd::dis(&s, &tx, "MESSAGE_CREATE", serde_json::json!({"id":"600","channel_id":"20","author":{"id":"200","username":"b"},"content":"x","timestamp":"2024-01-01T00:00:00+00:00"}));
    while rx.try_recv().is_ok() {}
    hd::dis(&s, &tx, "GUILD_ROLE_UPDATE", serde_json::json!({"guild_id":"10","role":{"id":"11","name":"m","permissions":"0","position":1}}));
    let mut last = None;
    while let Ok(e) = rx.try_recv() {
        if let Ev::Chan(c) = e {
            if &*c.id.raw == "20" {
                last = c.last;
            }
        }
    }
    assert_eq!(&*last.unwrap().raw, "600");
}

#[test]
fn partial_update_without_author() {
    let m = map::msg_p(1, &serde_json::json!({"id":"5","channel_id":"20","embeds":[{"title":"t"}]}), false).unwrap();
    assert_eq!((m.ts, m.body.as_str(), m.embs.len()), (0, "", 1));
    assert!(map::msg(1, &serde_json::json!({"id":"5","channel_id":"20"})).is_none());
}

#[test]
fn inflate_limits() {
    let mut i = zl::Inf::new();
    assert!(i.push(&vec![1u8; 70 << 20]).is_err());
    assert!(i.push(&[1, 2, 3]).unwrap().is_none());
}

#[test]
fn sys_and_mentions() {
    let j = serde_json::json!({
        "id":"1250000000000000000","channel_id":"20","content":"","timestamp":"2024-02-29T12:30:45+00:00","type":7,
        "author":{"id":"200","username":"bob","global_name":"Bobby","discriminator":"0"},"member":{"nick":"Bobo"}
    });
    let m = map::msg(1, &j).unwrap();
    assert!(m.sys && m.body.contains("Bobo"));
    assert_eq!(m.auth.disp.as_deref(), Some("Bobo"));
    let c = serde_json::json!({
        "id":"5","channel_id":"20","content":"","timestamp":"2024-02-29T12:00:00+00:00","type":3,
        "author":{"id":"200","username":"bob","discriminator":"0"},"call":{"ended_timestamp":"2024-02-29T12:05:30+00:00"}
    });
    assert_eq!(map::msg(1, &c).unwrap().body, "bob started a call that lasted 5 minutes.");
    let n = serde_json::json!({
        "id":"6","channel_id":"20","content":"hi <@300>","timestamp":"2024-02-29T12:00:00+00:00","type":0,
        "author":{"id":"200","username":"bob","discriminator":"0"},
        "mentions":[{"id":"300","username":"kai","global_name":"Kai","member":{"nick":"K"}}],
        "embeds":[{"type":"gifv","url":"https://tenor.com/x","thumbnail":{"url":"https://t/a.png","proxy_url":"https://media.discordapp.net/a.png"},"video":{"url":"https://t/a.mp4","proxy_url":"https://media.discordapp.net/a.mp4"}}]
    });
    let m = map::msg(1, &n).unwrap();
    assert_eq!(m.ments[0].disp.as_deref(), Some("K"));
    assert_eq!(m.embs[0].kind.as_deref(), Some("gifv"));
    assert_eq!(m.embs[0].vid.as_deref(), Some("https://media.discordapp.net/a.mp4"));
    assert_eq!(m.body, "hi <@300>");
}

#[test]
fn guild_folders_proto() {
    fn var(mut v: u64, o: &mut Vec<u8>) {
        loop {
            let b = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                o.push(b);
                break;
            }
            o.push(b | 0x80);
        }
    }
    fn ld(f: u32, b: &[u8], o: &mut Vec<u8>) {
        var(((f as u64) << 3) | 2, o);
        var(b.len() as u64, o);
        o.extend_from_slice(b);
    }
    let mut ids = vec![];
    ids.extend_from_slice(&111u64.to_le_bytes());
    ids.extend_from_slice(&222u64.to_le_bytes());
    let mut idw = vec![];
    var(1 << 3, &mut idw);
    var(77, &mut idw);
    let mut nmw = vec![];
    ld(1, b"Fun", &mut nmw);
    let mut colw = vec![];
    var(1 << 3, &mut colw);
    var(0x5865f2, &mut colw);
    let mut f1 = vec![];
    ld(1, &ids, &mut f1);
    ld(2, &idw, &mut f1);
    ld(3, &nmw, &mut f1);
    ld(4, &colw, &mut f1);
    let mut f2 = vec![];
    ld(1, &333u64.to_le_bytes(), &mut f2);
    let mut gf = vec![];
    ld(1, &f1, &mut gf);
    ld(1, &f2, &mut gf);
    let mut root = vec![];
    var(1 << 3, &mut root);
    var(5, &mut root);
    ld(14, &gf, &mut root);
    use base64::Engine;
    let b = base64::engine::general_purpose::STANDARD.encode(&root);
    let l = crate::hd::folds(1, &b).unwrap();
    assert_eq!(l.len(), 2);
    assert_eq!(l[0].id.as_deref(), Some("dcf:1:77"));
    assert_eq!(l[0].name.as_deref(), Some("Fun"));
    assert_eq!(l[0].col, Some(0x5865f2));
    assert_eq!(l[0].glds.iter().map(|g| g.raw.to_string()).collect::<Vec<_>>(), vec!["111", "222"]);
    assert_eq!(l[1].id, None);
    assert_eq!(l[1].glds[0].raw.as_ref(), "333");
}

#[test]
fn activity_text() {
    let p = |a: serde_json::Value| map::pres(&serde_json::json!({"status":"online","activities":a})).act;
    assert_eq!(p(serde_json::json!([{"type":4,"name":"Custom Status","emoji":{"name":"🔥"}}])), Some("🔥".into()));
    assert_eq!(p(serde_json::json!([{"type":4,"name":"Custom Status"},{"type":0,"name":"Roblox"}])), Some("Playing Roblox".into()));
    assert_eq!(p(serde_json::json!([{"type":2,"name":"Spotify"}])), Some("Listening to Spotify".into()));
    assert_eq!(p(serde_json::json!([{"type":4,"name":"Custom Status","state":"hi","emoji":{"id":"5","name":"pog"}}])), Some("hi".into()));
}

#[test]
fn forward_poll_reply() {
    let f = serde_json::json!({
        "id":"7","channel_id":"20","content":"","timestamp":"2024-02-29T12:00:00+00:00","type":0,
        "author":{"id":"200","username":"bob","discriminator":"0"},
        "message_reference":{"type":1,"message_id":"3","channel_id":"9"},
        "message_snapshots":[{"message":{"content":"look","attachments":[{"url":"http://a/x.png","filename":"x.png","size":1}],"embeds":[]}}]
    });
    let m = map::msg(1, &f).unwrap();
    assert!(m.fwd && m.reply.is_none());
    assert_eq!(m.body, "look");
    assert_eq!(m.atts.len(), 1);
    let p = serde_json::json!({
        "id":"8","channel_id":"20","content":"","timestamp":"2024-02-29T12:00:00+00:00","type":0,
        "author":{"id":"200","username":"bob","discriminator":"0"},
        "poll":{"question":{"text":"Pizza?"},"answers":[{"poll_media":{"text":"yes"}},{"poll_media":{"text":"no","emoji":{"name":"🍕"}}}]}
    });
    assert_eq!(map::msg(1, &p).unwrap().body, "**Pizza?**\n- yes\n- 🍕 no");
    let r = serde_json::json!({
        "id":"9","channel_id":"20","content":"ok","timestamp":"2024-02-29T12:00:00+00:00","type":19,
        "author":{"id":"200","username":"bob","discriminator":"0"},
        "message_reference":{"message_id":"4"},
        "referenced_message":{"id":"4","content":"orig","author":{"id":"300","username":"kai","discriminator":"0"}}
    });
    let m = map::msg(1, &r).unwrap();
    assert_eq!(m.reply.unwrap().raw.as_ref(), "4");
    assert_eq!(m.rqb, "orig");
    assert_eq!(m.rq.unwrap().name, "kai");
}

#[test]
fn embed_gallery_and_suppress() {
    let e = |i: &str, t: Option<&str>| {
        let mut v = serde_json::json!({"type":"rich","url":"https://x.com/a/1","image":{"url":i,"width":800,"height":600}});
        if let Some(t) = t {
            v["description"] = t.into();
        }
        v
    };
    let m = serde_json::json!({
        "id":"5","channel_id":"20","content":"https://x.com/a/1","timestamp":"2024-02-29T12:00:00+00:00","type":0,
        "author":{"id":"200","username":"bob","discriminator":"0"},
        "embeds":[e("http://i/1", Some("post")), e("http://i/2", None), e("http://i/3", None)]
    });
    let x = map::msg(1, &m).unwrap();
    assert_eq!(x.embs.len(), 1);
    assert_eq!(x.embs[0].more, vec!["http://i/2".to_string(), "http://i/3".to_string()]);
    assert_eq!((x.embs[0].iw, x.embs[0].ih), (800, 600));
    let mut s = m.clone();
    s["flags"] = 4.into();
    assert!(map::msg(1, &s).unwrap().embs.is_empty());
}
