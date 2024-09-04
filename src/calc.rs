use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;
use leafwing_input_manager::prelude::*;

use crate::{Acceleration, Action, NextStop, RotAcceleration, Rotating, RotatingM};

pub fn paddle_sim(
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
        next_stop.0 = 90.0f32.mul_add((rotation_deg / 90.0).round(), -90.0);
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
