use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Go,
    Stop,
}

pub trait Hk: Send + Sync {
    fn id(&self) -> &'static str;
    fn on_ev(&self, _e: &mut Ev) -> Flow {
        Flow::Go
    }
    fn on_out(&self, _chan: &Rid, _o: &mut Out) -> Flow {
        Flow::Go
    }
}

#[derive(Default)]
pub struct Hks(Vec<Box<dyn Hk>>);

impl Hks {
    pub fn add(&mut self, h: Box<dyn Hk>) {
        self.0.push(h);
    }
    pub fn ev(&self, e: &mut Ev) -> Flow {
        for h in &self.0 {
            if h.on_ev(e) == Flow::Stop {
                return Flow::Stop;
            }
        }
        Flow::Go
    }
    pub fn out(&self, c: &Rid, o: &mut Out) -> Flow {
        for h in &self.0 {
            if h.on_out(c, o) == Flow::Stop {
                return Flow::Stop;
            }
        }
        Flow::Go
    }
}
