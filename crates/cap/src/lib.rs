pub mod aud;
pub mod dec;
pub mod enc;
#[cfg(target_os = "linux")]
pub mod scr;
#[cfg(not(target_os = "linux"))]
#[path = "scr_mac.rs"]
pub mod scr;

use std::sync::Once;

pub type Res<T> = anyhow::Result<T>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cod {
    H264,
    Vp8,
    Av1,
}

#[derive(Clone, Debug)]
pub struct Au {
    pub data: Vec<u8>,
    pub pts_us: u64,
    pub key: bool,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

pub fn init() -> Res<()> {
    static O: Once = Once::new();
    let mut r = Ok(());
    O.call_once(|| r = gstreamer::init().map_err(Into::into));
    r
}

pub fn has(name: &str) -> bool {
    init().is_ok() && gstreamer::ElementFactory::find(name).is_some()
}

pub fn first(names: &[&'static str]) -> Option<&'static str> {
    names.iter().copied().find(|n| has(n))
}
