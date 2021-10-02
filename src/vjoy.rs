use crate::config::{Settings, JOY_DOWN_RANGE, JOY_UP_RANGE};
use crossbeam::queue::SegQueue;
use env_logger;
use evdev_rs::{
    enums::{EventCode, EventType, InputProp, EV_ABS, EV_KEY, EV_SYN},
    AbsInfo, Device, DeviceWrapper, InputEvent, ReadFlag, TimeVal, UInputDevice, UninitDevice,
};
use libc::{gettimeofday, timeval};
use log;
use modular_bitfield::{
    bitfield,
    specifiers::{B4, B6},
};
use std::error::Error;
use std::fs::File;
use std::io::Result;
use std::ptr;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

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
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_TR))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_EAST))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_WEST))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_NORTH))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_SOUTH))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_TL))?;
        inp.enable(&EventCode::EV_KEY(EV_KEY::BTN_Z))?;
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

    pub fn joystick(&self, key: EV_ABS, prev_value: f32, value: f32) -> Result<()> {
        const JOY_UP_RANGE_F32: f32 = JOY_UP_RANGE as f32;
        if prev_value != value {
            self.device.write_event(&InputEvent {
                time: self.now,
                event_code: EventCode::EV_ABS(key),
                value: (value * JOY_UP_RANGE_F32) as i32,
            })?;
        }
        Ok(())
    }

    pub fn trigger(&self, key: EV_ABS, prev_depth: f32, depth: f32) -> Result<()> {
        if prev_depth != depth {
            self.device.write_event(&InputEvent {
                time: self.now,
                event_code: EventCode::EV_ABS(key),
                value: (127.0 + depth * 128.0) as i32, // makes no sense, but it works :)
            })?;
        }
        Ok(())
    }
}
