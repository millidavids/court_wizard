//! Staging shield: attackers still marching to their staging points are
//! immune to spells (enforced by `Without<StagingAttacker>` filters across
//! the spell systems). The player-facing indicator is `StagingShieldGlow`,
//! a dedicated visual-only marker rendered by `update_persistent_effect_visuals`
//! with the exact same golden pulse as the shielder's blessing — but with no
//! gameplay coupling: unlike `ShielderDamageReduction` it grants no damage
//! reduction, and clearing it can never race with a real shielder's shield
//! landing on the activation frame.
//!
//! Corpses keep `StagingAttacker` forever (the wave-activation removal loop
//! is `Without<Corpse>`-gated), so `RemovedComponents` never fires for units
//! that die while staging — death cleanup strips the glow on corpse
//! conversion instead (covers lava deaths), and resurrection despawns into a
//! fresh unit without it.

use bevy::prelude::*;

use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::Corpse;

/// Visual-only marker: golden shielder-style pulse on a staging attacker.
#[derive(Component)]
pub(in crate::game) struct StagingShieldGlow;

/// Gives newly staging attackers the shield glow.
pub(super) fn apply_staging_shield_glow(
    mut commands: Commands,
    new_stagers: Query<Entity, (Added<StagingAttacker>, Without<Corpse>)>,
) {
    for unit in &new_stagers {
        if let Ok(mut ec) = commands.get_entity(unit) {
            ec.try_insert(StagingShieldGlow);
        }
    }
}

/// Clears the glow when a wave activates (`check_wave_activation` removes
/// `StagingAttacker` from every living unit of the wave in one frame).
pub(super) fn clear_staging_shield_glow(
    mut commands: Commands,
    mut removed: RemovedComponents<StagingAttacker>,
) {
    for unit in removed.read() {
        if let Ok(mut ec) = commands.get_entity(unit) {
            ec.try_remove::<StagingShieldGlow>();
        }
    }
}

/// Dev-only regression net: no spell effect (harmful or beneficial) should
/// ever land on a staging unit. Plain `apply_damage_to_unit` paths leave no
/// marker, so this can't catch everything — the query filters are the actual
/// enforcement; this just surfaces future misses early in playtests.
///
/// `TemporaryHitPoints` is deliberately absent: the Fortified Horde challenge
/// legitimately stamps it on wave-0 attackers before they're tagged as
/// staging, so it would cry wolf for the whole opening march.
#[cfg(debug_assertions)]
#[allow(clippy::type_complexity)]
pub(super) fn warn_on_staging_spell_effects(
    stagers: Query<
        (
            Entity,
            (
                Has<crate::game::units::components::SpellDamaged>,
                Has<crate::game::units::components::PendingDamageEffect>,
                Has<crate::game::units::components::FireDoT>,
                Has<crate::game::units::components::Shocked>,
                Has<crate::game::units::components::Airborne>,
                Has<crate::game::units::components::MindControlled>,
                Has<crate::game::units::components::SleepModifier>,
                Has<crate::game::units::components::RootedModifier>,
                Has<crate::game::units::components::PolymorphedModifier>,
                Has<crate::game::units::components::BanishedModifier>,
            ),
            (
                Has<crate::game::units::king::components::SpellShield>,
                Has<crate::game::units::components::MarkedForDeathModifier>,
                Has<crate::game::units::components::HasteModifier>,
                Has<crate::game::units::components::BattleHymnModifier>,
                Has<crate::game::units::components::BerserkerRageModifier>,
                Has<crate::game::units::components::FogEvasionModifier>,
                Has<crate::game::cauldron::components::CauldronSpeedModifier>,
            ),
        ),
        (With<StagingAttacker>, Without<Corpse>),
    >,
) {
    for (
        entity,
        (damaged, pending, fire, shocked, airborne, mind, sleep, rooted, poly, banish),
        (shield, marked, haste, hymn, rage, evasion, brew_speed),
    ) in &stagers
    {
        let leaks = [
            (damaged, "SpellDamaged"),
            (pending, "PendingDamageEffect"),
            (fire, "FireDoT"),
            (shocked, "Shocked"),
            (airborne, "Airborne"),
            (mind, "MindControlled"),
            (sleep, "SleepModifier"),
            (rooted, "RootedModifier"),
            (poly, "PolymorphedModifier"),
            (banish, "BanishedModifier"),
            (shield, "SpellShield"),
            (marked, "MarkedForDeathModifier"),
            (haste, "HasteModifier"),
            (hymn, "BattleHymnModifier"),
            (rage, "BerserkerRageModifier"),
            (evasion, "FogEvasionModifier"),
            (brew_speed, "CauldronSpeedModifier"),
        ];
        for (present, name) in leaks {
            if present {
                warn!("staging-immunity leak: {name} landed on staging unit {entity:?}");
            }
        }
    }
}
