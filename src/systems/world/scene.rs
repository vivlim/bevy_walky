use bevy::prelude::*;

use bevy_xpbd_3d::prelude::*;

use crate::components::{
    camera::{OrbitCameraTarget, ViewpointMappable, ViewpointMappedInput},
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
    asset_server: Res<AssetServer>,
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

    commands.spawn(SceneBundle {
        scene: asset_server.load("walky_objs.glb#Scene0"),
        ..default()
    });
    // .insert(Collider::from_bevy_mesh(
    //     asset_server.load("walky_objs.glb#Mesh0"),
    // ));
}
pub fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* Create the ground. */
    commands
        .spawn(PbrBundle {
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

    commands
        .spawn(RigidBody::Kinematic)
        .insert(Collider::capsule(1.0, 0.5))
        .insert(RigidBody::Kinematic)
        .insert(Collider::capsule(1.0, 0.4))
        // .insert(
        // // Cast the player shape downwards to detect when the player is grounded
        // ShapeCaster::new(
        // Collider::capsule(0.9, 0.35),
        // Vector::ZERO,
        // Quaternion::default(),
        // Vector::NEG_Y,
        // )
        // .with_max_time_of_impact(0.11)
        // .with_max_hits(1),
        //     )
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
            acceleration_speed: 1.00,
            deceleration_speed: 1.00,
            top_speed: 15.0,
            friction_speed: 0.30,
            gravity: 1.0,
        })
        .insert(OrbitCameraTarget {
            distance: 5.0,
            active: true,
            yaw: 0.0,
            pitch: 0.0,
        })
        .insert(ViewpointMappable {
            forward: Quat::default(),
        })
        .insert(ViewpointMappedInput {
            move_input: Vec2::ZERO,
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 1.0, 0.0)));
}
