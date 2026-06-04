use bevy::prelude::*;

use super::{
    components::ArcanoRouterBonuses,
    messages::SliderAdjustMessage,
    resources::{ArcanoRouterSetupBaseline, ArcanoRouterState, SliderType},
};
use crate::game::units::wizard::components::Wizard;

/// Processes slider adjustment messages and updates the allocation state.
///
/// This system reads `SliderAdjustMessage` events and applies the requested
/// adjustments to the `ArcanoRouterState` resource, which automatically
/// handles redistribution and normalization.
///
/// During the multiplayer setup stage the Range allocation is pinned to its
/// match-start baseline: Range-targeted adjustments are ignored, and any rise in
/// range caused by adjusting the other sliders is reverted (range can never grow
/// past the baseline while setup is active). Mana/power/speed stay adjustable.
pub(super) fn update_allocations(
    mut state: ResMut<ArcanoRouterState>,
    mut reader: MessageReader<SliderAdjustMessage>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    kill_stats: Res<crate::game::resources::KillStats>,
    baseline: Option<Res<ArcanoRouterSetupBaseline>>,
) {
    let in_setup = session.is_some()
        && kill_stats.elapsed_time < crate::game::run_conditions::MP_SETUP_DURATION;

    for message in reader.read() {
        if in_setup {
            // Block range from being pumped up during the setup stage.
            if message.slider == SliderType::Range {
                continue;
            }
            state.adjust_allocation(message.slider, message.delta);
            // Adjusting another slider redistributes into range (up OR down);
            // re-pin it to the baseline in either direction so range stays put.
            if let Some(ref baseline) = baseline
                && (state.range_allocation - baseline.0).abs() > 0.01
            {
                state.range_allocation = baseline.0;
                state.normalize_excluding_range();
            }
        } else {
            state.adjust_allocation(message.slider, message.delta);
        }
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
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    const BASE_SPELL_RANGE: f32 = 3000.0;

    // Multiplayer grants every wizard +5% spell range. The Arcanorouter fully
    // owns its range here, so the buff must be re-applied each recompute.
    let mp_range_mult = if session.is_some() {
        crate::game::units::wizard::constants::MP_SPELL_RANGE_MULTIPLIER
    } else {
        1.0
    };

    for (mut wizard, bonuses) in wizard_query.iter_mut() {
        // Apply bonuses to wizard stats
        wizard.spell_range = BASE_SPELL_RANGE * bonuses.get_range_multiplier() * mp_range_mult;
        wizard.mana_cost_multiplier = bonuses.get_mana_cost_multiplier();
        wizard.spell_power_multiplier = bonuses.get_spell_power_multiplier();
        wizard.cast_speed_multiplier = bonuses.get_cast_speed_multiplier();
    }
}

/// Resets allocations to default when game ends.
///
/// This ensures each new game starts with balanced allocations.
pub(super) fn reset_allocations_on_game_over(mut state: ResMut<ArcanoRouterState>) {
    state.reset();
}
