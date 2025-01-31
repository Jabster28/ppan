use bevy::prelude::Commands;
use bevy::prelude::*;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

// use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::{
    calc,
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
                acceleration: calc::ACCELERATION,
                rot_acceleration: calc::ROT_ACCELERATION,
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
        calc::paddle_sim(
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
