use crate::config::{Binds, Settings};
use crate::dir8::{Dir8, ToDir8};
use evdev_rs::{enums::EV_KEY, InputEvent, TimeVal};
use modular_bitfield::{bitfield, specifiers::B6};

#[derive(Copy, Clone, Default, Debug)]
pub struct JoyState {
    // control stick
    pub control_stick: DPadState,
    pub control_stick_x: f32,
    pub control_stick_y: f32,
    // c stick
    pub c_stick: DPadState,
    pub c_stick_x: f32,
    pub c_stick_y: f32,
    // dpad
    pub dpad: DPadState,
    // analog l trigger
    pub l_trigger: f32,
    // digital buttons
    pub btn: JoyButtons,
    // modifiers
    pub m: Modifiers,
    pub updated: UpdatedTimeVal,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct UpdatedTimeVal(pub TimeVal);

impl Default for UpdatedTimeVal {
    fn default() -> UpdatedTimeVal {
        UpdatedTimeVal(TimeVal {
            tv_sec: 0,
            tv_usec: 0,
        })
    }
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct DPadState {
    pub up_held: bool,
    pub down_held: bool,
    pub left_held: bool,
    pub right_held: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Modifiers {
    pub mod1: bool,
    pub mod2: bool,
    #[skip]
    __: B6,
}

impl DPadState {
    #[inline]
    pub fn is_active(&self) -> bool {
        self.left() || self.right() || self.up() || self.down()
    }

    pub fn update<B: ToDir8>(&mut self, binds: &B, key: EV_KEY, value: bool) -> bool {
        use Dir8::*;
        let dir = if let Some(dir) = binds.dir(key) {
            dir
        } else {
            return false;
        };
        match dir {
            N => {
                self.on_up(value);
            }
            S => {
                self.on_down(value);
            }
            W => {
                self.on_left(value);
            }
            E => {
                self.on_right(value);
            }
            NW => {
                self.on_up(value);
                self.on_left(value);
            }
            SW => {
                self.on_down(value);
                self.on_left(value);
            }
            NE => {
                self.on_up(value);
                self.on_right(value);
            }
            SE => {
                self.on_down(value);
                self.on_right(value);
            }
        };
        true
    }

    #[inline]
    fn on_up(&mut self, value: bool) {
        self.set_up(value);
        self.set_up_held(value);
        if value {
            self.set_down(false);
        } else {
            self.set_down(self.down_held());
        }
    }

    #[inline]
    fn on_down(&mut self, value: bool) {
        self.set_down(value);
        self.set_down_held(value);
        if value {
            self.set_up(false);
        } else {
            self.set_up(self.up_held());
        }
    }

    #[inline]
    fn on_left(&mut self, value: bool) {
        self.set_left(value);
        self.set_left_held(value);
        if value {
            self.set_right(false);
        } else {
            self.set_right(self.right_held());
        }
    }

    #[inline]
    fn on_right(&mut self, value: bool) {
        self.set_right(value);
        self.set_right_held(value);
        if value {
            self.set_left(false);
        } else {
            self.set_left(self.left_held());
        }
    }
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct JoyButtons {
    // buttons
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub start: bool,
    // digital triggers
    pub l: bool,
    pub r: bool,
}

impl JoyState {
    pub fn update_flags(&mut self, cfg: &Binds, event: InputEvent) -> bool {
        if let evdev_rs::enums::EventCode::EV_KEY(k) = event.event_code {
            let value = event.value;
            if value > 1 {
                // skip a repeat key event entirely, we would have already
                // processed it being pressed anyway
                return false;
            }
            let value = value != 0;
            if self.control_stick.update(&cfg.control_stick, k, value) {
            } else if self.c_stick.update(&cfg.c_stick, k, value) {
            } else if self.dpad.update(&cfg.dpad, k, value) {
            } else {
                match k {
                    _ if k == cfg.a => self.btn.set_a(value),
                    _ if k == cfg.b => self.btn.set_b(value),
                    _ if k == cfg.x => self.btn.set_x(value),
                    _ if k == cfg.y => self.btn.set_y(value),
                    _ if k == cfg.z => self.btn.set_z(value),
                    _ if k == cfg.start => self.btn.set_start(value),
                    _ if k == cfg.l => self.btn.set_l(value),
                    _ if k == cfg.r => self.btn.set_r(value),
                    _ if k == cfg.mod1 => self.m.set_mod1(value),
                    _ if k == cfg.mod2 => self.m.set_mod2(value),
                    _ => {}
                }
            }
            return true;
        }
        false
    }

    pub fn update_analog(&mut self, cfg: &Settings, prev: &Self) {
        let mod1 = self.m.mod1();
        let mod2 = self.m.mod2();
        let mod_changed = self.m != prev.m;
        let j_mul = if mod2 { cfg.mod2_mul } else { 1.0 };

        // update control stick
        if self.control_stick != prev.control_stick || mod_changed {
            let active = self.control_stick.is_active();
            if active {
                let vx = (self.control_stick.right() as i8) - (self.control_stick.left() as i8);
                let vy = (self.control_stick.down() as i8) - (self.control_stick.up() as i8);
                let (mut x, mut y) = (vx as f32, vy as f32);
                if mod1 {
                    let active0 = prev.control_stick.is_active();
                    let (x0, y0) = (self.control_stick_x, self.control_stick_y);
                    let (vhoriz, vvert) = (vx.abs() == 1, vy.abs() == 1);
                    if !active0 {
                        if vhoriz && vvert {
                            y *= cfg.mod1_around_y;
                        }
                    } else {
                        if y0.abs() > 0.01 {
                            x = x0 + cfg.mod1_incr * vx as f32;
                        }
                        if x0.abs() > 0.01 {
                            y = y0 + cfg.mod1_around_y * vy as f32;
                        }
                    }
                }
                if mod2 && x.abs() < 0.99 && y.abs() < 0.99 {
                    if x.abs() > y.abs() {
                        x = vx as f32;
                    } else {
                        y = vy as f32;
                    }
                }
                self.control_stick_x = (x * j_mul).clamp(-1.0, 1.0);
                self.control_stick_y = (y * j_mul).clamp(-1.0, 1.0);
            } else {
                self.control_stick_x = 0.0;
                self.control_stick_y = 0.0;
            }
        }

        // update C stick
        if self.c_stick != prev.c_stick || mod2 != prev.m.mod2() {
            self.c_stick_x = ((self.c_stick.right() as i8) - (self.c_stick.left() as i8)) as f32;
            self.c_stick_y = ((self.c_stick.down() as i8) - (self.c_stick.up() as i8)) as f32;
            self.c_stick_x *= j_mul;
            self.c_stick_y *= j_mul;
        }

        // update L trigger (R trigger is just digital here)
        self.l_trigger = if self.btn.l() {
            if mod1 {
                cfg.mod1_trigger_mul
            } else {
                1.0
            }
        } else {
            0.0
        };
    }

    pub fn sanity(&self) {
        let control_stick_is_active = self.control_stick.is_active();
        if !control_stick_is_active {
            let control_stick_xy = (self.control_stick_x, self.control_stick_y);
            assert_eq!(control_stick_xy, (0.0, 0.0));
        }
    }
}
