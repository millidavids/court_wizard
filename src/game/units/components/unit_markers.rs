use bevy::prelude::*;

#[derive(Component)]
pub struct Corpse;

/// Marker component for permanent corpses that cannot be resurrected.
///
/// Applied to undead corpses to prevent them from being raised again.
#[derive(Component)]
pub struct PermanentCorpse;

/// Marker component for units that can be teleported.
///
/// Applied to all combat units (defenders, attackers, undead) but not the wizard.
#[derive(Component)]
pub struct Teleportable;

/// Marker component for flying units that render above the battlefield.
///
/// Flying units ignore wall obstacles (no wall avoidance, collision, or LOS suppression)
/// and cannot be targeted by melee ground units (infantry, king, kingsguard).
/// Archers can still target and hit flying units.
#[derive(Component)]
pub struct Flying;

/// Shadow entity that tracks a unit's XZ position at ground level.
#[derive(Component)]
pub struct UnitShadow {
    pub owner: Entity,
}

/// Marker on a unit that already has a shadow spawned for it.
#[derive(Component)]
pub struct HasShadow;

/// Persistent color glow for special unit types (dispeller, healer, shielder, commander, brute).
///
/// The visual system uses this to apply a pulsing color tint to the unit's material.
/// This is separate from elite glow and shield buff glow, allowing them to stack.
#[derive(Component, Clone)]
pub struct UnitTypeGlow {
    pub color: Color,
}

/// King's Guard unit. Stores the slot index for positioning around the King.
#[derive(Component)]
pub struct KingsGuard(pub u32);

/// Mind control effect — unit targets allies instead of enemies.
/// Used by both the Hag boss (Martina) and the player's Mind Control spell.
#[derive(Component)]
pub struct MindControlled {
    /// Time elapsed since mind control was applied.
    pub time_elapsed: f32,
    /// Duration before mind control wears off.
    pub wear_off_duration: f32,
    /// Original defender spawn position for restoring flow field on wear-off.
    pub original_spawn_pos: Option<Vec2>,
    /// Damage multiplier for controlled unit's attacks (Deep Domination talent).
    pub damage_multiplier: f32,
}

/// Marks a unit as wanting to retaliate against a specific entity.
/// Inserted when a mind-controlled unit attacks a same-team ally, causing
/// the victim to consider the attacker a valid target despite being on the same team.
#[derive(Component)]
pub struct RetaliationTarget(pub Entity);
