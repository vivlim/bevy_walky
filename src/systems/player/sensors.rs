use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use strum::IntoEnumIterator;

use crate::components::player::{
    physics::PlatformingCharacterPhysics,
    sensors::{CharacterSensor, CharacterSensorArray, CharacterSensorCaster},
};

pub fn sensor_bundle(sensor: CharacterSensor) -> impl Bundle {
    (
        ShapeCaster::new(
            Collider::ball(0.35),
            Vec3::ZERO,
            Quat::default(),
            Vec3::NEG_Y,
        ),
        CharacterSensorCaster {},
    )
}

pub fn update_sensors(
    mut characters: Query<(&mut CharacterSensorArray, With<PlatformingCharacterPhysics>)>,
    mut casters: Query<(&ShapeCaster, &CharacterSensorCaster, &ShapeHits)>,
) {
    for (mut sensor_array, _) in characters.iter_mut() {
        for sensor in CharacterSensor::iter() {
            let sensor_index = sensor as usize;
            let caster_id = sensor_array.sensors[sensor_index];
            match casters.get(caster_id) {
                Ok((caster, _, _hits)) => {
                    let (caster, _, hits) = casters.get(caster_id).unwrap();
                    sensor_array.collision[sensor_index] = !hits.is_empty();
                }
                Err(e) => {
                    warn!("Sensor query failed for {:?}, {:?}", sensor_index, e);
                    sensor_array.collision[sensor_index] = false; // If there's no sensor, it can't collide...
                }
            }
        }
        //info!("sensors: {:?}", sensor_array);
    }
}
