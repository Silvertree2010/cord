#[cxx_qt::bridge]
pub mod q {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;
        include!("img.h");
        fn reg_img(e: Pin<&mut QQmlApplicationEngine>);
    }
    extern "Rust" {
        fn media_get(u: &str) -> Vec<u8>;
        fn frame_get(u: &str, w: &mut u32, h: &mut u32) -> Vec<u8>;
    }
}

pub use q::reg_img;

fn media_get(u: &str) -> Vec<u8> {
    let u = u.to_string();
    let a: Option<u32> = u.split_once('/').and_then(|(a, _)| a.parse().ok());
    let url = u.split_once('/').map(|(_, r)| r.to_string()).unwrap_or(u.clone());
    let m = a.and_then(crate::bks::mx_of).or_else(crate::bks::mx_any);
    let Some(m) = m else { return vec![] };
    crate::rt::rt().block_on(async move { m.media(&url).await.unwrap_or_default() })
}

fn frame_get(u: &str, w: &mut u32, h: &mut u32) -> Vec<u8> {
    match crate::frames::get(u) {
        Some(f) => {
            *w = f.w;
            *h = f.h;
            f.rgba.clone()
        }
        None => vec![],
    }
}
