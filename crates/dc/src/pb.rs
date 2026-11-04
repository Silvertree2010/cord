pub struct Rd<'a> {
    b: &'a [u8],
    i: usize,
}

pub enum V<'a> {
    Int(u64),
    F64(u64),
    Bytes(&'a [u8]),
    F32,
}

impl<'a> Rd<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Rd { b, i: 0 }
    }

    fn var(&mut self) -> Option<u64> {
        let mut v = 0u64;
        for s in 0..10 {
            let c = *self.b.get(self.i)?;
            self.i += 1;
            v |= ((c & 0x7f) as u64) << (7 * s);
            if c & 0x80 == 0 {
                return Some(v);
            }
        }
        None
    }

    pub fn next(&mut self) -> Option<(u32, V<'a>)> {
        if self.i >= self.b.len() {
            return None;
        }
        let k = self.var()?;
        let f = (k >> 3) as u32;
        let v = match k & 7 {
            0 => V::Int(self.var()?),
            1 => {
                let s = self.b.get(self.i..self.i + 8)?;
                self.i += 8;
                V::F64(u64::from_le_bytes(s.try_into().ok()?))
            }
            2 => {
                let n = self.var()? as usize;
                let s = self.b.get(self.i..self.i.checked_add(n)?)?;
                self.i += n;
                V::Bytes(s)
            }
            5 => {
                self.b.get(self.i..self.i + 4)?;
                self.i += 4;
                V::F32
            }
            _ => return None,
        };
        Some((f, v))
    }
}

fn wrap_int(b: &[u8]) -> Option<u64> {
    let mut r = Rd::new(b);
    while let Some((f, v)) = r.next() {
        if let (1, V::Int(x)) = (f, v) {
            return Some(x);
        }
    }
    Some(0)
}

fn wrap_str(b: &[u8]) -> Option<String> {
    let mut r = Rd::new(b);
    while let Some((f, v)) = r.next() {
        if let (1, V::Bytes(s)) = (f, v) {
            return Some(String::from_utf8_lossy(s).into_owned());
        }
    }
    Some(String::new())
}

#[derive(Debug, Default, PartialEq)]
pub struct Fd {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub col: Option<u64>,
    pub glds: Vec<u64>,
}

fn fixed64s(b: &[u8]) -> Vec<u64> {
    b.chunks_exact(8).filter_map(|c| c.try_into().ok().map(u64::from_le_bytes)).collect()
}

fn folder(b: &[u8]) -> Fd {
    let mut o = Fd::default();
    let mut r = Rd::new(b);
    while let Some((f, v)) = r.next() {
        match (f, v) {
            (1, V::Bytes(s)) => o.glds.extend(fixed64s(s)),
            (1, V::F64(x)) => o.glds.push(x),
            (2, V::Bytes(s)) => o.id = wrap_int(s).map(|x| x as i64),
            (3, V::Bytes(s)) => o.name = wrap_str(s).filter(|x| !x.is_empty()),
            (4, V::Bytes(s)) => o.col = wrap_int(s),
            _ => {}
        }
    }
    o
}

pub fn folders(settings: &[u8]) -> Option<(Vec<Fd>, Vec<u64>)> {
    let mut r = Rd::new(settings);
    while let Some((f, v)) = r.next() {
        if let (14, V::Bytes(gf)) = (f, v) {
            let mut fs = vec![];
            let mut pos = vec![];
            let mut q = Rd::new(gf);
            while let Some((f, v)) = q.next() {
                match (f, v) {
                    (1, V::Bytes(s)) => fs.push(folder(s)),
                    (2, V::Bytes(s)) => pos.extend(fixed64s(s)),
                    (2, V::F64(x)) => pos.push(x),
                    _ => {}
                }
            }
            return Some((fs, pos));
        }
    }
    None
}
