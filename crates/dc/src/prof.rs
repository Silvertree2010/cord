use crate::map::{CDN, deco, s, tag};
use serde_json::{Value, json};

fn img(kind: &str, id: &str, h: Option<&str>, sz: u32) -> Value {
    match h.filter(|x| !x.is_empty()) {
        Some(h) => {
            let ext = if h.starts_with("a_") { "gif" } else { "png" };
            json!(format!("{CDN}/{kind}/{id}/{h}.{ext}?size={sz}"))
        }
        None => Value::Null,
    }
}

pub fn conn_url(t: &str, id: &str, name: &str) -> Option<String> {
    Some(match t {
        "github" => format!("https://github.com/{name}"),
        "twitter" => format!("https://x.com/{name}"),
        "steam" => format!("https://steamcommunity.com/profiles/{id}"),
        "spotify" => format!("https://open.spotify.com/user/{id}"),
        "youtube" => format!("https://www.youtube.com/channel/{id}"),
        "twitch" => format!("https://www.twitch.tv/{name}"),
        "reddit" => format!("https://www.reddit.com/user/{name}"),
        "tiktok" => format!("https://www.tiktok.com/@{name}"),
        "instagram" => format!("https://www.instagram.com/{name}"),
        "bluesky" => format!("https://bsky.app/profile/{name}"),
        "ebay" => format!("https://www.ebay.com/usr/{name}"),
        "paypal" => return None,
        "domain" => format!("https://{name}"),
        "roblox" => format!("https://www.roblox.com/users/{id}/profile"),
        "mastodon" => {
            let (u, h) = name.trim_start_matches('@').split_once('@')?;
            format!("https://{h}/@{u}")
        }
        _ => return None,
    })
}

