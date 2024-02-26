use std::time::Duration;

use bevy::prelude::*;

use crate::components::player::{
    animation::Animated,
    physics::{
        FloorInfo, KinematicCharacterPhysics, PlatformingCharacterControl,
        PlatformingCharacterPhysics,
    },
};

pub fn character_animation(
    mut characters: Query<(&PlatformingCharacterControl, &PlatformingCharacterPhysics)>,
    mut char_anims: Query<(&mut Animated, &mut Transform, &Parent)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
) {
    for (mut anim_state, mut anim_transform, parent) in char_anims.iter_mut() {
        if let Ok((control, physics)) = characters.get(parent.get()) {
            anim_transform.rotation = physics.overall_rotation;

            match physics.air_speed {
                crate::components::player::physics::AirSpeed::Grounded { angle, slope_quat } => {
                    if physics.ground_speed.length() > 1.0 {
                        anim_state.current_animation = 1;
                    } else {
                        anim_state.current_animation = 0;
                    }
                }
                crate::components::player::physics::AirSpeed::InAir(air_speed) => {
                    if air_speed > 0.0 {
                        anim_state.current_animation = 2;
                    } else {
                        anim_state.current_animation = 3;
                    }
                }
            }

            let current = &animations.0[anim_state.current_animation];

            for mut player in animation_players.iter_mut() {
                if !player.is_playing_clip(current) {
                    // switch player
                    info!("switch animation to {:?}", anim_state.current_animation);
                    player
                        .play_with_transition(current.clone_weak(), Duration::from_millis(300))
                        .repeat();
                }
            }

            // anim_transform.rotation = match physics.air_speed {
            //     crate::components::player::physics::AirSpeed::Grounded { angle, slope_quat } => {
            //         kinematic_physics.orientation
            //     }
            //     crate::components::player::physics::AirSpeed::InAir(_) => {
            //         let dir_3d =
            //             Vec3::new(physics.ground_direction.x, 0.0, physics.ground_direction.y);
            //         Quat::from_rotation_arc(Vec3::Z, dir_3d)
            //     }
            // }
        }
    }
}

#[derive(Resource)]
pub struct Animations(Vec<Handle<AnimationClip>>);

pub fn setup_animations(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Animations(vec![
        asset_server.load("degauss.glb#Animation0"),
        asset_server.load("degauss.glb#Animation1"),
        asset_server.load("degauss.glb#Animation2"),
        asset_server.load("degauss.glb#Animation3"),
        asset_server.load("degauss.glb#Animation4"),
    ]));
}
