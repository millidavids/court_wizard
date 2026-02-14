use bevy::prelude::*;

use crate::game::resources::GameOutcome;

/// Message sent when a battle ends (entering GameOver state).
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

/// Message sent when the player clears their progress from the progress screen.
/// Triggers resetting all in-memory achievement resources back to locked.
#[derive(Message)]
pub(crate) struct ClearProgressMessage;
