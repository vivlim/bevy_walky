use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{
    camera::OrbitCameraTarget,
    player::physics::{
        PlatformingCharacterControl, PlatformingCharacterPhysics, PlatformingCharacterPhysicsAccel,
        PlatformingCharacterValues,
    },
};

/// set up a simple 3D scene
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Circle::new(9.0).into()),
    //     material: materials.add(Color::WHITE.into()),
    //     transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    //     ..default()
    // })
    //     .insert(TransformBundle::from(Transform::from_xyz(0.0,-2.0,0.0)));
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
pub fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    commands
        .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -50.0,
                max_x: 50.0,
                min_y: -0.05,
                max_y: 0.05,
                min_z: -50.0,
                max_z: 50.0,
            })),
            material: materials.add(Color::WHITE.into()),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    /* Create the bouncing ball. */
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(TransformBundle::from(Transform::from_xyz(4.0, 4.0, 0.0)));

    commands
        .spawn(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule_y(1.0, 0.5))
        .insert(KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            ..default()
        })
        .insert(PlatformingCharacterPhysics {
            ground_speed: Vec2::ZERO,
            air_speed: crate::components::player::physics::AirSpeed::Grounded,
        })
        .insert(PlatformingCharacterPhysicsAccel {
            ground_acceleration: Vec2::ZERO,
            ground_friction: 0.0,
            air_acceleration: 0.0,
        })
        .insert(PlatformingCharacterControl {
            move_input: Vec2::ZERO,
            jump_pressed: false,
        })
        .insert(PlatformingCharacterValues {
            acceleration_speed: 0.02,
            deceleration_speed: 0.02,
            top_speed: 1.0,
            friction_speed: 0.08,
            gravity: 1.0,
        })
        .insert(OrbitCameraTarget {
            distance: 5.0,
            active: true,
            yaw: 0.0,
            pitch: 0.0,
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 1.0, 0.0)));
}
