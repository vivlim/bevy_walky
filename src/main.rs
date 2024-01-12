pub mod components;
pub mod systems;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use systems::player::physics::{
    update_platforming_accel_from_controls, update_platforming_kinematic_from_physics,
    update_platforming_physics,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // fill the entire browser window
                fit_canvas_to_parent: true,
                // don't hijack keyboard shortcuts like F5, F6, F12, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .register_type::<components::player::physics::PlatformingCharacterPhysics>()
        .register_type::<components::player::physics::PlatformingCharacterPhysicsAccel>()
        .register_type::<components::player::physics::PlatformingCharacterValues>()
        .register_type::<components::player::physics::PlatformingCharacterControl>()
        .add_systems(Startup, systems::world::camera::setup_camera)
        .add_systems(Startup, systems::world::scene::setup_scene)
        .add_systems(Startup, systems::world::scene::setup_physics)
        .add_systems(Update, systems::player::physics::character_movement)
        .add_systems(Update, systems::player::physics::read_result_system)
        .add_systems(Update, update_platforming_physics)
        .add_systems(Update, update_platforming_accel_from_controls)
        .add_systems(Update, update_platforming_kinematic_from_physics)
        .run();
}
