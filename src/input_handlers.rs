use ggez::{input::keyboard, Context};
use glam::*;

pub trait InputHandler {
    fn is_up(&self) -> bool;
    fn is_down(&self) -> bool;
    fn is_left(&self) -> bool;
    fn is_right(&self) -> bool;
    fn is_rotating_cw(&self) -> bool;
    fn is_rotating_acw(&self) -> bool;
    fn tick(&mut self, ctx: &mut Context) -> Result<(), String>;
    // add a snapshot fn that returns a readonly copy of the state of the input handler
    fn snapshot(&self) -> CurrentInputs;
}
pub struct CurrentInputs {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    rotate_cw: bool,
    rotate_acw: bool,
}
#[derive(Clone)]
pub struct EmptyInputHandler {}

impl InputHandler for EmptyInputHandler {
    fn is_up(&self) -> bool {
        false
    }

    fn is_down(&self) -> bool {
        false
    }

    fn is_left(&self) -> bool {
        false
    }

    fn is_right(&self) -> bool {
        false
    }

    fn is_rotating_cw(&self) -> bool {
        false
    }

    fn is_rotating_acw(&self) -> bool {
        false
    }

    fn tick(&mut self, _ctx: &mut Context) -> Result<(), String> {
        Ok(())
    }
    fn snapshot(&self) -> CurrentInputs {
        CurrentInputs {
            up: false,
            down: false,
            left: false,
            right: false,
            rotate_cw: false,
            rotate_acw: false,
        }
    }
}
#[derive(Clone)]

pub struct KeyboardInputHandler {
    up_key: keyboard::KeyCode,
    down_key: keyboard::KeyCode,
    left_key: keyboard::KeyCode,
    right_key: keyboard::KeyCode,
    rotate_cw_key: keyboard::KeyCode,
    rotate_acw_key: keyboard::KeyCode,
    going_up: bool,
    going_down: bool,
    going_left: bool,
    going_right: bool,
    rotating_cw: bool,
    rotating_acw: bool,
}

impl KeyboardInputHandler {
    pub fn new(
        up_key: keyboard::KeyCode,
        down_key: keyboard::KeyCode,
        left_key: keyboard::KeyCode,
        right_key: keyboard::KeyCode,
        rotate_cw_key: keyboard::KeyCode,
        rotate_acw_key: keyboard::KeyCode,
    ) -> KeyboardInputHandler {
        KeyboardInputHandler {
            up_key,
            down_key,
            left_key,
            right_key,
            rotate_cw_key,
            rotate_acw_key,
            going_up: false,
            going_down: false,
            going_left: false,
            going_right: false,
            rotating_cw: false,
            rotating_acw: false,
        }
    }
}

impl InputHandler for KeyboardInputHandler {
    fn is_up(&self) -> bool {
        self.going_up
    }

    fn is_down(&self) -> bool {
        self.going_down
    }

    fn is_left(&self) -> bool {
        self.going_left
    }

    fn is_right(&self) -> bool {
        self.going_right
    }

    fn is_rotating_cw(&self) -> bool {
        self.rotating_cw
    }

    fn is_rotating_acw(&self) -> bool {
        self.rotating_acw
    }

    fn tick(&mut self, ctx: &mut Context) -> Result<(), String> {
        self.going_up = keyboard::is_key_pressed(ctx, self.up_key);
        self.going_down = keyboard::is_key_pressed(ctx, self.down_key);
        self.going_left = keyboard::is_key_pressed(ctx, self.left_key);
        self.going_right = keyboard::is_key_pressed(ctx, self.right_key);
        self.rotating_cw = keyboard::is_key_pressed(ctx, self.rotate_cw_key);
        self.rotating_acw = keyboard::is_key_pressed(ctx, self.rotate_acw_key);
        Ok(())
    }
    fn snapshot(&self) -> CurrentInputs {
        CurrentInputs {
            up: self.going_up,
            down: self.going_down,
            left: self.going_left,
            right: self.going_right,
            rotate_cw: self.rotating_cw,
            rotate_acw: self.rotating_acw,
        }
    }
}