pub fn norm(v: &Value, eff: Option<&Value>) -> Value {
    let u = &v["user"];
    let id = s(u, "id").unwrap_or("");
    let up = &v["user_profile"];
    let gp = v.get("guild_member_profile").filter(|x| x.is_object());
    let pick = |k: &str| -> Value {
        if let Some(g) = gp {
            if let Some(x) = g.get(k).filter(|x| !x.is_null() && x.as_str() != Some("") && x.as_array().is_none_or(|a| !a.is_empty())) {
                return x.clone();
            }
        }
        up.get(k).filter(|x| !x.is_null()).cloned().or_else(|| u.get(k).filter(|x| !x.is_null()).cloned()).unwrap_or(Value::Null)
    };
    let gid = v.get("guild_member").and_then(|m| s(m, "guild_id")).or_else(|| gp.and_then(|g| s(g, "guild_id")));
    let banner = match (gp.and_then(|g| s(g, "banner")).filter(|x| !x.is_empty()), gid) {
        (Some(b), Some(g)) => json!(format!("{CDN}/guilds/{g}/users/{id}/banners/{b}.{}?size=600", if b.starts_with("a_") { "gif" } else { "png" })),
        _ => img("banners", id, pick("banner").as_str(), 600),
    };
    let badges: Vec<Value> = v["badges"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|b| {
                    let ic = s(b, "icon")?;
                    Some(json!({"id": s(b, "id").unwrap_or(""), "d": s(b, "description").unwrap_or(""), "icon": format!("{CDN}/badge-icons/{ic}.png"), "link": s(b, "link")}))
                })
                .collect()
        })
        .unwrap_or_default();
    let conns: Vec<Value> = v["connected_accounts"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|c| {
                    let t = s(c, "type").unwrap_or("");
                    let cid = s(c, "id").unwrap_or("");
                    let n = s(c, "name").unwrap_or("");
                    json!({"t": t, "id": cid, "name": n, "ok": c["verified"].as_bool().unwrap_or(false), "url": conn_url(t, cid, n)})
                })
                .collect()
        })
        .unwrap_or_default();
    let effect_id = pick("profile_effect").get("id").and_then(Value::as_str).map(str::to_string);
    let effect = effect_id.as_ref().and_then(|e| eff.and_then(|x| x["profile_effect_configs"].as_array()).and_then(|a| a.iter().find(|c| s(c, "id") == Some(e)).cloned()));
    let mutual: Vec<Value> = v["mutual_guilds"].as_array().map(|a| a.iter().filter_map(|g| s(g, "id").map(|x| json!(x))).collect()).unwrap_or_default();
    json!({
        "id": id,
        "bio": pick("bio"),
        "pronouns": pick("pronouns"),
        "banner": banner,
        "accent": pick("accent_color"),
        "theme": pick("theme_colors"),
        "effect": effect,
        "badges": badges,
        "conns": conns,
        "mutual": mutual,
        "mfu": v["mutual_friends"].as_array().map(|a| a.iter().take(3).map(|x| { let m = crate::map::usr(0, x); json!({"name": m.name, "av": m.av}) }).collect::<Vec<_>>()).unwrap_or_default(),
        "mfriends": v["mutual_friends_count"].as_u64().or_else(|| v["mutual_friends"].as_array().map(|a| a.len() as u64)),
        "since": v["premium_since"],
        "boost": v["premium_guild_since"],
        "joined": v.get("guild_member").map(|m| m["joined_at"].clone()).unwrap_or(Value::Null),
        "deco": deco(u),
        "tag": tag(u),
        "plate": crate::map::plate(u),
        "ncol": u["display_name_styles"]["colors"].clone(),
        "nfx": u["display_name_styles"]["effect_id"].clone(),
        "legacy": v["legacy_username"],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_profile() {
        let v = json!({
            "user": {"id": "5", "username": "x", "bio": "hi", "banner": "a_b1", "accent_color": 255,
                "avatar_decoration_data": {"asset": "dec1"}, "primary_guild": {"identity_guild_id": "9", "identity_enabled": true, "tag": "ASND", "badge": "bd"}},
            "user_profile": {"bio": "hello", "pronouns": "they/them", "theme_colors": [1, 2], "profile_effect": {"id": "e1"}},
            "badges": [{"id": "hypesquad", "description": "HypeSquad", "icon": "ic1", "link": null}],
            "connected_accounts": [{"type": "github", "id": "1", "name": "octo", "verified": true}],
            "mutual_guilds": [{"id": "77"}],
            "mutual_friends_count": 3,
            "premium_since": "2020-01-01T00:00:00Z"
        });
        let e = json!({"profile_effect_configs": [{"id": "e1", "effects": []}]});
        let p = norm(&v, Some(&e));
        assert_eq!(p["bio"], "hello");
        assert_eq!(p["pronouns"], "they/them");
        assert_eq!(p["banner"], "https://cdn.discordapp.com/banners/5/a_b1.gif?size=600");
        assert_eq!(p["badges"][0]["icon"], "https://cdn.discordapp.com/badge-icons/ic1.png");
        assert_eq!(p["conns"][0]["url"], "https://github.com/octo");
        assert_eq!(p["effect"]["id"], "e1");
        assert_eq!(p["deco"], "dec1");
        assert_eq!(p["tag"]["t"], "ASND");
        assert_eq!(p["mutual"][0], "77");
        assert_eq!(p["mfriends"], 3);
    }

    #[test]
    fn guild_profile_overrides() {
        let v = json!({
            "user": {"id": "5", "username": "x"},
            "user_profile": {"bio": "global", "pronouns": ""},
            "guild_member": {"guild_id": "8", "joined_at": "2021-02-02T00:00:00Z"},
            "guild_member_profile": {"guild_id": "8", "bio": "server bio", "banner": "gb", "pronouns": ""}
        });
        let p = norm(&v, None);
        assert_eq!(p["bio"], "server bio");
        assert_eq!(p["banner"], "https://cdn.discordapp.com/guilds/8/users/5/banners/gb.png?size=600");
        assert_eq!(p["joined"], "2021-02-02T00:00:00Z");
        assert!(p["effect"].is_null());
    }
}
