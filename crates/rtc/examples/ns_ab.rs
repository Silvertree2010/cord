use deep_filter_rt::tract::DfTract;
use ndarray::Array2;
use nnnoiseless::DenoiseState;
use std::io::Read;

fn rd(p: &str) -> Vec<f32> {
    let mut b = Vec::new();
    std::fs::File::open(p).unwrap().read_to_end(&mut b).unwrap();
    b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

fn wr(p: &str, v: &[f32]) {
    let b: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
    std::fs::write(p, b).unwrap();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let x = rd(&a[1]);
    let mut rn = DenoiseState::new();
    let mut o = vec![0.0f32; x.len()];
    let mut tmp = [0.0f32; DenoiseState::FRAME_SIZE];
    for (i, c) in x.chunks(480).enumerate() {
        if c.len() < 480 {
            break;
        }
        let s: Vec<f32> = c.iter().map(|v| v * 32768.0).collect();
        rn.process_frame(&mut tmp, &s);
        for (k, v) in tmp.iter().enumerate() {
            o[i * 480 + k] = v / 32768.0;
        }
    }
    wr(&a[2], &o);
    let mut df = DfTract::default();
    let t = std::time::Instant::now();
    let mut d = vec![0.0f32; x.len()];
    for (i, c) in x.chunks(480).enumerate() {
        if c.len() < 480 {
            break;
        }
        let inp = Array2::from_shape_vec((1, 480), c.to_vec()).unwrap();
        let mut out = Array2::<f32>::zeros((1, 480));
        df.process(inp.view(), out.view_mut()).unwrap();
        for (k, v) in out.iter().enumerate() {
            d[i * 480 + k] = *v;
        }
    }
    eprintln!("dfn: {:.2}s for {:.2}s audio", t.elapsed().as_secs_f32(), x.len() as f32 / 48000.0);
    wr(&a[3], &d);
}
