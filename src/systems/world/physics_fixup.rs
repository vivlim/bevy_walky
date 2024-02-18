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
                info!(
                    "Cloning parent rigidbody ({:?} from {:?}) onto {:?}",
                    parent, collider_parent_entity, e
                );
                info!(
                    "child transform {:?}, parent {:?}",
                    collider_transform, parent_transform
                );
                let rb = parent.clone();
                commands.entity(e).insert(rb);
                commands.entity(e).insert(collider_transform.clone());
            }
            Err(err) => {
                warn!("Failed to get parent rigidbody for {:?}: {:?}", e, err);
            }
        }
        // Mark this entity so we don't visit it again.
        commands.entity(e).insert(ColliderFixupVisited {});
    }
}

#[derive(Component, Reflect)]
pub struct ColliderFixupVisited {}
