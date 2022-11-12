use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy_rapier3d::prelude::*;
use bevy::ecs::component::StorageType;

pub struct PropsPlugin;
impl Plugin for PropsPlugin {
    fn build(&self, app: &mut App) {
//      app.add_system_set(
//         );
    }
}


pub trait ComponentAdder {
    fn add_components(entity_commands: &mut EntityCommands);
}


#[derive(Component)]
pub struct Plate;
#[derive(Component)]
pub struct BrokenPlate;

impl ComponentAdder for Plate {
    fn add_components(entity_commands: &mut EntityCommands) {
        entity_commands
            .insert(Restitution::coefficient(0.9))
            .insert(ColliderMassProperties::Density(0.01))
            .insert(Collider::cuboid(0.3, 0.05, 0.3))
            .insert(Velocity::default())
            .insert(Plate)
            //.insert(DampPhysics(2.0))
            .insert(RigidBody::Dynamic);
    }
}


impl ComponentAdder for BrokenPlate {
    fn add_components(entity_commands: &mut EntityCommands) {
        entity_commands
            .insert(Restitution::coefficient(0.9))
            .insert(ColliderMassProperties::Density(0.01))
            .insert(Collider::cuboid(0.3, 0.05, 0.3))
            .insert(Velocity::default())
            .insert(BrokenPlate)
            //.insert(DampPhysics(2.0))
            .insert(RigidBody::Dynamic);
    }
}
