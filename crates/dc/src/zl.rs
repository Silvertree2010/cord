use flate2::{Decompress, FlushDecompress};

pub struct Inf {
    d: Decompress,
    b: Vec<u8>,
}

impl Default for Inf {
    fn default() -> Self {
        Self::new()
    }
}

impl Inf {
    pub fn new() -> Self {
        Self { d: Decompress::new(true), b: Vec::new() }
    }

    pub fn push(&mut self, x: &[u8]) -> anyhow::Result<Option<String>> {
        self.b.extend_from_slice(x);
        if !self.b.ends_with(&[0, 0, 0xff, 0xff]) {
            return Ok(None);
        }
        let inp = std::mem::take(&mut self.b);
        let mut o: Vec<u8> = Vec::with_capacity(inp.len() * 4 + 1024);
        let mut p = 0usize;
        loop {
            if o.capacity() - o.len() < 4096 {
                o.reserve(16384);
            }
            let (i0, o0) = (self.d.total_in(), self.d.total_out());
            self.d.decompress_vec(&inp[p..], &mut o, FlushDecompress::Sync)?;
            let u = (self.d.total_in() - i0) as usize;
            p += u;
            let w = self.d.total_out() - o0;
            if p >= inp.len() && o.len() < o.capacity() {
                break;
            }
            if u == 0 && w == 0 {
                break;
            }
        }
        Ok(Some(String::from_utf8(o)?))
    }
}
