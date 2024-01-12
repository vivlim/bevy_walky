use bevy::prelude::*;

#[derive(Component)]
pub struct PlatformingCharacterControl {
    pub move_input: Vec2,
    pub jump_pressed: bool,
}

#[derive(Component)]
pub struct PlatformingCharacterValues {
    pub acceleration_speed: f32,
    pub deceleration_speed: f32,
    pub top_speed: f32,
    pub friction_speed: f32,
    pub gravity: f32,
}

#[derive(Component)]
pub struct PlatformingCharacterPhysics {
    pub ground_speed: Vec2,
    pub air_speed: AirSpeed,
}

#[derive(Component)]
pub struct PlatformingCharacterPhysicsAccel {
    pub ground_acceleration: Vec2,
    pub ground_friction: f32,
    pub air_acceleration: f32,
}

#[derive(Component)]
pub struct KinematicCharacterPhysics {
    pub velocity: Vec3,
    pub orientation: Quat,
}

pub enum AirSpeed {
    Grounded,
    InAir(f32),
}
