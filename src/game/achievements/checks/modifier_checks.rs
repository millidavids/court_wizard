use super::super::helpers::do_unlock;
use bevy::prelude::*;

use crate::config::input_bindings::BindingContext;
use crate::config::save_data::grant_achievement_insight;
use crate::config::{GameConfig, InputBindings, WizardType};
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::{CurrentLevel, GameOutcome};

use super::super::messages::BattleEndedMessage;
use super::super::resources::*;

// ---------------------------------------------------------------------------
// Roguelite modifier achievements — checked on battle end
// ---------------------------------------------------------------------------

/// Checks all roguelite modifier achievements when a roguelite run is completed.
/// A roguelite run is "completed" when the player wins the final level (level 25).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn check_roguelite_modifier_achievements(
    mut msg: MessageReader<BattleEndedMessage>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    current_level: Res<CurrentLevel>,
    mut wave_speed: (
        ResMut<ModWaveSpeedMinAch>,
        ResMut<ModWaveSpeed100Ach>,
        ResMut<ModWaveSpeed200Ach>,
        ResMut<ModWaveSpeed300Ach>,
    ),
    mut enemy_str: (
        ResMut<ModEnemyStrengthMinAch>,
        ResMut<ModEnemyStrength100Ach>,
        ResMut<ModEnemyStrength200Ach>,
        ResMut<ModEnemyStrength300Ach>,
    ),
    mut enemy_count: (
        ResMut<ModEnemyCountMinAch>,
        ResMut<ModEnemyCount100Ach>,
        ResMut<ModEnemyCount200Ach>,
        ResMut<ModEnemyCount300Ach>,
    ),
    mut combos: (
        ResMut<ModAllMinAch>,
        ResMut<ModAll200Ach>,
        ResMut<ModAllMaxAch>,
        ResMut<ModMixedExtremesAch>,
    ),
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    use crate::game::game_mode::components::{ROGUELITE_MAX_LEVEL, is_roguelite_mode};

    for m in msg.read() {
        // Must be a roguelite victory at the final level
        if m.outcome != GameOutcome::Victory {
            continue;
        }
        if !is_roguelite_mode(game_mode.as_deref()) {
            continue;
        }
        if current_level.0 != ROGUELITE_MAX_LEVEL {
            continue;
        }

        let Some(mods) = &modifiers else {
            continue;
        };

        let gs = mods.game_speed;
        let ee = mods.enemy_effectiveness;
        let ec = mods.enemy_count;

        // Helper to check approximate equality
        let near = |a: f32, b: f32| (a - b).abs() < 0.05;

        // Wave Speed
        if near(gs, 0.2) && wave_speed.0.is_locked() {
            do_unlock(&mut wave_speed.0, &mut events);
        }
        if near(gs, 1.0) && wave_speed.1.is_locked() {
            do_unlock(&mut wave_speed.1, &mut events);
        }
        if near(gs, 2.0) && wave_speed.2.is_locked() {
            do_unlock(&mut wave_speed.2, &mut events);
        }
        if near(gs, 3.0) && wave_speed.3.is_locked() {
            do_unlock(&mut wave_speed.3, &mut events);
            grant_achievement_insight(ModWaveSpeed300Ach::achievement_id());
        }

        // Enemy Strength
        if near(ee, 0.2) && enemy_str.0.is_locked() {
            do_unlock(&mut enemy_str.0, &mut events);
        }
        if near(ee, 1.0) && enemy_str.1.is_locked() {
            do_unlock(&mut enemy_str.1, &mut events);
        }
        if near(ee, 2.0) && enemy_str.2.is_locked() {
            do_unlock(&mut enemy_str.2, &mut events);
        }
        if near(ee, 3.0) && enemy_str.3.is_locked() {
            do_unlock(&mut enemy_str.3, &mut events);
            grant_achievement_insight(ModEnemyStrength300Ach::achievement_id());
        }

        // Enemy Count
        if near(ec, 0.2) && enemy_count.0.is_locked() {
            do_unlock(&mut enemy_count.0, &mut events);
        }
        if near(ec, 1.0) && enemy_count.1.is_locked() {
            do_unlock(&mut enemy_count.1, &mut events);
        }
        if near(ec, 2.0) && enemy_count.2.is_locked() {
            do_unlock(&mut enemy_count.2, &mut events);
        }
        if near(ec, 3.0) && enemy_count.3.is_locked() {
            do_unlock(&mut enemy_count.3, &mut events);
            grant_achievement_insight(ModEnemyCount300Ach::achievement_id());
        }

        // Combo: All at minimum (20%)
        if near(gs, 0.2) && near(ee, 0.2) && near(ec, 0.2) && combos.0.is_locked() {
            do_unlock(&mut combos.0, &mut events);
        }

        // Combo: All at 200%+
        if gs >= 2.0 - 0.05 && ee >= 2.0 - 0.05 && ec >= 2.0 - 0.05 && combos.1.is_locked() {
            do_unlock(&mut combos.1, &mut events);
            grant_achievement_insight(ModAll200Ach::achievement_id());
        }

        // Combo: All at 300% (max)
        if near(gs, 3.0) && near(ee, 3.0) && near(ec, 3.0) && combos.2.is_locked() {
            do_unlock(&mut combos.2, &mut events);
            grant_achievement_insight(ModAllMaxAch::achievement_id());
        }

        // Combo: Mixed extremes (one at 300%, another at 20%)
        let values = [gs, ee, ec];
        let has_max = values.iter().any(|v| near(*v, 3.0));
        let has_min = values.iter().any(|v| near(*v, 0.2));
        if has_max && has_min && combos.3.is_locked() {
            do_unlock(&mut combos.3, &mut events);
        }
    }
}

