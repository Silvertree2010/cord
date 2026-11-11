use image::{AnimationDecoder, ImageFormat, RgbaImage, codecs::png::PngDecoder, imageops::FilterType};
use serde::Serialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::Cursor,
    path::PathBuf,
    time::Duration,
};

const MAX_TEX: u32 = 8192;
const MAX_FRAMES: usize = 240;

#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq)]
pub struct Sheet {
    pub path: String,
    pub n: u32,
    pub w: u32,
    pub h: u32,
    pub cols: u32,
    pub ms: u32,
    pub total: u32,
}

fn key(url: &str, max: u32) -> String {
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    max.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn dir() -> PathBuf {
    let d = crate::cfg::data().join("sheets");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn build(bytes: &[u8], max: u32, out: PathBuf) -> anyhow::Result<Sheet> {
    let dec = PngDecoder::new(Cursor::new(bytes))?;
    let mut frames: Vec<(RgbaImage, u32)> = Vec::new();
    if dec.is_apng()? {
        for f in dec.apng()?.into_frames().take(MAX_FRAMES) {
            let f = f?;
            let (n, d) = f.delay().numer_denom_ms();
            let ms = if d == 0 { 0 } else { n / d };
            frames.push((f.into_buffer(), ms.max(10)));
        }
    } else {
        frames.push((image::load_from_memory_with_format(bytes, ImageFormat::Png)?.to_rgba8(), 1000));
    }
    anyhow::ensure!(!frames.is_empty(), "no frames");
    let (sw, sh) = frames[0].0.dimensions();
    let sc = if max > 0 && sw.max(sh) > max { max as f32 / sw.max(sh) as f32 } else { 1.0 };
    let (w, h) = (((sw as f32 * sc).round() as u32).max(1), ((sh as f32 * sc).round() as u32).max(1));
    let n = frames.len() as u32;
    let cols = (MAX_TEX / w).clamp(1, n);
    let rows = n.div_ceil(cols);
    anyhow::ensure!(rows * h <= MAX_TEX, "animation too large");
    let mut img = RgbaImage::new(cols * w, rows * h);
    let mut total = 0u32;
    for (i, (f, ms)) in frames.iter().enumerate() {
        let f = if sc < 1.0 { image::imageops::resize(f, w, h, FilterType::Triangle) } else { f.clone() };
        let (x, y) = ((i as u32 % cols) * w, (i as u32 / cols) * h);
        image::imageops::overlay(&mut img, &f, x as i64, y as i64);
        total += ms;
    }
    img.save_with_format(&out, ImageFormat::Png)?;
    Ok(Sheet { path: out.to_string_lossy().into_owned(), n, w, h, cols, ms: (total / n).max(10), total })
}

fn webm_to_apng(bytes: &[u8], tmp: &std::path::Path) -> anyhow::Result<Vec<u8>> {
    let src = tmp.with_extension("webm");
    let dst = tmp.with_extension("apng");
    std::fs::write(&src, bytes)?;
    let run = |dec: Option<&str>| {
        let mut c = std::process::Command::new("ffmpeg");
        c.args(["-hide_banner", "-loglevel", "error", "-y"]);
        if let Some(d) = dec {
            c.args(["-c:v", d]);
        }
        c.arg("-i").arg(&src).args(["-plays", "0", "-f", "apng"]).arg(&dst);
        c.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        c.status().map(|s| s.success()).unwrap_or(false)
    };
    let ok = run(Some("libvpx-vp9")) || run(Some("libvpx")) || run(None);
    let out = if ok { std::fs::read(&dst).map_err(anyhow::Error::from) } else { Err(anyhow::anyhow!("ffmpeg could not convert the animation")) };
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&dst);
    out
}

pub async fn get(url: String, max: u32) -> anyhow::Result<Sheet> {
    let k = key(&url, max);
    let d = dir();
    let meta = d.join(format!("{k}.json"));
    if let Ok(s) = std::fs::read_to_string(&meta) {
        if let Ok(sh) = serde_json::from_str::<Sheet>(&s) {
            if std::path::Path::new(&sh.path).exists() {
                return Ok(sh);
            }
        }
    }
    let bytes = reqwest::Client::new().get(&url).timeout(Duration::from_secs(30)).send().await?.error_for_status()?.bytes().await?.to_vec();
    let out = d.join(format!("{k}.png"));
    let webm = url.split('?').next().unwrap_or("").ends_with(".webm");
    let tmp = d.join(format!("{k}.tmp"));
    let sh = tokio::task::spawn_blocking(move || {
        let png = if webm { webm_to_apng(&bytes, &tmp)? } else { bytes };
        build(&png, max, out)
    })
    .await??;
    let _ = std::fs::write(&meta, serde_json::to_string(&sh)?);
    Ok(sh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_png_becomes_one_frame() {
        let mut b = Vec::new();
        RgbaImage::from_pixel(20, 10, image::Rgba([255, 0, 0, 255])).write_to(&mut Cursor::new(&mut b), ImageFormat::Png).unwrap();
        let out = std::env::temp_dir().join("cord-sheet-test.png");
        let s = build(&b, 0, out.clone()).unwrap();
        assert_eq!((s.n, s.w, s.h, s.cols), (1, 20, 10, 1));
        let s = build(&b, 10, out.clone()).unwrap();
        assert_eq!((s.w, s.h), (10, 5));
        let _ = std::fs::remove_file(out);
    }

    #[test]
    fn apng_frames_from_env_file() {
        let Some(p) = std::env::var_os("CORD_APNG") else { return };
        let b = std::fs::read(p).unwrap();
        let out = std::env::temp_dir().join("cord-sheet-apng.png");
        let s = build(&b, 0, out).unwrap();
        assert!(s.n > 1);
        println!("{s:?}");
    }
}
