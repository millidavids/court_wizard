use bevy::prelude::*;

/// Marker component for the battlemage's field avatar entity.
#[derive(Component)]
pub struct BattlemageAvatar;

/// Tracks the avatar's last non-zero facing direction on the XZ plane.
/// Attack direction (missile + sword) reads this — the archetype no longer
/// separates aim from movement; both share the left stick / WASD input.
#[derive(Component, Debug, Clone, Copy)]
pub struct BattlemageFacing(pub Vec2);

impl Default for BattlemageFacing {
    fn default() -> Self {
        // Default facing away from the castle toward the battlefield (-X direction).
        Self(Vec2::new(-1.0, 0.0))
    }
}

/// Sword swing arc visual effect entity.
#[derive(Component)]
pub struct SwordArc {
    pub time_alive: f32,
    pub duration: f32,
    pub direction: Vec2,
    pub damage_dealt: bool,
}

/// Cooldown tracker for battlemage missile attacks.
#[derive(Component)]
pub struct BattlemageMissileCooldown {
    pub remaining: f32,
}

/// Cooldown tracker for battlemage sword swings.
#[derive(Component)]
pub struct BattlemageSwordCooldown {
    pub remaining: f32,
}

/// Marker component for the "Enter the Fray" button UI root.
#[derive(Component)]
pub struct EnterFrayRoot;

/// Marker component for the "Enter the Fray" button itself.
#[derive(Component)]
pub struct EnterFrayButton;

/// Marker component for the "Enter the Fray" button text.
#[derive(Component)]
pub struct EnterFrayButtonText;

/// Component for the battlemage health bar UI container.
#[derive(Component)]
pub struct BattlemageHealthBar;

/// Component for the battlemage health bar fill node.
#[derive(Component)]
pub struct BattlemageHealthBarFill;

/// Cached material handle for sword arc visuals (created once, reused per swing).
#[derive(Resource)]
pub struct SwordArcMaterial(pub Handle<StandardMaterial>);
