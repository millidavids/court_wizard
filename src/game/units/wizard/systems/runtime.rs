use bevy::math::Affine2;
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants;
use super::super::messages::*;
use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::components::ConcentrationSpell;
use crate::game::input::MouseButtonState;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Updates the wizard sprite sheet animation.
pub fn update_wizard_animation(
    time: Res<Time>,
    mut wizard_query: Query<
        (&mut WizardAnimation, &MeshMaterial3d<StandardMaterial>),
        With<Wizard>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const FRAME_SCALE: f32 = 1.0 / constants::WIZARD_SPRITE_GRID_SIZE as f32;
    let frame_scale_vec = Vec2::splat(FRAME_SCALE);

    for (mut animation, material_handle) in &mut wizard_query {
        if animation.tick(time.delta_secs())
            && let Some(material) = materials.get_mut(material_handle)
        {
            let (offset_x, offset_y) = animation.uv_offset();
            material.uv_transform = Affine2::from_scale_angle_translation(
                frame_scale_vec,
                0.0,
                Vec2::new(offset_x, offset_y),
            );
        }
    }
}

/// Regenerates wizard mana over time, scaled by cauldron buffs.
/// Mana is capped at `max - reserved` where reserved = sum of active concentration spell costs.
/// Skipped entirely when the Mana Drought toggle is active.
pub fn regenerate_mana(
    time: Res<Time>,
    cauldron_buffs: Res<CauldronBuffs>,
    #[cfg(debug_assertions)] infinite_mana: Res<crate::ui::action_bar::InfiniteMana>,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    mut wizards: Query<(&mut Mana, &ManaRegen), With<Wizard>>,
    concentration_spells: Query<&ConcentrationSpell>,
) {
    let reserved: f32 = concentration_spells.iter().map(|c| c.mana_cost).sum();

    #[cfg(debug_assertions)]
    let cheat_active = infinite_mana.0;
    #[cfg(not(debug_assertions))]
    let cheat_active = false;

    // Mana Drought: no passive regen (mana comes from kills instead)
    if active_toggles.as_ref().is_some_and(|t| {
        t.is_active(crate::game::game_mode::components::ToggleModifier::ManaDrought)
    }) {
        // Still apply infinite mana cheat if active
        if cheat_active {
            for (mut mana, _) in &mut wizards {
                let effective_max = (mana.max - reserved).max(0.0);
                mana.current = effective_max;
            }
        }
        return;
    }

    for (mut mana, regen) in &mut wizards {
        let effective_max = (mana.max - reserved).max(0.0);
        if cheat_active {
            mana.current = effective_max;
        } else {
            let rate = regen.rate * cauldron_buffs.mana_regen_multiplier();
            mana.regenerate(rate * time.delta_secs());
            mana.current = mana.current.min(effective_max);
        }
    }
}

/// Spell Rotation: detects completed spell casts by watching for mana consumption
/// followed by returning to CastingState::Resting. This two-step approach ensures
/// channeled/multi-step spells aren't blocked mid-cast.
///
/// Phase 1: When mana decreases significantly (> 1.0), record the spell as "pending".
/// Phase 2: When CastingState returns to Resting with a pending spell, commit it
///          as the last cast spell (blocking re-cast via spell_is_primed).
pub fn track_spell_casts_for_rotation(
    wizard_query: Query<(&Mana, &CastingState, Option<&PrimedSpell>), With<Wizard>>,
    mut last_cast: Option<ResMut<crate::game::game_mode::components::LastCastSpell>>,
) {
    let Some(ref mut lc) = last_cast else {
        return;
    };

    for (mana, casting_state, primed) in &wizard_query {
        // Initialize prev_mana on first frame
        if lc.prev_mana == f32::MAX {
            lc.prev_mana = mana.current;
            continue;
        }

        // Phase 1: Detect mana consumption → mark pending.
        // Any decrease counts (channeled spells drain small amounts per frame).
        if mana.current < lc.prev_mana - f32::EPSILON
            && let Some(primed_spell) = primed
        {
            lc.pending_spell = Some(primed_spell.spell);
            // Mana was spent — clear rune bypass since the spell is now casting
            lc.bypass_until_cast = false;
        }

        // Phase 2: When wizard returns to Resting, commit pending spell as last cast
        if matches!(casting_state, CastingState::Resting)
            && let Some(pending) = lc.pending_spell.take()
        {
            lc.spell = Some(pending);
        }

        lc.prev_mana = mana.current;
    }
}

/// Mana Drought toggle: restores mana when enemies die.
/// Each kill restores a fixed amount of mana, capped by concentration reservation.
pub fn mana_on_kill(
    mut kill_events: MessageReader<crate::game::achievements::messages::EnemyKilledMessage>,
    mut wizards: Query<&mut Mana, With<Wizard>>,
    concentration_spells: Query<&ConcentrationSpell>,
) {
    let kill_count = kill_events.read().count();
    if kill_count == 0 {
        return;
    }

    const MANA_PER_KILL: f32 = 3.0;
    let mana_restore = kill_count as f32 * MANA_PER_KILL;
    let reserved: f32 = concentration_spells.iter().map(|c| c.mana_cost).sum();

    for mut mana in &mut wizards {
        let effective_max = (mana.max - reserved).max(0.0);
        mana.regenerate(mana_restore);
        mana.current = mana.current.min(effective_max);
    }
}

