use crate::config::{DPad8Binds, DPadBinds};
use evdev_rs::enums::EV_KEY;

pub trait ToDir8 {
    fn dir(&self, key: EV_KEY) -> Option<Dir8>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir8 {
    NW,
    N,
    NE,
    W,
    E,
    SW,
    S,
    SE,
}

impl ToDir8 for DPad8Binds {
    #[inline(always)]
    fn dir(&self, k: EV_KEY) -> Option<Dir8> {
        match k {
            _ if k == self.up => Some(Dir8::N),
            _ if k == self.upleft => Some(Dir8::NW),
            _ if k == self.upright => Some(Dir8::NE),
            _ if k == self.down => Some(Dir8::S),
            _ if k == self.downleft => Some(Dir8::SW),
            _ if k == self.downright => Some(Dir8::SE),
            _ if k == self.left => Some(Dir8::W),
            _ if k == self.right => Some(Dir8::E),
            _ => None,
        }
    }
}

impl ToDir8 for DPadBinds {
    #[inline(always)]
    fn dir(&self, k: EV_KEY) -> Option<Dir8> {
        match k {
            _ if k == self.up => Some(Dir8::N),
            _ if k == self.down => Some(Dir8::S),
            _ if k == self.left => Some(Dir8::W),
            _ if k == self.right => Some(Dir8::E),
            _ => None,
        }
    }
}
