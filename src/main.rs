use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
#[cfg(feature = "discord")]
use discord_game_sdk::Discord;
// mod input_handlers;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Left,
    Right,
    Up,
    Down,
    RotateClockwise,
    RotateAntiClockwise,
}

use bevy_asset::{AssetServer, AssetServerSettings, Handle};
#[cfg(debug_assertions)]
use bevy_editor_pls::prelude::*;
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Bindings {
    Hotkeys(HotkeysInput),
    Movement(MovementInput),
    Camera(CameraInput),
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum MovementInput {
    Forward,
    Right,
    Up,
}
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum CameraInput {
    Yaw,
    Pitch,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum HotkeysInput {
    Test,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    Paused,
}
#[cfg(feature = "discord")]

struct DiscordState<'a>(Discord<'a, ()>);

#[derive(PartialEq, Inspectable)]

enum RotatingM {
    Clockwise,
    AntiClockwise,
    Neither,
}
#[derive(Component)]
struct Paddle;

#[derive(Inspectable, Component)]
struct RotationVelocity(f32);

#[derive(Inspectable, Component)]
struct Velocity(f32, f32);

#[derive(Inspectable, Component)]
struct Acceleration(f32);

#[derive(Inspectable, Component)]
struct NextStop(f32);

#[derive(Inspectable, Component)]
struct Rotating(RotatingM);

#[derive(Bundle)]
struct PaddleBundle {
    rotation_velocity: RotationVelocity,
    velocity: Velocity,
    acceleration: Acceleration,
    next_stop: NextStop,
    rotating: Rotating,
    #[bundle]
    sprite: SpriteBundle,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_state(AppState::InGame)
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game))
        .add_system_set(SystemSet::on_update(AppState::InGame).with_system(movement));
    // .add_plugin(WorldInspectorPlugin::new())
    #[cfg(feature = "discord")]
    app.add_startup_system(setup_discord.exclusive_system())
        .add_system(discord_update);
    #[cfg(debug_assertions)]
    app.add_plugin(EditorPlugin);

    app.run();
}
fn setup_game(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
    server: Res<AssetServer>,
    mut server_settings: ResMut<AssetServerSettings>,
) {
    let default_map = InputMap::new([
        (KeyCode::A, Action::Left),
        (KeyCode::D, Action::Right),
        (KeyCode::W, Action::Up),
        (KeyCode::S, Action::Down),
        (KeyCode::C, Action::RotateAntiClockwise),
        (KeyCode::V, Action::RotateClockwise),
    ]);
    server_settings.asset_folder =
    // chanegs for each os
    if cfg!(target_os = "windows") || cfg!(target_os = "linux") || cfg!(debug_asdsertions){
        "assets"
    } else if cfg!(target_os = "macos") {
        "../Resources/assets"
    } else {
        panic!("unsupported os")
    }.to_string();
    let font: Handle<Font> = server.load("Blazma/Blazma-Regular.ttf");

    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(Text2dBundle {
        // set font
        text: Text::from_section(
            "Hello World",
            TextStyle {
                font,
                font_size: 40.0,
                color: Color::WHITE,
            },

        ),
        transform:
        // centre of screen
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),

        ..Default::default()
    });
    for _ in 0..1 {
        // let mut commands = world.get_resource_mut::<Commands>().unwrap();
        commands
            .spawn_bundle(PaddleBundle {
                rotation_velocity: RotationVelocity(0.0),
                velocity: Velocity(0.0, 0.0),
                acceleration: Acceleration(10.0),
                next_stop: NextStop(0.0),
                rotating: Rotating(RotatingM::Neither),
                sprite: SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.5, 0.5, 1.0),
                        custom_size: Some(Vec2::new(10.0, 50.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            })
            .insert(Paddle)
            .insert_bundle(InputManagerBundle::<Action> {
                // Stores "which actions are currently pressed"
                action_state: ActionState::default(),
                // Describes how to convert from player inputs into those actions
                input_map: default_map.clone(),
            });
        // world.resource_scope(|_, mut table: Mut<Table>| {
        // table.paddles[i].push(paddle);
        // });
    }
}

