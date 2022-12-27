use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::{Velocity, *};
#[cfg(feature = "discord")]
use discord_game_sdk::Discord;
// mod input_handlers;
use bevy_egui::{egui, EguiContext, EguiPlugin};
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

use bevy_asset::{AssetServer, Handle};
#[cfg(debug_assertions)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    Paused,
}
#[cfg(feature = "discord")]

struct DiscordState<'a>(Discord<'a, ()>);

#[derive(PartialEq, Inspectable, Reflect, Default, Clone, Debug)]

enum RotatingM {
    Clockwise,
    AntiClockwise,
    #[default]
    Neither,
}
#[derive(Component)]
struct Paddle;
#[derive(Component)]
struct Ball;

#[derive(Inspectable, Component)]
struct RotationVelocity(f32);

#[derive(Inspectable, Component)]
struct Acceleration(f32);
#[derive(Inspectable, Component)]
struct RotAcceleration(f32);

#[derive(Inspectable, Component, Reflect, Default)]
#[reflect(Component)]
struct NextStop(f32);

#[derive(Reflect, Default)]
#[reflect(Component)]
#[derive(Inspectable, Component)]
struct Rotating(RotatingM);

#[derive(Bundle)]
struct PaddleBundle {
    flags: ActiveEvents,
    active_collision_types: ActiveCollisionTypes,
    rotation_velocity: RotationVelocity,
    acceleration: Acceleration,
    rot_acceleration: RotAcceleration,
    next_stop: NextStop,
    rotating: Rotating,
    #[bundle]
    sprite: SpriteBundle,
}

// impl PhysicsHooksWithQuery<Paddle> for BallHitIncrease {
//     fn modify_solver_contacts(
//         &self,
//         context: ContactModificationContextView,
//         paddles: &Query<Paddle>,
//     ) {
//         SolverFlags::all()
//         // This is a silly example of contact modifier that does silly things
//         // for illustration purpose:
//         // - Flip all the contact normals.
//         // - Delete the first contact.
//         // - Set the friction coefficients to 0.3
//         // - Set the restitution coefficients to 0.4
//         // - Set the tangent velocities to X * 10.0
//         // *context.normal = -*context.normal;

//         // if !context.solver_contacts.is_empty() {
//         //     context.solver_contacts.swap_remove(0);
//         // }

//         // for solver_contact in &mut *context.solver_contacts {
//         //     solver_contact.friction = 0.3;
//         //     solver_contact.restitution = 0.4;
//         //     solver_contact.tangent_velocity.x = 10.0;
//         // }

//         // // Use the persistent user-data to count the number of times
//         // // contact modification was called for this contact manifold
//         // // since its creation.
//         // *context.user_data += 1;
//         // println!(
//         //     "Contact manifold has been modified {} times since its creation.",
//         //     *context.user_data
//         // );
//         println!(
//             "yo something happened between {} and {}",
//             context.rigid_body1().unwrap().id(),
//             context.rigid_body2().unwrap().id()
//         );
//         if self
//             .paddles
//             .iter()
//             .any(|x| &x.id() == &context.rigid_body2().unwrap().id())
//         {
//             println!("paddle hit");
//             // triple the velocity
//             for solver_contact in &mut *context.raw.solver_contacts {
//                 solver_contact.restitution = 5.0;
//                 solver_contact.tangent_velocity.x = 2.0;
//             }
//         }
//         // println!("{}", context.raw.rigid_body1.unwrap())
//     }
// }

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            asset_folder: if cfg!(target_os = "windows")
                || cfg!(target_os = "linux")
                || cfg!(debug_assertions)
            {
                "assets"
            } else if cfg!(target_os = "macos") {
                "../Resources/assets"
            } else {
                panic!("unsupported os")
            }
            .to_string(),
            ..default()
        }),
    )
    .add_plugin(EguiPlugin)
    .add_plugin(InputManagerPlugin::<Action>::default())
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
    .add_plugin(RapierDebugRenderPlugin::default())
    .add_state(AppState::MainMenu)
    .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game))
    .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(ui))
    .add_startup_system(setup)
    // .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup))
    .add_system_set(SystemSet::on_update(AppState::InGame).with_system(movement))
    .add_system(ball_collision_detection);
    // if debug
    // #[cfg(debug_assertions)]
    // app.add_plugin(EditorPlugin);
    #[cfg(feature = "discord")]
    app.add_startup_system(setup_discord)
        .add_system(discord_update);

    app.run();
}

fn ui(mut egui_context: ResMut<EguiContext>, mut app_state: ResMut<State<AppState>>) {
    egui::Window::new("main menu").show(egui_context.ctx_mut(), |ui| {
        ui.label("hi");
        if ui.button("start game").clicked() {
            app_state.set(AppState::InGame).unwrap();
        }
    });
}

