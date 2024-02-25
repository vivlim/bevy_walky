use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct PlatformingCharacterControl {
    pub move_input: Vec2,
    pub facing_2d: Vec2,
    pub jump_pressed: bool,
}

#[derive(Component, Reflect)]
pub struct PlatformingCharacterValues {
    pub acceleration_speed: f32,
    pub air_acceleration_speed: f32,
    pub deceleration_speed: f32,
    pub top_speed: f32,
    pub friction_speed: f32,
    pub gravity: f32,
    pub jump_speed: f32,
}

#[derive(Component, Reflect)]
pub struct PlatformingCharacterPhysics {
    pub ground_speed: Vec2,
    /// grounded facing
    pub ground_direction: Vec2,
    /// direction the ground is in
    pub ground_cast_direction: Vec3,
    pub air_speed: AirSpeed,
    pub wall_running: bool,
    /// feedback from 3d collisions to use for damping ground speed
    pub wall_collision_normal: Option<Vec3>,
}

#[derive(Component, Reflect)]
pub struct PlatformingCharacterPhysicsAccel {
    pub ground_acceleration: Vec2,
    pub ground_friction: f32,
    pub air_acceleration: f32,
}

#[derive(Component, Reflect)]
pub struct PlatformingCharacterAnimationFlags {
    pub skidding: bool,
}

#[derive(Component, Reflect)]
pub struct KinematicCharacterPhysics {
    pub velocity: Vec3,
    pub orientation: Quat,
}

#[derive(Reflect)]
pub enum AirSpeed {
    Grounded { angle: f32 },
    InAir(f32),
}

#[derive(Component, Reflect)]
pub struct FloorInfo {
    pub up: Vec3,
    pub floor_sensor_origin_slope: Vec3,
    pub floor_sensor_cast_slope: Vec3,
    pub slope_pivot: Vec3,
}
