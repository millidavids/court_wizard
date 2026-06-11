use super::super::helpers::{do_unlock, unlock_and_notify_wizard_type};
use bevy::prelude::*;

use crate::config::WizardType;
use crate::config::save_data::grant_achievement_insight;
use crate::game::messages::AchievementUnlockedMessage;
use crate::game::resources::BattleInsightData;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::components::{CastingState, PrimedSpell, Wizard};
use crate::ui::main_menu::settings::components::SliderAdjusted;

use super::super::messages::{
    CloseCallMessage, DefenderKilledBySpellMessage, EnemyKilledMessage, EntangleHitDefenderMessage,
    GuardianCircleHitAttackerMessage, OutOfRangeMessage, QwerKeyPressedMessage,
    ScorchedEarthMessage, SpellCastMessage, StormbringerMessage, WizardTypeUnlockedMessage,
};
use super::super::resources::*;

use crate::game::units::wizard::spells::lightning_rod::components::LightningRod;
use crate::game::units::wizard::spells::squall::components::SquallStorm;

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
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        unlock_and_notify_wizard_type(WizardType::Arcanorouter, &mut wizard_unlocked);
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
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut res: ResMut<RandomMagicSurgeAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    use rand::Rng;
    for _ in msg.read() {
        if game_rng.0.random_range(1..=100) == 1 {
            do_unlock(&mut res, &mut events);
            unlock_and_notify_wizard_type(WizardType::Randomancer, &mut wizard_unlocked);
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
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        unlock_and_notify_wizard_type(WizardType::RuneCaster, &mut wizard_unlocked);
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
// Close Call — enemy killed within 300 units of the wizard (unlocks Swordcerer)
// ---------------------------------------------------------------------------

pub(crate) fn check_close_call(
    mut msg: MessageReader<CloseCallMessage>,
    mut res: ResMut<CloseCallAchievement>,
    mut events: MessageWriter<AchievementUnlockedMessage>,
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        unlock_and_notify_wizard_type(WizardType::Swordcerer, &mut wizard_unlocked);
    }
}

// ---------------------------------------------------------------------------
// Stormbringer — Lightning Rod placed within Squall AoE (unlocks Meteorologist)
// ---------------------------------------------------------------------------

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
    mut wizard_unlocked: MessageWriter<WizardTypeUnlockedMessage>,
) {
    if msg.read().next().is_some() {
        do_unlock(&mut res, &mut events);
        unlock_and_notify_wizard_type(WizardType::Meteorologist, &mut wizard_unlocked);
    }
}
