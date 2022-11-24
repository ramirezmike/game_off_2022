use bevy::prelude::*;
use crate::props;

pub struct GroupPlugin;
impl Plugin for GroupPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_groups)
           .add_event::<RestoreGroupEvent>()
           .add_system(restore_group_handler);
    }
}

#[derive(Component)]
pub struct GroupMarker(pub usize);

#[derive(Component, Debug)]
pub struct GroupMember {
    pub group_id: usize,
    pub original_global_transform: Transform,
    pub original_transform: Transform
}

pub struct RestoreGroupEvent {
    pub group_id: usize,
}

fn set_groups(
    mut commands: Commands,
    markers: Query<(Entity, &Transform, &GlobalTransform, &GroupMarker)>,
) {
    for (entity, transform, global_transform, marker) in &markers {
        println!("found marker!!!!");
        commands.entity(entity)
                .remove::<GroupMarker>()
                .insert(GroupMember {
                    group_id: marker.0,
                    original_global_transform: global_transform.compute_transform(), 
                    original_transform: transform.clone(),
                });
    }
}

fn restore_group_handler(
    mut commands: Commands,
    mut restore_group_event_handler: EventReader<RestoreGroupEvent>,
    mut group_members: Query<(Entity, &mut Transform, &GroupMember)>,
) {
    for group in restore_group_event_handler.iter() {
        for (entity, mut transform, group_member) in &mut group_members {
            if group_member.group_id == group.group_id {
                *transform = group_member.original_transform;

                let mut entity_commands = commands.entity(entity);
                props::restore_dynamic_rapier_components(&mut entity_commands);
            }
        }
    }
}