// ---------------------------------------------------------------------------
// Clicker — win a roguelite run with all keybindings unbound (mouse only)
// ---------------------------------------------------------------------------

/// Checks if the player won a roguelite run with all relevant keys unbound.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_clicker(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ClickerAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    bindings: Res<InputBindings>,
    game_config: Res<GameConfig>,
    game_mode: Option<Res<crate::game::game_mode::components::GameMode>>,
    current_level: Res<CurrentLevel>,
) {
    use crate::game::game_mode::components::{ROGUELITE_MAX_LEVEL, is_roguelite_mode};

    for m in msg.read() {
        if m.outcome != GameOutcome::Victory {
            continue;
        }
        if !is_roguelite_mode(game_mode.as_deref()) {
            continue;
        }
        if current_level.0 != ROGUELITE_MAX_LEVEL {
            continue;
        }

        // All universal bindings must be unbound
        if !bindings.all_universal_unbound() {
            continue;
        }

        // Wizard-specific bindings must be unbound (if the wizard type has any)
        if let Some(ctx) = wizard_type_to_context(game_config.wizard_type)
            && !bindings.all_context_unbound(ctx)
        {
            continue;
        }

        do_unlock(&mut res, &mut events);
        grant_achievement_insight(ClickerAchievement::achievement_id());
    }
}

/// Maps a WizardType to its BindingContext, if it has archetype-specific bindings.
fn wizard_type_to_context(wizard_type: WizardType) -> Option<BindingContext> {
    match wizard_type {
        WizardType::RuneCaster => Some(BindingContext::RuneCaster),
        WizardType::Swordcerer => Some(BindingContext::Swordcerer),
        WizardType::Arcanorouter => Some(BindingContext::ArcanoRouter),
        WizardType::Meteorologist => Some(BindingContext::Meteorologist),
        WizardType::Warglock => Some(BindingContext::Warglock),
        // BoringOleMage, Randomancer, Excremage, Shepherd, Psychopath, Alchemist
        // have no archetype-specific keybindings
        _ => None,
    }
}
