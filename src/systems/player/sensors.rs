use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use strum::IntoEnumIterator;

use crate::components::player::{
    physics::{PlatformingCharacterControl, PlatformingCharacterPhysics},
    sensors::{CharacterSensor, CharacterSensorArray, CharacterSensorCaster, MyCollisionLayers},
};

pub fn sensor_bundle(sensor: CharacterSensor, character_entity: Entity) -> impl Bundle {
    (
        match sensor {
            CharacterSensor::FloorFront => ShapeCaster::new(
                Collider::ball(0.1),
                Vec3 {
                    x: 0.0,
                    y: -0.10,
                    z: 0.1,
                },
                Quat::default(),
                Vec3::NEG_Y,
            ),
            CharacterSensor::FloorBack => ShapeCaster::new(
                Collider::ball(0.1),
                Vec3 {
                    x: 0.0,
                    y: -0.1,
                    z: -0.1,
                },
                Quat::default(),
                Vec3::NEG_Y,
            ),
        }
        .with_query_filter(SpatialQueryFilter::new().with_masks([MyCollisionLayers::Environment])), // don't count self-collisions
        CollisionLayers::new(
            [MyCollisionLayers::Player],
            [MyCollisionLayers::Environment, MyCollisionLayers::Enemy],
        ),
        CharacterSensorCaster {
            kind: sensor,
            character_entity,
        },
        SpatialBundle::default(),
    )
}

pub fn update_sensors(
    mut characters: Query<(
        Entity,
        &PlatformingCharacterControl,
        &mut Transform,
        With<PlatformingCharacterPhysics>,
    )>,
    mut sensors: Query<(
        &mut CharacterSensorArray,
        Without<PlatformingCharacterControl>,
    )>,
    mut casters: Query<(
        &ShapeCaster,
        &CharacterSensorCaster,
        &ShapeHits,
        &GlobalTransform,
    )>,
    mut gizmos: Gizmos,
) {
    for (mut sensor_array, _) in sensors.iter_mut() {
        let (sensor_owner, control, transform, _) =
            characters.get_mut(sensor_array.character).unwrap();
        for sensor in CharacterSensor::iter() {
            let sensor_index = sensor as usize;
            let caster_id = sensor_array.sensors[sensor_index];
            match casters.get(caster_id) {
                Ok((caster, _, hits, gt)) => {
                    if hits.is_empty() {
                        gizmos.sphere(gt.translation(), Quat::default(), 0.5, Color::GREEN);
                        sensor_array.collisions[sensor_index] = None;
                        continue;
                    }
                    for hit in hits.iter() {
                        gizmos.sphere(gt.translation(), Quat::default(), 0.5, Color::ORANGE);
                        gizmos.line(hit.point1, hit.point2, Color::ORANGE);
                        sensor_array.collisions[sensor_index] = Some(hit.clone());
                        break;
                    }
                }
                Err(e) => {
                    warn!("Sensor query failed for {:?}, {:?}", sensor_index, e);
                }
            }
        }
        //info!("sensors: {:?}", sensor_array);
    }
}
