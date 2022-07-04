#![feature(unboxed_closures)]
use ggez_egui::egui::ProgressBar;
use ggez_egui::{egui, EguiBackend};

use derive_new::new;
use ggez::event::{self, KeyCode};
use ggez::graphics::{self, Color, Rect};
use ggez::input::keyboard;
// use ggez::mint::Point2;
use ggez::{Context, GameResult};
use glam::*;
mod input_handlers;
use input_handlers::{InputHandler, KeyboardInputHandler};

struct GameState {
    left_paddles: Vec<Paddle>,
    right_paddles: Vec<Paddle>,
    egui: EguiBackend,
    ui: UiState,
}
struct UiState {
    debug: DebugState,
}
struct DebugState {
    show_debug: bool,
    show_playarea: bool,
}
#[derive(new)]
struct Paddle {
    x: f32,
    #[new(value = "300.0")]
    y: f32,
    #[new(value = "20.0")]
    width: f32,
    #[new(value = "100.0")]
    height: f32,
    // #[new(default)]
    // texture_id: u16,

    // rotation in radians
    #[new(default)]
    rotation: f32,
    #[new(default)]
    rotation_velocity: f32,
    #[new(default)]
    velocity_x: f32,
    #[new(default)]
    velocity_y: f32,
    #[new(value = "0.1")]
    friction: f32,
    #[new(value = "70.0")]
    acceleration: f32,
    #[new(default)]
    next_stop: f32,
    #[new(value = "false")]
    going_acw: bool,
    #[new(value = "false")]
    going_cw: bool,
    input_handler: Box<dyn InputHandler>,
}

// impl Paddle {
//     fn new(left: bool) -> Paddle {
//         Paddle {
//             x: if left { 20.0 } else { 760.0 },
//             y: 300.0,
//             width: 20.0,
//             height: 100.0,
//             texture_id: 0,
//             rotation: 0.0,
//             rotation_velocity: 0.0,
//             velocity_x: 0.0,
//             velocity_y: 0.0,
//             friction: 0.1,
//             acceleration: 70.0,
//             next_stop: 0.0,
//             going_acw: false,
//             going_cw: false,
//         }
//     }
// }

