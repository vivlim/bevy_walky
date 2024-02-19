use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

pub fn fixup_nested_colliders(
    mut nested_colliders: Query<(
        &ColliderParent,
        Entity,
        &ColliderTransform,
        Without<RigidBody>,
        Without<ColliderFixupVisited>,
    )>,
    mut commands: Commands,
    scene_bodies: Query<(&RigidBody, &Transform, Entity, &Children, &Handle<Scene>)>,
) {
    for (mut nc, e, collider_transform, _, _) in nested_colliders.iter_mut() {
        match scene_bodies.get(nc.get()) {
            Ok((parent, parent_transform, collider_parent_entity, _, _)) => {
                /*info!(
                    "Cloning parent rigidbody ({:?} from {:?}) onto {:?}",
                    parent, collider_parent_entity, e
                );
                info!(
                    "child transform {:?}, parent {:?}",
                    collider_transform, parent_transform
                );
                */
                let rb = parent.clone();
                commands.entity(e).insert(rb);
                commands.entity(e).insert(ReapplyColliderTransform {
                    desired: collider_transform.clone(),
                    lgtm_remaining: 5,
                });
            }
            Err(err) => {
                warn!("Failed to get parent rigidbody for {:?}: {:?}", e, err);
            }
        }
        // Mark this entity so we don't visit it again.
        commands.entity(e).insert(ColliderFixupVisited {});
    }
}

// Crudely counteract collider_backend::update_collider_parents' tendency to reset collider transforms after we add a rigidbody to a child
pub fn reapply_collider_transform(
    mut to_reapply_to: Query<(&mut ReapplyColliderTransform, &ColliderTransform, Entity)>,
    mut commands: Commands,
) {
    for (mut reapply, current, entity) in to_reapply_to.iter_mut() {
        // If the transforms are different, reapply
        if reapply.desired != *current {
            /*
            info!(
                "Reapplying collider transform {:?} over {:?} for entity {:?}",
                reapply, current, entity
            ); */
            commands.entity(entity).insert(reapply.desired.clone());
        } else {
            reapply.lgtm_remaining -= 1;
            if (reapply.lgtm_remaining <= 0) {
                commands.entity(entity).remove::<ReapplyColliderTransform>(); // you can rest now
            }
        }
    }
}

#[derive(Component, Reflect)]
pub struct ColliderFixupVisited {}

#[derive(Component, Reflect, Debug)]
pub struct ReapplyColliderTransform {
    desired: ColliderTransform,
    lgtm_remaining: u8,
}
