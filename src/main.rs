#![feature(thread_is_running)]
pub mod compute;

use local_ip_address::local_ip;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::mem::{swap, take, MaybeUninit};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use ggez_egui::egui::ProgressBar;

use bincode::{deserialize, serialize};
use derive_new::new;
use ggez::event::{self, KeyCode};
use ggez::graphics::{self, Color, Rect};
use ggez::input::keyboard;
use ggez_egui::{egui, EguiBackend};
use ggrs::{
    Config,
    GGRSEvent,
    P2PSession,
    PlayerType,
    SessionBuilder,
    SessionState,
    UdpNonBlockingSocket,
};
// use ggez::mint::Point2;
use ggez::{Context, GameResult};
use glam::*;
use input_handlers::{EmptyInputHandler, InputHandler, KeyboardInputHandler};

use input_handlers::NetworkInputHandler;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ShareData {
    hostname: String,
    port: u16,
    ip: String,
    version: String,
}
enum MultiplayerMode {
    LocalClient,
    LocalHost,
    LocalMode,
    Server,
    Manual,
    None,
}
use lazy_static::lazy_static;

use crate::compute::compute;
#[derive(Debug)]
pub struct GGRSConfig;

pub const PORT: u16 = 7101;
lazy_static! {
    pub static ref IPV4: IpAddr = Ipv4Addr::new(224, 0, 0, 47).into();
    pub static ref IPV6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0047).into();
}

// this will be common for all our sockets
fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;

    // we're going to use read timeouts so that we don't hang waiting for packets
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    Ok(socket)
}
fn new_sender(addr: &SocketAddr) -> io::Result<Socket> {
    let socket = new_socket(addr)?;

    if addr.is_ipv4() {
        socket.bind(&SockAddr::from(SocketAddr::new(
            Ipv4Addr::new(0, 0, 0, 0).into(),
            0,
        )))?;
    } else {
        socket.bind(&SockAddr::from(SocketAddr::new(
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(),
            0,
        )))?;
    }

    Ok(socket)
}

fn join_multicast(addr: SocketAddr) -> io::Result<Socket> {
    let ip_addr = addr.ip();

    let socket = new_socket(&addr)?;

    // depending on the IP protocol we have slightly different work
    match ip_addr {
        IpAddr::V4(ref mdns_v4) => {
            // join to the multicast address, with all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
        }
        IpAddr::V6(ref mdns_v6) => {
            // join to the multicast address, with all interfaces (ipv6 uses indexes not addresses)
            socket.join_multicast_v6(mdns_v6, 0)?;
            socket.set_only_v6(true)?;
        }
    };

    // bind us to the socket address.
    socket.bind(&SockAddr::from(addr))?;
    Ok(socket)
}

