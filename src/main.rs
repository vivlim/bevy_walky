pub mod components;
pub mod systems;

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::prelude::*;
use components::camera::{OrbitCameraTarget, ViewpointMappable, ViewpointMappedInput};
use smooth_bevy_cameras::{
    controllers::{orbit::OrbitCameraPlugin, unreal::UnrealCameraPlugin},
    LookTransform, LookTransformBundle, LookTransformPlugin,
};
use systems::{
    player::physics::{
        handle_collisions, update_platforming_accel_from_controls,
        update_platforming_kinematic_from_physics, update_platforming_physics,
    },
    world::camera::{project_input_camera, update_camera},
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
        .add_plugins(LookTransformPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(UnrealCameraPlugin::default())
        .add_plugins(PhysicsPlugins::default())
        .register_type::<components::player::physics::PlatformingCharacterPhysics>()
        .register_type::<components::player::physics::PlatformingCharacterPhysicsAccel>()
        .register_type::<components::player::physics::PlatformingCharacterValues>()
        .register_type::<components::player::physics::PlatformingCharacterControl>()
        .register_type::<LookTransform>()
        .register_type::<OrbitCameraTarget>()
        .register_type::<ViewpointMappable>()
        .register_type::<ViewpointMappedInput>()
        .add_systems(Startup, systems::world::camera::setup_camera)
        .add_systems(Startup, systems::world::scene::setup_scene)
        .add_systems(Startup, systems::world::scene::setup_physics)
        .add_systems(Update, systems::player::physics::character_movement)
        .add_systems(Update, systems::player::physics::character_gamepad)
        .add_systems(Update, update_camera)
        .add_systems(Update, project_input_camera)
        .add_systems(
            // constraints avoid camera jitter: https://github.com/Jondolf/bevy_xpbd/issues/211#issuecomment-1789342920
            PostUpdate,
            update_camera
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        )
        .add_systems(FixedUpdate, update_platforming_accel_from_controls)
        .add_systems(
            FixedUpdate,
            update_platforming_physics.after(update_platforming_accel_from_controls),
        )
        .add_systems(
            FixedUpdate,
            update_platforming_kinematic_from_physics.after(update_platforming_physics),
        )
        .add_systems(
            FixedUpdate,
            handle_collisions.after(update_platforming_kinematic_from_physics),
        )
        .run();
}
