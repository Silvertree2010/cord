#[cxx_qt::bridge]
pub mod q {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;
        include!("img.h");
        fn reg_img(e: Pin<&mut QQmlApplicationEngine>);
    }
    extern "Rust" {
        type FrameRef;
        fn media_get(u: &str) -> Vec<u8>;
        fn frame_ref(u: &str) -> Box<FrameRef>;
        fn fr_w(self: &FrameRef) -> u32;
        fn fr_h(self: &FrameRef) -> u32;
        fn fr_data(self: &FrameRef) -> &[u8];
    }
}

pub use q::reg_img;
use std::time::Duration;

pub struct FrameRef(Option<crate::frames::Fr>);

impl FrameRef {
    fn fr_w(&self) -> u32 {
        self.0.as_ref().map(|f| f.w).unwrap_or(0)
    }
    fn fr_h(&self) -> u32 {
        self.0.as_ref().map(|f| f.h).unwrap_or(0)
    }
    fn fr_data(&self) -> &[u8] {
        self.0.as_ref().map(|f| f.rgba.as_slice()).unwrap_or(&[])
    }
}

fn frame_ref(u: &str) -> Box<FrameRef> {
    Box::new(FrameRef(crate::frames::get(u)))
}

pub fn split_media(u: &str) -> Option<(u32, &str)> {
    let (a, r) = u.split_once('/')?;
    Some((a.parse().ok()?, r))
}

fn media_get(u: &str) -> Vec<u8> {
    let (a, url) = match split_media(u) {
        Some((a, r)) => (Some(a), r.to_string()),
        None => (None, u.to_string()),
    };
    let m = match a {
        Some(a) => crate::bks::mx_of(a).or_else(|| if a == 0 { crate::bks::mx_any() } else { None }),
        None => crate::bks::mx_any(),
    };
    let Some(m) = m else { return vec![] };
    crate::rt::rt().block_on(async move { tokio::time::timeout(Duration::from_secs(20), m.media(&url)).await.ok().and_then(|r| r.ok()).unwrap_or_default() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_id_split() {
        assert_eq!(split_media("3/mxc://hs/abc"), Some((3, "mxc://hs/abc")));
        assert_eq!(split_media("mxc://hs/abc"), None);
    }
}
