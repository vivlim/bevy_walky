use bevy::prelude::*;
use smooth_bevy_cameras::{
    controllers::{
        orbit::{OrbitCameraBundle, OrbitCameraController},
        unreal::{UnrealCameraBundle, UnrealCameraController},
    },
    LookTransform, LookTransformBundle, Smoother,
};

use crate::components::camera::{MyCameraMarker, OrbitCameraTarget};

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(UnrealCameraBundle::new(
            smooth_bevy_cameras::controllers::unreal::UnrealCameraController::default(),
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::Y,
        ))
        .insert(Camera3dBundle::default());
}

pub fn update_camera(
    mut cameras: Query<(
        &mut UnrealCameraController,
        &mut LookTransform,
        &mut Transform,
        Without<OrbitCameraTarget>,
    )>,
    targets: Query<(&OrbitCameraTarget, &Transform, Without<LookTransform>)>,
    mut gizmos: Gizmos,
) {
    let mut last_target: Option<&OrbitCameraTarget> = None;
    for (mut unreal_camera, mut look_transform, mut camera_transform, _) in cameras.iter_mut() {
        for (target, target_transform, _) in targets.iter() {
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
