#[allow(non_snake_case)]
use env_logger;
use evdev_rs::{
    enums::{EventCode, EV_ABS, EV_KEY, EV_SYN},
    Device, ReadFlag,
};
use std::error::Error;
use std::fs::File;
use std::time::Instant;

mod config;
mod dir8;
mod state;
mod vjoy;
use crate::config::Settings;
use crate::state::*;
use crate::vjoy::*;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let settings = Settings::new()?;

    let kbd = {
        let path = settings.keyboard_path.clone();
        let file = File::open(path).expect("Could not open keyboard device");
        Device::new_from_file(file).expect("Could not create keyboard device")
    };

    let mut vjoy = VJoy::new(&settings)?;

    let mut state = JoyState::default();
    let mut prev = JoyState::default();

    'outer: loop {
        let mut updated = None;

        if let Ok((_status, ev)) = kbd.next_event(ReadFlag::NORMAL) {
            if !ev.is_code(&EventCode::EV_SYN(EV_SYN::SYN_REPORT)) {
                let have_changes = state.update_flags(&settings.binds, &ev);
                if have_changes {
                    state.update_analog(&settings, &prev);
                    updated = Some(ev.time);
                }
            }
        }

        if updated.is_none() {
            continue 'outer;
        }

        // control stick
        vjoy.now = updated.unwrap();
        vjoy.joystick(EV_ABS::ABS_X, prev.control_stick_x, state.control_stick_x)?;
        vjoy.joystick(EV_ABS::ABS_Y, prev.control_stick_y, state.control_stick_y)?;

        // L trigger
        vjoy.trigger(EV_ABS::ABS_Z, prev.l_trigger, state.l_trigger)?;

        // buttons
        if prev.btn.into_bytes() != state.btn.into_bytes() {
            vjoy.key(EV_KEY::BTN_EAST, prev.btn.a(), state.btn.a())?;
            vjoy.key(EV_KEY::BTN_SOUTH, prev.btn.b(), state.btn.b())?;
            vjoy.key(EV_KEY::BTN_NORTH, prev.btn.x(), state.btn.x())?;
            vjoy.key(EV_KEY::BTN_TL, prev.btn.y(), state.btn.y())?;
            vjoy.key(EV_KEY::BTN_TR, prev.btn.r(), state.btn.r())?;
            vjoy.key(EV_KEY::BTN_Z, prev.btn.z(), state.btn.z())?;
            vjoy.key(EV_KEY::BTN_START, prev.btn.start(), state.btn.start())?;
        }

        // c stick
        vjoy.joystick(EV_ABS::ABS_RX, prev.c_stick_x, state.c_stick_x)?;
        vjoy.joystick(EV_ABS::ABS_RY, prev.c_stick_y, state.c_stick_y)?;

        // // dpad
        if prev.dpad.into_bytes() != state.dpad.into_bytes() {
            vjoy.key(EV_KEY::BTN_DPAD_UP, prev.dpad.up(), state.dpad.up())?;
            vjoy.key(EV_KEY::BTN_DPAD_DOWN, prev.dpad.down(), state.dpad.down())?;
            vjoy.key(EV_KEY::BTN_DPAD_LEFT, prev.dpad.left(), state.dpad.left())?;
            vjoy.key(
                EV_KEY::BTN_DPAD_RIGHT,
                prev.dpad.right(),
                state.dpad.right(),
            )?;
        }

        vjoy.sync()?;
        prev = state;
    }
}
