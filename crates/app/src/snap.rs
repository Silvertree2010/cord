use cord_hub::Sta;
use serde_json::{Map, Value, json};

pub fn boot(s: &Sta) -> Value {
    let pres: Map<String, Value> = s.pres.iter().map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap_or(Value::Null))).collect();
    let unr: Map<String, Value> = s.unr.iter().map(|(k, v)| (k.to_string(), serde_json::to_value(v).unwrap_or(Value::Null))).collect();
    json!({
        "accs": s.accs.values().collect::<Vec<_>>(),
        "up": s.up.iter().collect::<Vec<_>>(),
        "glds": s.glds.values().map(|g| s.gld_v(g)).collect::<Vec<_>>(),
        "chans": s.vis_chans(),
        "rels": s.rels.values().collect::<Vec<_>>(),
        "pres": pres,
        "unr": unr,
        "vs": s.vs.values().collect::<Vec<_>>(),
        "call": s.call,
        "folds": s.all_folds(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cord_core::*;

    #[test]
    fn boot_has_all_sections_and_filters_hidden() {
        let mut s = Sta::default();
        let rid = |r: &str| Rid::new(Svc::Dc, 1, r);
        let c = |id: &str, hidden: bool| Chan { id: rid(id), gld: None, par: None, k: ChK::Txt, name: id.into(), topic: None, pos: 0, nsfw: false, hidden, last: None, rcp: vec![], icon: None, enc: false, mute: false };
        s.apply(Ev::Chan(c("a", false)));
        s.chans.insert(rid("b"), c("b", true));
        let v = boot(&s);
        for k in ["accs", "up", "glds", "chans", "rels", "pres", "unr", "vs", "call", "folds"] {
            assert!(v.get(k).is_some(), "{k}");
        }
        assert_eq!(v["chans"].as_array().unwrap().len(), 1);
        assert_eq!(v["chans"][0]["rid"], "dc:1:a");
    }
}
