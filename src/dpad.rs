use super::dir8::Dir8;
use fixed::types::I1F7;
use modular_bitfield::bitfield;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct JoyStickState {
    pub dpad: DPadState,
    pub x: I1F7,
    pub y: I1F7,
}

impl JoyStickState {
    #[inline]
    pub fn update_x(&mut self, mul: Option<I1F7>) {
        let cx = (self.dpad.right() as i8) - (self.dpad.left() as i8);
        if let Some(mul) = mul {
            self.x = I1F7::saturating_from_num(cx).saturating_mul(mul);
        } else {
            self.x = I1F7::saturating_from_num(cx);
        }
    }

    #[inline]
    pub fn update_y(&mut self, mul: Option<I1F7>) {
        let cy = (self.dpad.down() as i8) - (self.dpad.up() as i8);
        if let Some(mul) = mul {
            self.y = I1F7::saturating_from_num(cy).saturating_mul(mul);
        } else {
            self.y = I1F7::saturating_from_num(cy);
        }
    }

    #[inline]
    pub fn step(&mut self, x_incr: I1F7, y_incr: I1F7) {
        let vx = (self.dpad.right() as i8) - (self.dpad.left() as i8);
        let vy = (self.dpad.down() as i8) - (self.dpad.up() as i8);
        let (x0, y0) = (self.x, self.y);
        let (mut x1, mut y1) = (I1F7::saturating_from_num(vx), I1F7::saturating_from_num(vy));

        match (x0.is_zero(), y0.is_zero()) {
            (true, true) => {}
            (true, false) => {
                x1 = self.x.saturating_add(x1 * x_incr);
            }
            (false, true) => {
                y1 = self.y.saturating_add(y1 * y_incr);
            }
            (false, false) => {
                x1 = self.x.saturating_add(x1 * x_incr);
                y1 = self.y.saturating_add(y1 * y_incr);
            }
        }

        self.x = x1;
        self.y = y1;

        // match (vx, vy) {
        //     (0, 0) => {
        //         self.x = I1F7::ZERO;
        //         self.y = I1F7::ZERO;
        //     }
        //     (0, _) => {
        //         self.x = I1F7::ZERO;
        //         self.y = y1;
        //     }
        //     (_, 0) => {
        //         self.x = x1;
        //         self.y = I1F7::ZERO;
        //     }
        //     _ => {
        //     }
        // }
    }
}

#[bitfield]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct DPadState {
    pub up_held: bool,
    pub down_held: bool,
    pub left_held: bool,
    pub right_held: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl DPadState {
    #[inline]
    pub fn on_up(&mut self, value: bool) {
        self.set_up(value);
        self.set_up_held(value);
        if value {
            self.set_down(false);
        } else {
            self.set_down(self.down_held());
        }
    }

    #[inline]
    pub fn on_down(&mut self, value: bool) {
        self.set_down(value);
        self.set_down_held(value);
        if value {
            self.set_up(false);
        } else {
            self.set_up(self.up_held());
        }
    }

    #[inline]
    pub fn on_left(&mut self, value: bool) {
        self.set_left(value);
        self.set_left_held(value);
        if value {
            self.set_right(false);
        } else {
            self.set_right(self.right_held());
        }
    }

    #[inline]
    pub fn on_right(&mut self, value: bool) {
        self.set_right(value);
        self.set_right_held(value);
        if value {
            self.set_left(false);
        } else {
            self.set_left(self.left_held());
        }
    }
}
