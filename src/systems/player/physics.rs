use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn read_result_system(controllers: Query<(Entity, &KinematicCharacterControllerOutput)>) {
    for (entity, output) in controllers.iter() {
        println!(
            "Entity {:?} moved by {:?} and touches the ground: {:?}",
            entity, output.effective_translation, output.grounded
        );
    }
}

pub fn character_movement(
    mut controllers: Query<&mut KinematicCharacterController>,
    keys: Res<Input<KeyCode>>,
) {
    for mut controller in controllers.iter_mut() {
        if keys.just_pressed(KeyCode::Space) {
            // jump
            controller.translation = Some(Vec3::new(0.0, 1.5, 0.0));
        } else {
            // todo: apply gravity instead of just setting this
            controller.translation = Some(Vec3::new(0.0, -0.5, 0.0));
        }
    }
}
