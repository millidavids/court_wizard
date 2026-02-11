use bevy::prelude::*;

use super::{
    components::ArcanoRouterBonuses, messages::SliderAdjustMessage, resources::ArcanoRouterState,
};
use crate::game::units::wizard::components::{PrimedSpell, Wizard};

/// Processes slider adjustment messages and updates the allocation state.
///
/// This system reads `SliderAdjustMessage` events and applies the requested
/// adjustments to the `ArcanoRouterState` resource, which automatically
/// handles redistribution and normalization.
pub(super) fn update_allocations(
    mut state: ResMut<ArcanoRouterState>,
    mut reader: MessageReader<SliderAdjustMessage>,
) {
    for message in reader.read() {
        state.adjust_allocation(message.slider, message.delta);
    }
}

/// Syncs the ArcanoRouterState to the ArcanoRouterBonuses component.
///
/// This system updates the bonuses component whenever the allocation state changes.
pub(super) fn sync_state_to_bonuses(
    state: Res<ArcanoRouterState>,
    mut bonuses_query: Query<&mut ArcanoRouterBonuses>,
) {
    // Only update when allocations actually change
    if !state.is_changed() {
        return;
    }

    for mut bonuses in bonuses_query.iter_mut() {
        bonuses.range_bonus = state.range_allocation;
        bonuses.mana_cost_bonus = state.mana_allocation;
        bonuses.spell_power_bonus = state.power_allocation;
        bonuses.cast_speed_bonus = state.speed_allocation;
    }
}

/// Applies ArcanoRouterBonuses to the Wizard's stats.
///
/// This is the single source of truth - all wizard stats are calculated here
/// by applying bonuses to base values.
pub(super) fn apply_bonuses_to_wizard_stats(
    mut wizard_query: Query<(&mut Wizard, &ArcanoRouterBonuses), Changed<ArcanoRouterBonuses>>,
) {
    const BASE_SPELL_RANGE: f32 = 3000.0;

    for (mut wizard, bonuses) in wizard_query.iter_mut() {
        // Apply bonuses to wizard stats
        wizard.spell_range = BASE_SPELL_RANGE * bonuses.get_range_multiplier();
        wizard.mana_cost_multiplier = bonuses.get_mana_cost_multiplier();
        wizard.spell_power_multiplier = bonuses.get_spell_power_multiplier();
        wizard.cast_speed_multiplier = bonuses.get_cast_speed_multiplier();
    }
}

/// Applies the wizard's multipliers to the primed spell.
///
/// This system runs whenever the Wizard component changes, recalculating
/// the effective cast time and empowerment based on the wizard's multipliers.
pub(super) fn apply_wizard_stats_to_primed_spell(
    mut wizard_query: Query<(&Wizard, &mut PrimedSpell), Changed<Wizard>>,
) {
    for (wizard, mut primed_spell) in wizard_query.iter_mut() {
        // Get base values from spell config
        let base_cast_time = primed_spell.spell.primed_config().cast_time;

        // Apply wizard's cast speed multiplier to cast time
        primed_spell.cast_time = base_cast_time / wizard.cast_speed_multiplier;

        // Apply wizard's spell power multiplier to empowerment
        primed_spell.empowerment = wizard.spell_power_multiplier;
    }
}

/// Resets allocations to default when game ends.
///
/// This ensures each new game starts with balanced allocations.
pub(super) fn reset_allocations_on_game_over(mut state: ResMut<ArcanoRouterState>) {
    state.reset();
}
