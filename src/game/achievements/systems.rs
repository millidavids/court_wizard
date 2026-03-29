use bevy::prelude::*;

use crate::config::save_data::{
    accumulate_kill_stats, get_total_levels_completed, grant_achievement_insight, grant_insight,
    increment_games_played, increment_levels_completed, unlock_achievement, unlock_unit,
};
use crate::config::{GameConfig, WizardType};
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::{
    BattleInsightData, CurrentLevel, GameOutcome, KillStats, RetryTracker,
};
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::components::{CastingState, PrimedSpell, Wizard};
use crate::ui::main_menu::settings::components::SliderAdjusted;

use super::messages::{
    BattleEndedMessage, CloseCallMessage, DefenderKilledBySpellMessage, EnemyKilledMessage,
    EntangleHitDefenderMessage, GuardianCircleHitAttackerMessage, OutOfRangeMessage,
    QwerKeyPressedMessage, ScorchedEarthMessage, SpellCastMessage, StormbringerMessage,
};
use super::resources::*;

/// Helper to unlock an achievement: persists to save, updates resource, sends popup.
fn do_unlock<T: AchievementResource>(
    res: &mut ResMut<T>,
    achievement_events: &mut MessageWriter<AchievementUnlockedMessage>,
) {
    let id = T::achievement_id();
    res.unlock();
    unlock_achievement(id);
    achievement_events.write(AchievementUnlockedMessage { id });
}

// ---------------------------------------------------------------------------
// send_battle_ended — runs OnEnter(InGameState::ScoreScreen)
// ---------------------------------------------------------------------------

/// Collects battle data, updates meta-progression counters, and writes `BattleEndedMessage`.
/// Replaces the counter-incrementing part of the old `check_victory_progression_achievements`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn send_battle_ended(
    game_outcome: Res<GameOutcome>,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    kill_stats: Res<KillStats>,
    mut retry_tracker: ResMut<RetryTracker>,
    mut message: MessageWriter<BattleEndedMessage>,
    mut battle_insight: ResMut<BattleInsightData>,
    talent_progress: Option<
        Res<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
) {
    let is_victory = *game_outcome == GameOutcome::Victory;

    // Always increment games played and accumulate kill stats
    increment_games_played();
    accumulate_kill_stats(
        kill_stats.defenders_killed,
        kill_stats.attackers_killed,
        kill_stats.undead_killed,
    );

    if is_victory {
        increment_levels_completed();
        retry_tracker.level = 0;
        retry_tracker.attempts = 0;
    } else {
        // Track retries for this level
        if retry_tracker.level == current_level.0 {
            retry_tracker.attempts += 1;
        } else {
            retry_tracker.level = current_level.0;
            retry_tracker.attempts = 1;
        }
    }

    let total_wins = get_total_levels_completed();
    let effective_highest = if is_victory {
        config.highest_level_achieved.max(current_level.0 + 1)
    } else {
        config.highest_level_achieved
    };

    // Calculate and grant Arcane Insight
    let total_defenders = (crate::game::constants::INITIAL_DEFENDER_COUNT
        + crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT)
        as f32;
    let defenders_lost = kill_stats.defenders_killed as f32;
    let efficiency = 1.0 - (defenders_lost / total_defenders).min(1.0);

    let mut insight = 5 + (current_level.0 * 2); // Base: 5 + level * 2
    if is_victory {
        insight += 10; // Victory bonus
    }
    insight += (efficiency * 10.0) as u32; // Efficiency bonus: 0-10

    battle_insight.insight_earned = insight;
    grant_insight(insight);

    // Flush accumulated talent progress to save data
    if let Some(tp) = &talent_progress {
        tp.flush_to_save();
    }

    message.write(BattleEndedMessage {
        outcome: *game_outcome,
        total_wins,
        highest_level: effective_highest,
        elapsed_time: kill_stats.elapsed_time,
        attackers_killed: kill_stats.attackers_killed,
        total_attackers_spawned: kill_stats.total_attackers_spawned,
        defenders_killed_by_spell: kill_stats.defenders_killed_by_spell,
        king_killed_by_spell: kill_stats.king_killed_by_spell,
        first_defender_death_time: kill_stats.first_defender_death_time,
        retry_attempts: retry_tracker.attempts,
        wizard_damaged_enemies: kill_stats.wizard_damaged_enemies,
    });
}

// ---------------------------------------------------------------------------
// Victory & Progression achievements (triggered by BattleEndedMessage)
// ---------------------------------------------------------------------------

