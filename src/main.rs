use std::net::SocketAddr;
use std::time::{Duration, Instant};
use ggez_egui::egui::ProgressBar;
use ggez_egui::{egui, EguiBackend};
use ggrs::{
    Config, GGRSEvent, P2PSession, PlayerType, SessionBuilder, SessionState, UdpNonBlockingSocket,
};

use derive_new::new;
use ggez::event::{self, KeyCode};
use ggez::graphics::{self, Color, Rect};
use ggez::input::keyboard;
// use ggez::mint::Point2;
use ggez::{Context, GameResult};
use glam::*;
mod input_handlers;
use input_handlers::{InputHandler, KeyboardInputHandler, EmptyInputHandler};

use crate::input_handlers::NetworkInputHandler;
#[derive(Debug)]
pub struct GGRSConfig;

impl Config for GGRSConfig {
    type Input = u8; // Copy + Clone + PartialEq + bytemuck::Pod + bytemuck::Zeroable
    type State = TableState; // Clone
    type Address = SocketAddr; // Clone + PartialEq + Eq + Hash
}

struct Player {
    addr: PlayerType<SocketAddr>,
    txt: String,
}
struct PpanState {
    table: TableState,
    egui: EguiBackend,
    ui: UiState,
    handlers: Vec<Handler>,
    network_session: Option<P2PSession<GGRSConfig>>,
    sess_builder: SessionBuilder<GGRSConfig>,
    skipping_frames: u32,
    last_update: Instant,
    accumulator: Duration,
    reversed_table: bool,
    players: Vec<Player>,
}

struct UiState {
    debug: DebugState,
}
#[derive(Clone)]
pub struct TableState {
    paddles: Vec<Paddle>,
}
struct DebugState {
    show_debug: bool,
    show_playarea: bool,
}
#[derive(new, Clone)]

struct Paddle {
    id: u16,
    x: f32,
    left: bool,
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
    // input_handler: Box<dyn InputHandler>,
}
struct Handler {
    input_handler: Box<dyn InputHandler>,
    affected_paddles: Vec<u16>,
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

impl PpanState {
    fn new(sess: SessionBuilder<GGRSConfig>, reversed_table: bool) -> GameResult<PpanState> {
        let s = PpanState {
            table: TableState {
                paddles: vec![
                    Paddle::new(
                        0,
                        40.0,
                        true
                        // Box::new(KeyboardInputHandler::new(
                        //     KeyCode::W,
                        //     KeyCode::S,
                        //     KeyCode::A,
                        //     KeyCode::D,
                        //     KeyCode::V,
                        //     KeyCode::C,
                        // )),
                    ),
                    Paddle::new(
                        1,
                        760.0,
                        false
                        // Box::new(KeyboardInputHandler::new(
                        //     KeyCode::I,
                        //     KeyCode::K,
                        //     KeyCode::J,
                        //     KeyCode::L,
                        //     KeyCode::Period,
                        //     KeyCode::Comma,
                        // )),
                    ),
                ],
            },
            egui: EguiBackend::default(),
            ui: UiState {
                debug: DebugState {
                    show_debug: true,
                    show_playarea: false,
                },
            },
            handlers: vec![
                Handler {
                    input_handler: Box::new(KeyboardInputHandler::new(
                        KeyCode::W,
                        KeyCode::S,
                        // reverse L and R if the table is reversed
                        if reversed_table {
                            KeyCode::D
                        } else {
                            KeyCode::A
                        },
                        if reversed_table {
                            KeyCode::A
                        } else {
                            KeyCode::D
                        },
                        if reversed_table {
                            KeyCode::C
                        } else {
                            KeyCode::V
                        },
                        if reversed_table {
                            KeyCode::V
                        } else {
                            KeyCode::C
                        },
                    )),
                    affected_paddles: vec![0],
                },
                Handler {
                    // input_handler: Box::new(KeyboardInputHandler::new(
                    //     KeyCode::I,
                    //     KeyCode::K,
                    //     KeyCode::J,
                    //     KeyCode::L,
                    //     KeyCode::Period,
                    //     KeyCode::Comma,
                    // )),
                    input_handler: Box::new(EmptyInputHandler {}),
                    affected_paddles: vec![1],
                },
            ],
            sess_builder: sess,
            network_session: None,
            skipping_frames: 0,
            last_update: Instant::now(),
            accumulator: Duration::new(0, 0),
            reversed_table,
            players: vec![],
        };
        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for PpanState {
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
                    "p1 x: {} y: {} state: {} up: {}",
                    self.table.paddles[0].x,
                    self.table.paddles[0].y,
                    self.handlers[0].input_handler.snapshot(),
                    self.handlers[0].input_handler.is_up()
                ));

