use bevy::prelude::*;

use crate::components::camera::MyCameraMarker;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 12.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                ..default()
            }),
            ..default()
        },
        MyCameraMarker,
    ));
}