impl GameState {
    fn new() -> GameResult<GameState> {
        let s = GameState {
            left_paddles: vec![Paddle::new(
                40.0,
                Box::new(KeyboardInputHandler::new(
                    KeyCode::W,
                    KeyCode::S,
                    KeyCode::A,
                    KeyCode::D,
                    KeyCode::V,
                    KeyCode::C,
                )),
            )],
            right_paddles: vec![Paddle::new(
                760.0,
                Box::new(KeyboardInputHandler::new(
                    KeyCode::I,
                    KeyCode::K,
                    KeyCode::J,
                    KeyCode::L,
                    KeyCode::Period,
                    KeyCode::Comma,
                )),
            )],
            egui: EguiBackend::default(),
            ui: UiState {
                debug: DebugState {
                    show_debug: true,
                    show_playarea: false,
                },
            },
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // update window size
        let (width, height) = graphics::drawable_size(_ctx);
        graphics::set_screen_coordinates(
            _ctx,
            graphics::Rect::new(0.0, 0.0, width as f32, height as f32),
        )?;

        let egui_ctx = self.egui.ctx();
        egui::Window::new("Debug Menu")
            .open(&mut self.ui.debug.show_debug)
            .show(&egui_ctx, |ui| {
                let fps = ProgressBar::new((ggez::timer::fps(_ctx) / 60.0) as f32)
                    .text(format!("{} FPS", ggez::timer::fps(_ctx).round()));
                ui.add(fps);

                ui.label(format!(
                    "x: {} y: {}",
                    self.left_paddles[0].x, self.left_paddles[0].y
                ));

                if ui.button("quit").clicked() {
                    std::process::exit(0);
                }

                ui.checkbox(&mut self.ui.debug.show_playarea, "show playarea");

                ui.allocate_space(ui.available_size());
            });

        let rot_accel = 0.8;
        // let targetfps = 6000;
        // while ggez::timer::check_update_time(_ctx, targetfps) {

        if keyboard::is_key_pressed(_ctx, KeyCode::Q) {
            // quit the game
            println!("Quitting game!");
            std::process::exit(0);
        }

        if keyboard::is_key_pressed(_ctx, KeyCode::Backslash) {
            self.ui.debug.show_debug = true;
        }
        let delta_time = ggez::timer::delta(_ctx).as_secs_f32();

        // update paddles
        let paddles = self
            .left_paddles
            .iter_mut()
            .chain(self.right_paddles.iter_mut());

        for paddle in paddles {
            paddle.input_handler.tick(_ctx).unwrap();
            // input handling

            if paddle.input_handler.is_right() {
                paddle.velocity_x += paddle.acceleration;
                // cap velocity to 1500
                if paddle.velocity_x > 1500.0 {
                    paddle.velocity_x = 1500.0;
                }
            }
            if paddle.input_handler.is_left() {
                paddle.velocity_x -= paddle.acceleration;
                // cap velocity to -1500
                if paddle.velocity_x < -1500.0 {
                    paddle.velocity_x = -1500.0;
                }
            }
            if paddle.input_handler.is_up() {
                paddle.velocity_y -= paddle.acceleration;
                // cap velocity to -1500
                if paddle.velocity_y < -1500.0 {
                    paddle.velocity_y = -1500.0;
                }
            }
            if paddle.input_handler.is_down() {
                paddle.velocity_y += paddle.acceleration;
                // cap velocity to 1500
                if paddle.velocity_y > 1500.0 {
                    paddle.velocity_y = 1500.0;
                }
            }

            if paddle.going_acw && (paddle.rotation - paddle.next_stop).abs() < 30.0 {
                paddle.going_acw = false;
            } else if paddle.going_cw && (paddle.rotation - paddle.next_stop).abs() < 30.0 {
                paddle.going_cw = false;
            }
            if paddle.going_cw && paddle.going_acw {
                // only keep cw
                paddle.going_acw = false;
            }

            if paddle.input_handler.is_rotating_acw() {
                paddle.going_acw = true;
                // get next 90 degree rotation to the left
                paddle.next_stop = (90.0 * (paddle.rotation / 90.0).floor()) as f32;
                if paddle.next_stop == paddle.rotation {
                    paddle.next_stop -= 90.0
                }
                while paddle.next_stop < 0.0 {
                    paddle.next_stop += 360.0;
                }
                paddle.next_stop %= 360.0;
            }

            if paddle.input_handler.is_rotating_cw() {
                paddle.going_cw = true;
                // get next 90 degree rotation to the right
                paddle.next_stop = (90.0 * (paddle.rotation / 90.0).ceil()) as f32;
                if paddle.next_stop == paddle.rotation {
                    paddle.next_stop += 90.0
                }
                paddle.next_stop %= 360.0;
            }

            // calculations
            let mut initial_velocity = 0.0;

            if (paddle.next_stop - paddle.rotation).abs() > 0.5 {
                while paddle.rotation < 0.0 {
                    paddle.rotation += 360.0;
                }
                paddle.rotation %= 360.0;

                // first, calculate clockwise and anticlockwise rotations
                let mut first_displacement = paddle.next_stop - paddle.rotation;
                let mut second_displacement = paddle.next_stop - paddle.rotation - 180.0;
                // if our current rotation is greater than the next stop, we need to add 360 to both displacements
                if first_displacement < 0.0 && second_displacement < 0.0 {
                    while first_displacement < 0.0 && second_displacement < 0.0 {
                        first_displacement += 180.0;
                        second_displacement += 180.0;
                    }
                }
                if first_displacement > 0.0 && second_displacement > 0.0 {
                    while first_displacement > 0.0 && second_displacement > 0.0 {
                        first_displacement -= 180.0;
                        second_displacement -= 180.0;
                    }
                }
                // cw will always be positive, acw will always be negative

                //  if the paddle's attempted rotation is left, its rotational velocity should decrease as it reaches the next 90 degree mark
                // we'll use v^2 = u^2 + 2as to figure out the "initial" velocity, since we know the final velocity is 0 and acceleration is 10, and the displacement is just the rotation's distance from the nearest 90 degree mark
                // we'll calculate two velocities, one for the rotation to the left and one for the rotation to the right
                // and we'll use the one that is shortest
                let initial_velocity_squared_first =
                    -(0.0 - 2.0 * rot_accel * first_displacement) % 360.0;
                let initial_velocity_squared_second =
                    -(0.0 - 2.0 * rot_accel * second_displacement) % 360.0;

                let init_vel_sq_cw =
                    if initial_velocity_squared_first > initial_velocity_squared_second {
                        initial_velocity_squared_first
                    } else {
                        initial_velocity_squared_second
                    };
                let init_vel_sq_acw =
                    if initial_velocity_squared_first > initial_velocity_squared_second {
                        initial_velocity_squared_second
                    } else {
                        initial_velocity_squared_first
                    };
                let initial_velocity_squared = if paddle.going_acw {
                    init_vel_sq_acw
                } else if paddle.going_cw {
                    init_vel_sq_cw
                } else {
                    // use the shortest one
                    if init_vel_sq_acw.abs() > init_vel_sq_cw.abs() {
                        init_vel_sq_cw
                    } else {
                        init_vel_sq_acw
                    }
                };

                initial_velocity = if initial_velocity_squared < 0.0 {
                    -(initial_velocity_squared.abs().sqrt())
                } else {
                    initial_velocity_squared.sqrt()
                };
            } else {
                // if we're really close, just silently snap to the next stop
                // should save us a couple cpu cycles
                paddle.rotation = paddle.next_stop;
            }
            // println!("initial_velocity: {}", initial_velocity);
            paddle.rotation_velocity = initial_velocity;

            // cap rotation velocity
            let max_rotation_velocity = 8.0;

            if paddle.input_handler.is_rotating_cw() {
                paddle.rotation_velocity += max_rotation_velocity;
            }
            if paddle.input_handler.is_rotating_acw() {
                paddle.rotation_velocity -= max_rotation_velocity;
            }

            if paddle.rotation_velocity > max_rotation_velocity {
                paddle.rotation_velocity = max_rotation_velocity;
            } else if paddle.rotation_velocity < -max_rotation_velocity {
                paddle.rotation_velocity = -max_rotation_velocity;
            }

            paddle.rotation += paddle.rotation_velocity;

            // if the paddle's rotating right, its rotational velocity should also decrease as it reaches the next 90 degree mark
            // println!(
            //     "x: {: >4} y: {: >4} gl: {: >5} gr: {: >5} rot: {: >4} rotvel: {: >4} nxtstop: {: >4} fps: {: >4}",
            //     // pad start to 3 chars
            //     paddle.x.round(),
            //     paddle.y.round(),
            //     paddle.going_acw,
            //     paddle.going_cw,
            //     paddle.rotation.round(),
            //     paddle.rotation_velocity.round(),
            //     paddle.next_stop.round(),
            //     ggez::timer::fps(_ctx).round()
            // );!!

            // speed calculations
            paddle.x += paddle.velocity_x * delta_time;
            paddle.velocity_x *= 1.0 - paddle.friction;

            paddle.y += paddle.velocity_y * delta_time;
            paddle.velocity_y *= 1.0 - paddle.friction;

            // ensure x is in bounds, and reset velocity if it is
            if paddle.x < paddle.width / 2.0 {
                paddle.x = paddle.width / 2.0;
                paddle.velocity_x = 0.0;
            } else if paddle.x > 800.0 - paddle.width / 2.0 {
                paddle.x = 800.0 - paddle.width / 2.0;
                paddle.velocity_x = 0.0;
            }
            // 0 > y > 600
            if paddle.y < 0.0 {
                paddle.y = 0.0;
                paddle.velocity_y = 0.0;
            } else if paddle.y > 600.0 {
                paddle.y = 600.0;
                paddle.velocity_y = 0.0;
            }
            // }
        }
        // calculate delta time, but factor in the framerate

        // for loop with both left and right paddles

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (width, height) = graphics::drawable_size(ctx);

        graphics::clear(ctx, [0.1, 0.2, 0.0, 1.0].into());

        // we have to scale everything by the screen's size over 800x600, since that's what the game's expecting

        // step 1: check if we need letterboxing
        let real_ratio = width as f32 / height as f32;
        let dummy_ratio = 800.0 / 600.0;
        let extra_width: f32;
        let extra_height: f32;
        if real_ratio > dummy_ratio {
            // we need letterboxing
            extra_height = 0.0;
            extra_width = width - (height * dummy_ratio);
        } else if real_ratio < dummy_ratio {
            // we need letterboxing
            extra_width = 0.0;
            extra_height = height - (width / dummy_ratio);
        } else {
            // we don't need letterboxing
            extra_width = 0.0;
            extra_height = 0.0;
        }
        let playarea_width = width - extra_width;
        let playarea_height = height - extra_height;

        // step 2: draw the playarea (if we need it)
        if self.ui.debug.show_playarea {
            // visualise actual play area
            let playarearect = graphics::Rect::new(
                extra_width / 2.0,
                extra_height / 2.0,
                playarea_width,
                playarea_height,
            );

            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                playarearect,
                // need to convert [0.1, 0.2, 0.3, 1.0] into ints by * by 255
                Color::from_rgba(
                    (0.1_f32 * 255.0_f32).round() as u8,
                    (0.2_f32 * 255.0_f32).round() as u8,
                    (0.3_f32 * 255.0_f32).round() as u8,
                    255,
                ),
            )?;
            graphics::draw(ctx, &rect, graphics::DrawParam::default())?;
        }

        let paddles = self
            .left_paddles
            .iter_mut()
            .chain(self.right_paddles.iter_mut());

        // step 3: draw paddles
        for paddle in paddles {
            let rectangle = Rect {
                x: 0.0,
                y: 0.0,
                w: paddle.width * playarea_width / 800.0,
                h: paddle.height * playarea_height / 600.0,
            };

            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                rectangle,
                Color::WHITE,
            )?;
            graphics::draw(
                ctx,
                &rect,
                graphics::DrawParam::new()
                    .dest(Vec2::new(
                        extra_width / 2.0 + paddle.x * playarea_width / 800.0,
                        extra_height / 2.0 + paddle.y * playarea_height / 600.0,
                    ))
                    .rotation(paddle.rotation * (std::f64::consts::PI as f32) / 180.0)
                    .offset(Vec2::new(
                        (paddle.width * playarea_width / 800.0) / 2.0,
                        (paddle.height * playarea_height / 600.0) / 2.0,
                    )),
            )?;
        }
        graphics::draw(ctx, &self.egui, graphics::DrawParam::default())?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.egui.input.mouse_button_down_event(button);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.egui.input.mouse_button_up_event(button);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.egui.input.mouse_motion_event(x, y);
    }
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: event::KeyMods,
        _repeat: bool,
    ) {
        self.egui.input.key_down_event(keycode, _keymods)
    }
    fn text_input_event(&mut self, _ctx: &mut Context, character: char) {
        self.egui.input.text_input_event(character);
    }
}

fn main() -> GameResult {
    // uncap fps

    let cb = ggez::ContextBuilder::new("pong", "Jabster28").window_mode(
        ggez::conf::WindowMode::default()
            .resizable(true)
            .maximized(true),
    );

    let (mut ctx, event_loop) = cb.build()?;
    // set fullscreen
    ggez::graphics::set_fullscreen(&mut ctx, ggez::conf::FullscreenType::Windowed)?;

    let state: GameState = GameState::new()?;

    event::run(ctx, event_loop, state)
}
