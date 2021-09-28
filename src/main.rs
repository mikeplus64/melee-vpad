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

const RATE_TARGET: Duration = Duration::from_micros(500);

#[derive(Copy, Clone, Default, Debug)]
struct JoyState {
    // control stick
    control_stick_up: bool,
    control_stick_down: bool,
    control_stick_left: bool,
    control_stick_right: bool,
    control_stick_active: bool,
    control_stick_x: f32,
    control_stick_y: f32,
    control_stick_mul: f32,
    control_stick_update: bool,

    // c stick
    c_up: bool,
    c_down: bool,
    c_left: bool,
    c_right: bool,
    c_x: f32,
    c_y: f32,

    // triggers
    l_trigger: bool,
    r_trigger: bool,
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

    // tag
    tag: u32,
}

impl JoyState {
    fn update_flags(&mut self, event: evdev::InputEvent) {
        use evdev::{InputEventKind, Key};
        if let InputEventKind::Key(k) = event.kind() {
            if let Some(r) = match k {
                // wasd -> control stick
                Key::KEY_W => Some(&mut self.control_stick_up),
                Key::KEY_S => Some(&mut self.control_stick_down),
                Key::KEY_A => Some(&mut self.control_stick_left),
                Key::KEY_D => Some(&mut self.control_stick_right),
                Key::KEY_LEFTSHIFT => Some(&mut self.mod1),
                Key::KEY_ENTER => Some(&mut self.mod2),
                _ => None,
            } {
                let cur = *r;
                let next = event.value() != 0;
                self.control_stick_update = self.control_stick_update || cur != next;
                *r = event.value() != 0;
            } else if let Some(r) = match k {
                // hbnm -> C stick
                // Key::KEY_H => Some(&mut self.c_up),
                // Key::KEY_N => Some(&mut self.c_down),
                // Key::KEY_B => Some(&mut self.c_left),
                // Key::KEY_M => Some(&mut self.c_right),
                // jkl -> ABZ buttons
                // Key::KEY_J => Some(&mut self.a),
                // Key::KEY_K => Some(&mut self.b),
                // Key::KEY_L => Some(&mut self.z),
                // io -> triggers
                // Key::KEY_I => Some(&mut self.l_trigger),
                // Key::KEY_O => Some(&mut self.r_trigger),

                // hbnm -> C stick
                Key::KEY_K => Some(&mut self.c_up),
                Key::KEY_COMMA => Some(&mut self.c_down),
                Key::KEY_M => Some(&mut self.c_left),
                Key::KEY_DOT => Some(&mut self.c_right),

                // p spc -> ABZX buttons
                Key::KEY_L => Some(&mut self.a),
                Key::KEY_SEMICOLON => Some(&mut self.b),
                Key::KEY_APOSTROPHE => Some(&mut self.z),
                Key::KEY_SPACE => Some(&mut self.x),

                // -= -> triggers
                Key::KEY_P => Some(&mut self.l_trigger),
                Key::KEY_LEFTBRACE => Some(&mut self.r_trigger),

                Key::KEY_T => Some(&mut self.start),
                // dpad
                Key::KEY_UP => Some(&mut self.dpad_up),
                Key::KEY_DOWN => Some(&mut self.dpad_down),
                Key::KEY_LEFT => Some(&mut self.dpad_left),
                Key::KEY_RIGHT => Some(&mut self.dpad_right),
                // modifiers
                Key::KEY_LEFTSHIFT => Some(&mut self.mod1),
                _ => None,
            } {
                *r = event.value() != 0;
            }
        }
        self.tag += 1;
    }

    fn update_analog(&mut self) {
        if self.control_stick_update {
            let active0 = self.control_stick_active;

            self.control_stick_mul = if self.mod2 { J_MOD2 } else { 1.0 };
            self.control_stick_active = self.control_stick_up
                || self.control_stick_down
                || self.control_stick_left
                || self.control_stick_right;

            let vx = (self.control_stick_right as i8) - (self.control_stick_left as i8);
            let vy = (self.control_stick_down as i8) - (self.control_stick_up as i8);
            let mut x = vx as f32;
            let mut y = vy as f32;
            if self.mod1 && self.control_stick_active {
                let x0 = self.control_stick_x;
                let y0 = self.control_stick_y;
                if !active0 {
                    if vx.abs() == 1 && vy.abs() == 1 {
                        y = 0.31 * vy as f32;
                    }
                } else {
                    if !(y0.abs() < 0.01) {
                        x = x0 + 0.3875 * vx as f32;
                    }
                    if !(x0.abs() < 0.01) {
                        y = y0 + 0.31 * vy as f32;
                    }
                }
                if !self.mod2 && x.abs() < 0.99 && y.abs() < 0.99 {
                    if x.abs() > y.abs() {
                        x = vx as f32;
                    } else {
                        y = vy as f32;
                    }
                }
            }
            self.control_stick_x = (x * self.control_stick_mul).clamp(-1.0, 1.0);
            self.control_stick_y = (y * self.control_stick_mul).clamp(-1.0, 1.0);
        }
        self.c_x = self.c_joyval(self.c_right, self.c_left);
        self.c_y = self.c_joyval(self.c_down, self.c_up);
        self.l_trigger_depth = self.triggerval();
    }

    // only used for C stick
    fn c_joy_modval(&self) -> f32 {
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

    fn c_joyval(&self, high: bool, low: bool) -> f32 {
        if high && low {
            0.0
        } else if high {
            1.0 * self.c_joy_modval()
        } else if low {
            -1.0 * self.c_joy_modval()
        } else {
            0.0
        }
    }

    fn triggerval(&self) -> f32 {
        // light shield and full shield and that's it
        if self.l_trigger {
            if self.mod1 {
                TRIGGER_MOD1
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
            state.control_stick_update = false;
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
                (prev.r_trigger, state.r_trigger, C::GamePad(GP::TR)),
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
