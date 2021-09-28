use crossbeam::queue::SegQueue;
use env_logger;
use evdev;
use evdev::Key;
use log;
use modular_bitfield::{
    bitfield,
    specifiers::{B3, B6},
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
struct DPadBinds {
    up: Key,
    down: Key,
    left: Key,
    right: Key,
}

const CONTROL_STICK: DPadBinds = DPadBinds {
    up: Key::KEY_W,
    down: Key::KEY_S,
    left: Key::KEY_A,
    right: Key::KEY_D,
};

const C_STICK: DPadBinds = DPadBinds {
    up: Key::KEY_K,
    down: Key::KEY_COMMA,
    left: Key::KEY_M,
    right: Key::KEY_DOT,
};

const DPAD: DPadBinds = DPadBinds {
    up: Key::KEY_UP,
    down: Key::KEY_DOWN,
    left: Key::KEY_LEFT,
    right: Key::KEY_RIGHT,
};

const BTN_A: Key = Key::KEY_L;
const BTN_B: Key = Key::KEY_SEMICOLON;
const BTN_Z: Key = Key::KEY_APOSTROPHE;
const BTN_X: Key = Key::KEY_SPACE;
const BTN_Y: Key = Key::KEY_LEFTALT; // I don't really use this at all
const BTN_START: Key = Key::KEY_T;
const BTN_L: Key = Key::KEY_P;
const BTN_R: Key = Key::KEY_LEFTBRACE;
// Modifiers
const BTN_MOD1: Key = Key::KEY_LEFTSHIFT;
const BTN_MOD2: Key = Key::KEY_ENTER;
// =================

const J_MOD1_INCR: f32 = 0.3875;
const J_MOD1_AROUND_Y: f32 = 0.31;
const J_MOD2: f32 = 48.0 / 128.0;
const TRIGGER_MOD1: f32 = 129.0 / 256.0;

const RATE_TARGET: Duration = Duration::from_micros(250);

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
struct DPadState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    updated: bool,
    #[skip]
    __: B3,
}

impl DPadState {
    pub fn is_active(&self) -> bool {
        self.left() || self.right() || self.up() || self.down()
    }

    pub fn update(&mut self, binds: DPadBinds, key: Key, value: bool) -> bool {
        if key == binds.up {
            let prev = self.up();
            self.set_up(value);
            self.set_updated(prev != value);
            true
        } else if key == binds.down {
            let prev = self.down();
            self.set_down(value);
            self.set_updated(prev != value);
            true
        } else if key == binds.left {
            let prev = self.left();
            self.set_left(value);
            self.set_updated(prev != value);
            true
        } else if key == binds.right {
            let prev = self.right();
            self.set_right(value);
            self.set_updated(prev != value);
            true
        } else {
            false
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
        self.control_stick.set_updated(false);
        self.c_stick.set_updated(false);
        self.dpad.set_updated(false);
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
        .min(-127)
        .max(128)
        .event(AbsolutePosition::Y)?
        .min(-127)
        .max(128)
        .event(AbsolutePosition::Z)?
        .min(0)
        .max(255)
        .event(AbsolutePosition::RX)?
        .min(-127)
        .max(128)
        .event(AbsolutePosition::RY)?
        .min(-127)
        .max(128)
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
            if prev.btn != state.btn {
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
            if state.dpad.updated() {
                update_btn(&mut vjoy, prev.dpad.up(), state.dpad.up(), &DPad::Up)?;
                update_btn(&mut vjoy, prev.dpad.down(), state.dpad.down(), &DPad::Up)?;
                update_btn(&mut vjoy, prev.dpad.left(), state.dpad.left(), &DPad::Up)?;
                update_btn(&mut vjoy, prev.dpad.right(), state.dpad.right(), &DPad::Up)?;
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

fn jval(value: f32) -> i32 {
    (value * 127.0) as i32
}

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
