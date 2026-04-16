use bevy::prelude::*;

use crate::game::resources::GameOutcome;

/// Message sent when a battle ends (entering ScoreScreen state).
/// Carries a snapshot of all data needed by achievement check systems.
#[derive(Message)]
pub(crate) struct BattleEndedMessage {
    pub outcome: GameOutcome,
    pub total_wins: u32,
    pub highest_level: u32,
    pub elapsed_time: f32,
    pub attackers_killed: u32,
    pub total_attackers_spawned: u32,
    pub defenders_killed_by_spell: u32,
    pub king_killed_by_spell: bool,
    pub first_defender_death_time: Option<f32>,
    pub retry_attempts: u32,
    pub wizard_damaged_enemies: bool,
}

/// Message sent when a defender is killed by the player's spell during battle.
#[derive(Message)]
pub(crate) struct DefenderKilledBySpellMessage;

/// Message sent when an enemy (attacker or undead) is killed.
#[derive(Message)]
pub(crate) struct EnemyKilledMessage;

/// Message sent when the wizard begins casting a spell (CastingState transitions to Casting).
#[derive(Message)]
pub(crate) struct SpellCastMessage;

/// Message sent when the player presses Q, W, E, or R during gameplay.
#[derive(Message)]
pub(crate) struct QwerKeyPressedMessage;

/// Message sent when a defender is beyond the wizard's spell range.
#[derive(Message)]
pub(crate) struct OutOfRangeMessage;

/// Message sent when a unit dies from residual fire damage (fireball ground fire).
#[derive(Message)]
pub(crate) struct ScorchedEarthMessage;

/// Message sent when Guardian Circle's buff hits an attacker.
#[derive(Message)]
pub(crate) struct GuardianCircleHitAttackerMessage;

/// Message sent when Entangle roots a friendly defender.
#[derive(Message)]
pub(crate) struct EntangleHitDefenderMessage;

/// Message sent when the player clears their progress from the progress screen.
/// Triggers resetting all in-memory achievement resources back to locked.
#[derive(Message)]
pub(crate) struct ClearProgressMessage;

/// Message sent when a unit becomes sickened (poison threshold reached).
#[derive(Message)]
pub(crate) struct UnitSickenedMessage;

/// Message sent when an enemy marked with Finger of Death's mark is killed.
#[derive(Message)]
pub(crate) struct MarkedForDeathKillMessage;

/// Message sent when an enemy dies within close range of the wizard.
#[derive(Message)]
pub(crate) struct CloseCallMessage;

/// Message sent when a Lightning Rod is placed within a Squall's AoE.
#[derive(Message)]
pub(crate) struct StormbringerMessage;

/// Message sent when a wizard type is newly unlocked via `unlock_wizard_type()`.
#[derive(Message)]
pub(crate) struct WizardTypeUnlockedMessage;
