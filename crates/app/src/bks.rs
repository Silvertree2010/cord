use crate::acc::Ac;
use cord_core::*;
use serde_json::Value;
use std::sync::Arc;

pub fn mk(_a: &Ac, _sec: String) -> Option<Arc<dyn Bk>> {
    None
}

pub async fn login(sv: Svc, _id: u32, v: &Value) -> Res<(String, String)> {
    match sv {
        Svc::Dc => {
            let t = v["token"].as_str().unwrap_or("").trim().trim_matches('"').to_string();
            anyhow::ensure!(!t.is_empty(), "token missing");
            Ok((String::new(), t))
        }
        _ => anyhow::bail!("{} login not available yet", sv.tag()),
    }
}
