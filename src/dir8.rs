use crate::config::{DPad8Binds, DPadBinds};
use evdev_rs::enums::EV_KEY;

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
