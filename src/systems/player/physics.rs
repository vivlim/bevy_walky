use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_xpbd_3d::prelude::*;

use crate::components::{
    camera::{OrbitCameraTarget, ViewpointMappedInput},
    player::physics::{
        AirSpeed, KinematicCharacterPhysics, PlatformingCharacterControl,
        PlatformingCharacterPhysics, PlatformingCharacterPhysicsAccel, PlatformingCharacterValues,
    },
};

pub fn character_movement(
    mut character_control: Query<(&mut PlatformingCharacterControl, &mut ViewpointMappedInput)>,
    mut camera_targets: Query<&mut OrbitCameraTarget>,
    keys: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
) {
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
    if keyboardDirection.length() > 0.0 {
        for (_, mut vmi) in character_control.iter_mut() {
            vmi.move_input = keyboardDirection.normalize_or_zero();
        }
    }

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse.read() {
        cursor_delta += event.delta;
    }

    const mouse_look_factor: f32 = 0.001;
    if cursor_delta.length() > 0.3 {
        for mut c in &mut camera_targets {
            if c.active {
                c.pitch += cursor_delta.y * mouse_look_factor;
                c.yaw += cursor_delta.x * mouse_look_factor;
            }
        }
    }
}

pub fn character_gamepad(
    mut character_control: Query<(&mut PlatformingCharacterControl, &mut ViewpointMappedInput)>,
    mut camera_targets: Query<&mut OrbitCameraTarget>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepads: Res<Gamepads>,
) {
    for gamepad in gamepads.iter() {
        // The joysticks are represented using a separate axis for X and Y
        let axis_lx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickX,
        };
        let axis_ly = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::LeftStickY,
        };
        let axis_rx = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::RightStickX,
        };
        let axis_ry = GamepadAxis {
            gamepad,
            axis_type: GamepadAxisType::RightStickY,
        };
        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            // combine X and Y into one vector
            let left_stick_pos = Vec2::new(x, y);
            //info!("{:?} LeftStickX value is {}", gamepad, left_stick_pos);

            // Example: check if the stick is pushed up
            if left_stick_pos.length() > 0.3 {
                for (_, mut vmi) in character_control.iter_mut() {
                    vmi.move_input = left_stick_pos.normalize_or_zero();
                }
            }
        }
        if let (Some(rx), Some(ry)) = (axes.get(axis_rx), axes.get(axis_ry)) {
            if f32::abs(ry) > 0.1 {
                for mut c in &mut camera_targets {
                    info!("adjusting camera pitch");
                    c.pitch -= ry * 0.003
                }
            }
            if f32::abs(rx) > 0.1 {
                for mut c in &mut camera_targets {
                    info!("adjusting camera yaw");
                    c.yaw -= rx * 0.005
                }
            }
        }
        // In a real game, the buttons would be configurable, but here we hardcode them
        let jump_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::South,
        };
        let heal_button = GamepadButton {
            gamepad,
            button_type: GamepadButtonType::East,
        };

        if buttons.just_pressed(jump_button) {
            // button just pressed: make the player jump
            info!("pushed jump");
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
            lv.y = 0.0;

            //gizmos.ray(transform.translation, lv., Color::RED);
        } else {
            lv.x = 0.0;
            lv.z = 0.0;
            lv.y = 0.0;
        }
    }
}