                ui.label(format!(
                    "p2 x: {} y: {} state: {} up: {}",
                    self.table.paddles[1].x,
                    self.table.paddles[1].y,
                    self.handlers[1].input_handler.snapshot(),
                    self.handlers[1].input_handler.is_up()
                ));
                        ui.end_row();
                if self.network_session.is_none()
            {

                for player in self.players.iter_mut() {
                    ui.label(format!(
                        "player type: {}",
                        match player.addr {
                            PlayerType::Local => 
                                "local",
                            PlayerType::Remote(_) => 
                                "remote",
                            PlayerType::Spectator(_) => 
                                "spectator",
                        },
                        // match player.addr {
                        //     PlayerType::Local => 
                        //         "localhost".to_string(),
                        //     PlayerType::Remote(addr) => 
                        //         addr.to_string(),
                        //     PlayerType::Spectator(addr) => 
                        //         addr.to_string(),
                        // }
                    ));
                    let response = ui.add(egui::TextEdit::singleline(&mut player.txt));
                        if (response).changed() {
                            println!("maybe i should write rn");
                            println!("{}", player.txt);
                        }
                        if (response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) ) {
                        let socketaddr: Option<SocketAddr> = match player.txt.parse() {
                            Ok(addr) => Some(addr),
                            Err(_) => None,
                        };

                        if socketaddr.is_none() {
                            println!("invalid address");
                            player.txt = match player.addr {
                                PlayerType::Local => 
                                    "me".to_string(),
                                PlayerType::Remote(addr) => 
                                    addr.to_string(),
                                PlayerType::Spectator(addr) => 
                                    addr.to_string(),
                            };
                            return;
                        } else {
                            println!("new address: {}", socketaddr.unwrap());
                            player.addr = PlayerType::Remote(socketaddr.unwrap());
                        }
                    }
        ui.end_row();
                }

                if ui.button("add player").clicked() {
                    self.players.push(Player {
                        addr: PlayerType::Local,
                        txt: "me".to_string(),
                    });
                } if ui.button("start").clicked() {
                    // TODO: figure out a way to mess with ownership so that this works
                    // self.players.iter_mut().enumerate().for_each(|(i, player)| {
                    //     self.sess_builder = self.sess_builder.add_player(
                    //         match player.addr {
                    //             PlayerType::Local => 
                    //                 PlayerType::Local,
                    //             PlayerType::Remote(addr) => 
                    //                 PlayerType::Remote(addr),
                    //             PlayerType::Spectator(addr) => 
                    //                 PlayerType::Spectator(addr),
                    //         },
                    //         i
                    //     ).unwrap();
                    // });
                    // // self.network_session = Some(2);
                }
            }

                if ui.button("quit").clicked() {
                    std::process::exit(0);
                }
                if self.network_session.is_some() {
                let stats = self
                    .network_session.as_ref().unwrap()
                    .network_stats(self.network_session.as_ref().unwrap().remote_player_handles()[0]);
                match stats {
                    Ok(stats) => {
                        ui.label(format!(
                            "{} kbps, send queue is {}. {}ms ping. we're around {: >2} frames {: >6}, and the other player is {: >2} frames {: >6}.",
                            stats.kbps_sent,
                            stats.send_queue_len,
                            stats.ping,
                            stats.local_frames_behind.abs(),
                            if stats.local_frames_behind > 0 {
                                "behind"
                            } else {
                                "ahead"
                            },
                            stats.remote_frames_behind.abs(),
                            if stats.remote_frames_behind > 0 {
                                "behind"
                            } else {
                                "ahead"
                            },
                        ));
                    }
                    Err(e) => {
                        ui.label(format!("network stats unavailable: {}", e));
                    },
                }
                         } else {
                    ui.label("no network session active");
                }
                ui.checkbox(&mut self.ui.debug.show_playarea, "show playarea");

