use super::super::components::EntangleGroundEffect;
use super::super::constants;
use super::root_effects::apply_entangle_to_unit;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::multiplayer::components::{GhostEntity, GhostSpellEffect};
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{Corpse, RootedModifier, Team};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Ticks entangle ground effect timer and handles Overgrowth expansion.
pub fn tick_entangle_ground_effect(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut effects: Query<&mut EntangleGroundEffect>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let delta = time.delta_secs();
    let mote_interval = vfx::constants::MOTE_SPAWN_INTERVAL;
    let mote_count = vfx::constants::MOTE_COUNT_PER_SPAWN;

    for mut effect in &mut effects {
        let prev_remaining = effect.time_remaining;
        effect.time_remaining -= delta;

        let prev_elapsed = effect.duration - prev_remaining;
        let curr_elapsed = effect.duration - effect.time_remaining;
        if effect.time_remaining > 0.0
            && (curr_elapsed / mote_interval).floor() != (prev_elapsed / mote_interval).floor()
        {
            vfx::systems::spawn_floating_motes_synced(
                &mut commands,
                &visual_assets,
                &mut pending_cast_events,
                &visual_assets.nature_mote,
                crate::networking::snapshot::MoteMaterial::Nature,
                effect.center,
                effect.current_radius,
                mote_count,
                time.elapsed_secs(),
            );
        }

        // Overgrowth: expand zone over its lifetime
        if effect.talent_params.overgrowth {
            let progress = (effect.time_remaining / effect.duration).max(0.0);
            let elapsed_fraction = 1.0 - progress;
            let growth =
                effect.base_radius * constants::OVERGROWTH_GROWTH_FRACTION * elapsed_fraction;
            effect.current_radius = effect.base_radius + growth;
        }
    }
}

/// Overgrowth: periodically root new units entering the expanding zone.
pub fn overgrowth_root_new_units(
    time: Res<Time>,
    mut commands: Commands,
    // Gameplay is host-authoritative: skip the guest's mirror of a remote zone
    // (`GhostSpellEffect`) and never root ghost units (`GhostEntity`). The guest
    // forwards its own cast-time roots; re-rooting must not double-apply.
    mut effects: Query<&mut EntangleGroundEffect, Without<GhostSpellEffect>>,
    targets: Query<
        (Entity, &Transform, &Team, Option<&RootedModifier>),
        (
            Without<Wizard>,
            Without<Corpse>,
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut defender_hit_msg: MessageWriter<EntangleHitDefenderMessage>,
) {
    let delta = time.delta_secs();
    for mut effect in &mut effects {
        if !effect.talent_params.overgrowth {
            continue;
        }
        effect.overgrowth_check_timer += delta;
        if effect.overgrowth_check_timer < constants::OVERGROWTH_CHECK_INTERVAL {
            continue;
        }
        effect.overgrowth_check_timer -= constants::OVERGROWTH_CHECK_INTERVAL;

        let remaining_duration = effect.time_remaining;
        if remaining_duration <= 0.0 {
            continue;
        }

        let talent_params = effect.talent_params;
        let center = effect.center;
        let radius = effect.current_radius;

        for (entity, transform, team, rooted) in &targets {
            if rooted.is_some() {
                continue;
            }
            let distance = transform.translation.distance(center);
            if distance <= radius {
                apply_entangle_to_unit(
                    &mut commands,
                    entity,
                    team,
                    remaining_duration,
                    &talent_params,
                    &mut defender_hit_msg,
                );
            }
        }
    }
}

/// Despawns expired entangle ground effects.
pub fn cleanup_entangle_ground_effect(
    mut commands: Commands,
    effects: Query<(Entity, &EntangleGroundEffect)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, effect) in &effects {
        if effect.time_remaining <= 0.0 {
            // Notify pathfinding that this zone is removed
            let buffered_radius = effect.current_radius + OBSTACLE_BUFFER;
            let origin_2d = Vec2::new(effect.center.x, effect.center.z);
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}
