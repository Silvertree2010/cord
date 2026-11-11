use sonora::config::EchoCanceller;
use sonora::{AudioProcessing, Config, StreamConfig};
use std::io::Read;

fn rd(p: &str) -> Vec<f32> {
    let mut b = Vec::new();
    std::fs::File::open(p).unwrap().read_to_end(&mut b).unwrap();
    b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let far = rd(&a[1]);
    let mic = rd(&a[2]);
    let cfg = Config { echo_canceller: Some(EchoCanceller::default()), ..Default::default() };
    let mut apm = AudioProcessing::builder().config(cfg).capture_config(StreamConfig::new(48000, 1)).render_config(StreamConfig::new(48000, 1)).build();
    let n = far.len().min(mic.len());
    let mut out = vec![0.0f32; n];
    for i in 0..n / 480 {
        let r = &far[i * 480..i * 480 + 480];
        let c = &mic[i * 480..i * 480 + 480];
        let mut sink = [0.0f32; 480];
        apm.process_render_f32(&[r], &mut [&mut sink]).unwrap();
        let mut o = [0.0f32; 480];
        apm.process_capture_f32(&[c], &mut [&mut o]).unwrap();
        out[i * 480..i * 480 + 480].copy_from_slice(&o);
    }
    let b: Vec<u8> = out.iter().flat_map(|x| x.to_le_bytes()).collect();
    std::fs::write(&a[3], b).unwrap();
}
