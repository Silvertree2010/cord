use cord_hub::Hub;
use serde_json::Value;
use std::sync::Arc;

pub fn install(_h: &Arc<Hub>, _c: &Value) {}

pub async fn set(_h: &Arc<Hub>, _k: &str, _on: bool) {}
