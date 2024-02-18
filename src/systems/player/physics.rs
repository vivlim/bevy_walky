use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_xpbd_3d::{math::Scalar, prelude::*};

use crate::components::{
    camera::{OrbitCameraTarget, ViewpointMappedInput},
    player::physics::{
        AirSpeed, KinematicCharacterPhysics, PlatformingCharacterControl,
        PlatformingCharacterPhysics, PlatformingCharacterPhysicsAccel, PlatformingCharacterValues,
    },
};

pub fn update_platforming_accel_from_controls(
    mut query: Query<(
        &mut PlatformingCharacterPhysicsAccel,
        &PlatformingCharacterPhysics,
        &mut PlatformingCharacterControl,
        &PlatformingCharacterValues,
    )>,
) {
    for (mut accel, platforming, mut control, values) in query.iter_mut() {
        if control.move_input.length() > 0.0 {
            // Moving in a direction.
            let mut accel_amount = values.acceleration_speed;
            // If moving in a direction opposite the player's ground speed, apply deceleration
            // speed too.
            if platforming.ground_speed.length() > 0.0 {
                let angle_between_input_and_speed =
                    platforming.ground_speed.angle_between(control.move_input);
                if angle_between_input_and_speed.abs() > PI / 2.0 {
                    accel_amount += values.deceleration_speed;
                }
            }
            accel.ground_acceleration = accel_amount * control.move_input;
            accel.ground_friction = 0.0;

            // Consume the input
            control.move_input = Vec2::ZERO;
        } else {
            accel.ground_acceleration = Vec2::ZERO;
            accel.ground_friction = values.friction_speed;
        }

        match (&platforming.air_speed, control.jump_pressed) {
            (AirSpeed::Grounded, true) => {
                accel.air_acceleration = values.jump_speed;
            }
            (AirSpeed::InAir(_), false) => {
                accel.air_acceleration = 0.0; // TODO: this blocks any contribution to air accel
                                              // other than jumping.
            }
            _ => (),
        }
    }
}

pub fn update_platforming_physics(
    mut query: Query<(
        &mut PlatformingCharacterPhysics,
        &mut PlatformingCharacterPhysicsAccel,
        &PlatformingCharacterValues,
    )>,
) {
    for (mut platforming, mut accel, values) in query.iter_mut() {
        if accel.air_acceleration > 0.0 {
            if let AirSpeed::Grounded = platforming.air_speed {
                // Trying to jump, and on the ground.
                platforming.air_speed = AirSpeed::InAir(accel.air_acceleration);
            }
        }

        let ground_accel = match platforming.air_speed {
            AirSpeed::Grounded => accel.ground_acceleration,
            AirSpeed::InAir(_) => accel.ground_acceleration * 0.5,
        };
        //let initial_speed = platforming.ground_speed.length() > values.top_speed;
        // Apply acceleration if we aren't over top speed.
        platforming.ground_speed += accel.ground_acceleration;
        // Actually for now just clamp ground speed to top speed. tune it later.
        platforming.ground_speed = platforming.ground_speed.clamp_length(0.0, values.top_speed);

        match platforming.air_speed {
            AirSpeed::Grounded => {
                // Apply friction
                if (accel.ground_friction > 0.0) {
                    // Get friction vector - start with a unit vector that's facing the direction
                    // of ground speed.
                    let ground_friction_direction = platforming.ground_speed.normalize_or_zero();
                    // flip it
                    let ground_friction_direction = Vec2 {
                        x: ground_friction_direction.x * -1.0,
                        y: ground_friction_direction.y * -1.0,
                    };
                    // multiply it by friction_speed
                    let ground_friction = accel.ground_friction * ground_friction_direction;
                    // add the friction vector to the ground speed.
                    platforming.ground_speed += ground_friction;

                    // if the ground speed is now facing the same direction as the friction vector was,
                    // we should stop.
                    if platforming.ground_speed.normalize_or_zero() == ground_friction_direction {
                        platforming.ground_speed = Vec2::ZERO;
                    }
                }
            }
            AirSpeed::InAir(air_speed) => {
                // Apply acceleration and gravity
                let air_speed = air_speed + accel.air_acceleration + values.gravity;
                // TODO: consider separate top speed for air.
                let air_speed = air_speed.clamp(values.top_speed * -1.0, values.top_speed);

                platforming.air_speed = AirSpeed::InAir(air_speed);
            }
        }

        // accel.ground_acceleration = Vec2::ZERO;
        // accel.ground_friction = 0.0;
        // accel.air_acceleration = 0.0;
    }
}

pub fn update_platforming_kinematic_from_physics(
    mut query: Query<(
        &PlatformingCharacterPhysics,
        &RigidBody,
        &mut LinearVelocity,
        &Rotation,
        &Transform,
    )>,
    mut gizmos: Gizmos,
) {
    for (physics, rb, mut lv, rot, transform) in query.iter_mut() {
        if physics.ground_speed.length() > 1.0 {
            // Map the ground speed into 3d space
            lv.x = physics.ground_speed.x;
            lv.z = physics.ground_speed.y;

            //gizmos.ray(transform.translation, lv., Color::RED);
        } else {
            lv.x = 0.0;
            lv.z = 0.0;
        }

        if let AirSpeed::InAir(air_speed) = physics.air_speed {
            lv.y = air_speed;
        } else {
            lv.y = 0.0;
        }
    }
}

pub fn handle_collisions(
    collisions: Res<Collisions>,
    mut bodies: Query<(
        Option<&RigidBody>,
        &mut Position,
        &Rotation,
        Option<&mut PlatformingCharacterPhysics>,
        Without<AsyncSceneCollider>,
    )>,
    mut scene_bodies: Query<(&RigidBody, &Children, &Handle<Scene>)>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // If the collision didn't happen during this substep, skip the collision
        if !contacts.during_current_substep {
            continue;
        }
        if let Ok(
            [(rb1, mut position1, rotation1, mut maybe_physics1, _), (rb2, mut position2, _, mut maybe_physics2, _)],
        ) = bodies.get_many_mut([contacts.entity1, contacts.entity2])
        {
            for manifold in contacts.manifolds.iter() {
                for contact in manifold.contacts.iter() {
                    if contact.penetration <= Scalar::EPSILON {
                        continue;
                    }

                    if let (Some(rb1), Some(rb2)) = (rb1, rb2) {
                        if rb1.is_kinematic() && !rb2.is_kinematic() {
                            position1.0 -= contact.global_normal1(rotation1) * contact.penetration;

                            if let Some(ref mut physics) = maybe_physics1 {
                                if let AirSpeed::InAir(_) = physics.air_speed {
                                    physics.air_speed = AirSpeed::Grounded;
                                    info!("now grounded (entity 1)");
                                }
                            }
                        } else if rb2.is_kinematic() && !rb1.is_kinematic() {
                            position2.0 += contact.global_normal1(rotation1) * contact.penetration;
                            if let Some(ref mut physics) = maybe_physics2 {
                                if let AirSpeed::InAir(_) = physics.air_speed {
                                    physics.air_speed = AirSpeed::Grounded;
                                    info!("now grounded (entity 2)");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
