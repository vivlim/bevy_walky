use std::{f32::consts::PI, ops::Mul};

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_xpbd_3d::{math::Scalar, prelude::*};

use crate::components::{
    camera::{OrbitCameraTarget, ViewpointMappedInput},
    player::{
        physics::{
            AirSpeed, FloorInfo, KinematicCharacterPhysics, PlatformingCharacterAnimationFlags,
            PlatformingCharacterControl, PlatformingCharacterPhysics,
            PlatformingCharacterPhysicsAccel, PlatformingCharacterValues,
        },
        sensors::{CharacterSensor, CharacterSensorArray, MyCollisionLayers},
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
            (AirSpeed::Grounded{..}, true) => {
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
            if let AirSpeed::Grounded {..} = platforming.air_speed {
                // Trying to jump, and on the ground.
                platforming.air_speed = AirSpeed::InAir(accel.air_acceleration);
            }
        }

        let ground_accel = match platforming.air_speed {
            AirSpeed::Grounded {..}=> accel.ground_acceleration,
            AirSpeed::InAir(_) => accel.ground_acceleration * 0.5,
        };
        //let initial_speed = platforming.ground_speed.length() > values.top_speed;
        // Apply acceleration if we aren't over top speed.
        platforming.ground_speed += accel.ground_acceleration;
        // Actually for now just clamp ground speed to top speed. tune it later.
        platforming.ground_speed = platforming.ground_speed.clamp_length(0.0, values.top_speed);

        match platforming.air_speed {
            AirSpeed::Grounded {..}=> {
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
        &mut PlatformingCharacterPhysics,
        &RigidBody,
        &mut LinearVelocity,
        &Rotation,
        &mut Transform,
        &FloorInfo,
        &GlobalTransform,
        &PlatformingCharacterControl,
    )>,
    mut gizmos: Gizmos,
    spatial_query: SpatialQuery,
) {
    for (mut physics, rb, mut lv, rot, mut transform, floor_info, global_transform, control) in
        query.iter_mut()
    {
        if physics.ground_speed.length() > 1.0 {
            physics.ground_direction = physics.ground_speed.normalize();
        }
        let mut direction = 
            // Map the ground direction into 3d space
            Vec3 {
                x: physics.ground_direction.x,
                y: 0.0,
                z: physics.ground_direction.y,
            }
        ;


        if let AirSpeed::Grounded { angle, .. } = physics.air_speed {
            if (angle > PI/4.0 || angle < -PI/4.0){
                // Cast a ray in the direction we are trying to go. If it hits something, use it as a new ground cast direction

                if let Some(running_up_wall_cast) = spatial_query.cast_ray(
                    global_transform.translation(),
                    lv.0,
                    1.0,
                    true,
                    SpatialQueryFilter::new().with_masks([MyCollisionLayers::Environment]),
                ){
                    let mut new_ground_direction = Vec3::ZERO - running_up_wall_cast.normal;
                    if (physics.wall_running){
                        // Leaving a wall. Keep only Y
                        new_ground_direction.x = 0.0;
                        new_ground_direction.z = 0.0;
                        // Positive Y is ceiling running.
                        if new_ground_direction.y > 0.0 {
                            new_ground_direction.y = 1.0;
                        }
                        else {
                            new_ground_direction.y = -1.0;
                        }
                        physics.wall_running = false;
                    } else {
                        // Remove y component. Only want x/z (unless we are doing ceiling running)
                        new_ground_direction.y = 0.0;
                        if let Some(n) = new_ground_direction.try_normalize(){
                            new_ground_direction = n;
                            physics.wall_running = true;
                        } else {
                            // Return to default (ground is down)
                            new_ground_direction = Vec3::NEG_Y;
                            physics.wall_running = false;
                        }
                    }
                    physics.ground_cast_direction = new_ground_direction;
                }
            }
        }
        let cast_origin_rotation = Quat::from_rotation_arc(Vec3::NEG_Y, physics.ground_cast_direction);
        direction = cast_origin_rotation.mul(direction);

        // Cast ahead and behind to get the slope from where we're standing now.
        let slope_cast_direction = physics.ground_cast_direction;
        let slope_cast_distance = 2.0;
        let radius = 0.25;
        let ground_cast_overshoot = 0.1;
        let front_slope_cast_origin = global_transform.translation() + (direction * (radius));
        let back_slope_cast_origin = global_transform.translation() + (direction * (radius * -1.0));
        let ground_cast_origin = global_transform.translation();
        let mut ground_cast_length = 0.0; // Set this using the longer slope cast
        let front_slope_cast = spatial_query.cast_ray(
            front_slope_cast_origin,
            slope_cast_direction,
            slope_cast_distance,
            true,
            SpatialQueryFilter::new().with_masks([MyCollisionLayers::Environment]),
        );
        let back_slope_cast = spatial_query.cast_ray(
            back_slope_cast_origin,
            slope_cast_direction,
            slope_cast_distance,
            true,
            SpatialQueryFilter::new().with_masks([MyCollisionLayers::Environment]),
        );

        gizmos.ray(front_slope_cast_origin, slope_cast_direction * (slope_cast_distance), Color::LIME_GREEN);
        gizmos.ray(back_slope_cast_origin, slope_cast_direction * (slope_cast_distance), Color::DARK_GREEN);
        match (front_slope_cast, back_slope_cast) {
            (Some(front), Some(back)) => {
                ground_cast_length = f32::max(front.time_of_impact, back.time_of_impact);
                let front_contact =
                    front_slope_cast_origin + (slope_cast_direction * front.time_of_impact);
                gizmos.sphere(front_contact, Quat::default(), 0.1, Color::LIME_GREEN);
                let back_contact =
                    back_slope_cast_origin + (slope_cast_direction * back.time_of_impact);
                gizmos.sphere(back_contact, Quat::default(), 0.1, Color::DARK_GREEN);

                let slope = Vec3::normalize(front_contact - back_contact);

                gizmos.ray(back_contact, slope, Color::GREEN);

                if let AirSpeed::Grounded{ref mut angle} = physics.air_speed {
                    *angle = direction.angle_between(slope);
                }

                let slope_quat = Quat::from_rotation_arc(direction, slope);
                // info!("slope quat {:?}", slope_quat);
                direction = slope_quat.mul_vec3(direction);
            }
            _ => {
                gizmos.circle(
                    global_transform.translation(),
                    Vec3::Y,
                    1.0,
                    Color::ALICE_BLUE,
                );
            }
        }

        gizmos.ray(ground_cast_origin, slope_cast_direction * (ground_cast_length + ground_cast_overshoot), Color::BISQUE);
        gizmos.ray(ground_cast_origin + (slope_cast_direction * ground_cast_length), slope_cast_direction * ground_cast_overshoot, Color::SEA_GREEN);
        let ground_cast = spatial_query.cast_ray(
            ground_cast_origin,
            slope_cast_direction,
            radius + ground_cast_overshoot,
            true,
            SpatialQueryFilter::new().with_masks([MyCollisionLayers::Environment]),
        );

        match ground_cast {
            Some(ground) => {
                match physics.air_speed {
                    AirSpeed::Grounded { .. } => {
                        // floating above the ground, but close enough.
                        // pull the character into the ground so they stick to it
                        if ground.time_of_impact > radius {
                            let dist_away_from_ground = -1.0 * (ground.time_of_impact - radius);
                            if dist_away_from_ground < -0.0001 {
                                // info!("pull down by {:?}", dist_away_from_ground);
                                transform.translation = transform.translation + (ground.normal.normalize() * dist_away_from_ground);
                            }
                        }
                    },
                    AirSpeed::InAir(_) => {
                        if ground.time_of_impact <= radius {
                            info!("just grounded");
                            physics.air_speed = AirSpeed::Grounded{angle: 0.0 /* TODO: does it need to be computed here? */};
                        }
                    }
                }
                // fell into the ground. push out.
                if ground.time_of_impact < radius {
                    let dist_inside_ground = radius - ground.time_of_impact;
                    transform.translation = transform.translation + (ground.normal.normalize() * dist_inside_ground);
                }
            },
            None => {
                if let AirSpeed::Grounded{..} = physics.air_speed {
                    physics.air_speed = AirSpeed::InAir(0.0);
                    physics.wall_running = false;
                    physics.ground_cast_direction = Vec3::NEG_Y;
                }
            }

        }

        lv.0 = direction * physics.ground_speed.length();

        //gizmos.ray(transform.translation, lv., Color::RED);
        if let AirSpeed::InAir(air_speed) = physics.air_speed {
            lv.y = air_speed;
            physics.ground_cast_direction = Vec3::NEG_Y;
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
                                    physics.air_speed = AirSpeed::Grounded{angle: 0.0};
                                    info!("now grounded (entity 1)");
                                }
                            }
                        } else if rb2.is_kinematic() && !rb1.is_kinematic() {
                            position2.0 += contact.global_normal1(rotation1) * contact.penetration;
                            if let Some(ref mut physics) = maybe_physics2 {
                                if let AirSpeed::InAir(_) = physics.air_speed {
                                    physics.air_speed = AirSpeed::Grounded{angle:0.0};
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
        &PlatformingCharacterControl,
        &PlatformingCharacterPhysics,
        &mut FloorInfo,
        Without<CharacterSensorArray>,
    )>,
    mut sensor_arrays: Query<(
        &mut CharacterSensorArray,
        &mut Transform,
        &GlobalTransform,
        Without<PlatformingCharacterControl>,
        Without<ShapeCaster>,
    )>,
    targets: Query<(
        &GlobalTransform,
        With<Collider>,
        Without<PlatformingCharacterControl>,
        Without<ShapeCaster>,
    )>,
    sensors: Query<(&GlobalTransform, &ShapeCaster)>,
    mut gizmos: Gizmos,
) {
    for (mut sensor_array, mut transform, global_transform, ..) in sensor_arrays.iter_mut() {
        let (control, platforming_physics, mut floor_info, ..) =
            characters.get_mut(sensor_array.character).unwrap();
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
                match (
                    targets.get(front.entity),
                    targets.get(back.entity),
                    sensors.get(sensor_array.sensors[CharacterSensor::FloorFront as usize]),
                    sensors.get(sensor_array.sensors[CharacterSensor::FloorBack as usize]),
                ) {
                    (
                        Ok((front_target, ..)),
                        Ok((back_target, ..)),
                        Ok((front_sensor, front_sensor_caster, ..)),
                        Ok((back_sensor, back_sensor_caster, ..)),
                    ) => {
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

                        let floor_sensor_origins_back_to_front =
                            Vec3::normalize(front_sensor_caster.origin - back_sensor_caster.origin);

                        let slope_pivot = floor_sensor_back_to_front.cross(up);

                        //let up = Vec3::normalize(Vec3::ZERO - front_point - back_point);

                        back_point.angle_between(front_point);
                        floor_info.up = up;
                        floor_info.floor_sensor_origin_slope = floor_sensor_origins_back_to_front;
                        floor_info.floor_sensor_cast_slope = floor_sensor_back_to_front;
                        floor_info.slope_pivot = slope_pivot;

                        //gizmos.ray(transform.translation, (up * 2.0), Color::ALICE_BLUE);
                        //let mut target = transform.clone();
                        let rotation = //Quat::from_rotation_arc(Vec3::X, floor_sensor_back_to_front)
                    //* Quat::from_rotation_arc(Vec3::Y, up)
                     //Quat::from_rotation_arc(back_point, front_point) *
                     //Quat::from_axis_angle(up, direction_angle);
                     //Quat::from_rotation_arc(Vec3::Z, Vec3 { x: control.facing_2d.x, y: 0.0, z: control.facing_2d.y })
                      //Quat::from_rotation_arc(floor_sensor_origins_back_to_front, floor_sensor_back_to_front);
                      Quat::from_rotation_arc(Vec3::Z, Vec3 { x: platforming_physics.ground_speed.x, y: 0.0, z: platforming_physics.ground_speed.y}.normalize());
                        //info!("angle {:?} quat: {:?}", direction_angle, rotation);
                        if !rotation.is_nan() {
                            transform.rotation = rotation;
                        }

                        //target.look_at(transform.translation + direction, up);
                        //transform.look_at(transform.translation + direction, up);
                        gizmos.ray(
                            global_transform.translation(),
                            rotation.mul_vec3(Vec3::Z) * 2.0,
                            Color::PURPLE,
                        );
                        gizmos.ray(
                            global_transform.translation()
                                + Vec3 {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                },
                            floor_sensor_back_to_front,
                            Color::PURPLE,
                        );
                        gizmos.ray(
                            global_transform.translation()
                                + Vec3 {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                },
                            up,
                            Color::ALICE_BLUE,
                        );
                        gizmos.ray(
                            global_transform.translation()
                                + Vec3 {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                },
                            slope_pivot,
                            Color::TEAL,
                        );
                    }
                    _ => {
                        warn!("fail");
                    }
                }
            }
            (None, Some(_)) => warn!("no collision for front sensor"),
            (Some(_), None) => warn!("no collision for back sensor"),
            (None, None) => warn!("no collision for front or back sensor"),
        }
    }
}
