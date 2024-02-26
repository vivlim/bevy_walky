use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Animated {
    pub current_animation: usize,
    pub speed: f32,
}

//pub fn load_anims(asset_server: &mut AssetServer, vec![(dh)])
