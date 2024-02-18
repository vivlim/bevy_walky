use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::components::player::sensors::{CharacterSensor, CharacterSensorCaster};

pub fn sensor_bundle(sensor: CharacterSensor) -> impl Bundle {
    (
        ShapeCaster::new(Collider::ball(0.35), Vec3::ZERO, Quat::default(), Vec3::X),
        CharacterSensorCaster {},
    )
}
