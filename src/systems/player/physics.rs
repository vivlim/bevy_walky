use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_xpbd_3d::{math::Scalar, prelude::*};

use crate::components::{
    camera::{OrbitCameraTarget, ViewpointMappedInput},
    player::{
        physics::{
            AirSpeed, KinematicCharacterPhysics, PlatformingCharacterAnimationFlags,
            PlatformingCharacterControl, PlatformingCharacterPhysics,
            PlatformingCharacterPhysicsAccel, PlatformingCharacterValues,
        },
        sensors::{CharacterSensor, CharacterSensorArray},
    },
};

pub fn update_platforming_accel_from_controls(
    mut query: Query<(
        &mut PlatformingCharacterPhysicsAccel,
        &PlatformingCharacterPhysics,
        &mut PlatformingCharacterControl,
        &PlatformingCharacterValues,
        &mut PlatformingCharacterAnimationFlags,
    )>,
) {
    for (mut accel, platforming, mut control, values, mut animation_flags) in query.iter_mut() {
        if control.move_input.length() > 0.0 {
            control.facing_2d = control.move_input;
            // Moving in a direction.
            let mut accel_amount = values.acceleration_speed;
            // If moving in a direction opposite the player's ground speed, apply deceleration
            // speed too.
            if platforming.ground_speed.length() > 0.0 {
                let angle_between_input_and_speed =
                    platforming.ground_speed.angle_between(control.move_input);
                if angle_between_input_and_speed.abs() > PI / 2.0 {
                    accel_amount += values.deceleration_speed;
                    animation_flags.skidding = true;
                } else {
                    animation_flags.skidding = false;
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

pub fn update_floor(
    mut characters: Query<(
        &CharacterSensorArray,
        &PlatformingCharacterControl,
        &mut Transform,
        &GlobalTransform,
    )>,
    targets: Query<(
        &GlobalTransform,
        With<Collider>,
        Without<PlatformingCharacterControl>,
    )>,
    mut gizmos: Gizmos,
) {
    for (mut sensor_array, control, mut transform, global_transform) in characters.iter_mut() {
        // determine slope
        match (
            sensor_array.collisions[CharacterSensor::FloorFront as usize],
            sensor_array.collisions[CharacterSensor::FloorBack as usize],
        ) {
            (Some(front), Some(back)) => {
                let direction = Vec3 {
                    x: control.facing_2d.x,
                    y: 0.0,
                    z: control.facing_2d.y,
                };
                let (front_target, _, _) = targets.get(front.entity).unwrap();
                let (back_target, _, _) = targets.get(back.entity).unwrap();
                // let front_point = global_transform.transform_point(front.point1);
                // let back_point = global_transform.transform_point(back.point1);
                // let front_normal = front_target.transform_point(front.normal1);
                // let back_normal = back_target.transform_point(back.normal1);
                let front_point = front.point1;
                let back_point = back.point1;
                let front_normal = front.normal1;
                let back_normal = back.normal1;
                let direction_angle = control.facing_2d.angle_between(Vec2::Y);
                let floor_sensor_back_to_front = Vec3::normalize(front_point - back_point);
                let floor_normals = Vec3::normalize(front_normal + back_normal);
                let up = floor_normals.reject_from_normalized(floor_sensor_back_to_front);

                back_point.angle_between(front_point);

                //gizmos.ray(transform.translation, (up * 2.0), Color::ALICE_BLUE);
                //let mut target = transform.clone();
                let rotation = //Quat::from_rotation_arc(Vec3::X, floor_sensor_back_to_front)
                    //* Quat::from_rotation_arc(Vec3::Y, up)
                     Quat::from_rotation_arc(back_point, front_point) *
                     Quat::from_axis_angle(up, direction_angle);
                info!("angle {:?} quat: {:?}", direction_angle, rotation);
                if !rotation.is_nan() {
                    transform.rotation = rotation;
                }

                //target.look_at(transform.translation + direction, up);
                //transform.look_at(transform.translation + direction, up);
                gizmos.ray(
                    transform.translation,
                    rotation.mul_vec3(Vec3::Z) * 2.0,
                    Color::PURPLE,
                );
                gizmos.ray(
                    transform.translation
                        + Vec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                    floor_sensor_back_to_front,
                    Color::PURPLE,
                );
                gizmos.ray(
                    transform.translation
                        + Vec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                    up,
                    Color::ALICE_BLUE,
                );
            }
            (None, Some(_)) => (),
            (Some(_), None) => (),
            (None, None) => (),
        }
    }
}
