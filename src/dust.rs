use bevy::prelude::*;
use crate::{
    assets::GameAssets,
    billboard,
    ingame,
    cutscene,
};
use rand::Rng;

pub struct DustPlugin;
impl Plugin for DustPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DustSpawnEvent>()
           .add_system(handle_dust)
           .add_system(handle_emitters)
           .add_system(handle_dust_spawn_event);
    }
}

pub struct DustSpawnEvent {
    pub position: Vec3,
    pub count: usize,
    pub rate: f32,
    pub emitter_time_to_live: f32,
    pub dust_time_to_live: f32,
    pub spread: f32,
    pub size: f32,
    pub speed: f32,
    pub image: Handle<Image>,
}

impl Default for DustSpawnEvent {
    fn default() -> Self {
        DustSpawnEvent {
            position: Vec3::default(),
            count: 1,
            rate: 0.0,
            emitter_time_to_live: 0.0,
            dust_time_to_live: 3.0,
            spread: 0.0,
            speed: 1.0,
            size: 1.0,
            image: Handle::<Image>::default(),
        }
    }
}

#[derive(Component)]
struct DustEmitter {
    pub count: usize,
    pub current_rate: f32,
    pub rate: f32,
    pub current_life_time: f32,
    pub time_to_live: f32,
    pub dust_time_to_live: f32,
    pub spread: f32,
    pub speed: f32,
    pub size: f32,
    pub image: Handle<Image>,
}

#[derive(Component)]
struct Dust {
    speed: f32,
    current_life_time: f32,
    time_to_live: f32,
}

fn handle_dust_spawn_event(
    mut commands: Commands,
    mut dust_spawn_event_reader: EventReader<DustSpawnEvent>,
) {
    for event in dust_spawn_event_reader.iter() {
        commands.spawn(
            (Transform::from_xyz(event.position.x, event.position.y, event.position.z),
            DustEmitter {
                count: event.count,
                current_rate: 0.0,
                rate: event.rate,
                speed: event.speed,
                current_life_time: 0.0,
                time_to_live: event.emitter_time_to_live,
                dust_time_to_live: event.dust_time_to_live,
                spread: event.spread,
                size: event.size,
                image: event.image.clone(),
            },
            cutscene::CutsceneCleanupMarker,
            ingame::CleanupMarker
            )
        );
    }
}

fn handle_dust(
    mut commands: Commands,
    mut dusts: Query<(Entity, &mut Transform, &mut Dust, &Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut dust, material_handle) in &mut dusts {
        dust.current_life_time += time.delta_seconds();
        if dust.current_life_time > dust.time_to_live {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        transform.translation.y += dust.speed * time.delta_seconds();
        if let Some(mut material) = materials.get_mut(material_handle) {
            material.alpha_mode = AlphaMode::Blend;
            material.base_color = Color::Rgba { 
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: (1.0 - dust.current_life_time / dust.time_to_live),
            };
        }
    }
}

fn handle_emitters(
    mut commands: Commands,
    mut emitters: Query<(Entity, &Transform, &mut DustEmitter)>,
    game_assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, transform, mut emitter) in &mut emitters {
        let mut rng = rand::thread_rng();

        emitter.current_rate -= time.delta_seconds();
        emitter.current_rate = emitter.current_rate.clamp(0.0, emitter.rate);

        if emitter.current_rate <= 0.0 {
            let material = materials.add(StandardMaterial {
                               base_color_texture: Some(emitter.image.clone()),
                               alpha_mode: AlphaMode::Blend,
                               ..Default::default()
                           });

            for _ in 0..emitter.count {
                let (random_x, random_z) =
                    if emitter.spread > 0.0 {
                        (rng.gen_range(0.0..emitter.spread), rng.gen_range(0.0..emitter.spread))
                    } else {
                        (0.0, 0.0)
                    };
                let x = transform.translation.x + random_x;
                let z = transform.translation.z + random_z;
                let mut transform = Transform::from_xyz(x, transform.translation.y, z);
                transform.scale *= emitter.size; 

                commands
                    .spawn(
                        SpatialBundle::from_transform(transform)
                    )
                    .with_children(|parent| {
                        parent.spawn(PbrBundle {
                            mesh: game_assets.dust.clone(),
                            material: material.clone(),
                            transform: Transform::from_rotation(
                                Quat::from_axis_angle(Vec3::X, (3.0 * std::f32::consts::PI) / 2.0)),
                            ..Default::default()
                        })
                        .insert(bevy::pbr::NotShadowCaster)
                        .insert(Dust {
                            speed: emitter.speed,
                            current_life_time: 0.0,
                            time_to_live: emitter.dust_time_to_live,
                        });
                    })
                .insert(billboard::Billboard)
                .insert(cutscene::CutsceneCleanupMarker)
                .insert(ingame::CleanupMarker);
            }

            emitter.current_rate = emitter.rate;
        }

        emitter.current_life_time += time.delta_seconds();
        if emitter.current_life_time >= emitter.time_to_live {
            commands.entity(entity).despawn_recursive();
        }
    }
}