fn setup(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    server: Res<AssetServer>,
) {
    rapier_config.gravity = Vec2::new(0.0, 0.0);
    commands.spawn_bundle(Camera2dBundle::default());
    // server.().unwrap();

    // server_settings.asset_folder =
    // // changes for each os
    // if cfg!(target_os = "windows") || cfg!(target_os = "linux") || cfg!(debug_assertions){
    //     "assets"
    // } else if cfg!(target_os = "macos") {
    //     "../Resources/assets"
    // } else {
    //     panic!("unsupported os")
    // }.to_string();
    let blazma: Handle<Font> = server.load("Blazma/Blazma-Regular.ttf");
    let noto_sans: Handle<Font> =
        server.load("Noto_Sans_Mono/NotoSansMono-VariableFont_wdth,wght.ttf");

    // commands.spawn_bundle(Text2dBundle {
    //     // set font
    //     text: Text::from_section(
    //         "/ppɒŋ/",
    //         TextStyle {
    //             font: noto_sans,
    //             font_size: 40.0,
    //             color: Color::WHITE,
    //         },

    //     ),
    //     transform:
    //     // centre of screen
    //     Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),

    //     ..Default::default()
    // });
}

fn setup_game(mut commands: Commands) {
    let default_map = InputMap::new([
        (KeyCode::A, Action::Left),
        (KeyCode::D, Action::Right),
        (KeyCode::W, Action::Up),
        (KeyCode::S, Action::Down),
        (KeyCode::C, Action::RotateAntiClockwise),
        (KeyCode::V, Action::RotateClockwise),
    ]);
    commands
        .spawn(TransformBundle::from(Transform::from_xyz(-500.0, 0.0, 0.0)))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.0, 1000.0))
        .insert(Restitution::coefficient(0.0));
    commands
        .spawn(TransformBundle::from(Transform::from_xyz(500.0, 0.0, 0.0)))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.0, 1000.0))
        .insert(Restitution::coefficient(0.0));
    commands
        .spawn(TransformBundle::from(Transform::from_xyz(0.0, -250.0, 0.0)))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(1000.0, 0.0))
        .insert(Restitution::coefficient(0.0));
    commands
        .spawn(TransformBundle::from(Transform::from_xyz(0.0, 250.0, 0.0)))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(1000.0, 0.0))
        .insert(Restitution::coefficient(0.0));

    commands
        .spawn(TransformBundle::from(Transform::from_xyz(50.0, 0.0, 0.0)))
        .insert(RigidBody::Dynamic)
        .insert(Ball)
        .insert(Collider::ball(15.0))
        .insert(CollidingEntities::default())
        // add external imp
        .insert(ExternalImpulse::default())
        // .insert(ActiveHooks::MODIFY_SOLVER_CONTACTS)
        .insert(Restitution::coefficient(1.2))
        .insert(Velocity {
            linvel: Vec2::new(1.0, 2.0),
            angvel: 0.4,
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::all())
        .insert_bundle(TransformBundle::from(Transform::from_xyz(400.0, 0.0, 0.0)));

    for _ in 0..1 {
        // let mut commands = world.get_resource_mut::<Commands>().unwrap();
        commands
            .spawn_bundle(PaddleBundle {
                flags: ActiveEvents::COLLISION_EVENTS,
                active_collision_types: ActiveCollisionTypes::default(),
                rotation_velocity: RotationVelocity(0.0),
                acceleration: Acceleration(60.0),
                rot_acceleration: RotAcceleration(0.0005),
                next_stop: NextStop(0.0),
                rotating: Rotating(RotatingM::Neither),
                sprite: SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.5, 0.5, 1.0),
                        custom_size: Some(Vec2::new(30.0, 150.0)),
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
            })
            .insert(RigidBody::Dynamic)
            .insert(Damping {
                linear_damping: 1.7,
                angular_damping: 1.0,
            })
            .insert(Velocity {
                linvel: Vec2::new(0.0, 0.0),
                angvel: 0.0,
            })
            .insert(Collider::cuboid(15.0, 75.0))
            .insert(CollidingEntities::default())
            .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)));
        // world.resource_scope(|_, mut table: Mut<Table>| {
        // table.paddles[i].push(paddle);
        // });
    }
    // commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(BallHitIncrease {
    //     paddles: paddles,
    // })));
}

