use bevy::prelude::*;

#[derive(Component)]
pub struct MyCameraMarker;

#[derive(Component, Reflect)]
pub struct OrbitCameraTarget {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub active: bool,
}

#[derive(Component, Reflect)]
pub struct ViewpointMappedInput {
    pub move_input: Vec2,
}
