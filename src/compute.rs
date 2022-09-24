use input_handlers::InputHandler;
extern crate test;

macro_rules! debug {
    ($x:expr) => {
        dbg!($x)
    };
}

use crate::Paddle;

#[allow(clippy::too_many_lines)]
pub fn compute(handler: &dyn InputHandler, paddle: &mut Paddle, delta_time: f32) {
    let rot_accel = 0.8;

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
        if (paddle.next_stop - paddle.rotation).abs() < f32::EPSILON {
            paddle.next_stop -= 90.0;
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
        // check if same
        if (paddle.next_stop - paddle.rotation).abs() < f32::EPSILON {
            paddle.next_stop += 90.0;
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
        #[cfg(debug_assertions)]
        if (first_displacement > 0.0 && second_displacement > 0.0)
            || (first_displacement < 0.0 && second_displacement < 0.0)
        {
            debug!("woah there, that's a lot of rotation");
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
        let initial_velocity_squared_first = -(0.0 - 2.0 * rot_accel * first_displacement) % 360.0;
        let initial_velocity_squared_second =
            -(0.0 - 2.0 * rot_accel * second_displacement) % 360.0;

        // if they're both positive, something went wrong. log
        #[cfg(debug_assertions)]
        if initial_velocity_squared_first > 0.0 && initial_velocity_squared_second > 0.0 {
            debug!("the fuck?");
        }

        let init_vel_sq_cw = if initial_velocity_squared_first > initial_velocity_squared_second {
            initial_velocity_squared_first
        } else {
            initial_velocity_squared_second
        };
        let init_vel_sq_acw = if initial_velocity_squared_first > initial_velocity_squared_second {
            initial_velocity_squared_second
        } else {
            initial_velocity_squared_first
        };
        #[cfg(debug_assertions)]
        debug!(format!(
            "so if we're going clockwise, we'll need a velocity of {:?}, but if we're going \
             anticlockwise, we'd need a velocity of {:?}",
            init_vel_sq_cw.sqrt(),
            -(init_vel_sq_acw.abs().sqrt()),
        ));
        // check nan
        #[cfg(debug_assertions)]
        if (-init_vel_sq_acw.abs().sqrt()).is_nan() || init_vel_sq_cw.sqrt().is_nan() {
            debug!("one of the velocities is nan");
        }

        let initial_velocity_squared = if paddle.going_acw {
            #[cfg(debug_assertions)]
            debug!("we need to go left, so we're using anticlockwise");
            init_vel_sq_acw
        } else if paddle.going_cw {
            #[cfg(debug_assertions)]

            debug!("we need to go right, so we're using clockwise");
            init_vel_sq_cw
        } else {
            // use the shortest one
            #[cfg(debug_assertions)]

            debug!("we're not aiming anywhere, so we're using the shortest one");
            if init_vel_sq_acw.abs() > init_vel_sq_cw.abs() {
                #[cfg(debug_assertions)]

                debug!(format!("using clockwise, {:?}", init_vel_sq_cw));
                init_vel_sq_cw
            } else {
                #[cfg(debug_assertions)]

                debug!(format!("using anticlockwise, {:?}", init_vel_sq_acw));
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
    // debug!("initial_velocity: {}", initial_velocity);
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
    // debug!(
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
#[cfg(test)]
mod tests {
    use test::Bencher;

    use input_handlers::NetworkInputHandler;

    use super::*;

    #[test]
    fn rot_right() {
        let mut paddle = Paddle::new(0, 0.0, false);
        let handler = NetworkInputHandler::new(16);
        compute(&handler, &mut paddle, 16.0 / 1000.0);
        let handler = NetworkInputHandler::new(0);

        for _i in 1..100 {
            compute(&handler, &mut paddle, 16.0 / 1000.0);
        }
        assert_eq!(paddle.rotation, 90.0);
    }

    #[test]
    fn rot_left() {
        let mut paddle = Paddle::new(0, 0.0, false);
        let handler = NetworkInputHandler::new(32);
        compute(&handler, &mut paddle, 16.0 / 1000.0);
        let handler = NetworkInputHandler::new(0);

        for _i in 1..100 {
            compute(&handler, &mut paddle, 16.0 / 1000.0);
        }
        assert_eq!(paddle.rotation, 270.0);
    }

    #[bench]
    fn bench_rotations(b: &mut Bencher) {
        let handler = NetworkInputHandler::new(16);
        let mut paddle = Paddle::new(0, 0.0, false);
        let delta = 16.0 / 1000.0;
        b.iter(|| {
            compute(&handler, &mut paddle, delta);
        });
    }
}