fn movement(
    mut query: Query<
        (
            &ActionState<Action>,
            &Acceleration,
            &mut NextStop,
            &mut Rotating,
            &mut Velocity,
            &mut Transform,
            &RotAcceleration,
        ),
        With<Paddle>,
    >,
    _time: Res<Time>,
) {
    for (
        action_state,
        acceleration,
        mut next_stop,
        mut rotating,
        mut vel,
        mut transform,
        rot_acceleration,
    ) in query.iter_mut()
    {
        // convert to degrees
        let mut rotation_deg = 180.0 - transform.rotation.to_euler(EulerRot::YXZ).2.to_degrees();
        // if the diff between rotation and the next stop is less than .1, set the rotation to the next stop
        if ((rotating.0 == RotatingM::AntiClockwise || rotating.0 == RotatingM::Clockwise)
            && (rotation_deg - next_stop.0).abs() < 20.0)
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
            next_stop.0 = 90.0 * (rotation_deg / 90.0).round() - 90.0;
            while next_stop.0 < 0.0 {
                next_stop.0 += 360.0;
            }
            next_stop.0 %= 360.0;
        }

        if action_state.pressed(Action::RotateClockwise) {
            rotating.0 = RotatingM::Clockwise;
            // get next 90 degree rotation to the right
            next_stop.0 = 90.0 * (rotation_deg / 90.0).round() + 90.0;
            while next_stop.0 < 0.0 {
                next_stop.0 += 360.0;
            }
            next_stop.0 %= 360.0;
        }
        if (rotation_deg - next_stop.0).abs() < 0.1 {
            transform.rotate_z((next_stop.0 - rotation_deg).to_radians());
            rotation_deg = next_stop.0;
            rotating.0 = RotatingM::Neither;
        }
        // let rotation = {
        //     let mut newrot = rotation;
        //     while newrot < 0.0 {
        //         newrot += 360.0;
        //     }
        //     newrot % 360.0
        // };

        // println!("rotation: {}", rotation);

        // let _rot_accel = 0.8;
        // let (width, _height) = (10.0, 10.0);
        // let mut vel = (0.0, 0.0);
        // let rotation = transform.rotation;
        if action_state.pressed(Action::Right) {
            vel.linvel.x += acceleration.0;
        }
        if action_state.pressed(Action::Left) {
            vel.linvel.x -= acceleration.0;
        }
        if action_state.pressed(Action::Down) {
            vel.linvel.y -= acceleration.0;
        }
        if action_state.pressed(Action::Up) {
            vel.linvel.y += acceleration.0;
        }
        if (rotation_deg - next_stop.0).abs() > 2.0 {
            let displacement_cw = if rotation_deg < next_stop.0 {
                next_stop.0 - rotation_deg
            } else {
                next_stop.0 + (360.0 - rotation_deg)
            };
            let displacement_ccw = if rotation_deg > next_stop.0 {
                next_stop.0 - rotation_deg
            } else {
                next_stop.0 - rotation_deg - 360.0
            };
            println!("{}d {} {}", rotation_deg, displacement_cw, displacement_ccw);

            let displacement = match rotating.0 {
                RotatingM::Neither => {
                    // go for the closest one
                    if (displacement_cw).abs() < (displacement_ccw).abs() {
                        displacement_cw
                    } else {
                        displacement_ccw
                    }
                }
                RotatingM::Clockwise => displacement_cw,
                RotatingM::AntiClockwise => displacement_ccw,
            };

            let mut rotation_velocity = (2.0 * rot_acceleration.0 * displacement.abs()).sqrt();
            if displacement < 0.0 {
                rotation_velocity *= -1.0;
            }
            println!(
                "nxs {} disp {} vel {} d {:?}",
                next_stop.0, displacement, rotation_velocity, rotating.0
            );

            transform.rotate_z(-rotation_velocity);
        } else {
            // just get closer to the next stop
            transform.rotate_z(-(next_stop.0 - rotation_deg).to_radians() / 5.0);
        }
    }
}

fn ball_collision_detection(
    // commands: Commands,
    mut ball_query: Query<(&mut Velocity, Entity), With<Ball>>,
    colliding_entities_query: Query<&CollidingEntities, With<Paddle>>,
) {
    for colliding_entities in colliding_entities_query.iter() {
        for (mut vel, ball_ent) in ball_query.iter_mut() {
            if colliding_entities.contains(ball_ent) {
                println!("collision detected");
                // commands.despawn(ball_ent);
                // commands.spawn((Ball, Transform::from_translation(Vec3::new(400.0, 300.0, 0.0))));
                // make the ball go faster
                // imp.impulse += Vec2::new(100.0, 100.0)
                vel.linvel *= 2.5;
            }
            // println!("vel: {:?}", vel.linvel);
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
    // TODO: add more states
    if discord.is_none() {
        return;
    }
    let mut discord = discord.unwrap();
    discord.0.run_callbacks().unwrap();
    let mut activity = discord_game_sdk::Activity::empty();
    let activity = activity
        // party status
        .with_state("idle")
        .with_large_image_key("logo")
        .with_large_image_key("logo")
        // player status
        .with_details("in the menus");
    discord.0.update_activity(activity, |_, result| {
        if let Err(e) = result {
            println!("Error updating activity: {}", e);
        }
    });
}
