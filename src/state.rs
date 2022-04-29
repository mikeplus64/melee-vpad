use crate::config::{Binds, Settings};
use crate::dpad::{DPadState, JoyStickState};
use crate::vjoy::VJoy;
use evdev_rs::enums::{EV_ABS, EV_KEY};
use fixed::types::I1F7;
use modular_bitfield::{bitfield, specifiers::B6};
// use std::collections::HashMap;

#[derive(Copy, Clone, Default, Debug)]
pub struct JoyState {
    // control stick
    pub control_stick: JoyStickState,
    // c stick
    pub c_stick: JoyStickState,
    // dpad
    pub dpad: DPadState,
    // analog l trigger
    pub l_trigger: I1F7,
    // digital buttons
    pub btn: JoyButtons,
    // modifiers
    pub m: Modifiers,
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct JoyButtons {
    pub l: bool,
    pub r: bool,
    #[skip]
    __: B6,
    // dont store other buttons; they're not stateful enough to need to carry around
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Modifiers {
    pub mod1: bool,
    pub mod2: bool,
    #[skip]
    __: B6,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum StateUpdateKind {
    Noop,
    ControlStickLeft,
    ControlStickDownLeft,
    ControlStickUpLeft,
    ControlStickRight,
    ControlStickDownRight,
    ControlStickUpRight,
    ControlStickUp,
    ControlStickDown,
    CStickLeft,
    CStickRight,
    CStickUp,
    CStickDown,
    DPadLeft,
    DPadRight,
    DPadUp,
    DPadDown,
    BtnA,
    BtnB,
    BtnZ,
    BtnX,
    BtnY,
    BtnStart,
    BtnL,
    BtnR,
    Mod1,
    Mod2,
}

use StateUpdateKind::*;

impl Default for StateUpdateKind {
    fn default() -> StateUpdateKind {
        Noop
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StateUpdate {
    kind: StateUpdateKind,
    value: bool,
}

#[repr(transparent)]
#[derive(Clone)]
pub struct BindsMap {
    binds: [StateUpdateKind; 594],
}

impl BindsMap {
    pub fn create(cfg: &Binds) -> BindsMap {
        let mut r: [StateUpdateKind; 594] = [Noop; 594];

        r[cfg.control_stick.down as usize] = ControlStickDown;
        r[cfg.control_stick.up as usize] = ControlStickUp;
        r[cfg.control_stick.left as usize] = ControlStickLeft;
        r[cfg.control_stick.right as usize] = ControlStickRight;
        r[cfg.control_stick.downleft as usize] = ControlStickDownLeft;
        r[cfg.control_stick.downright as usize] = ControlStickDownRight;
        r[cfg.control_stick.upleft as usize] = ControlStickUpLeft;
        r[cfg.control_stick.upright as usize] = ControlStickUpRight;
        r[cfg.c_stick.down as usize] = CStickDown;
        r[cfg.c_stick.up as usize] = CStickUp;
        r[cfg.c_stick.left as usize] = CStickLeft;
        r[cfg.c_stick.right as usize] = CStickRight;
        r[cfg.dpad.down as usize] = DPadDown;
        r[cfg.dpad.up as usize] = DPadUp;
        r[cfg.dpad.left as usize] = DPadLeft;
        r[cfg.dpad.right as usize] = DPadRight;
        r[cfg.a as usize] = BtnA;
        r[cfg.b as usize] = BtnB;
        r[cfg.x as usize] = BtnX;
        r[cfg.y as usize] = BtnY;
        r[cfg.z as usize] = BtnZ;
        r[cfg.start as usize] = BtnStart;
        r[cfg.l as usize] = BtnL;
        r[cfg.r as usize] = BtnR;
        r[cfg.mod1 as usize] = Mod1;
        r[cfg.mod2 as usize] = Mod2;

        // let mut r = HashMap::<EV_KEY, StateUpdateKind>::new();
        // r.insert(cfg.control_stick.down, ControlStickDown);
        // r.insert(cfg.control_stick.up, ControlStickUp);
        // r.insert(cfg.control_stick.left, ControlStickLeft);
        // r.insert(cfg.control_stick.right, ControlStickRight);
        // r.insert(cfg.control_stick.downleft, ControlStickDownLeft);
        // r.insert(cfg.control_stick.downright, ControlStickDownRight);
        // r.insert(cfg.control_stick.upleft, ControlStickUpLeft);
        // r.insert(cfg.control_stick.upright, ControlStickUpRight);
        // r.insert(cfg.c_stick.down, CStickDown);
        // r.insert(cfg.c_stick.up, CStickUp);
        // r.insert(cfg.c_stick.left, CStickLeft);
        // r.insert(cfg.c_stick.right, CStickRight);
        // r.insert(cfg.dpad.down, DPadDown);
        // r.insert(cfg.dpad.up, DPadUp);
        // r.insert(cfg.dpad.left, DPadLeft);
        // r.insert(cfg.dpad.right, DPadRight);
        // r.insert(cfg.a, BtnA);
        // r.insert(cfg.b, BtnB);
        // r.insert(cfg.x, BtnX);
        // r.insert(cfg.y, BtnY);
        // r.insert(cfg.z, BtnZ);
        // r.insert(cfg.start, BtnStart);
        // r.insert(cfg.l, BtnL);
        // r.insert(cfg.r, BtnR);
        // r.insert(cfg.mod1, Mod1);
        // r.insert(cfg.mod2, Mod2);
        //
        BindsMap { binds: r }
    }

    #[inline]
    pub fn lookup_key(&self, key: EV_KEY, value: bool) -> Option<StateUpdate> {
        let kind = self.binds[key as usize];
        Some(StateUpdate { kind, value })
    }
}

impl StateUpdate {
    #[inline]
    pub fn run(self, state: &mut JoyState, vjoy: &VJoy, settings: &Settings) {
        match self.kind {
            Noop => {
                return;
            }

            BtnA => {
                // state.btn.set_a(self.value);
                vjoy.key(EV_KEY::BTN_EAST, self.value);
            }

            BtnB => {
                // state.btn.set_b(self.value);
                vjoy.key(EV_KEY::BTN_SOUTH, self.value);
            }

            BtnX => {
                // state.btn.set_x(self.value);
                vjoy.key(EV_KEY::BTN_NORTH, self.value);
            }

            BtnY => {
                // state.btn.set_y(self.value);
                vjoy.key(EV_KEY::BTN_TL, self.value);
            }

            BtnL => {
                state.btn.set_l(self.value);
                state.l_trigger = if self.value {
                    if state.m.mod1() {
                        I1F7::from_num(settings.mod1_trigger_mul)
                    } else {
                        I1F7::MAX
                    }
                } else {
                    I1F7::ZERO
                };
                vjoy.trigger(EV_ABS::ABS_Z, state.l_trigger);
            }

            BtnR => {
                state.btn.set_r(self.value);
                vjoy.key(EV_KEY::BTN_TR, self.value);
            }

            BtnZ => {
                // state.btn.set_z(self.value);
                vjoy.key(EV_KEY::BTN_Z, self.value);
            }

            BtnStart => {
                // state.btn.set_start(self.value);
                vjoy.key(EV_KEY::BTN_START, self.value);
            }

            DPadLeft => {
                state.dpad.on_left(self.value);
                vjoy.key(EV_KEY::BTN_DPAD_LEFT, self.value);
            }

            DPadRight => {
                state.dpad.on_right(self.value);
                vjoy.key(EV_KEY::BTN_DPAD_RIGHT, self.value);
            }

            DPadUp => {
                state.dpad.on_up(self.value);
                vjoy.key(EV_KEY::BTN_DPAD_UP, self.value);
            }

            DPadDown => {
                state.dpad.on_down(self.value);
                vjoy.key(EV_KEY::BTN_DPAD_DOWN, self.value);
            }

            CStickLeft => {
                state.c_stick.dpad.on_left(self.value);
                state.c_stick.update_x(None);
                vjoy.joystick(EV_ABS::ABS_RX, state.c_stick.x);
            }

            CStickRight => {
                state.c_stick.dpad.on_right(self.value);
                state.c_stick.update_x(None);
                vjoy.joystick(EV_ABS::ABS_RX, state.c_stick.x);
            }

            CStickUp => {
                state.c_stick.dpad.on_up(self.value);
                state.c_stick.update_y(None);
                vjoy.joystick(EV_ABS::ABS_RY, state.c_stick.y);
            }

            CStickDown => {
                state.c_stick.dpad.on_down(self.value);
                state.c_stick.update_y(None);
                vjoy.joystick(EV_ABS::ABS_RY, state.c_stick.y);
            }

            ////////////////////////////////////////////////////////////////////////////////
            // Control stick
            ControlStickLeft => {
                state.control_stick.dpad.on_left(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else {
                    state
                        .control_stick
                        .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                }
                vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
            }

            ControlStickRight => {
                state.control_stick.dpad.on_right(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else {
                    state
                        .control_stick
                        .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                }
                vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
            }

            ControlStickUp => {
                state.control_stick.dpad.on_up(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else {
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                }
                vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
            }

            ControlStickDown => {
                state.control_stick.dpad.on_down(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else if state.m.mod2() && (state.btn.l() || state.btn.r()) {
                    const SHIELD_DROP_Y_MUL: f32 = 0.43_f32;
                    // shield drop special case
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), I1F7::from_num(SHIELD_DROP_Y_MUL)));
                } else {
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                }
                vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
            }

            ControlStickDownLeft => {
                state.control_stick.dpad.on_down(self.value);
                state.control_stick.dpad.on_left(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else {
                    state
                        .control_stick
                        .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                }
                vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
                vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
            }

            ControlStickDownRight => {
                state.control_stick.dpad.on_down(self.value);
                state.control_stick.dpad.on_right(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else {
                    state
                        .control_stick
                        .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                }
                vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
                vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
            }

            ControlStickUpLeft => {
                state.control_stick.dpad.on_up(self.value);
                state.control_stick.dpad.on_left(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else {
                    state
                        .control_stick
                        .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                }
                vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
                vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
            }

            ControlStickUpRight => {
                state.control_stick.dpad.on_up(self.value);
                state.control_stick.dpad.on_right(self.value);
                if state.m.mod1() {
                    state
                        .control_stick
                        .step(settings.mod1_incr, settings.mod1_around_y);
                } else {
                    state
                        .control_stick
                        .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                }
                vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
                vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
            }

            ////////////////////////////////////////////////////////////////////////////////
            Mod1 => {
                state.m.set_mod1(self.value);
                if !self.value {
                    state
                        .control_stick
                        .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                    state
                        .control_stick
                        .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                    vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
                    vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
                }
            }

            Mod2 => {
                state.m.set_mod2(self.value);
                state
                    .control_stick
                    .update_x(mod2_mul(state.m.mod2(), settings.mod2_x_mul));
                state
                    .control_stick
                    .update_y(mod2_mul(state.m.mod2(), settings.mod2_y_mul));
                vjoy.joystick(EV_ABS::ABS_X, state.control_stick.x);
                vjoy.joystick(EV_ABS::ABS_Y, state.control_stick.y);
            }
        }
        vjoy.sync();
    }
}

#[inline(always)]
fn mod2_mul(mod2: bool, mul: I1F7) -> Option<I1F7> {
    if mod2 {
        Some(mul)
    } else {
        None
    }
}
