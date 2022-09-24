//! This module contains the input handlers for ppan.
//!
//!  Initially while designing the game, the inputs were hard-coded and I had planned to re-use the computation code per player. However, modularising the code was proven to make things easier, and I decided to make a separate module for storing traits of (input handlers)[InputHandler]. This allows for easy re-use of the code, and also allows for easy addition of new input handlers such as controllers, or AI.

use bytemuck::Zeroable;
use ggez::{input::keyboard, Context};
use glam::*;

/// A trait that can store the current inputs of a player and update them.
///
/// This abstracts away the input method, so that the same code can be used for real players, AI, and networked players.
pub trait InputHandler {
    /// Returns true if the player is pressing the "up" key.
    fn is_up(&self) -> bool;
    /// Returns true if the player is pressing the "down" key.
    fn is_down(&self) -> bool;
    /// Returns true if the player is pressing the "left" key.
    fn is_left(&self) -> bool;
    /// Returns true if the player is pressing the "right" key.
    fn is_right(&self) -> bool;
    /// Returns true if the player is pressing the "rotate clockwise" key.
    fn is_rotating_cw(&self) -> bool;
    /// Returns true if the player is pressing the "rotate anti-clockwise" key.
    fn is_rotating_acw(&self) -> bool;
    /// Update the state of the inputs.
    ///
    /// This is where the main logic for the input handler should go. This should almost always be deterministic, to ensure that players don't get different game states.
    ///
    /// # Example:
    /// ```ignore
    /// use input_handlers::*;
    ///
    /// for i in players.iter_mut() {
    ///     let player = KeyboardInputHandler::new(/* stuff and things... */);
    ///     player.tick(&mut ctx).unwrap();
    ///     // now we can do game logic!
    /// }
    fn tick(&mut self, ctx: &mut Context) -> Result<(), String>;
    /// Returns a readonly copy of the state of the input handler.
    ///
    /// Useful for situations where you'd want to save the state of the game, perhaps in replays or for networking.
    ///
    /// # Example:
    /// ```
    /// use input_handlers::*;
    ///
    /// let mut inputs = vec![];
    /// let player = NetworkInputHandler::new(9);
    ///
    /// # assert_eq!(player.snapshot(), 9);
    /// // keep list of inputs, for generating a replay file later
    /// inputs.push(player.snapshot());
    ///
    /// // game logic...
    /// ```

    fn snapshot(&self) -> u8;
}

#[repr(C)]
#[derive(bytemuck::Pod, Copy, Clone, Zeroable)]
/// Dummy input handler that doesn't take any input.
///
/// Mainly used for debugging and in some edge-cases.
///
/// Could also be used for disconnected players, although that should probably be handled by the game code.
///
/// # Example:
/// ```
/// # fn something_bad_happens_to(thisguy: NetworkInputHandler) -> bool {
/// #     true
/// # }
/// use input_handlers::*;
/// let player1 = NetworkInputHandler::new(7);
/// let player2 = NetworkInputHandler::new(11);
///
/// // player2 disconnects
/// if something_bad_happens_to(player2) {
///     let player2 = EmptyInputHandler {};
/// }
/// // game logic..
/// ```
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

    fn snapshot(&self) -> u8 {
        0
    }
}
#[derive(Clone)]

/// Input handler that takes input from the keyboard.
///
/// Provides a way of checking multiple [ggez keyboard](ggez::input::keyboard) KeyCodes.
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
    /// Creates a new KeyboardInputHandler.
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

    fn snapshot(&self) -> u8 {
        // each bit should represent a different input
        let results = vec![
            self.going_up,
            self.going_down,
            self.going_left,
            self.going_right,
            self.rotating_cw,
            self.rotating_acw,
        ];
        let mut snapshot: u8 = 0;
        for (i, result) in results.iter().enumerate() {
            if *result {
                snapshot += 1 * 2u8.pow(i as u32);
            }
        }
        snapshot
    }
}

/// An Input handler that takes input from a state snapshot, intended to be used in a network.
///
/// This isn't much different from the EmptyInputHandler, but its new() function takes a snapshot of the input state which makes it a tad more useful.
pub struct NetworkInputHandler {
    going_up: bool,
    going_down: bool,
    pub going_left: bool,
    pub going_right: bool,
    pub rotating_cw: bool,
    pub rotating_acw: bool,
}
impl NetworkInputHandler {
    /// Creates a new NetworkInputHandler from a snapshot.
    ///
    /// This should be used once the inputs from other players have been received, in order to simulate a traditional input handler.
    pub fn new(state: u8) -> NetworkInputHandler {
        NetworkInputHandler {
            going_up: state % 2 == 1,
            going_down: state % 4 / 2 == 1,
            going_left: state % 8 / 4 == 1,
            going_right: state % 16 / 8 == 1,
            rotating_cw: state % 32 / 16 == 1,
            rotating_acw: state % 64 / 32 == 1,
        }
    }
}
impl InputHandler for NetworkInputHandler {
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

    fn tick(&mut self, _ctx: &mut Context) -> Result<(), String> {
        Ok(())
    }

    fn snapshot(&self) -> u8 {
        // so each bit represents a different input
        let results = vec![
            self.going_up,
            self.going_down,
            self.going_left,
            self.going_right,
            self.rotating_cw,
            self.rotating_acw,
        ];
        let mut snapshot: u8 = 0;
        for (i, result) in results.iter().enumerate() {
            if *result {
                snapshot += 1 * 2u8.pow(i as u32);
            }
        }
        snapshot
    }
}
