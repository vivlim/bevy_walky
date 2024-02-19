use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use strum::IntoEnumIterator;

use crate::components::player::{
    physics::{PlatformingCharacterControl, PlatformingCharacterPhysics},
    sensors::{CharacterSensor, CharacterSensorArray, CharacterSensorCaster},
};

pub fn sensor_bundle(sensor: CharacterSensor, character_entity: Entity) -> impl Bundle {
    (
        match sensor {
            CharacterSensor::FloorFront => ShapeCaster::new(
                Collider::cuboid(0.25, 0.02, 0.05),
                Vec3 {
                    x: 0.0,
                    y: -0.10,
                    z: 0.1,
                },
                Quat::default(),
                Vec3::NEG_Y,
            ),
            CharacterSensor::FloorBack => ShapeCaster::new(
                Collider::cuboid(0.25, 0.02, 0.05),
                Vec3 {
                    x: 0.0,
                    y: -0.1,
                    z: -0.1,
                },
                Quat::default(),
                Vec3::NEG_Y,
            ),
        }
        .with_query_filter(SpatialQueryFilter::new().without_entities([character_entity])), // don't count self-collisions
        CharacterSensorCaster {
            kind: sensor,
            character_entity,
        },
        SpatialBundle::default(),
    )
}
pub fn position_sensors(
    characters: Query<(
        &CharacterSensorArray,
        &mut Transform,
        &PlatformingCharacterControl,
        With<PlatformingCharacterPhysics>,
    )>,
    mut casters: Query<(
        &mut ShapeCaster,
        &CharacterSensorCaster,
        &mut Transform, // transform doesn't seem to matter.
        Without<CharacterSensorArray>,
        Without<PlatformingCharacterPhysics>,
    )>,
) {
    for (mut caster, sensor, mut transform, _, _) in casters.iter_mut() {
        let (array, char_transform, control, _) = characters.get(sensor.character_entity).unwrap();
    }
}

pub fn update_sensors(
    mut characters: Query<(
        &mut CharacterSensorArray,
        Entity,
        &PlatformingCharacterControl,
        &mut Transform,
        With<PlatformingCharacterPhysics>,
    )>,
    mut casters: Query<(&ShapeCaster, &CharacterSensorCaster, &ShapeHits)>,
    mut gizmos: Gizmos,
) {
    for (mut sensor_array, sensor_owner, control, transform, _) in characters.iter_mut() {
        for sensor in CharacterSensor::iter() {
            let sensor_index = sensor as usize;
            let caster_id = sensor_array.sensors[sensor_index];
            match casters.get(caster_id) {
                Ok((caster, _, _hits)) => {
                    let (caster, _, hits) = casters.get(caster_id).unwrap();
                    if hits.is_empty() {
                        gizmos.sphere(caster.origin, Quat::default(), 5.0, Color::GREEN);
                        sensor_array.collisions[sensor_index] = None;
                        continue;
                    }
                    for hit in hits.iter() {
                        gizmos.sphere(caster.origin, Quat::default(), 5.0, Color::ORANGE);
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
