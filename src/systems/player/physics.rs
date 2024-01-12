use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::player::physics::{
    AirSpeed, KinematicCharacterPhysics, PlatformingCharacterControl, PlatformingCharacterPhysics,
    PlatformingCharacterPhysicsAccel, PlatformingCharacterValues,
};

pub fn read_result_system(controllers: Query<(Entity, &KinematicCharacterControllerOutput)>) {
    for (entity, output) in controllers.iter() {
        println!(
            "Entity {:?} moved by {:?} and touches the ground: {:?}",
            entity, output.effective_translation, output.grounded
        );
    }
}

pub fn character_movement(
    mut controllers: Query<&mut KinematicCharacterController>,
    mut character_control: Query<&mut PlatformingCharacterControl>,
    keys: Res<Input<KeyCode>>,
) {
    for mut controller in controllers.iter_mut() {
        if keys.just_pressed(KeyCode::Space) {
            // jump
            //controller.translation = Some(Vec3::new(0.0, 1.5, 0.0));
        } else {
            // todo: apply gravity instead of just setting this
            //controller.translation = Some(Vec3::new(0.0, -0.5, 0.0));
        }
        let mut keyboardDirection = Vec2::new(0.0, 0.0);
        if keys.pressed(KeyCode::Up) {
            keyboardDirection += Vec2 { x: 0.0, y: 1.0 }
        }
        if keys.pressed(KeyCode::Down) {
            keyboardDirection += Vec2 { x: 0.0, y: -1.0 }
        }
        if keys.pressed(KeyCode::Left) {
            keyboardDirection += Vec2 { x: -1.0, y: 0.0 }
        }
        if keys.pressed(KeyCode::Right) {
            keyboardDirection += Vec2 { x: 1.0, y: 0.0 }
        }
        for mut cc in character_control.iter_mut() {
            cc.move_input = keyboardDirection.normalize_or_zero();
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

        match platforming.air_speed {
            AirSpeed::Grounded => {
                //let initial_speed = platforming.ground_speed.length() > values.top_speed;
                // Apply acceleration if we aren't over top speed.
                platforming.ground_speed += accel.ground_acceleration;
                // Actually for now just clamp ground speed to top speed. tune it later.
                platforming.ground_speed =
                    platforming.ground_speed.clamp_length(0.0, values.top_speed);
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
                // Apply gravity
                todo!("implement being in the air")
            }
        }

        // accel.ground_acceleration = Vec2::ZERO;
        // accel.ground_friction = 0.0;
        // accel.air_acceleration = 0.0;
    }
}

pub fn update_platforming_accel_from_controls(
    mut query: Query<(
        &mut PlatformingCharacterPhysicsAccel,
        &PlatformingCharacterPhysics,
        &PlatformingCharacterControl,
        &PlatformingCharacterValues,
    )>,
) {
    for (mut accel, platforming, control, values) in query.iter_mut() {
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
        } else {
            accel.ground_acceleration = Vec2::ZERO;
            accel.ground_friction = values.friction_speed;
        }
    }
}

pub fn update_platforming_kinematic_from_physics(
    mut query: Query<(
        &PlatformingCharacterPhysics,
        &mut KinematicCharacterController,
    )>,
) {
    for (physics, mut kinematic) in query.iter_mut() {
        kinematic.translation = if physics.ground_speed.length() > 0.0 {
            // Map the ground speed into 3d space
            let ground_speed = Vec3 {
                x: physics.ground_speed.x,
                y: 0.0,
                z: physics.ground_speed.y,
            };

            Some(ground_speed)
        } else {
            None
        }
    }
}
