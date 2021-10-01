use crossbeam::queue::SegQueue;
use env_logger;
use evdev;
use evdev::Key;
use log;
use modular_bitfield::{
    bitfield,
    specifiers::{B4, B6},
};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use uinput;
use uinput::event::absolute::Position as AbsolutePosition;
use uinput::event::controller::{DPad, GamePad as GP};

const KEYBOARD_NAME: &str = "CATEX TECH. 84EC-XRGB";

// =================
// Key Bindings
// =================
// GCC

const CONTROL_STICK: DPad8Binds = DPad8Binds {
    up: Key::KEY_W,
    upleft: Key::KEY_Q,
    upright: Key::KEY_E,
    down: Key::KEY_S,
    downleft: Key::KEY_Z,
    downright: Key::KEY_C,
    left: Key::KEY_A,
    right: Key::KEY_D,
};
const C_STICK: DPadBinds = DPadBinds {
    up: Key::KEY_H,
    down: Key::KEY_N,
    left: Key::KEY_B,
    right: Key::KEY_M,
};
const DPAD: DPadBinds = DPadBinds {
    up: Key::KEY_UP,
    down: Key::KEY_DOWN,
    left: Key::KEY_LEFT,
    right: Key::KEY_RIGHT,
};
// =================
// Digital binds
const BTN_A: Key = Key::KEY_J;
const BTN_B: Key = Key::KEY_K;
const BTN_Z: Key = Key::KEY_L;
const BTN_X: Key = Key::KEY_SPACE;
const BTN_Y: Key = Key::KEY_LEFTALT; // I don't really use this at all
const BTN_START: Key = Key::KEY_T;
const BTN_L: Key = Key::KEY_I;
const BTN_R: Key = Key::KEY_O;
// Modifiers
const BTN_MOD1: Key = Key::KEY_LEFTSHIFT;
const BTN_MOD2: Key = Key::KEY_SLASH;
// =================
// Special parameters
const J_MOD1_INCR: f32 = 0.3875;
const J_MOD1_AROUND_Y: f32 = 0.31;
const J_MOD2: f32 = 48.0 / 128.0;
const TRIGGER_MOD1: f32 = 129.0 / 256.0;
const JOY_UP_RANGE: i32 = 127;
const JOY_DOWN_RANGE: i32 = -JOY_UP_RANGE;
// =================

const RATE_TARGET: Duration = Duration::from_micros(250);

struct DPadBinds {
    up: Key,
    down: Key,
    left: Key,
    right: Key,
}

