#[allow(non_snake_case)]
use crossbeam::queue::SegQueue;
use env_logger;
use evdev_rs::{
    enums::{EventCode, EventType, InputProp, EV_ABS, EV_KEY, EV_SYN},
    AbsInfo, Device, DeviceWrapper, ReadFlag, UInputDevice, UninitDevice,
};
use log;
use modular_bitfield::{
    bitfield,
    specifiers::{B4, B6},
};
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// use uinput;
// use uinput::event::absolute::Position as AbsolutePosition;
// use uinput::event::controller::{DPad, GamePad as GP};

mod config;
mod state;
mod vjoy;
use crate::config::*;
use crate::state::*;
use crate::vjoy::*;

// use crate::config::{Config, DPad8Binds, DPadBinds, Dir8, DirectionalBinds};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let settings = Settings::new()?;

    let q = Arc::new(SegQueue::<JoyState>::new());

    {
        let q = q.clone();
        let settings = settings.clone();
        thread::spawn(move || {
            let binds = settings.binds.clone();
            let kbd = {
                let path = settings.keyboard_path.clone();
                let file = File::open(path).expect("Could not open keyboard device");
                Device::new_from_file(file).expect("Could not create keyboard device")
            };
            let mut state = JoyState::default();
            let mut prev = state;
            loop {
                let mut changes = false;
                while let Ok((_status, ev)) = kbd.next_event(ReadFlag::BLOCKING) {
                    if ev.is_code(&EventCode::EV_SYN(EV_SYN::SYN_REPORT)) {
                        if changes {
                            state.updated = UpdatedTimeVal(ev.time);
                            state.update_analog(&settings, &prev);
                            state.sanity();
                            q.push(state);
                            prev = state;
                            changes = false;
                        }
                    }
                    changes = state.update_flags(&binds, ev) || changes;
                }
            }
        });
    }

    let mut vjoy = VJoy::new(&settings)?;
    let mut prev = JoyState::default();
    loop {
        let t0 = Instant::now();

        while let Some(state) = q.pop() {
            // control stick
            vjoy.now = state.updated.0;
            vjoy.joystick(EV_ABS::ABS_X, prev.control_stick_x, state.control_stick_x)?;
            vjoy.joystick(EV_ABS::ABS_Y, prev.control_stick_y, state.control_stick_y)?;

            // R trigger, put it here to marginally reduce latency on wavedash
            // which is the only 1 frame input I can think of at the moment
            vjoy.key(EV_KEY::BTN_TR, prev.btn.r(), state.btn.r())?;

            // // L trigger
            vjoy.trigger(EV_ABS::ABS_Z, prev.l_trigger, state.l_trigger)?;

            // buttons
            if prev.btn.into_bytes() != state.btn.into_bytes() {
                vjoy.key(EV_KEY::BTN_EAST, prev.btn.a(), state.btn.a())?;
                vjoy.key(EV_KEY::BTN_SOUTH, prev.btn.b(), state.btn.b())?;
                vjoy.key(EV_KEY::BTN_NORTH, prev.btn.x(), state.btn.x())?;
                vjoy.key(EV_KEY::BTN_TL, prev.btn.y(), state.btn.y())?;
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

        let dt = t0.elapsed();
        if dt < settings.poll_rate {
            thread::sleep(settings.poll_rate - dt);
        }
    }
}