/// Handles PrimeSpellMessage to update the wizard's primed spell.
/// This allows UI systems to request spell changes without directly accessing components.
/// Applies cauldron spell power buff as a base empowerment multiplier.
/// Blocks same-spell priming when Spell Rotation toggle is active.
pub fn handle_prime_spell_messages(
    mut commands: Commands,
    mut messages: MessageReader<PrimeSpellMessage>,
    cauldron_buffs: Res<CauldronBuffs>,
    // `Option<&mut PrimedSpell>` so archetypes that start with no primed spell
    // (Randomancer, Shepherd) get one INSERTED on their first prime instead of the
    // message being silently dropped.
    mut wizard_query: Query<(Entity, Option<&mut PrimedSpell>), With<LocalWizard>>,
    active_talents: Option<Res<ActiveTalents>>,
    last_cast: Option<Res<crate::game::game_mode::components::LastCastSpell>>,
) {
    for message in messages.read() {
        // Spell Rotation: block priming the same spell that was just cast
        if let Some(ref lc) = last_cast
            && !lc.bypass_until_cast
            && lc.spell == Some(message.spell.spell)
        {
            continue;
        }

        let Ok((wizard_entity, primed)) = wizard_query.single_mut() else {
            continue;
        };

        let mut spell = message.spell;
        if cauldron_buffs.spell_power_multiplier() > 1.0 {
            spell =
                spell.with_empowerment(spell.empowerment * cauldron_buffs.spell_power_multiplier());
        }
        let cast_speed = cauldron_buffs.cast_speed_multiplier();
        if cast_speed > 1.0 {
            spell.cast_time /= cast_speed;
        }
        let range_mult = cauldron_buffs.spell_range_multiplier();
        if range_mult > 1.0 {
            spell.range_multiplier *= range_mult;
        }
        // Apply talent-based cast time modifiers
        if let Some(ref talents) = active_talents {
            let mult = talent_cast_time_multiplier(spell.spell, talents);
            if mult < 1.0 {
                spell.cast_time *= mult;
            }
        }

        match primed {
            Some(mut primed_spell) => *primed_spell = spell,
            None => {
                commands.entity(wizard_entity).insert(spell);
            }
        }
    }
}

/// Returns the talent-based cast time multiplier for a spell (1.0 = no change).
fn talent_cast_time_multiplier(spell: Spell, talents: &ActiveTalents) -> f32 {
    match spell {
        Spell::BlackHole => {
            if talents.get_selection(Spell::BlackHole, 0) == Some(2) {
                super::super::spells::black_hole_constants::QUICK_COLLAPSE_CAST_TIME_MULT
            } else {
                1.0
            }
        }
        _ => 1.0,
    }
}

/// Manages empowerment consumption and reset based on casting state transitions.
///
/// When a cast starts (transition to Casting), marks empowerment as consumed.
/// When returning to Resting, resets empowerment if it was previously consumed.
/// This ensures empowerment only applies to a single cast (including full channel duration).
pub fn reset_empowerment_after_cast(
    mut wizard_query: Query<
        (&CastingState, &mut PrimedSpell),
        (With<Wizard>, Changed<CastingState>),
    >,
) {
    for (casting_state, mut primed_spell) in &mut wizard_query {
        match casting_state {
            // When starting a cast, mark empowerment as consumed
            CastingState::Casting { .. } => {
                primed_spell.consume_empowerment();
            }
            // When returning to rest, reset empowerment if it was consumed
            CastingState::Resting => {
                if primed_spell.should_reset_empowerment() {
                    primed_spell.reset_empowerment();
                }
            }
            // Channeling state doesn't affect empowerment
            CastingState::Channeling { .. } => {}
        }
    }
}

/// Cancels any active casting when leaving the Running state.
///
/// Prevents spells from continuing to cast when entering menus or paused state.
/// Also resets the mouse button state to prevent lingering input.
pub fn cancel_active_casts(
    mut wizard_query: Query<&mut CastingState, With<Wizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    for mut casting_state in &mut wizard_query {
        if !matches!(*casting_state, CastingState::Resting) {
            casting_state.cancel();
        }
    }
    // Reset mouse state when exiting running state
    mouse_state.left_consumed = false;
}

/// Applies the wizard's stat multipliers to the currently primed spell.
///
/// Runs for all archetypes whenever the Wizard component changes. Recalculates
/// the effective cast time and empowerment based on the wizard's multipliers.
/// For archetypes with default 1.0 multipliers this is effectively a no-op.
pub fn apply_wizard_stats_to_primed_spell(
    mut wizard_query: Query<(&Wizard, &mut PrimedSpell), Changed<Wizard>>,
) {
    for (wizard, mut primed_spell) in wizard_query.iter_mut() {
        let base_cast_time = primed_spell.spell.primed_config().cast_time;
        primed_spell.cast_time = base_cast_time / wizard.cast_speed_multiplier;
        primed_spell.empowerment = wizard.spell_power_multiplier;
    }
}

/// Mirrors `Wizard.mana_cost_multiplier` onto `Mana.cost_multiplier` so the
/// per-wizard discount is applied centrally by `Mana::consume`/`can_afford` and
/// thus reaches EVERY spell — the Arcanorouter's mana routing, the BoringOleMage
/// discount, and insight bonuses. Only the few spells that read the Wizard field
/// directly honored it before.
pub fn sync_mana_cost_multiplier(mut wizards: Query<(&Wizard, &mut Mana), Changed<Wizard>>) {
    for (wizard, mut mana) in &mut wizards {
        if mana.cost_multiplier != wizard.mana_cost_multiplier {
            mana.cost_multiplier = wizard.mana_cost_multiplier;
        }
    }
}
