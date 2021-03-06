use evdev_rs::enums::EV_KEY;
use fixed::types::I1F7;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSecondsWithFrac};
use std::error::Error;
use std::fs;
use std::time::Duration;
use xdg;

pub const JOY_UP_RANGE: i32 = 127;
pub const JOY_DOWN_RANGE: i32 = -JOY_UP_RANGE;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub keyboard_path: String,
    #[serde_as(as = "DurationMilliSecondsWithFrac")]
    pub poll_rate: Duration,
    pub mod1_incr: I1F7,
    pub mod1_around_y: I1F7,
    pub mod1_trigger_mul: I1F7,
    pub mod2_x_mul: I1F7,
    pub mod2_y_mul: I1F7,
    pub binds: Binds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binds {
    pub a: EV_KEY,
    pub b: EV_KEY,
    pub z: EV_KEY,
    pub x: EV_KEY,
    pub y: EV_KEY,
    pub l: EV_KEY,
    pub r: EV_KEY,
    pub start: EV_KEY,
    pub mod1: EV_KEY,
    pub mod2: EV_KEY,
    pub control_stick: DPad8Binds,
    pub c_stick: DPadBinds,
    pub dpad: DPadBinds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPadBinds {
    pub up: EV_KEY,
    pub down: EV_KEY,
    pub left: EV_KEY,
    pub right: EV_KEY,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPad8Binds {
    pub upleft: EV_KEY,
    pub up: EV_KEY,
    pub upright: EV_KEY,
    pub downleft: EV_KEY,
    pub down: EV_KEY,
    pub downright: EV_KEY,
    pub left: EV_KEY,
    pub right: EV_KEY,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            keyboard_path: "/dev/input/by-id/usb-CATEX_TECH._84EC-XRGB_CA2017090002-event-kbd"
                .to_string(),
            poll_rate: Duration::from_micros(250),
            mod1_incr: I1F7::saturating_from_num(0.3875_f32),
            mod1_around_y: I1F7::saturating_from_num(0.31_f32),
            mod1_trigger_mul: I1F7::saturating_from_num(129.0 / 256.0 as f32),
            mod2_x_mul: I1F7::saturating_from_num(48.0 / 128.0 as f32),
            mod2_y_mul: I1F7::saturating_from_num(48.0 / 128.0 as f32),
            binds: Binds {
                a: EV_KEY::KEY_J,
                b: EV_KEY::KEY_K,
                z: EV_KEY::KEY_L,
                x: EV_KEY::KEY_SPACE,
                y: EV_KEY::KEY_LEFTALT, // I don't really use this at all
                start: EV_KEY::KEY_T,
                l: EV_KEY::KEY_I,
                r: EV_KEY::KEY_O,
                mod1: EV_KEY::KEY_LEFTSHIFT,
                mod2: EV_KEY::KEY_SLASH,
                control_stick: DPad8Binds {
                    up: EV_KEY::KEY_W,
                    upleft: EV_KEY::KEY_Q,
                    upright: EV_KEY::KEY_E,
                    down: EV_KEY::KEY_S,
                    downleft: EV_KEY::KEY_Z,
                    downright: EV_KEY::KEY_C,
                    left: EV_KEY::KEY_A,
                    right: EV_KEY::KEY_D,
                },
                c_stick: DPadBinds {
                    up: EV_KEY::KEY_H,
                    down: EV_KEY::KEY_N,
                    left: EV_KEY::KEY_B,
                    right: EV_KEY::KEY_M,
                },
                dpad: DPadBinds {
                    up: EV_KEY::KEY_UP,
                    down: EV_KEY::KEY_DOWN,
                    left: EV_KEY::KEY_LEFT,
                    right: EV_KEY::KEY_RIGHT,
                },
            },
        }
    }
}

impl Settings {
    pub fn new() -> Result<Settings, Box<dyn Error>> {
        let pathbuf = xdg::BaseDirectories::new()?.place_config_file("melee-vpad.toml")?;
        let cfg = if pathbuf.exists() {
            let contents = fs::read_to_string(pathbuf)?;
            let cfg = toml::from_str(&contents)?;
            cfg
        } else {
            log::info!("Creating config file from defaults {:?}", pathbuf);
            let def = Settings::default();
            fs::write(pathbuf, toml::to_string_pretty(&def)?)?;
            def
        };
        log::info!("{:#?}", cfg);
        Ok(cfg)
    }
}
