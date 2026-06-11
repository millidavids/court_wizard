use bevy::prelude::*;

use crate::game::units::components::{Health, Team};
use crate::game::units::wizard::components::Wizard;

/// Resource representing a pending heal to be applied to the nearest injured defender.
///
/// Used by spells that deal damage and heal defenders as a side-effect (e.g., Void Siphon,
/// Siphon Life). The heal is deferred to a separate system to avoid query conflicts when
/// both the damage system and healing system need mutable access to Health components.
#[derive(Resource)]
pub(crate) struct PendingDefenderHeal {
    /// Total heal amount to apply.
    pub amount: f32,
    /// World-space origin position (for finding the nearest injured defender).
    pub origin: Vec3,
}

/// Applies wizard spell healing to a unit and records it for the multiplayer
/// score screen's per-wizard heal tally. Returns the HP actually restored
/// (after `healing_reduction` and clamping to `health.max`).
///
/// Call this from wizard heal spells instead of `health.heal(amount)`. Non-spell
/// heal sources (healer units, cauldron regen, melee lifesteal) keep calling
/// `health.heal` directly so they are excluded from the wizard's stat. Like the
/// damage tally, this only fires on the peer running the spell, so it captures
/// exactly the local wizard's healing output.
pub(crate) fn apply_spell_heal(
    commands: &mut Commands,
    entity: Entity,
    health: &mut Health,
    amount: f32,
) -> f32 {
    let before = health.current;
    health.heal(amount);
    let actual = health.current - before;
    if actual > 0.0 {
        // `entry` + `and_modify` accumulates multiple same-frame heals on one
        // unit instead of overwriting; the score-screen consumer clears it each frame.
        commands
            .entity(entity)
            .entry::<crate::game::units::spell_stats::SpellHealTally>()
            .and_modify(move |mut tally| tally.0 += actual)
            .or_insert(crate::game::units::spell_stats::SpellHealTally(actual));
    }
    actual
}

/// Applies a pending defender heal to the nearest injured defender, then removes the resource.
///
/// Finds the closest defender (by world-space distance to `origin`) that is alive but not
/// at full health, and heals them for the pending amount.
pub(crate) fn apply_pending_defender_heal(
    mut commands: Commands,
    pending: Option<Res<PendingDefenderHeal>>,
    mut defenders: Query<(Entity, &Transform, &mut Health, &Team), Without<Wizard>>,
) {
    let Some(heal) = pending else {
        return;
    };

    let mut best: Option<(Entity, f32)> = None;
    for (entity, transform, health, team) in defenders.iter() {
        if *team != Team::Defenders || health.current <= 0.0 || health.current >= health.max {
            continue;
        }
        let dist = transform.translation.distance(heal.origin);
        match best {
            None => best = Some((entity, dist)),
            Some((_, best_dist)) if dist < best_dist => best = Some((entity, dist)),
            _ => {}
        }
    }

    if let Some((entity, _)) = best
        && let Ok((_, _, mut health, _)) = defenders.get_mut(entity)
    {
        apply_spell_heal(&mut commands, entity, &mut health, heal.amount);
    }

    commands.remove_resource::<PendingDefenderHeal>();
}
