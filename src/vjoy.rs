use crate::config::{Settings, JOY_DOWN_RANGE, JOY_UP_RANGE};
use evdev_rs::{
    enums::{EventCode, EventType, EV_ABS, EV_KEY, EV_SYN},
    AbsInfo, DeviceWrapper, InputEvent, TimeVal, UInputDevice, UninitDevice,
};
use fixed::types::I1F7;
use log;
use std::io::Result;

pub struct VJoy {
    pub device: UInputDevice,
    pub now: TimeVal,
}

impl VJoy {
    pub fn new(_cfg: &Settings) -> Result<VJoy> {
        let inp = UninitDevice::new().unwrap();
        inp.set_name("melee-vpad");
        inp.enable(&EventType::EV_SYN)?;

        inp.enable(&EventType::EV_KEY)?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_EAST))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_WEST))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_NORTH))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_SOUTH))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_Z))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_TL))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_TR))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_START))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_UP))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_DOWN))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_LEFT))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_DPAD_RIGHT))?;

        inp.enable(&EventType::EV_ABS)?;

        inp.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_X),
            Some(&AbsInfo {
                value: 0,
                minimum: JOY_DOWN_RANGE,
                maximum: JOY_UP_RANGE,
                fuzz: 0,
                flat: 0,
                resolution: 255,
            }),
        )?;

        inp.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_Y),
            Some(&AbsInfo {
                value: 0,
                minimum: JOY_DOWN_RANGE,
                maximum: JOY_UP_RANGE,
                fuzz: 0,
                flat: 0,
                resolution: 255,
            }),
        )?;

        inp.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_RX),
            Some(&AbsInfo {
                value: 0,
                minimum: JOY_DOWN_RANGE,
                maximum: JOY_UP_RANGE,
                fuzz: 0,
                flat: 0,
                resolution: 255,
            }),
        )?;

        inp.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_RY),
            Some(&AbsInfo {
                value: 0,
                minimum: JOY_DOWN_RANGE,
                maximum: JOY_UP_RANGE,
                fuzz: 0,
                flat: 0,
                resolution: 255,
            }),
        )?;

        inp.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_Z),
            Some(&AbsInfo {
                value: 0,
                minimum: JOY_DOWN_RANGE,
                maximum: JOY_UP_RANGE,
                fuzz: 0,
                flat: 0,
                resolution: 255,
            }),
        )?;

        inp.enable_event_code(
            &EventCode::EV_ABS(EV_ABS::ABS_Z),
            Some(&AbsInfo {
                value: 0,
                minimum: 0,
                maximum: 255,
                fuzz: 0,
                flat: 0,
                resolution: 255,
            }),
        )?;

        let device = UInputDevice::create_from_device(&inp)?;
        log::info!("Created virtual gamepad device {:?}", device.devnode());
        Ok(VJoy {
            device,
            now: TimeVal {
                tv_sec: 0,
                tv_usec: 0,
            },
        })
    }

    pub fn sync(&self) -> Result<()> {
        self.device.write_event(&InputEvent {
            time: self.now,
            event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
            value: 0,
        })
    }

    pub fn key(&self, key: EV_KEY, prev_value: bool, value: bool) -> Result<()> {
        if prev_value != value {
            self.device.write_event(&InputEvent {
                time: self.now,
                event_code: EventCode::EV_KEY(key),
                value: value as i32,
            })?;
        }
        Ok(())
    }

    const JOY_UP_RANGE_f32: f32 = JOY_UP_RANGE as f32;

    pub fn joystick(&self, key: EV_ABS, prev_value: I1F7, value: I1F7) -> Result<()> {
        if prev_value != value {
            return self.joystick_always(key, value);
        }
        Ok(())
    }

    pub fn joystick_always(&self, key: EV_ABS, value: I1F7) -> Result<()> {
        self.device.write_event(&InputEvent {
            time: self.now,
            event_code: EventCode::EV_ABS(key),
            value: (value.to_num::<f32>() * Self::JOY_UP_RANGE_f32) as i32,
        })?;
        Ok(())
    }

    pub fn trigger(&self, key: EV_ABS, prev_depth: I1F7, depth: I1F7) -> Result<()> {
        if prev_depth != depth {
            self.device.write_event(&InputEvent {
                time: self.now,
                event_code: EventCode::EV_ABS(key),
                value: (127.0 + depth.to_num::<f32>() * 128.0) as i32, // makes no sense, but it works :)
            })?;
        }
        Ok(())
    }
}
