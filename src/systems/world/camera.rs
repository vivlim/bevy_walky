use bevy::{math::vec3, prelude::*};
use smooth_bevy_cameras::{
    controllers::{
        orbit::{OrbitCameraBundle, OrbitCameraController},
        unreal::{UnrealCameraBundle, UnrealCameraController},
    },
    LookTransform, LookTransformBundle, Smoother,
};

use crate::components::{
    camera::{MyCameraMarker, OrbitCameraTarget, ViewpointMappable, ViewpointMappedInput},
    player::physics::PlatformingCharacterControl,
};

pub fn setup_camera(mut commands: Commands) {
    let light = commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 3.0, 0.0),
            ..default()
        })
        .id();
    commands
        .spawn(UnrealCameraBundle::new(
            smooth_bevy_cameras::controllers::unreal::UnrealCameraController::default(),
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::Y,
        ))
        .add_child(light)
        .insert(Camera3dBundle::default());
}

pub fn update_camera(
    mut cameras: Query<(
        &mut UnrealCameraController,
        &mut LookTransform,
        &mut Transform,
        Without<OrbitCameraTarget>,
    )>,
    mut targets: Query<(
        &OrbitCameraTarget,
        &Transform,
        &mut ViewpointMappable,
        Without<LookTransform>,
    )>,
    mut gizmos: Gizmos,
) {
    let mut last_target: Option<&OrbitCameraTarget> = None;
    for (mut unreal_camera, mut look_transform, mut camera_transform, _) in cameras.iter_mut() {
        for (target, target_transform, mut viewpoint_mappable, _) in targets.iter_mut() {
            // Get camera target yaw and pitch, and compute vector
            let xz_len = f32::cos(target.pitch);
            let direction = Vec3::new(
                xz_len * f32::cos(target.yaw),
                f32::sin(target.pitch),
                xz_len * f32::sin(target.yaw * -1.0),
            );
            // Multiply it by the desired distance and add it to the target's position.
            let camera_target_position =
                (direction * target.distance) + target_transform.translation;

            if target.active {
                unreal_camera.enabled = false;
                look_transform.target = target_transform.translation;
                // Move the camera there.
                camera_transform.translation = camera_target_position;
                camera_transform.look_at(target_transform.translation, Vec3::Y);

                viewpoint_mappable.forward = camera_transform.rotation;
                return;
            }

            // It's not active, let's draw a gizmo so we can examine it.
            gizmos.ray(
                target_transform.translation,
                direction * target.distance,
                Color::BLUE,
            );
            gizmos.sphere(camera_target_position, Quat::default(), 0.5, Color::BLUE);
        }

        unreal_camera.enabled = true;
    }
}

pub fn project_input_camera(
    mut targets: Query<(
        &mut ViewpointMappedInput,
        &ViewpointMappable,
        &mut PlatformingCharacterControl,
        &Transform,
    )>,
    mut gizmos: Gizmos,
) {
    for (mut input_to_map, orientation, mut control, transform) in targets.iter_mut() {
        let input = Vec3::new(
            input_to_map.move_input.x,
            0.0,
            input_to_map.move_input.y * -1.0,
        );
        let forward = orientation.forward.mul_vec3(input);

        // gizmos.ray(
        //     transform.translation,
        //     orientation.forward.mul_vec3(Vec3::X),
        //     Color::RED,
        // );
        // gizmos.ray(
        //     transform.translation,
        //     orientation.forward.mul_vec3(Vec3::Y),
        //     Color::GREEN,
        // );
        // gizmos.ray(
        //     transform.translation,
        //     orientation.forward.mul_vec3(Vec3::Z),
        //     Color::BLUE,
        // );

        let result = forward.xz().normalize_or_zero() * input.length();
        control.move_input = result;

        input_to_map.move_input = Vec2::ZERO;
    }
}
