#[allow(non_snake_case)]
use env_logger;
use evdev_rs::{enums::EventCode, Device, ReadFlag};
use std::error::Error;
use std::fs::File;

mod config;
mod dir8;
mod dpad;
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

    let vjoy = VJoy::new(&settings)?;

    let mut state = JoyState::default();
    let binds_map = BindsMap::create(&settings.binds);

    let poll_rate = settings.poll_rate;

    if poll_rate.as_millis() > 0 {
        log::debug!("using polling event loop");
        loop {
            let t0 = std::time::Instant::now();
            if !kbd.has_event_pending() {
                // do nothing
            } else if let Ok((_status, ev)) = kbd.next_event(ReadFlag::NORMAL) {
                match ev.event_code {
                    EventCode::EV_KEY(key) if !(ev.value > 1) => {
                        let value = ev.value != 0;
                        if let Some(update) = binds_map.lookup_key(key, value) {
                            update.run(&mut state, &vjoy, &settings);
                        }
                    }
                    _ => {}
                }
            }
            let dt = t0.elapsed();
            if dt < poll_rate {
                std::thread::sleep(poll_rate - dt);
            }
        }
    } else {
        log::debug!("using blocking event loop");
        while let Ok((_status, ev)) = kbd.next_event(ReadFlag::BLOCKING) {
            match ev.event_code {
                EventCode::EV_KEY(key) if !(ev.value > 1) => {
                    let value = ev.value != 0;
                    if let Some(update) = binds_map.lookup_key(key, value) {
                        update.run(&mut state, &vjoy, &settings);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
