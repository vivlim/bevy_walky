use bevy::{
    prelude::{Component, Entity},
    reflect::{impl_type_path, Reflect},
};
use bevy_xpbd_3d::prelude::{RayHitData, ShapeHitData};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter, EnumTable, FromRepr};

/// Different kinds of sensor a character has
#[derive(EnumCountMacro, EnumIter, FromRepr, Reflect, Debug)]
pub enum CharacterSensor {
    FloorFront,
    FloorBack,
}

// wait for bevy 13 in bevy_xpbd
//impl_reflect!(::bevy_xpbd_3d::plugins::spatial_query::ShapeHitData);

/// Collection of sensors belonging to a character
#[derive(Component, Reflect, Debug)]
pub struct CharacterSensorArray {
    pub sensors: [Entity; CharacterSensor::COUNT],
    #[reflect(ignore)]
    pub collisions: [Option<ShapeHitData>; CharacterSensor::COUNT],
    pub character: Entity,
}

/// Marks some sensor entities as character sensors, for queries
#[derive(Component, Reflect)]
pub struct CharacterSensorCaster {
    pub kind: CharacterSensor,
    pub character_entity: Entity,
}
