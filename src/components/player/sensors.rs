use bevy::{
    prelude::{Component, Entity},
    reflect::Reflect,
};
use strum::EnumCount;
use strum_macros::{EnumCount as EnumCountMacro, EnumIter, EnumTable, FromRepr};

/// Different kinds of sensor a character has
#[derive(EnumCountMacro, EnumIter, FromRepr, Reflect, Debug)]
pub enum CharacterSensor {
    Floor,
}

/// Collection of sensors belonging to a character
#[derive(Component, Reflect)]
pub struct CharacterSensorArray {
    pub sensors: [Entity; CharacterSensor::COUNT],
    pub collision: [bool; CharacterSensor::COUNT],
}

/// Marks some sensor entities as character sensors, for queries
#[derive(Component, Reflect)]
pub struct CharacterSensorCaster {}
