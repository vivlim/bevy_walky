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
    /// Radius for slope detection, and the amount of distance we want to have from the ground. it's a 'cushion' around the actual collider.
    pub cushion_radius: f32,
    /// How big our 'footprint' is.
    pub ground_detection_radius: f32,
    /// How big our radius for bonking into stuff is.
    pub obstacle_detection_radius: f32,
    /// How far to cast when computing slopes
    pub slope_cast_distance: f32,
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
    pub overall_rotation: Quat,
    pub show_gizmos: bool,
    /// When ceiling running, this quaternion represents the rotation from ground to ceiling that passes through the wall that was climbed.
    pub ceiling_run_quat: Option<Quat>,
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
    Grounded { angle: f32, slope_quat: Quat },
    InAir(f32),
}

#[derive(Component, Reflect)]
pub struct FloorInfo {
    pub up: Vec3,
    pub floor_sensor_origin_slope: Vec3,
    pub floor_sensor_cast_slope: Vec3,
    pub slope_pivot: Vec3,
}