fn movement(
    mut query: Query<
        (
            &ActionState<Action>,
            &mut Transform,
            &mut Velocity,
            &mut Acceleration,
            &mut NextStop,
            &mut Rotating,
            &mut RotationVelocity,
        ),
        With<Paddle>,
    >,
    _time: Res<Time>,
) {
    for (
        action_state,
        mut transform,
        mut velocity,
        acceleration,
        mut next_stop,
        mut rotating,
        mut rotation_velocity,
    ) in query.iter_mut()
    {
        let delta_time = 16.0 / 1000.0;
        let friction = 0.1;
        // cpnvert to degrees
        let rotation = transform.rotation.xyz().z;
        let rotation = {
            let mut newrot = rotation;
            while newrot < 0.0 {
                newrot += 360.0;
            }
            newrot % 360.0
        };

        println!("rotation: {}", rotation);

        let rot_accel = 0.8;
        let (width, _height) = (10.0, 10.0);

        if action_state.pressed(Action::Right) {
            velocity.0 += acceleration.0;
            // cap velocity to 1500
            if velocity.0 > 1500.0 {
                velocity.0 = 1500.0;
            }
        }
        if action_state.pressed(Action::Left) {
            velocity.0 -= acceleration.0;
            // cap velocity to -1500
            if velocity.0 < -1500.0 {
                velocity.0 = -1500.0;
            }
        }
        if action_state.pressed(Action::Down) {
            velocity.1 -= acceleration.0;
            // cap velocity to -1500
            if velocity.1 < -1500.0 {
                velocity.1 = -1500.0;
            }
        }
        if action_state.pressed(Action::Up) {
            velocity.1 += acceleration.0;
            // cap velocity to 1500
            if velocity.1 > 1500.0 {
                velocity.1 = 1500.0;
            }
        }

        if rotating.0 == RotatingM::AntiClockwise
            && (rotation * 180.0 / std::f32::consts::PI - next_stop.0).abs() < 30.0
        {
            rotating.0 = RotatingM::Neither;
        } else if rotating.0 == RotatingM::Clockwise
            && (rotation * 180.0 / std::f32::consts::PI - next_stop.0).abs() < 30.0
        {
            rotating.0 = RotatingM::Neither;
        }

        if rotating.0 == RotatingM::Clockwise && rotating.0 == RotatingM::AntiClockwise {
            // only keep cw
            rotating.0 = RotatingM::Clockwise;
        }

        if action_state.pressed(Action::RotateAntiClockwise) {
            rotating.0 = RotatingM::AntiClockwise;
            // get next 90 degree rotation to the left
            next_stop.0 =
                (90.0 * ((rotation * 180.0 / std::f32::consts::PI) / 90.0).floor()) as f32;
            if (next_stop.0 - (rotation * 180.0 / std::f32::consts::PI)).abs() < f32::EPSILON {
                next_stop.0 -= 90.0;
            }
            while next_stop.0 < 0.0 {
                next_stop.0 += 360.0;
            }
            next_stop.0 %= 360.0;
        }

        if action_state.pressed(Action::RotateClockwise) {
            rotating.0 = RotatingM::Clockwise;

            // get next 90 degree rotation to the right
            next_stop.0 = (90.0 * (rotation * 180.0 / std::f32::consts::PI / 90.0).ceil()) as f32;
            // check if same
            if (next_stop.0 - rotation * 180.0 / std::f32::consts::PI).abs() < f32::EPSILON {
                next_stop.0 += 90.0;
            }
            next_stop.0 %= 360.0;
        }

        // #[cfg(debug_assertions)]
        println!(
            "next stop is {}, i'm at {} so i wanna set velocity to {}",
            next_stop.0, rotation, rotation_velocity.0
        );
        transform.rotate_z(rotation_velocity.0 * std::f32::consts::PI / 180.0);

        // if the paddle's rotating right, its rotational velocity should also decrease as it reaches the next 90 degree mark
        // println!(
        //     "x: {: >4} y: {: >4} gl: {: >5} gr: {: >5} rot: {: >4} rotvel: {: >4} nxtstop: {: >4} fps: {: >4}",
        //     // pad start to 3 chars
        //     x.round(),
        //     y.round(),
        //     rotating.0 == RotatingM::AntiClockwise,
        //     rotating.0 == RotatingM::Clockwise,
        //     rotation.round(),
        //     rotation_velocity.round(),
        //     next_stop.round(),
        //     ggez::timer::fps(_ctx).round()
        // );!!

        // speed calculations
        transform.translation.x += velocity.0 * delta_time;
        velocity.0 *= 1.0 - friction;

        transform.translation.y += velocity.1 * delta_time;
        velocity.1 *= 1.0 - friction;

        // ensure transform.translation.0 is in bounds, and reset velocittransform.translation.1if it is
        if transform.translation.x < width / 2.0 {
            transform.translation.x = width / 2.0;
            velocity.0 = 0.0;
        } else if transform.translation.x > 800.0 - width / 2.0 {
            transform.translation.x = 800.0 - width / 2.0;
            velocity.0 = 0.0;
        }
        // 0 >transform.translation.y> 600
        if transform.translation.y < 0.0 {
            transform.translation.y = 0.0;
            velocity.1 = 0.0;
        } else if transform.translation.y > 600.0 {
            transform.translation.y = 600.0;
            velocity.1 = 0.0;
        }
    }
}
#[cfg(feature = "discord")]

fn setup_discord(world: &mut World) {
    let discord = Discord::with_create_flags(
        1_023_380_299_821_875_210,
        discord_game_sdk::CreateFlags::NoRequireDiscord,
    );
    match discord {
        Ok(discord) => {
            world.insert_non_send_resource(DiscordState(discord));
        }
        Err(_e) => {
            println!("warning: discord setup failed...");
        }
    }
}

#[cfg(feature = "discord")]

fn discord_update(discord: Option<NonSendMut<DiscordState>>) {
    if discord.is_none() {
        return;
    }
    let mut discord = discord.unwrap();
    discord.0.run_callbacks().unwrap();
    let mut activity = discord_game_sdk::Activity::empty();
    let activity = activity
        // party status
        .with_state("idle")
        // player status
        .with_details("in the menus");
    discord.0.update_activity(activity, |_, result| {
        if let Err(e) = result {
            println!("Error updating activity: {}", e);
        }
    });
}
