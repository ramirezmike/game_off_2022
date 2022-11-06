use bevy::prelude::*;

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
//        app.add_system(link_animations);
    }
}

#[derive(Component)]
pub struct AnimationLink {
    pub entity: Option::<Entity>,
}

fn link_animations(
    mut animation_links: Query<(&mut AnimationLink, &Children)>,
    animations: Query<(&Parent, Entity), With<AnimationPlayer>>,
) {
    for (mut link, children) in &mut animation_links {
        let is_none = link.entity.is_none();

        if is_none {
            for child in children {
                for (parent, entity) in &animations {
                    if parent.get() == *child {
                        link.entity = Some(entity);
                    }
                }
            }
        }
    }
}
