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
        Option<&ColliderParent>,
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
            [(rb1, cp1, mut position1, rotation1, mut maybe_physics1, _), (rb2, cp2, mut position2, _, mut maybe_physics2, _)],
        ) = bodies.get_many_mut([contacts.entity1, contacts.entity2])
        {
            for manifold in contacts.manifolds.iter() {
                for contact in manifold.contacts.iter() {
                    if contact.penetration <= Scalar::EPSILON {
                        continue;
                    }

                    let rb1 = match (rb1, cp1) {
                        (None, Some(p)) => match scene_bodies.get_component::<RigidBody>(p.get()) {
                            Ok(rb) => Some(rb),
                            Err(e) => {
                                warn!("failed to get parent rigid body for {:?}: {:?}", p.get(), e);
                                None
                            }
                        },
                        (Some(r), _) => Some(r),
                        (None, None) => {
                            warn!(
                                "a colliding object was not a rigid body or parented to something"
                            );
                            None
                        }
                    };
                    let rb2 = match (rb2, cp2) {
                        (None, Some(p)) => match scene_bodies.get_component::<RigidBody>(p.get()) {
                            Ok(rb) => Some(rb),
                            Err(e) => {
                                warn!("failed to get parent rigid body for {:?}: {:?}", p.get(), e);
                                None
                            }
                        },
                        (Some(r), _) => Some(r),
                        (None, None) => {
                            warn!(
                                "a colliding object was not a rigid body or parented to something"
                            );
                            None
                        }
                    };

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