pub(crate) fn check_first_victory(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<FirstVictoryAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 1 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(FirstVictoryAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_apprentice_wizard(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ApprenticeWizardAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 5 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(ApprenticeWizardAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_court_wizard(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<CourtWizardAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 10 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(CourtWizardAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_archmage(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ArchmageAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 25 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(ArchmageAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_legends_speak_your_name(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<LegendsSpeakYourNameAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 50 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(LegendsSpeakYourNameAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_immortalized(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ImmortalizedAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 100 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_the_grind_never_stops(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TheGrindNeverStopsAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && m.total_wins >= 200 {
            do_unlock(&mut res, &mut events);
        }
    }
}

// --- Level-based achievements (checked on both victory and defeat) ---

pub(crate) fn check_one_more_level(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<OneMoreLevelAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 10 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_into_the_deep(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<IntoTheDeepAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 25 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_absurdity(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<AbsurdityAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 50 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_level_100(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<Level100Achievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.highest_level >= 100 {
            do_unlock(&mut res, &mut events);
        }
    }
}

// ---------------------------------------------------------------------------
// Defeat & Failure achievements (triggered by BattleEndedMessage)
// ---------------------------------------------------------------------------

pub(crate) fn check_tactical_retreat(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TacticalRetreatAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_the_king_is_dead(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TheKingIsDeadAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::DefeatKingDied {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(TheKingIsDeadAchievement::achievement_id());
        }
    }
}

pub(crate) fn check_total_wipe(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<TotalWipeAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Defeat {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_speedrun_wrong_direction(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<SpeedrunWrongDirectionAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.elapsed_time < 30.0 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_pyrrhic_defeat(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<PyrrhicDefeatAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.total_attackers_spawned > 0 {
            let kill_ratio = m.attackers_killed as f32 / m.total_attackers_spawned as f32;
            if kill_ratio >= 0.9 {
                do_unlock(&mut res, &mut events);
            }
        }
    }
}

pub(crate) fn check_it_was_going_so_well(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ItWasGoingSoWellAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory
            && let Some(first_death_time) = m.first_defender_death_time
            && first_death_time >= 120.0
        {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_friendly_fire_department(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<FriendlyFireDepartmentAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.defenders_killed_by_spell >= 10 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_accidental_regicide(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<AccidentalRegicideAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.king_killed_by_spell {
            do_unlock(&mut res, &mut events);
            crate::config::save_data::unlock_wizard_type(WizardType::Psychopath);
        }
    }
}

pub(crate) fn check_stubborn(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<StubbornAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.retry_attempts >= 5 {
            do_unlock(&mut res, &mut events);
        }
    }
}

pub(crate) fn check_extremely_stubborn(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<ExtremelyStubbornAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome != GameOutcome::Victory && m.retry_attempts >= 15 {
            do_unlock(&mut res, &mut events);
        }
    }
}

// ---------------------------------------------------------------------------
// Mid-battle achievements
// ---------------------------------------------------------------------------

pub(crate) fn check_friendly_fire(
    mut msg: MessageReader<DefenderKilledBySpellMessage>,
    mut res: ResMut<FriendlyFireAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(FriendlyFireAchievement::achievement_id());
    }
}

/// Updates the multi-kill tracker timer.
pub(crate) fn update_multi_kill_tracker(time: Res<Time>, mut tracker: ResMut<MultiKillTracker>) {
    tracker.update(time.delta_secs());
}

/// Tracks consecutive enemy kills for multi-kill achievements.
pub(crate) fn track_multi_kills(
    mut msg: MessageReader<EnemyKilledMessage>,
    mut tracker: ResMut<MultiKillTracker>,
    mut res: ResMut<ChainReactionAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for _ in msg.read() {
        let kill_count = tracker.register_kill();
        if kill_count >= 3 {
            do_unlock(&mut res, &mut events);
            grant_achievement_insight(ChainReactionAchievement::achievement_id());
        }
    }
}

// ---------------------------------------------------------------------------
// Meta achievement (triggered by SliderAdjusted)
// ---------------------------------------------------------------------------

pub(crate) fn check_slider_fiddler(
    mut msg: MessageReader<SliderAdjusted>,
    mut res: ResMut<SliderFiddlerAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        crate::config::save_data::unlock_wizard_type(WizardType::Arcanorouter);
    }
}

// ---------------------------------------------------------------------------
// Spell cast detection — writes SpellCastMessage when wizard starts casting
// ---------------------------------------------------------------------------

/// Detects when the wizard's CastingState transitions to Casting and writes a SpellCastMessage.
/// Also records the damage type of the cast spell for Insight tracking.
pub(crate) fn detect_spell_cast(
    wizard_query: Query<
        (&CastingState, Option<&PrimedSpell>),
        (With<Wizard>, Changed<CastingState>),
    >,
    mut msg: MessageWriter<SpellCastMessage>,
    mut battle_insight: ResMut<BattleInsightData>,
) {
    for (casting_state, primed_spell) in &wizard_query {
        if matches!(casting_state, CastingState::Casting { .. }) {
            msg.write(SpellCastMessage);
            if let Some(primed) = primed_spell {
                battle_insight
                    .damage_types_used
                    .insert(primed.spell.damage_type());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Random Magic Surge achievement (triggered by SpellCastMessage, 1/100 chance)
// ---------------------------------------------------------------------------

pub(crate) fn check_random_magic_surge(
    mut msg: MessageReader<SpellCastMessage>,
    mut res: ResMut<RandomMagicSurgeAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    use rand::Rng;
    for _ in msg.read() {
        if rand::thread_rng().gen_range(1..=100) == 1 {
            do_unlock(&mut res, &mut events);
            crate::config::save_data::unlock_wizard_type(WizardType::Randomancer);
        }
    }
}

// ---------------------------------------------------------------------------
// QWER achievement — triggered by pressing Q, W, E, or R during gameplay
// ---------------------------------------------------------------------------

/// Detects Q/W/E/R key presses during gameplay and writes a QwerKeyPressedMessage.
pub(crate) fn detect_qwer_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut msg: MessageWriter<QwerKeyPressedMessage>,
) {
    if keyboard.any_just_pressed([KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR]) {
        msg.write(QwerKeyPressedMessage);
    }
}

pub(crate) fn check_qwer(
    mut msg: MessageReader<QwerKeyPressedMessage>,
    mut res: ResMut<QwerAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        crate::config::save_data::unlock_wizard_type(WizardType::RuneCaster);
    }
}

// ---------------------------------------------------------------------------
// Spell-unlock achievements — mid-battle detection + check
// ---------------------------------------------------------------------------

/// Detects when a defender is beyond the wizard's spell range.
pub(crate) fn detect_out_of_range(
    wizard_query: Query<(&Transform, &Wizard)>,
    defenders: Query<(&Transform, &Team), (Without<Wizard>, Without<Corpse>)>,
    mut msg: MessageWriter<OutOfRangeMessage>,
) {
    let Ok((wizard_transform, wizard)) = wizard_query.single() else {
        return;
    };

    for (defender_transform, team) in &defenders {
        if *team != Team::Defenders {
            continue;
        }
        let distance = wizard_transform
            .translation
            .distance(defender_transform.translation);
        if distance > wizard.spell_range {
            msg.write(OutOfRangeMessage);
            return; // Only need one detection
        }
    }
}

pub(crate) fn check_out_of_range(
    mut msg: MessageReader<OutOfRangeMessage>,
    mut res: ResMut<OutOfRangeAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(OutOfRangeAchievement::achievement_id());
    }
}

pub(crate) fn check_scorched_earth(
    mut msg: MessageReader<ScorchedEarthMessage>,
    mut res: ResMut<ScorchedEarthAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(ScorchedEarthAchievement::achievement_id());
    }
}

pub(crate) fn check_protective_instincts(
    mut msg: MessageReader<GuardianCircleHitAttackerMessage>,
    mut res: ResMut<ProtectiveInstinctsAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(ProtectiveInstinctsAchievement::achievement_id());
    }
}

pub(crate) fn check_friendly_thorns(
    mut msg: MessageReader<EntangleHitDefenderMessage>,
    mut res: ResMut<FriendlyThornsAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        grant_achievement_insight(FriendlyThornsAchievement::achievement_id());
    }
}

// ---------------------------------------------------------------------------
// Unit encounter achievements
// ---------------------------------------------------------------------------

use crate::game::units::UnitType;
use crate::game::units::boss::hags::components::Hag;
use crate::game::units::boss::ogre::components::OgreEnrageState;
use crate::game::units::brute::components::Brute;
use crate::game::units::dispeller::components::Dispeller;
use crate::game::units::healer::components::Healer;
use crate::game::units::{Commander, EliteHealthBonus};

/// Macro to generate an encounter-detection system for a unit type.
macro_rules! encounter_system {
    ($fn_name:ident, $marker:ty, $res:ty, $unit_type:expr) => {
        pub(crate) fn $fn_name(
            query: Query<(), With<$marker>>,
            mut res: ResMut<$res>,
            mut events: MessageWriter<AchievementUnlockedMessage>,
        ) {
            if !query.is_empty() {
                unlock_unit($unit_type);
                do_unlock(&mut res, &mut events);
            }
        }
    };
}

encounter_system!(
    check_brute_encounter,
    Brute,
    MeetTheBruteAchievement,
    UnitType::Brute
);
encounter_system!(
    check_elite_encounter,
    EliteHealthBonus,
    EliteForcesAchievement,
    UnitType::Elite
);
encounter_system!(
    check_commander_encounter,
    Commander,
    CommanderOnTheFieldAchievement,
    UnitType::Commander
);
encounter_system!(
    check_healer_encounter,
    Healer,
    EnemyMedicAchievement,
    UnitType::Healer
);
encounter_system!(
    check_dispeller_encounter,
    Dispeller,
    MagicNullifierAchievement,
    UnitType::Dispeller
);
encounter_system!(
    check_hag_encounter,
    Hag,
    TheThreeHagsAchievement,
    UnitType::Hag
);
encounter_system!(
    check_ogre_encounter,
    OgreEnrageState,
    OgreWarlordAchievement,
    UnitType::Ogre
);

// ---------------------------------------------------------------------------
// Soiled Surprise — triggered by UnitSickenedMessage (first sickened event)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Master Brewer — triggered by IngredientCollectedMessage (all 18 ingredients)
// ---------------------------------------------------------------------------

pub(crate) fn check_master_brewer(
    mut msg: MessageReader<crate::game::messages::IngredientCollectedMessage>,
    mut res: ResMut<MasterBrewerAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    use crate::game::cauldron::brews::Ingredient;

    if msg.read().next().is_some() {
        let save = crate::config::save_data::load_unified_save();
        let unlocked = save
            .map(|s| s.player.unlocked_content.ingredients)
            .unwrap_or_default();

        let all_collected = Ingredient::all()
            .iter()
            .all(|i| unlocked.contains(&format!("{:?}", i)));

        if all_collected {
            do_unlock(&mut res, &mut events);
            crate::config::save_data::unlock_wizard_type(WizardType::Alchemist);
        }
    }
}

pub(crate) fn check_soiled_surprise(
    mut msg: MessageReader<super::messages::UnitSickenedMessage>,
    mut res: ResMut<SoiledSurpriseAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        crate::config::save_data::unlock_wizard_type(WizardType::Excremage);
    }
}

pub(crate) fn check_right_to_bear_arms(
    mut msg: MessageReader<super::messages::MarkedForDeathKillMessage>,
    mut res: ResMut<RightToBearArmsAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        crate::config::save_data::unlock_wizard_type(WizardType::Warglock);
    }
}

// ---------------------------------------------------------------------------
// Close Call — enemy killed within 300 units of the wizard (unlocks Battlemage)
// ---------------------------------------------------------------------------

pub(crate) fn check_close_call(
    mut msg: MessageReader<CloseCallMessage>,
    mut res: ResMut<CloseCallAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        crate::config::save_data::unlock_wizard_type(WizardType::Battlemage);
    }
}

// ---------------------------------------------------------------------------
// Stormbringer — Lightning Rod placed within Squall AoE (unlocks Meteorologist)
// ---------------------------------------------------------------------------

use crate::game::units::wizard::spells::lightning_rod::components::LightningRod;
use crate::game::units::wizard::spells::squall::components::SquallStorm;

/// Detects when a Lightning Rod exists within a Squall's AoE.
pub(crate) fn detect_stormbringer(
    lightning_rods: Query<&LightningRod>,
    squalls: Query<&SquallStorm>,
    mut msg: MessageWriter<StormbringerMessage>,
) {
    for rod in lightning_rods.iter() {
        for squall in squalls.iter() {
            let dx = rod.position.x - squall.position.x;
            let dz = rod.position.z - squall.position.z;
            let dist_sq = dx * dx + dz * dz;
            if dist_sq <= squall.radius * squall.radius {
                msg.write(StormbringerMessage);
                return;
            }
        }
    }
}

pub(crate) fn check_stormbringer(
    mut msg: MessageReader<StormbringerMessage>,
    mut res: ResMut<StormbringerAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        crate::config::save_data::unlock_wizard_type(WizardType::Meteorologist);
    }
}

// ---------------------------------------------------------------------------
// Pacifist — win a level without any spell damaging enemies (unlocks Shepherd)
// ---------------------------------------------------------------------------

pub(crate) fn check_pacifist(
    mut msg: MessageReader<BattleEndedMessage>,
    mut res: ResMut<PacifistAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
) {
    for m in msg.read() {
        if m.outcome == GameOutcome::Victory && !m.wizard_damaged_enemies {
            do_unlock(&mut res, &mut events);
            crate::config::save_data::unlock_wizard_type(WizardType::Shepherd);
        }
    }
}

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
