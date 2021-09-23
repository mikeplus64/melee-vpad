use crossbeam::queue::SegQueue;
use env_logger;
use evdev;
use log;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use uinput;
use uinput::event::absolute::Position as AbsolutePosition;
use uinput::event::controller::{Controller as C, DPad, GamePad as GP};

const KEYBOARD_NAME: &str = "CATEX TECH. 84EC-XRGB";

const J_MOD1: f32 = 64.0 / 128.0; // c stick only
const J_MOD2: f32 = 48.0 / 128.0;
const J_MOD12: f32 = 32.0 / 128.0; // c stick only

const TRIGGER_MOD1: f32 = 129.0 / 256.0;
const TRIGGER_MOD2: f32 = 92.0 / 256.0;
const TRIGGER_MOD12: f32 = 64.0 / 256.0;

#[derive(Copy, Clone, Default, Debug)]
struct JoyStateChange {
    l: bool,
    r: bool,
}

#[derive(Copy, Clone, Default, Debug)]
struct JoyState {
    // control stick
    l_up: bool,
    l_down: bool,
    l_left: bool,
    l_right: bool,
    l_active: bool,
    l_y: f32,
    l_x: f32,
    l_mul: f32,

    // c stick
    c_up: bool,
    c_down: bool,
    c_left: bool,
    c_right: bool,
    c_x: f32,
    c_y: f32,

    // triggers
    l_trigger: bool,
    l_trigger_depth: f32,
    c_trigger: bool,

    // buttons
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    z: bool,
    start: bool,

    // dpad
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,

    // modifiers
    mod1: bool,
    mod2: bool,
}

impl JoyState {
    fn l_state(&self) -> (bool, bool, bool, bool, bool) {
        (self.l_up, self.l_down, self.l_left, self.l_right, self.mod1)
    }

    fn update_flags(&mut self, event: evdev::InputEvent) {
        use evdev::{InputEventKind, Key};
        if let InputEventKind::Key(k) = event.kind() {
            // Control stick
            if let Some(r) = match k {
                Key::KEY_W => Some(&mut self.l_up),
                Key::KEY_S => Some(&mut self.l_down),
                Key::KEY_A => Some(&mut self.l_left),
                Key::KEY_D => Some(&mut self.l_right),
                // C stick
                Key::KEY_H => Some(&mut self.c_up),
                Key::KEY_N => Some(&mut self.c_down),
                Key::KEY_B => Some(&mut self.c_left),
                Key::KEY_M => Some(&mut self.c_right),
                // left trigger
                Key::KEY_I => Some(&mut self.l_trigger),
                // buttons
                Key::KEY_J => Some(&mut self.a),
                Key::KEY_K => Some(&mut self.b),
                Key::KEY_SPACE => Some(&mut self.x),
                Key::KEY_L => Some(&mut self.z),
                Key::KEY_T => Some(&mut self.start),
                // dpad
                Key::KEY_UP => Some(&mut self.dpad_up),
                Key::KEY_DOWN => Some(&mut self.dpad_down),
                Key::KEY_LEFT => Some(&mut self.dpad_left),
                Key::KEY_RIGHT => Some(&mut self.dpad_right),
                // modifiers
                Key::KEY_LEFTSHIFT => Some(&mut self.mod1),
                Key::KEY_SLASH => Some(&mut self.mod2),
                _ => None,
            } {
                *r = event.value() != 0;
            }
        }
    }

    fn update_analog(&mut self) {
        self.l_x = self.joyval(self.l_right, self.l_left);
        self.l_y = self.joyval(self.l_down, self.l_up);
        self.l_active = self.l_up || self.l_down || self.l_left || self.l_right;
        self.l_mul = if self.mod2 { J_MOD2 } else { 1.0 };
        self.c_x = self.joyval(self.c_right, self.c_left);
        self.c_y = self.joyval(self.c_down, self.c_up);
        self.l_trigger_depth = self.triggerval();
    }

    // only used for C stick
    fn joy_modval(&self) -> f32 {
        if self.mod1 && self.mod2 {
            J_MOD12
        } else if self.mod1 {
            J_MOD1
        } else if self.mod2 {
            J_MOD2
        } else {
            1.0
        }
    }

    fn joyval(&self, high: bool, low: bool) -> f32 {
        if high && low {
            0.0
        } else if high {
            1.0 * self.joy_modval()
        } else if low {
            -1.0 * self.joy_modval()
        } else {
            0.0
        }
    }

    fn triggerval(&self) -> f32 {
        if self.l_trigger {
            if self.mod1 && self.mod2 {
                TRIGGER_MOD12
            } else if self.mod1 {
                TRIGGER_MOD1
            } else if self.mod2 {
                TRIGGER_MOD2
            } else {
                1.0
            }
        } else {
            0.0
        }
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
        loop {
            let mut any_events = false;
            for ev in kbd.fetch_events().unwrap() {
                any_events = true;
                state.update_flags(ev);
            }
            if any_events {
                state.update_analog();
                q.push(state);
            }
        }
    });

    let mut prev = JoyState::default();
    let mut l_x = 0.0;
    let mut l_y = 0.0;
    loop {
        let t0 = Instant::now();
        while let Some(state) = q.pop() {
            // control stick
            if state.l_state() != prev.l_state() {
                let dx = ((state.l_right as i8) as f32) - ((state.l_left as i8) as f32);
                let dy = ((state.l_down as i8) as f32) - ((state.l_up as i8) as f32);
                if state.mod1 {
                    l_x = if dy != 0.0 { 0.5 * (dx + l_x) } else { dx };
                    l_y = if dx != 0.0 { 0.5 * (dy + l_y) } else { dy };
                } else {
                    l_x = dx;
                    l_y = dy;
                }
                vjoy.position(&AbsolutePosition::X, jval(l_x * state.l_mul))?;
                vjoy.position(&AbsolutePosition::Y, jval(l_y * state.l_mul))?;
            }

            // c stick
            update_joy(&mut vjoy, &AbsolutePosition::RX, prev.c_x, state.c_x)?;
            update_joy(&mut vjoy, &AbsolutePosition::RY, prev.c_y, state.c_y)?;

            // trigger
            if prev.l_trigger_depth != state.l_trigger_depth {
                vjoy.position(
                    &AbsolutePosition::Z,
                    // this doesn't make any sense to me but it works in dolphin
                    (127.0 + state.l_trigger_depth * 128.0) as i32,
                )?;
            }

            // buttons
            for (prev, cur, ev) in &[
                (prev.a, state.a, C::GamePad(GP::East)),
                (prev.b, state.b, C::GamePad(GP::South)),
                (prev.x, state.x, C::GamePad(GP::North)),
                (prev.z, state.z, C::GamePad(GP::West)),
                (prev.start, state.start, C::GamePad(GP::Start)),
                (prev.dpad_up, state.dpad_up, C::DPad(DPad::Up)),
                (prev.dpad_down, state.dpad_down, C::DPad(DPad::Down)),
                (prev.dpad_left, state.dpad_left, C::DPad(DPad::Left)),
                (prev.dpad_right, state.dpad_right, C::DPad(DPad::Right)),
            ] {
                if *prev && !*cur {
                    vjoy.release(ev)?;
                } else if !*prev && *cur {
                    vjoy.press(ev)?;
                }
            }

            vjoy.synchronize()?;

            prev = state;
        }

        let dt = t0.elapsed();
        if dt < Duration::from_micros(500) {
            thread::sleep(Duration::from_micros(500) - dt);
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
