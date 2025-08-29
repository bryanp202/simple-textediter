#![allow(dead_code)]

#[derive(Default)]
pub struct InputState {
    pub mouse: Mouse,
    pub keyboard: Keyboard,
}

#[derive(Default)]
pub struct Mouse {
    left: bool,
    right: bool,
    middle: bool,
}

impl Mouse {
    pub fn left_down(&self) -> bool {
        self.left
    }

    pub fn middle_down(&self) -> bool {
        self.middle
    }

    pub fn right_down(&self) -> bool {
        self.right
    }

    pub fn press_left(&mut self) {
        self.left = true;
    }

    pub fn release_left(&mut self) {
        self.left = false;
    }

    pub fn press_middle(&mut self) {
        self.middle = true;
    }

    pub fn release_middle(&mut self) {
        self.middle = false;
    }

    pub fn press_right(&mut self) {
        self.right = true;
    }

    pub fn release_right(&mut self) {
        self.right = false;
    }
}

#[derive(Default)]
pub struct Keyboard {
    ctrl: bool,
    shift: bool,
}

impl Keyboard {
    pub fn ctrl_down(&self) -> bool {
        self.ctrl
    }

    pub fn shift_down(&self) -> bool {
        self.shift
    }

    pub fn press_ctrl(&mut self) {
        self.ctrl = true;
    }

    pub fn release_ctrl(&mut self) {
        self.ctrl = false;
    }

    pub fn press_shift(&mut self) {
        self.shift = true;
    }

    pub fn release_shift(&mut self) {
        self.shift = false;
    }
}