struct DPad8Binds {
    upleft: Key,
    up: Key,
    upright: Key,
    downleft: Key,
    down: Key,
    downright: Key,
    left: Key,
    right: Key,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Dir8 {
    NW,
    N,
    NE,
    W,
    E,
    SW,
    S,
    SE,
}

trait DirectionalBinds {
    fn dir(&self, key: Key) -> Option<Dir8>;
}

impl DirectionalBinds for DPad8Binds {
    #[inline(always)]
    fn dir(&self, k: Key) -> Option<Dir8> {
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

impl DirectionalBinds for DPadBinds {
    #[inline(always)]
    fn dir(&self, k: Key) -> Option<Dir8> {
        match k {
            _ if k == self.up => Some(Dir8::N),
            _ if k == self.down => Some(Dir8::S),
            _ if k == self.left => Some(Dir8::W),
            _ if k == self.right => Some(Dir8::E),
            _ => None,
        }
    }
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
struct DPadState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    up_masked: bool,
    down_masked: bool,
    left_masked: bool,
    right_masked: bool,
}

impl DPadState {
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.left() || self.right() || self.up() || self.down()
    }

    #[inline]
    fn on_up(&mut self, value: bool) {
        self.set_up(value);
        self.set_up_masked(value);
        if value {
            self.set_down(false);
        } else {
            self.set_down(self.down_masked());
        }
    }

    #[inline]
    fn on_down(&mut self, value: bool) {
        self.set_down(value);
        self.set_down_masked(value);
        if value {
            self.set_up(false);
        } else {
            self.set_up(self.up_masked());
        }
    }

    #[inline]
    fn on_left(&mut self, value: bool) {
        self.set_left(value);
        self.set_left_masked(value);
        if value {
            self.set_right(false);
        } else {
            self.set_right(self.right_masked());
        }
    }

    #[inline]
    fn on_right(&mut self, value: bool) {
        self.set_right(value);
        self.set_right_masked(value);
        if value {
            self.set_left(false);
        } else {
            self.set_left(self.left_masked());
        }
    }

    #[inline(always)]
    pub fn update<B: DirectionalBinds>(&mut self, binds: B, key: Key, value: bool) -> bool {
        use Dir8::*;
        let dir = if let Some(dir) = binds.dir(key) {
            dir
        } else {
            return false;
        };
        match dir {
            N => {
                self.on_up(value);
                true
            }

            S => {
                self.on_down(value);
                true
            }

            W => {
                self.on_left(value);
                true
            }

            E => {
                self.on_right(value);
                true
            }

            NW => {
                self.on_up(value);
                self.on_left(value);
                true
            }

            SW => {
                self.on_down(value);
                self.on_left(value);
                true
            }

            NE => {
                self.on_up(value);
                self.on_right(value);
                true
            }

            SE => {
                self.on_down(value);
                self.on_right(value);
                true
            }

            _ => false,
        }
    }
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
struct JoyButtons {
    // buttons
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    z: bool,
    start: bool,
    // digital triggers
    l: bool,
    r: bool,
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
struct Modifiers {
    mod1: bool,
    mod2: bool,
    #[skip]
    __: B6,
}

#[derive(Copy, Clone, Default, Debug)]
struct JoyState {
    // control stick
    control_stick: DPadState,
    control_stick_x: f32,
    control_stick_y: f32,
    // c stick
    c_stick: DPadState,
    c_stick_x: f32,
    c_stick_y: f32,
    // dpad
    dpad: DPadState,
    // analog l trigger
    l_trigger: f32,
    // digital buttons
    btn: JoyButtons,
    // modifiers
    m: Modifiers,
}

impl JoyState {
    fn update_flags(&mut self, event: evdev::InputEvent) {
        if let evdev::InputEventKind::Key(k) = event.kind() {
            let value = event.value();
            if value > 1 {
                // skip a repeat key event entirely, we would have already
                // processed it being pressed anyway
                return;
            }
            let value = value != 0;
            if self.control_stick.update(CONTROL_STICK, k, value) {
            } else if self.c_stick.update(C_STICK, k, value) {
            } else if self.dpad.update(DPAD, k, value) {
            } else {
                match k {
                    _ if k == BTN_A => self.btn.set_a(value),
                    _ if k == BTN_B => self.btn.set_b(value),
                    _ if k == BTN_X => self.btn.set_x(value),
                    _ if k == BTN_Y => self.btn.set_y(value),
                    _ if k == BTN_Z => self.btn.set_z(value),
                    _ if k == BTN_START => self.btn.set_start(value),
                    _ if k == BTN_L => self.btn.set_l(value),
                    _ if k == BTN_R => self.btn.set_r(value),
                    _ if k == BTN_MOD1 => self.m.set_mod1(value),
                    _ if k == BTN_MOD2 => self.m.set_mod2(value),
                    _ => {}
                }
            }
        }
    }

    fn update_analog(&mut self, prev: &Self) {
        let mod1 = self.m.mod1();
        let mod2 = self.m.mod2();
        let mod_changed = self.m != prev.m;
        let j_mul = if mod2 { J_MOD2 } else { 1.0 };

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
                            y *= J_MOD1_AROUND_Y;
                        }
                    } else {
                        if y0.abs() > 0.01 {
                            x = x0 + J_MOD1_INCR * vx as f32;
                        }
                        if x0.abs() > 0.01 {
                            y = y0 + J_MOD1_AROUND_Y * vy as f32;
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
                TRIGGER_MOD1
            } else {
                1.0
            }
        } else {
            0.0
        };
    }
}

fn main() -> uinput::Result<()> {
    env_logger::init();
    log::info!("Melee virtual gamepad for Linux");
    let mut vjoy = uinput::default()?
        .name("melee-vpad")?
        .event(GP::North)?
        .event(GP::East)?
        .event(GP::South)?
        .event(GP::West)?
        .event(GP::TL)?
        .event(GP::TR)?
        .event(GP::Start)?
        .event(GP::Select)?
        .event(GP::Mode)?
        .event(GP::ThumbL)?
        .event(GP::ThumbR)?
        .event(DPad::Up)?
        .event(DPad::Down)?
        .event(DPad::Left)?
        .event(DPad::Right)?
        .event(AbsolutePosition::X)?
        .min(JOY_DOWN_RANGE)
        .max(JOY_UP_RANGE)
        .event(AbsolutePosition::Y)?
        .min(JOY_DOWN_RANGE)
        .max(JOY_UP_RANGE)
        .event(AbsolutePosition::Z)?
        .min(0)
        .max(255)
        .event(AbsolutePosition::RX)?
        .min(JOY_DOWN_RANGE)
        .max(JOY_UP_RANGE)
        .event(AbsolutePosition::RY)?
        .min(JOY_DOWN_RANGE)
        .max(JOY_UP_RANGE)
        .create()?;
    log::info!("Created virtual gamepad device");

    let q = Arc::new(SegQueue::<JoyState>::new());
    let q1 = q.clone();
    thread::spawn(move || {
        let q = q1;
        let mut kbd = get_keyboard();
        let mut state = JoyState::default();
        let mut prev = state;
        loop {
            let mut any_events = false;
            let events = kbd.fetch_events().unwrap();
            for ev in events {
                any_events = true;
                state.update_flags(ev);
            }
            if any_events {
                state.update_analog(&prev);
                q.push(state);
            }
            prev = state;
            // log::debug!("input loop took {:?}", now.elapsed());
        }
    });

    let mut prev = JoyState::default();
    loop {
        let t0 = Instant::now();

        while let Some(state) = q.pop() {
            // control stick
            update_joy(
                &mut vjoy,
                &AbsolutePosition::X,
                prev.control_stick_x,
                state.control_stick_x,
            )?;
            update_joy(
                &mut vjoy,
                &AbsolutePosition::Y,
                prev.control_stick_y,
                state.control_stick_y,
            )?;

            // R trigger, put it here to marginally reduce latency on wavedash
            // which is the only 1 frame input I can think of at the moment
            update_btn(&mut vjoy, prev.btn.r(), state.btn.r(), &GP::TR)?;

            // L trigger
            if prev.l_trigger != state.l_trigger {
                vjoy.position(
                    &AbsolutePosition::Z,
                    // this doesn't make any sense to me but it works in dolphin
                    (127.0 + state.l_trigger * 128.0) as i32,
                )?;
            }

            // buttons
            if prev.btn.into_bytes() != state.btn.into_bytes() {
                update_btn(&mut vjoy, prev.btn.a(), state.btn.a(), &GP::East)?;
                update_btn(&mut vjoy, prev.btn.b(), state.btn.b(), &GP::South)?;
                update_btn(&mut vjoy, prev.btn.x(), state.btn.x(), &GP::North)?;
                update_btn(&mut vjoy, prev.btn.y(), state.btn.y(), &GP::TL)?;
                update_btn(&mut vjoy, prev.btn.z(), state.btn.z(), &GP::West)?;
                update_btn(&mut vjoy, prev.btn.start(), state.btn.start(), &GP::Start)?;
            }

            // c stick
            update_joy(
                &mut vjoy,
                &AbsolutePosition::RX,
                prev.c_stick_x,
                state.c_stick_x,
            )?;
            update_joy(
                &mut vjoy,
                &AbsolutePosition::RY,
                prev.c_stick_y,
                state.c_stick_y,
            )?;

            // dpad
            if prev.dpad.into_bytes() != state.dpad.into_bytes() {
                update_btn(&mut vjoy, prev.dpad.up(), state.dpad.up(), &DPad::Up)?;
                update_btn(&mut vjoy, prev.dpad.down(), state.dpad.down(), &DPad::Down)?;
                update_btn(&mut vjoy, prev.dpad.left(), state.dpad.left(), &DPad::Left)?;
                update_btn(
                    &mut vjoy,
                    prev.dpad.right(),
                    state.dpad.right(),
                    &DPad::Right,
                )?;
            }

            vjoy.synchronize()?;

            prev = state;
        }

        let dt = t0.elapsed();
        if dt < RATE_TARGET {
            thread::sleep(RATE_TARGET - dt);
        }
    }
}

fn get_keyboard() -> evdev::Device {
    for dev in evdev::enumerate().into_iter() {
        if dev.name() == Some(KEYBOARD_NAME) {
            return dev;
        }
    }
    panic!("Cannot get keyboard device")
}

#[inline(always)]
fn jval(value: f32) -> i32 {
    const JOY_UP_RANGE_F32: f32 = JOY_UP_RANGE as f32;
    (value * JOY_UP_RANGE_F32) as i32
}

#[inline(always)]
fn update_btn<T: uinput::event::Press + uinput::event::Release>(
    device: &mut uinput::Device,
    prev: bool,
    cur: bool,
    ev: &T,
) -> uinput::Result<()> {
    if prev && !cur {
        device.release(ev)
    } else if !prev && cur {
        device.press(ev)
    } else {
        Ok(())
    }
}

#[inline(always)]
fn update_joy<T: uinput::event::Position>(
    device: &mut uinput::Device,
    event: &T,
    prevvalue: f32,
    value: f32,
) -> uinput::Result<()> {
    if prevvalue != value {
        device.position(event, jval(value))
    } else {
        Ok(())
    }
}
