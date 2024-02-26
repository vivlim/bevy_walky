pub mod components;
pub mod systems;

use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::transform::TransformSystem;
use bevy::{prelude::*, render::RenderPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::{prelude::*, PhysicsSchedule, SubstepSchedule, SubstepSet};
use components::camera::{OrbitCameraTarget, ViewpointMappable, ViewpointMappedInput};
use components::player::sensors::CharacterSensorArray;
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

#[cfg(target_os = "windows")]
fn render_plugin() -> RenderPlugin {
    // workaround for error spam on windows: https://github.com/bevyengine/bevy/issues/9975#issuecomment-1848050580
    RenderPlugin {
        render_creation: RenderCreation::Automatic(WgpuSettings {
            backends: Some(Backends::VULKAN), // vulkan spends less time compiling shaders, which is *important* when iterating
            ..default()
        }),
    }
}

#[cfg(not(target_os = "windows"))]
fn render_plugin() -> RenderPlugin {
    RenderPlugin {
        render_creation: RenderCreation::Automatic(Default::default()),
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // fill the entire browser window
                        fit_canvas_to_parent: true,
                        // don't hijack keyboard shortcuts like F5, F6, F12, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(render_plugin()),
        )
        .add_plugins(LookTransformPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(UnrealCameraPlugin::default())
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        .register_type::<components::player::physics::PlatformingCharacterPhysics>()
        .register_type::<components::player::physics::PlatformingCharacterPhysicsAccel>()
        .register_type::<components::player::physics::PlatformingCharacterValues>()
        .register_type::<components::player::physics::PlatformingCharacterControl>()
        .register_type::<components::player::physics::PlatformingCharacterAnimationFlags>()
        .register_type::<components::player::animation::Animated>()
        .register_type::<CharacterSensorArray>()
        .register_type::<LookTransform>()
        .register_type::<OrbitCameraTarget>()
        .register_type::<ViewpointMappable>()
        .register_type::<ViewpointMappedInput>()
        .add_systems(Startup, systems::world::camera::setup_camera)
        .add_systems(Startup, systems::world::scene::setup_scene)
        .add_systems(Startup, systems::player::spawn::spawn_player)
        .add_systems(Startup, systems::world::scene::setup_physics)
        .add_systems(Startup, systems::player::animation::setup_animations)
        .add_systems(
            Update,
            systems::world::physics_fixup::fixup_nested_colliders,
        )
        .add_systems(
            Update,
            systems::world::physics_fixup::reapply_collider_transform,
        )
        .add_systems(Update, systems::player::control::character_movement)
        .add_systems(Update, systems::player::control::character_gamepad)
        .add_systems(Update, systems::player::animation::character_animation)
        .add_systems(Update, update_camera)
        .add_systems(Update, project_input_camera)
        .add_systems(
            // constraints avoid camera jitter: https://github.com/Jondolf/bevy_xpbd/issues/211#issuecomment-1789342920
            PostUpdate,
            update_camera
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        )
        // .add_systems(
        //     PostProcessCollisions,
        //     systems::player::sensors::update_sensors, //.after(systems::player::sensors::position_sensors),
        // )
        // .add_systems(
        //     PostProcessCollisions,
        //     systems::player::sensors::position_sensors
        //         .after(TransformSystem::TransformPropagate)
        //         .after(PhysicsSet::Sync),
        // )
        .add_systems(FixedUpdate, systems::player::physics::update_floor)
        .add_systems(FixedUpdate, update_platforming_accel_from_controls)
        .add_systems(
            FixedUpdate,
            update_platforming_physics.after(update_platforming_accel_from_controls),
        )
        .add_systems(
            PostUpdate,
            update_platforming_kinematic_from_physics
                .after(update_platforming_physics)
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        )
        .add_systems(
            PostUpdate,
            systems::player::physics::push_out_of_ground
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        )
        .add_systems(
            SubstepSchedule,
            systems::player::physics::handle_collisions
                .in_set(SubstepSet::SolveUserConstraints)
                .after(systems::player::physics::push_out_of_ground),
        )
        .run();
}
