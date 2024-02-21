use bevy::prelude::*;

use bevy_xpbd_3d::math::*;
use bevy_xpbd_3d::prelude::*;
use strum::EnumCount;
use strum::IntoEnumIterator;

use crate::components::player::physics::FloorInfo;
use crate::components::player::physics::PlatformingCharacterAnimationFlags;
use crate::components::player::sensors::CharacterSensor;
use crate::components::player::sensors::CharacterSensorArray;
use crate::components::player::sensors::MyCollisionLayers;
use crate::components::{
    camera::{OrbitCameraTarget, ViewpointMappable, ViewpointMappedInput},
    player::physics::{
        PlatformingCharacterControl, PlatformingCharacterPhysics, PlatformingCharacterPhysicsAccel,
        PlatformingCharacterValues,
    },
};

use super::sensors::sensor_bundle;

/// set up a simple 3D scene
pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut player = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(Color::rgba_u8(124, 0, 255, 69).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        RigidBody::Kinematic,
        Collider::ball(0.35),
        ShapeCaster::new(
            Collider::ball(0.35),
            Vector::ZERO,
            Quaternion::default(),
            Vector::NEG_Y,
        )
        .with_max_time_of_impact(0.11)
        .with_max_hits(1),
    ));
    player
        .insert(PlatformingCharacterPhysics {
            ground_speed: Vec2::ZERO,
            ground_direction: Vec2::X,
            ground_cast_direction: Vec3::NEG_Y,
            air_speed: crate::components::player::physics::AirSpeed::InAir(0.0),
        })
        .insert(PlatformingCharacterPhysicsAccel {
            ground_acceleration: Vec2::ZERO,
            ground_friction: 0.0,
            air_acceleration: 0.0,
        })
        .insert(PlatformingCharacterControl {
            move_input: Vec2::ZERO,
            facing_2d: Vec2::X,
            jump_pressed: false,
        })
        .insert(PlatformingCharacterValues {
            acceleration_speed: 1.00,
            deceleration_speed: 1.00,
            top_speed: 15.0,
            friction_speed: 0.30,
            gravity: -0.2,
            jump_speed: 2.0,
        })
        .insert(PlatformingCharacterAnimationFlags { skidding: false })
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
        .insert(FloorInfo {
            up: Vec3::default(),
            floor_sensor_origin_slope: Vec3::default(),
            floor_sensor_cast_slope: Vec3::default(),
            slope_pivot: Vec3::default(),
        })
        .insert(CollisionLayers::new(
            [MyCollisionLayers::Player],
            [MyCollisionLayers::Enemy, MyCollisionLayers::Environment],
        ));

    let player_id = player.id();
    info!("Player is entity {:?}", player_id);

    // let sensors = CharacterSensorArray {
    //     sensors: CharacterSensor::iter()
    //         .map(|s| {
    //             let bundle = sensor_bundle(s, player_id);
    //             let sensor = commands.spawn((bundle));
    //             sensor.id()
    //         })
    //         .collect::<Vec<Entity>>()
    //         .try_into()
    //         .unwrap(),
    //     collisions: [None; CharacterSensor::COUNT],
    //     character: player_id,
    // };

    // let sc = sensors.sensors.clone();

    // let sensor_entity = commands
    //     .spawn((
    //         sensors,
    //         SpatialBundle::default(),
    //         Collider::default(), // Must be a collider for child colliders to work
    //         CollisionLayers::new(
    //             [MyCollisionLayers::Player],
    //             [MyCollisionLayers::Environment, MyCollisionLayers::Enemy],
    //         ),
    //     ))
    //     .push_children(&sc)
    //     .set_parent(player_id);
}
