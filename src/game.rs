use bevy::prelude::Commands;
use bevy::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

// mod input_handlers;
// use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::{
    Acceleration,
    Action,
    Ball,
    NextStop,
    Paddle,
    PaddleBundle,
    RotAcceleration,
    Rotating,
    RotatingM,
    RotationVelocity,
};

pub fn setup_game(mut commands: Commands) {
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
        .insert(TransformBundle::from(Transform::from_xyz(400.0, 0.0, 0.0)));

    for _ in 0..1 {
        // let mut commands = world.get_resource_mut::<Commands>().unwrap();
        commands
            .spawn(PaddleBundle {
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
            .insert(InputManagerBundle::<Action> {
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
            .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)));
        // world.resource_scope(|_, mut table: Mut<Table>| {
        // table.paddles[i].push(paddle);
        // });
    }
    // commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(BallHitIncrease {
    //     paddles: paddles,
    // })));
}
pub fn movement(
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
) {
    for (action_state, acceleration, next_stop, rotating, vel, transform, rot_acceleration) in
        &mut query
    {
        paddle_sim(
            transform,
            rotating,
            next_stop,
            action_state,
            vel,
            acceleration,
            rot_acceleration,
        );
    }
}

fn paddle_sim(
    mut transform: Mut<'_, Transform>,
    mut rotating: Mut<'_, Rotating>,
    mut next_stop: Mut<'_, NextStop>,
    action_state: &ActionState<Action>,
    mut vel: Mut<'_, Velocity>,
    acceleration: &Acceleration,
    rot_acceleration: &RotAcceleration,
) {
    // convert to degrees
    let mut rotation_deg = 180.0 - transform.rotation.to_euler(EulerRot::YXZ).2.to_degrees();

    // if the diff between rotation and the next stop is less than .1, set the rotation to the next stop
    if (rotating.0 == RotatingM::AntiClockwise || rotating.0 == RotatingM::Clockwise)
        && (rotation_deg - next_stop.0).abs() < 20.0
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
        let displacement_clockwise = if rotation_deg < next_stop.0 {
            next_stop.0 - rotation_deg
        } else {
            next_stop.0 + (360.0 - rotation_deg)
        };
        let displacement_counterclockwise = if rotation_deg > next_stop.0 {
            next_stop.0 - rotation_deg
        } else {
            next_stop.0 - rotation_deg - 360.0
        };
        println!("{rotation_deg}d {displacement_clockwise} {displacement_counterclockwise}");

        let displacement = match rotating.0 {
            RotatingM::Neither => {
                // go for the closest one
                if (displacement_clockwise).abs() < (displacement_counterclockwise).abs() {
                    displacement_clockwise
                } else {
                    displacement_counterclockwise
                }
            }
            RotatingM::Clockwise => displacement_clockwise,
            RotatingM::AntiClockwise => displacement_counterclockwise,
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
pub fn ball_collision_detection(
    // commands: Commands,
    mut ball_query: Query<(&mut Velocity, Entity), With<Ball>>,
    colliding_entities_query: Query<&CollidingEntities, With<Paddle>>,
) {
    for colliding_entities in colliding_entities_query.iter() {
        for (mut vel, ball_ent) in &mut ball_query {
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
