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
                    //info!("adjusting camera pitch");
                    c.pitch -= ry * 0.007
                }
            }
            if f32::abs(rx) > 0.1 {
                for mut c in &mut camera_targets {
                    //info!("adjusting camera yaw");
                    c.yaw -= rx * 0.01
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

        // If jump was pressed and is now released, update state
        for (mut pcc, _) in character_control.iter_mut() {
            if pcc.jump_pressed && !buttons.pressed(jump_button) {
                pcc.jump_pressed = false;
            }
        }
        // If jump was just pressed, update state
        if buttons.just_pressed(jump_button) {
            // button just pressed: make the player jump
            for (mut pcc, _) in character_control.iter_mut() {
                pcc.jump_pressed = true;
            }
        }
    }
}
