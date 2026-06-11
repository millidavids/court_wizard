//! Undead raise and awaiting-release cleanup logic.

use super::super::components::*;
use crate::game::units::components::{Corpse, PermanentCorpse, Team};
use bevy::prelude::*;

/// Computes talent-modified parameters for Finger of Death.
pub fn process_pending_undead_raises(
    mut commands: Commands,
    pending: Option<Res<PendingUndeadRaise>>,
    corpse_query: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: Option<Res<crate::game::units::undead::resources::UndeadAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(pending) = pending else {
        return;
    };
    let Some(undead_assets) = undead_assets else {
        commands.remove_resource::<PendingUndeadRaise>();
        return;
    };

    use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
    use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;

    let mut raised_entities = Vec::new();
    for kill_pos in &pending.kill_positions {
        let mut best: Option<(Entity, f32)> = None;
        for (entity, transform) in corpse_query.iter() {
            if raised_entities.contains(&entity) {
                continue;
            }
            let dist = transform.translation.distance(*kill_pos);
            if dist < 50.0
                && (best.is_none() || dist < best.as_ref().map(|(_, d)| *d).unwrap_or(f32::MAX))
            {
                best = Some((entity, dist));
            }
        }
        if let Some((corpse_entity, _)) = best {
            raised_entities.push(corpse_entity);
            crate::game::units::systems::resurrect_corpse_as_infantry(
                &mut commands,
                corpse_entity,
                *kill_pos,
                Team::Undead,
                UNIT_HEALTH,
                UNIT_MOVEMENT_SPEED * 0.5,
                UNDEAD_SPRITE_TINT,
                undead_assets.sprite_texture.clone(),
                undead_assets.sprite_mesh.clone(),
                &mut materials,
                Some(undead_assets.death_texture.clone()),
            );
        }
    }

    commands.remove_resource::<PendingUndeadRaise>();
}

/// Removes AwaitingFingerOfDeathRelease when the mouse is no longer held.
/// This runs independently of the casting system's run conditions.
pub fn clear_awaiting_fod_release(
    mut commands: Commands,
    mouse_held: Res<crate::game::input::components::MouseLeftHeldThisFrame>,
    query: Query<Entity, With<AwaitingFingerOfDeathRelease>>,
) {
    if mouse_held.held {
        return;
    }
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<AwaitingFingerOfDeathRelease>();
    }
}