                // ui.allocate_space(ui.available_size());
            });

        let rot_accel = 0.8;
        // let targetfps = 6000;
        // while ggez::timer::check_update_time(_ctx, targetfps) {

        if keyboard::is_key_pressed(_ctx, KeyCode::Q) {
            // quit the game
            println!("Quitting game!");
            // self.network_session.disconnect_player(self.network_session.local_player_handles()[0]);
            std::process::exit(0);
        }

        if keyboard::is_key_pressed(_ctx, KeyCode::Backslash) {
            self.ui.debug.show_debug = true;
        }

        // let mut delta_time = ggez::timer::delta(_ctx).as_secs_f32();
        if self.network_session.is_none() {
            return Ok(());
        }
        let sess = &mut self.network_session.as_mut().unwrap();
        sess.poll_remote_clients();
        // if sess.frames_ahead() > 0 {
        //     delta_time *= 1.1;
        // }
        // print GGRS events
        for event in sess.events() {
            if let GGRSEvent::WaitRecommendation { skip_frames } = event {
                self.skipping_frames += skip_frames
            }
            println!("Event: {:?}", event);
        }

        // this is to keep ticks between clients synchronized.
        // if a client is ahead, it will run frames slightly slower to allow catching up
        let delta_time = 1. / 60.0;
        // if sess.frames_ahead() > 0 {
        //     delta_time *= 1.1;
        // }

        // get delta time from last iteration and accumulate it
        let delta = Instant::now().duration_since(self.last_update);
        self.accumulator = self.accumulator.saturating_add(delta);
        self.last_update = Instant::now();

        if sess.current_state() != SessionState::Running {
            return Ok(());
        }

        for handle in sess.local_player_handles() {
            self.handlers[0].input_handler.tick(_ctx).unwrap();
            sess.add_local_input(handle, self.handlers[0].input_handler.snapshot())
                .unwrap();
        }

        match sess.advance_frame() {
            Ok(requests) => {
                println!("Request size: {:?}", requests.len());
                requests.iter().for_each(|req| {
                    match req {
                        ggrs::GGRSRequest::LoadGameState { cell, frame } => {
                            println!("REQ: Loading frame {}", frame);
                            self.table = cell.load().unwrap();
                        }
                        ggrs::GGRSRequest::SaveGameState { cell, frame } =>
                        {
                            println!("REQ: Saving frame {}", frame);
                            cell.save(*frame, Some(self.table.clone()), None);
                        },
                        ggrs::GGRSRequest::AdvanceFrame { inputs } => {
                            println!("REQ: Advancing frame");
                            if self.skipping_frames > 0 {
                                self.skipping_frames -= 1;
                                println!("Frame {} skipped: WaitRecommendation", sess.current_frame());
                                return;
                            };

                            for (i, input) in inputs.iter().enumerate() {
                                match input.1 {
                                    ggrs::InputStatus::Predicted => {
                                        println!("status: predicted input on frame {} for player {}", sess.current_frame(), i);
                                    },
                                    ggrs::InputStatus::Disconnected => {
                                        println!("status: disconnected input on frame {} for player {}", sess.current_frame(), i);
                                    }
                                    ggrs::InputStatus::Confirmed => {
                                        println!("status: confirmed input on frame {} for player {}", sess.current_frame(), i);
                                    },
                                }
                                let mut handler = NetworkInputHandler::new(input.0);
                                // if i == 1 {
                                //     // swap the left and right values for the second player
                                //     (handler.going_left, handler.going_right) =
                                //         (handler.going_right, handler.going_left);
                                //     (handler.rotating_acw, handler.rotating_cw) =
                                //         (handler.rotating_cw, handler.rotating_acw);
                                // }

                                //find paddle where id is i
                                let paddle = &mut self.table.paddles.iter_mut().find(|p| p.id as usize == i).unwrap();
                                handler.tick(_ctx).unwrap();
                                // input handling

                                if handler.is_right() {
                                    paddle.velocity_x += paddle.acceleration;
                                    // cap velocity to 1500
                                    if paddle.velocity_x > 1500.0 {
                                        paddle.velocity_x = 1500.0;
                                    }
                                }
                                if handler.is_left() {
                                    paddle.velocity_x -= paddle.acceleration;
                                    // cap velocity to -1500
                                    if paddle.velocity_x < -1500.0 {
                                        paddle.velocity_x = -1500.0;
                                    }
                                }
                                if handler.is_up() {
                                    paddle.velocity_y -= paddle.acceleration;
                                    // cap velocity to -1500
                                    if paddle.velocity_y < -1500.0 {
                                        paddle.velocity_y = -1500.0;
                                    }
                                }
                                if handler.is_down() {
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

                                if handler.is_rotating_acw() {
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

                                if handler.is_rotating_cw() {
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
                                    // lmk if they're both positive or negative
                                    if (first_displacement > 0.0 && second_displacement > 0.0)
                                        || (first_displacement < 0.0 && second_displacement < 0.0)
                                    {
                                        println!("woah there, that's a lot of rotation");
                                    }
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

                                    // if they're both positive, something went wrong. log
                                    if initial_velocity_squared_first > 0.0 && initial_velocity_squared_second > 0.0
                                    {
                                        println!("the fuck?");
                                    }

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
                                    println!(
                                        "so if we're going clockwise, we'll need a velocity of {:?}, but if we're going anticlockwise, we'd need a velocity of {:?}",
                                        init_vel_sq_cw.sqrt(),
                                        -(init_vel_sq_acw.abs().sqrt()),
                                    );
                                    // check nan
                                    if (-init_vel_sq_acw.abs().sqrt()).is_nan() || init_vel_sq_cw.sqrt().is_nan() {
                                        println!("one of the velocities is nan");
                                    }
                                    let initial_velocity_squared = if paddle.going_acw {
                                        println!("we need to go left, so we're using anticlockwise");
                                        init_vel_sq_acw
                                    } else if paddle.going_cw {
                                        println!("we need to go right, so we're using clockwise");
                                        init_vel_sq_cw
                                    } else {
                                        // use the shortest one
                                        println!("we're not aiming anywhere, so we're using the shortest one");
                                        if init_vel_sq_acw.abs() > init_vel_sq_cw.abs() {
                                            println!("using clockwise, {:?}", init_vel_sq_cw);
                                            init_vel_sq_cw
                                        } else {
                                            println!("using anticlockwise, {:?}", init_vel_sq_acw);
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

                                if handler.is_rotating_cw() {
                                    paddle.rotation_velocity += max_rotation_velocity;
                                }
                                if handler.is_rotating_acw() {
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
                            }
                        }
                    }
                });
                ()
            }
            Err(e) => panic!("{:?}", e),
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

        let paddles = self.table.paddles.iter_mut();

        // step 3: draw paddles
        for paddle in paddles {
            let mut paddle = paddle.clone();
            if self.reversed_table {
                // reverse the paddle's x values and rotation
                paddle.x = 800.0 - paddle.x;
                paddle.rotation = 360.0 - paddle.rotation;
            }
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
    let mut local_port = 7001;
    let mut sess = SessionBuilder::<GGRSConfig>::new()
        .with_num_players(2)
        .with_max_prediction_window(20);
    // uncap fps

    let cb = ggez::ContextBuilder::new("pong", "Jabster28").window_mode(
        ggez::conf::WindowMode::default()
            .resizable(true)
            .maximized(true),
    );

    let (mut ctx, event_loop) = cb.build()?;
    // set fullscreen
    ggez::graphics::set_fullscreen(&mut ctx, ggez::conf::FullscreenType::Windowed)?;

    let state: PpanState = PpanState::new(sess, local_port == 7002)?;

    event::run(ctx, event_loop, state)
}
