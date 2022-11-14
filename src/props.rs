use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy_rapier3d::prelude::*;
use bevy::ecs::component::StorageType;
use crate::{
    assets,
    AppState,
};
use bevy::gltf::Gltf;
use bevy_scene_hook::{SceneHook, HookedSceneBundle};

const PROP_BREAK_THRESHOLD: f32 = 0.13;

pub struct PropsPlugin;
impl Plugin for PropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(handle_breakables)
           );
    }
}


pub trait ComponentAdder {
    fn add_components(entity_commands: &mut EntityCommands);
}

#[derive(Component)]
pub struct Breakable {
    breakable_type: BreakableType,
}
#[derive(Component)]
pub enum BreakableType {
    Plate,
    Mug,
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
            .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
            .insert(ContactForceEventThreshold(PROP_BREAK_THRESHOLD))
            .insert(Breakable {
                breakable_type: BreakableType::Plate,
            })
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

fn handle_breakables(
    mut commands: Commands,
    breakables: Query<(&Breakable, &Transform, &Velocity)>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    assets_gltf: Res<Assets<Gltf>>,
    game_assets: Res<assets::GameAssets>,
) {
    for e in contact_force_events.iter() {
        println!("contact force event {:?}", e.total_force_magnitude);
        [e.collider1, e.collider2].iter()
            .for_each(|entity| {
                if let Ok((breakable, transform, velocity)) = breakables.get(*entity) {
                    println!("Got breakable {:?} {:?}", transform, velocity);
                    commands.entity(*entity).despawn_recursive();
                    let transform = transform.clone();
                    let velocity = velocity.clone();
                    let (asset, mesh_name, adder) = match breakable.breakable_type {
                        BreakableType::Plate => (&game_assets.broken_plate, "plate", BrokenPlate::add_components),
                        BreakableType::Mug => (&game_assets.broken_mug, "mug", BrokenPlate::add_components),
                    };

                    if let Some(gltf) = assets_gltf.get(asset) {
                        commands.spawn_bundle(HookedSceneBundle {
                            scene: SceneBundle { scene: gltf.scenes[0].clone(), ..default() },
                            hook: SceneHook::new(move |entity, cmds, _| {
                                if let Some(name) = entity.get::<Name>().map(|t|t.as_str()) {
                                    if name.contains(mesh_name) {
                                        adder(cmds); 
                                        cmds.insert(velocity.clone());
                                        cmds.insert(transform.clone());
                                    }
                                }
                            })
                        });
                    }
                }
            });
    }
}
