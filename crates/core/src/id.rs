use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Svc {
    Dc,
    Mx,
    St,
}

impl Serialize for Svc {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.tag())
    }
}

impl<'de> Deserialize<'de> for Svc {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Svc::from_tag(&s.to_ascii_lowercase()).ok_or_else(|| serde::de::Error::custom(format!("unknown service {s}")))
    }
}

impl Svc {
    pub fn tag(self) -> &'static str {
        match self {
            Svc::Dc => "dc",
            Svc::Mx => "mx",
            Svc::St => "st",
        }
    }
    pub fn from_tag(s: &str) -> Option<Self> {
        match s {
            "dc" => Some(Svc::Dc),
            "mx" => Some(Svc::Mx),
            "st" => Some(Svc::St),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
pub struct Rid {
    pub svc: Svc,
    pub acc: u32,
    pub raw: Arc<str>,
}

impl Rid {
    pub fn new(svc: Svc, acc: u32, raw: impl Into<Arc<str>>) -> Self {
        Self { svc, acc, raw: raw.into() }
    }
}

impl Serialize for Rid {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut o = s.serialize_struct("Rid", 4)?;
        o.serialize_field("svc", &self.svc)?;
        o.serialize_field("acc", &self.acc)?;
        o.serialize_field("raw", &*self.raw)?;
        o.serialize_field("rid", &self.to_string())?;
        o.end()
    }
}

impl fmt::Display for Rid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.svc.tag(), self.acc, self.raw)
    }
}

impl FromStr for Rid {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.splitn(3, ':');
        let sv = it.next().and_then(Svc::from_tag).ok_or_else(|| anyhow::anyhow!("bad svc"))?;
        let a = it.next().ok_or_else(|| anyhow::anyhow!("no acc"))?.parse()?;
        let r = it.next().ok_or_else(|| anyhow::anyhow!("no raw"))?;
        Ok(Rid::new(sv, a, r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rid_serializes_string_form_and_roundtrips() {
        let r = Rid::new(Svc::Mx, 3, "!a:b");
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["rid"], "mx:3:!a:b");
        assert_eq!(v["svc"], "mx");
        let old: Rid = serde_json::from_value(serde_json::json!({"svc": "Mx", "acc": 3, "raw": "!a:b"})).unwrap();
        assert_eq!(old, r);
        let back: Rid = serde_json::from_value(v).unwrap();
        assert_eq!(back, r);
    }
}
