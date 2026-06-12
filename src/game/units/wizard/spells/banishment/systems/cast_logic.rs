use std::cmp::Ordering;

use super::super::super::vfx;
use super::super::components::{
    BanishmentTalentParams, DimensionalShunt, Displacement, OneWayTrip, PainfulReturn,
};
use super::super::constants;
use super::vfx::spawn_banishment_vfx;
use crate::game::units::components::{BanishedModifier, Corpse, Health, Team, WasBanished};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Returns true if the target is within the wizard's spell range.
pub(super) fn is_in_spell_range(target_pos: Vec3, spell_range: f32, local_origin: Vec3) -> bool {
    let dx = target_pos.x - local_origin.x;
    let dz = target_pos.z - local_origin.z;
    (dx * dx + dz * dz) <= spell_range * spell_range
}

/// Banishes a single target entity, applying talent components as needed.
/// Also spawns the lensing VFX at the target's position.
#[allow(clippy::too_many_arguments)]
pub(super) fn banish_target(
    commands: &mut Commands,
    target: Entity,
    target_pos: Vec3,
    duration: f32,
    params: &BanishmentTalentParams,
    health: &Health,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) {
    // Spawn shrinking lensing VFX at target position
    spawn_banishment_vfx(commands, target_pos, visual_assets, pending);

    vfx::systems::spawn_smoke_poof_synced(
        commands,
        visual_assets,
        pending,
        &visual_assets.banishment_poof,
        crate::networking::snapshot::PoofVariant::Banishment,
        target_pos,
        8,
        time_secs,
    );

    // One-Way Trip: if below HP threshold, mark for death on return
    if params.one_way_trip && health.current <= health.max * constants::ONE_WAY_TRIP_HP_THRESHOLD {
        commands.entity(target).insert((
            BanishedModifier::new(0.0), // Expires immediately next tick
            Visibility::Hidden,
            OneWayTrip,
        ));
        return;
    }

    let mut entity_commands = commands.entity(target);
    entity_commands.insert((BanishedModifier::new(duration), Visibility::Hidden));

    if params.painful_return {
        entity_commands.insert(PainfulReturn {
            damage: constants::PAINFUL_RETURN_DAMAGE,
        });
    }
    if params.displacement {
        entity_commands.insert(Displacement {
            radius: constants::DISPLACEMENT_RADIUS,
        });
    }
    if params.dimensional_shunt {
        entity_commands.insert(DimensionalShunt {
            hp_fraction: constants::DIMENSIONAL_SHUNT_HP_FRACTION,
        });
    }
}

/// Standard single-target (or dual-target) banishment.
#[allow(clippy::too_many_arguments)]
pub(super) fn cast_single_banishment(
    commands: &mut Commands,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    cursor_pos: Vec3,
    empowerment: f32,
    mana: &mut crate::game::units::wizard::components::Mana,
    params: &BanishmentTalentParams,
    spell_range: f32,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    local_origin: Vec3,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
    caster_team: Team,
) -> u32 {
    let duration = params.duration * empowerment;
    let mut banished_count = 0u32;

    // Find candidates within spell range, sorted by distance to cursor
    let mut candidates: Vec<(Entity, f32, Vec3, &Health)> = enemies_query
        .iter()
        .filter(|(_, _, team, _)| caster_team.is_enemy(team))
        .filter(|(_, transform, _, _)| {
            is_in_spell_range(transform.translation, spell_range, local_origin)
        })
        .map(|(entity, transform, _, health)| {
            let xz_dist = crate::game::units::wizard::spells::utils::xz_distance(
                transform.translation,
                cursor_pos,
            );
            (entity, xz_dist * xz_dist, transform.translation, health)
        })
        .collect();
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

    // Banish first target (nearest to cursor)
    if let Some(&(target, _, pos, health)) = candidates.first() {
        banish_target(
            commands,
            target,
            pos,
            duration,
            params,
            health,
            visual_assets,
            time_secs,
            pending,
        );
        banished_count += 1;
    }

    // Dual Banishment: banish second target if we can afford it
    if params.dual_banishment && candidates.len() > 1 {
        let base_mana_cost = constants::MANA_COST * params.mana_mult;
        let second_mana_cost = base_mana_cost * constants::DUAL_BANISHMENT_SECOND_MANA_MULT;
        if mana.consume(second_mana_cost) {
            let (target, _, pos, health) = candidates[1];
            banish_target(
                commands,
                target,
                pos,
                duration,
                params,
                health,
                visual_assets,
                time_secs,
                pending,
            );
            banished_count += 1;
        }
    }

    banished_count
}

/// Mass Banishment: banishes all enemies in a radius. Short duration, high cost.
#[allow(clippy::too_many_arguments)]
pub(super) fn cast_mass_banishment(
    commands: &mut Commands,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    cursor_pos: Vec3,
    empowerment: f32,
    params: &BanishmentTalentParams,
    spell_range: f32,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    local_origin: Vec3,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
    caster_team: Team,
) -> u32 {
    let duration = constants::MASS_BANISHMENT_DURATION * empowerment;
    let mut banished_count = 0u32;

    for (entity, transform, team, health) in enemies_query.iter() {
        if !caster_team.is_enemy(team) {
            continue;
        }
        if !is_in_spell_range(transform.translation, spell_range, local_origin) {
            continue;
        }
        let xz_dist = crate::game::units::wizard::spells::utils::xz_distance(
            transform.translation,
            cursor_pos,
        );
        if xz_dist * xz_dist > constants::MASS_BANISHMENT_RADIUS * constants::MASS_BANISHMENT_RADIUS
        {
            continue;
        }

        banish_target(
            commands,
            entity,
            transform.translation,
            duration,
            params,
            health,
            visual_assets,
            time_secs,
            pending,
        );
        banished_count += 1;
    }

    banished_count
}
