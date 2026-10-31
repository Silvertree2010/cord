use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Pop {
    Pkt(Vec<u8>),
    Lost,
    Empty,
}

pub struct Jb {
    map: BTreeMap<i64, Vec<u8>>,
    next: i64,
    hi: i64,
    init: bool,
    run: bool,
    miss: u32,
    depth: usize,
    max: usize,
}

impl Jb {
    pub fn new(depth: usize, max: usize) -> Jb {
        Jb { map: BTreeMap::new(), next: 0, hi: 0, init: false, run: false, miss: 0, depth, max }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn push(&mut self, seq: u16, d: Vec<u8>) {
        if !self.init {
            self.init = true;
            self.hi = seq as i64;
        }
        let ext = self.hi + seq.wrapping_sub(self.hi as u16) as i16 as i64;
        if ext - self.hi > 1000 || self.hi - ext > 1000 {
            self.map.clear();
            self.run = false;
            self.hi = ext;
        }
        if self.run && ext < self.next {
            return;
        }
        self.map.insert(ext, d);
        if ext > self.hi {
            self.hi = ext;
        }
        while self.map.len() > self.max {
            if let Some((&k, _)) = self.map.iter().next() {
                self.map.remove(&k);
                if self.run && k >= self.next {
                    self.next = k + 1;
                }
            }
        }
    }

    pub fn pop(&mut self) -> Pop {
        if !self.run {
            if self.map.len() < self.depth {
                return Pop::Empty;
            }
            self.run = true;
            self.miss = 0;
            self.next = *self.map.keys().next().unwrap();
        }
        if let Some(d) = self.map.remove(&self.next) {
            self.next += 1;
            self.miss = 0;
            return Pop::Pkt(d);
        }
        if self.map.keys().next_back().is_some_and(|&k| k > self.next) {
            self.next += 1;
            return Pop::Lost;
        }
        self.miss += 1;
        if self.miss >= 5 {
            self.run = false;
            self.map.clear();
        }
        Pop::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(n: u8) -> Vec<u8> {
        vec![n]
    }

    #[test]
    fn prefill_then_order() {
        let mut j = Jb::new(3, 50);
        j.push(10, p(0));
        assert_eq!(j.pop(), Pop::Empty);
        j.push(12, p(2));
        j.push(11, p(1));
        assert_eq!(j.pop(), Pop::Pkt(p(0)));
        assert_eq!(j.pop(), Pop::Pkt(p(1)));
        assert_eq!(j.pop(), Pop::Pkt(p(2)));
        assert_eq!(j.pop(), Pop::Empty);
    }

    #[test]
    fn loss() {
        let mut j = Jb::new(3, 50);
        j.push(1, p(1));
        j.push(2, p(2));
        j.push(4, p(4));
        assert_eq!(j.pop(), Pop::Pkt(p(1)));
        assert_eq!(j.pop(), Pop::Pkt(p(2)));
        assert_eq!(j.pop(), Pop::Lost);
        assert_eq!(j.pop(), Pop::Pkt(p(4)));
    }

    #[test]
    fn wrap() {
        let mut j = Jb::new(3, 50);
        for s in [65534u16, 65535, 0, 1] {
            j.push(s, vec![s as u8]);
        }
        assert_eq!(j.pop(), Pop::Pkt(vec![254]));
        assert_eq!(j.pop(), Pop::Pkt(vec![255]));
        assert_eq!(j.pop(), Pop::Pkt(vec![0]));
        assert_eq!(j.pop(), Pop::Pkt(vec![1]));
    }

    #[test]
    fn late_dropped() {
        let mut j = Jb::new(2, 50);
        j.push(5, p(5));
        j.push(6, p(6));
        assert_eq!(j.pop(), Pop::Pkt(p(5)));
        assert_eq!(j.pop(), Pop::Pkt(p(6)));
        j.push(5, p(5));
        assert!(j.is_empty());
    }

    #[test]
    fn late_but_in_time() {
        let mut j = Jb::new(2, 50);
        j.push(1, p(1));
        j.push(2, p(2));
        assert_eq!(j.pop(), Pop::Pkt(p(1)));
        assert_eq!(j.pop(), Pop::Pkt(p(2)));
        assert_eq!(j.pop(), Pop::Empty);
        j.push(3, p(3));
        assert_eq!(j.pop(), Pop::Pkt(p(3)));
    }

    #[test]
    fn overflow_and_rebuffer() {
        let mut j = Jb::new(2, 4);
        for s in 0..10u16 {
            j.push(s, p(s as u8));
        }
        assert_eq!(j.len(), 4);
        assert_eq!(j.pop(), Pop::Pkt(p(6)));
        for _ in 0..3 {
            j.pop();
        }
        for _ in 0..5 {
            assert_eq!(j.pop(), Pop::Empty);
        }
        j.push(100, p(100));
        assert_eq!(j.pop(), Pop::Empty);
        j.push(101, p(101));
        assert_eq!(j.pop(), Pop::Pkt(p(100)));
    }

    #[test]
    fn jump_resets() {
        let mut j = Jb::new(1, 50);
        j.push(1, p(1));
        assert_eq!(j.pop(), Pop::Pkt(p(1)));
        j.push(30000, p(9));
        assert_eq!(j.pop(), Pop::Pkt(p(9)));
    }
}