impl Config for GGRSConfig {
    // Clone
    type Address = SocketAddr;
    type Input = u8;
    // Copy + Clone + PartialEq + bytemuck::Pod + bytemuck::Zeroable
    type State = TableState; // Clone + PartialEq + Eq + Hash
}
struct Player {
    addr: PlayerType<SocketAddr>,
    txt: String,
}
struct PpanState {
    mcast: Arc<AtomicBool>,
    mcastthread: Option<thread::JoinHandle<()>>,
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
    tx: mpsc::Sender<String>,
    mode: MultiplayerMode,
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
pub struct Paddle {
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
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            rx.try_recv().unwrap_or("".to_string());
            // println!("failed + L + ratio");
            // create ppan.log

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open("ppan.log")
                .unwrap();
            loop {
                // println!("attempting to recv");
                writeln!(file, "{}", rx.recv().unwrap()).unwrap();
            }
        });
        let s = PpanState {
            mcast: Arc::new(AtomicBool::new(false)),
            mcastthread: None,
            mode: MultiplayerMode::None,
            tx: tx.clone(),
            table: TableState {
                paddles: vec![
                    Paddle::new(
                        0, 40.0,
                        true, /* Box::new(KeyboardInputHandler::new(
                              *     KeyCode::W,
                              *     KeyCode::S,
                              *     KeyCode::A,
                              *     KeyCode::D,
                              *     KeyCode::V,
                              *     KeyCode::C,
                              * )), */
                    ),
                    Paddle::new(
                        1, 760.0,
                        false, /* Box::new(KeyboardInputHandler::new(
                               *     KeyCode::I,
                               *     KeyCode::K,
                               *     KeyCode::J,
                               *     KeyCode::L,
                               *     KeyCode::Period,
                               *     KeyCode::Comma,
                               * )), */
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
        let log = |msg: &str| match self.tx.send(format!(
            "{}: {}",
            // time
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            msg
        )) {
            Ok(_) => {}
            Err(e) => {
                panic!("Error sending log message: {}", e);
            }
        };
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
                let _ = match &self.mode {
                    MultiplayerMode::Manual => {
                        if ui.button("add player").clicked() {
                            self.players.push(Player {
                                addr: PlayerType::Local,
                                txt: "me".to_string(),
                            });
                        }
                        ui.end_row();
                        if self.network_session.is_none() {
                            for player in self.players.iter_mut() {
                                ui.label(format!(
                                    "player type: {}",
                                    match player.addr {
                                        PlayerType::Local => "local",
                                        PlayerType::Remote(_) => "remote",
                                        PlayerType::Spectator(_) => "spectator",
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
                                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter)
                                {
                                    let socketaddr: Option<SocketAddr> = match player.txt.parse() {
                                        Ok(addr) => Some(addr),
                                        Err(_) => None,
                                    };

                                    if socketaddr.is_none() {
                                        println!("invalid address");
                                        player.txt = match player.addr {
                                            PlayerType::Local => "me".to_string(),
                                            PlayerType::Remote(addr) => addr.to_string(),
                                            PlayerType::Spectator(addr) => addr.to_string(),
                                        };
                                        return;
                                    } else {
                                        println!("new address: {}", socketaddr.unwrap());
                                        player.addr = PlayerType::Remote(socketaddr.unwrap());
                                    }
                                }
                                ui.end_row();
                            }

                            if ui.button("start").clicked() {
                                println!("clicked");
                                // TODO: figure out a way to mess with ownership so that this works
                                self.players.iter_mut().enumerate().for_each(|(i, player)| {
                                    let mut sb: SessionBuilder<GGRSConfig> = SessionBuilder::new();
                                    swap(&mut self.sess_builder, &mut sb);
                                    self.sess_builder = sb
                                        .add_player(
                                            match player.addr {
                                                PlayerType::Local => PlayerType::Local,
                                                PlayerType::Remote(addr) => {
                                                    PlayerType::Remote(addr)
                                                }
                                                PlayerType::Spectator(addr) => {
                                                    PlayerType::Spectator(addr)
                                                }
                                            },
                                            i,
                                        )
                                        .unwrap();
                                });
                                self.network_session = Some(
                                    take(&mut self.sess_builder)
                                        .with_num_players(self.players.len())
                                        .with_sparse_saving_mode(true)
                                        .start_p2p_session(
                                            UdpNonBlockingSocket::bind_to_port(
                                                if self.reversed_table { 7102 } else { 7101 },
                                            )
                                            .unwrap(),
                                        )
                                        .unwrap(),
                                );
                                println!("started");
                            }
                            if ui.button("back").clicked() {
                                self.mode = MultiplayerMode::None;
                                self.network_session = None;
                                self.players = vec![];
                                return;
                            }
                        }
                    }
                    MultiplayerMode::LocalMode => {
                        if ui.button("join").clicked() {
                            self.mode = MultiplayerMode::LocalClient;
                        }
                        if ui.button("host").clicked() {
                            self.mode = MultiplayerMode::LocalHost;
                        }
                    }
                    MultiplayerMode::Server => todo!(),
                    MultiplayerMode::None => {
                        if ui.button("manual").clicked() {
                            self.mode = MultiplayerMode::Manual;
                        }
                        if ui.button("local").clicked() {
                            self.mode = MultiplayerMode::LocalMode;
                        }
                        if ui.button("server").clicked() {
                            self.mode = MultiplayerMode::Server;
                        }
                        ui.end_row();
                    }

                    MultiplayerMode::LocalHost => {
                        // hehe, localhost
                        let name = "main";
                        let addr = *IPV4;
                        let addr = SocketAddr::new(addr, PORT);
                        let multicasting = Arc::clone(&self.mcast);
                        if ui.button("back").clicked() {
                            self.mode = MultiplayerMode::None;
                            self.network_session = None;
                            self.players = vec![];
                            multicasting.store(false, Ordering::Relaxed);
                            return;
                        }

                        if self.mcast.load(Ordering::Relaxed) {
                            return;
                        }
                        multicasting.store(true, Ordering::Relaxed);
                        let thread_multicasting = Arc::clone(&self.mcast);

                        let thread =
                            std::thread::Builder::new()
                                .name(format!("{}", ""))
                                .spawn(move || {
                                    let name = "host";
                                    // socket creation will go here...

                                    // We'll be looping until the client indicates it is done.
                                    let listener = join_multicast(addr).unwrap();
                                    // test receive and response code will go here...
                                    let mut buf: [MaybeUninit<u8>; 64] =
                                        [MaybeUninit::<u8>::uninit(); 64];

                                    while thread_multicasting.load(Ordering::Relaxed) {
                                        // we're assuming failures were timeouts, the client_done loop will stop us
                                        match listener.recv_from(&mut buf) {
                                            Ok((len, remote_addr)) => {
                                                let data = &buf[..len];
                                                unsafe {
                                                    let data = data
                                                        .iter()
                                                        .map(|x| x.assume_init())
                                                        .collect::<Vec<u8>>();
                                                    let data = data.as_slice();

                                                    println!(
                                                        "{}: got data: {} from: {:?}",
                                                        name,
                                                        String::from_utf8_lossy(data),
                                                        remote_addr
                                                    );
                                                }

                                                // create a socket to send the response
                                                let responder =
                                                    new_socket(&remote_addr.as_socket().unwrap())
                                                        .expect("failed to create responder");

                                                // we send the response that was set at the method beginning
                                                responder
                                                    .send_to(name.as_bytes(), &remote_addr)
                                                    .expect("failed to respond");

                                                println!(
                                                    "{}: sent response to: {:?}",
                                                    name, remote_addr
                                                );
                                            }
                                            Err(err) => {
                                                match err.kind() {
                                                    // we're assuming failures were timeouts, the client_done loop will stop us
                                                    std::io::ErrorKind::TimedOut => {}
                                                    std::io::ErrorKind::WouldBlock => {}
                                                    _ => {
                                                        panic!("{}: error: {}", name, err);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    println!("{}: quitting", name);
                                });
                        self.mcastthread = Some(thread.unwrap());

                        println!("{}: joined: {}", name, addr);
                    }

                    MultiplayerMode::LocalClient => {
                        let name = "main";
                        let addr = *IPV4;
                        let addr = SocketAddr::new(addr, PORT);
                        let multicasting = Arc::clone(&self.mcast);
                        if ui.button("back").clicked() {
                            self.mode = MultiplayerMode::None;
                            self.network_session = None;
                            self.players = vec![];
                            multicasting.store(false, Ordering::Relaxed);
                            return;
                        }
                        if ui.button("refresh").clicked() {
                            multicasting.store(false, Ordering::Relaxed);
                            // wait for the thread to stop
                            while multicasting.load(Ordering::Relaxed) {}
                            return;
                        }
                        if self.mcast.load(Ordering::Relaxed) {
                            return;
                        }
                        multicasting.store(true, Ordering::Relaxed);
                        let thread_multicasting = Arc::clone(&self.mcast);

                        let thread =
                            std::thread::Builder::new()
                                .name(format!("{}", ""))
                                .spawn(move || {
                                    let name = "client";
                                    // socket creation will go here...

                                    // We'll be looping until the client indicates it is done.
                                    let listener = join_multicast(addr).unwrap();
                                    // test receive and response code will go here...
                                    let mut buf: [MaybeUninit<u8>; 64] =
                                        [MaybeUninit::<u8>::uninit(); 64];

                                    while thread_multicasting.load(Ordering::Relaxed) {
                                        // we're assuming failures were timeouts, the client_done loop will stop us
                                        match listener.recv_from(&mut buf) {
                                            Ok((len, remote_addr)) => {
                                                let data = &buf[..len];
                                                unsafe {
                                                    let data = data
                                                        .iter()
                                                        .map(|x| x.assume_init())
                                                        .collect::<Vec<u8>>();
                                                    let data = data.as_slice();

                                                    println!(
                                                        "{}: got data: {} from: {:?}",
                                                        name,
                                                        String::from_utf8_lossy(data),
                                                        remote_addr
                                                    );
                                                }
                                            }
                                            Err(err) => {
                                                match err.kind() {
                                                    // we're assuming failures were timeouts, the client_done loop will stop us
                                                    std::io::ErrorKind::TimedOut => {}
                                                    std::io::ErrorKind::WouldBlock => {}
                                                    _ => {
                                                        panic!("{}: error: {}", name, err);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    println!("{}: quitting", name);
                                });
                        self.mcastthread = Some(thread.unwrap());

                        println!("{}: joined: {}", name, addr);
                        let data = ShareData {
                            hostname: hostname::get()
                                .unwrap_or("Player".into())
                                .to_str()
                                .unwrap()
                                .to_string(),
                            port: 7101,
                            ip: local_ip().unwrap().to_string(),
                            version: env!("CURRENT_TAG").to_string(),
                        };
                        let data = bincode::serialize(&data).unwrap();
                        let data = data.as_slice();
                        let message = format!("{}", env!("CURRENT_TAG")).into_bytes();
                        let message = message.as_slice();

                        // create the sending socket
                        let socket = new_sender(&addr).expect("could not create sender!");
                        socket
                            .send_to(data, &SockAddr::from(SocketAddr::new(*IPV4, 7101)))
                            .expect("could not send_to!");
                    }
                };

                if self.network_session.is_some()
                    && self
                        .network_session
                        .as_ref()
                        .unwrap()
                        .remote_player_handles()
                        .len()
                        > 0
                {
                    // println!("we good");
                    let stats = self.network_session.as_ref().unwrap().network_stats(
                        self.network_session
                            .as_ref()
                            .unwrap()
                            .remote_player_handles()[0],
                    );
                    match stats {
                        Ok(stats) => {
                            let txt = format!(
                                "{} kbps, send queue is {} ({} confirmed frame{}). {}ms ping. \
                                 we're around {: >2} frames {: >6}, and the other player ({}) is \
                                 {: >2} frames {: >6}",
                                stats.kbps_sent,
                                stats.send_queue_len,
                                self.network_session.as_ref().unwrap().confirmed_frame(),
                                if self.network_session.as_ref().unwrap().confirmed_frame() == 1 {
                                    ""
                                } else {
                                    "s"
                                },
                                stats.ping,
                                stats.local_frames_behind.abs(),
                                if stats.local_frames_behind > 0 {
                                    "behind"
                                } else {
                                    "ahead"
                                },
                                self.network_session
                                    .as_ref()
                                    .unwrap()
                                    .remote_player_handles()[0],
                                stats.remote_frames_behind.abs(),
                                if stats.remote_frames_behind > 0 {
                                    "behind"
                                } else {
                                    "ahead"
                                },
                            );
                            ui.label(txt.clone());
                            // println!("stats {}", txt);
                        }
                        Err(e) => {
                            ui.label(format!("network stats unavailable: {}", e));
                            println!("unav {}", e);
                        }
                    }
                } else {
                    ui.label("no network session active");
                }
                ui.checkbox(&mut self.ui.debug.show_playarea, "show playarea");
                if ui.button("quit").clicked() {
                    std::process::exit(0);
                }

                // ui.allocate_space(ui.available_size());
            });

        // let targetfps = 6000;

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
        // // print GGRS events
        for event in sess.events() {
            match event {
                GGRSEvent::Synchronizing { addr, total, count } => println!(
                    "Synchronizing with player {} ({} of {})",
                    addr, count, total
                ),
                GGRSEvent::Synchronized { addr } => println!("Synchronized with player {}", addr),
                GGRSEvent::Disconnected { addr } => println!("Player {} disconnected", addr),
                GGRSEvent::NetworkInterrupted {
                    addr,
                    disconnect_timeout,
                } => println!(
                    "Player {} network interrupted, disconnecting in {}ms...",
                    addr, disconnect_timeout
                ),
                GGRSEvent::NetworkResumed { addr } => println!("Player {} network resumed!", addr),
                GGRSEvent::WaitRecommendation { skip_frames } => {
                    self.skipping_frames = skip_frames;
                    println!(
                        "Wait recommended, attempting to skip {} frames to catch up",
                        skip_frames
                    )
                }
            }
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
        let targetfps = 60;
        if !ggez::timer::check_update_time(_ctx, targetfps) {
            log("frame | skipped");
            return Ok(());
        }

        match sess.advance_frame() {
            Ok(requests) => {
                println!("Request size: {:?}", requests.len());
                log(&format!(
                    "req | start | {} --------------------",
                    sess.confirmed_frame()
                ));
                requests.iter().enumerate().for_each(|(i, req)| {
                    log(&format!("req | {}/{} ---", i + 1, requests.len()));
                    match req {
                        ggrs::GGRSRequest::LoadGameState { cell, frame } => {
                            println!("REQ: Loading frame {}", frame);
                            log(&format!("state | load | {}", frame));
                            self.table = cell.load().unwrap();
                        }
                        ggrs::GGRSRequest::SaveGameState { cell, frame } => {
                            println!("REQ: Saving frame {}", frame);
                            log(&format!("state | save | {}", frame));

                            cell.save(*frame, Some(self.table.clone()), None);
                        }
                        ggrs::GGRSRequest::AdvanceFrame { inputs } => {
                            println!("REQ: Advancing frame");
                            if self.skipping_frames > 0 {
                                self.skipping_frames -= 1;
                                println!(
                                    "skipped frame {}, planning to skip {} more",
                                    sess.current_frame(),
                                    self.skipping_frames
                                );
                                log(&format!("req | skip | {}", self.skipping_frames));
                                return;
                            };
                            println!("frame {}", sess.current_frame());
                            log(&format!("req | calc | {}", sess.current_frame()));

                            for (i, input) in inputs.iter().enumerate() {
                                match input.1 {
                                    ggrs::InputStatus::Predicted => {
                                        println!(
                                            "status: predicted input on frame {} for player {}",
                                            sess.current_frame(),
                                            i
                                        );
                                        log(&format!(
                                            "req | predicted | {} | {}",
                                            i,
                                            sess.current_frame(),
                                        ));
                                    }
                                    ggrs::InputStatus::Disconnected => {
                                        println!(
                                            "status: disconnected input on frame {} for player {}",
                                            sess.current_frame(),
                                            i
                                        );
                                        log(&format!(
                                            "req | disconnected | {} | {}",
                                            i,
                                            sess.current_frame(),
                                        ));
                                    }
                                    ggrs::InputStatus::Confirmed => {
                                        println!(
                                            "status: confirmed input on frame {} for player {}",
                                            sess.current_frame(),
                                            i
                                        );
                                        log(&format!(
                                            "req | confirmed | {} | {}",
                                            i,
                                            sess.current_frame(),
                                        ));
                                    }
                                }
                                let mut handler = NetworkInputHandler::new(input.0);
                                // if i == 1 {
                                //     // swap the left and right values for the second player
                                //     (handler.going_left, handler.going_right) =
                                //         (handler.going_right, handler.going_left);
                                //     (handler.rotating_acw, handler.rotating_cw) =
                                //         (handler.rotating_cw, handler.rotating_acw);
                                // }

                                // find paddle where id is i
                                let paddle = &mut self
                                    .table
                                    .paddles
                                    .iter_mut()
                                    .find(|p| p.id as usize == i)
                                    .unwrap();
                                handler.tick(_ctx).unwrap();
                                // input handling
                                compute(&handler, paddle, delta_time)
                            }
                        }
                    }
                });
                log("req | end");
                ()
            }
            Err(e) => match e {
                ggrs::GGRSError::PredictionThreshold => {
                    panic!("too many frames behind");
                }
                ggrs::GGRSError::InvalidRequest { info: _ } => todo!(),
                ggrs::GGRSError::MismatchedChecksum { frame: _ } => todo!(),
                ggrs::GGRSError::NotSynchronized => todo!(),
                ggrs::GGRSError::SpectatorTooFarBehind => todo!(),
                ggrs::GGRSError::SocketCreationFailed => todo!(),
                ggrs::GGRSError::PlayerDisconnected => todo!(),
                ggrs::GGRSError::DecodingError => todo!(),
            },
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
    let local_port = 7101;
    let sess = SessionBuilder::<GGRSConfig>::new()
        // .with_num_players(2)
        .with_max_prediction_window(120);
    // uncap fps

    let cb = ggez::ContextBuilder::new("pong", "Jabster28").window_mode(
        ggez::conf::WindowMode::default()
            .resizable(true)
            .maximized(true),
    );

    let (mut ctx, event_loop) = cb.build()?;
    // set fullscreen
    ggez::graphics::set_fullscreen(&mut ctx, ggez::conf::FullscreenType::Windowed)?;

    let state: PpanState = PpanState::new(sess, local_port == 7102)?;

    event::run(ctx, event_loop, state)
}
#[test]
fn test_ipv4_multicast() {
    assert!(IPV4.is_multicast());
}

#[test]
fn test_ipv6_multicast() {
    assert!(IPV6.is_multicast());
}
