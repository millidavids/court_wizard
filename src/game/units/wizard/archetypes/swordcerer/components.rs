use bevy::prelude::*;

/// Marker component for the swordcerer's field avatar entity.
#[derive(Component)]
pub struct SwordcererAvatar;

/// Tracks the avatar's last non-zero facing direction on the XZ plane.
/// Attack direction (missile + sword) reads this — the archetype no longer
/// separates aim from movement; both share the left stick / WASD input.
#[derive(Component, Debug, Clone, Copy)]
pub struct SwordcererFacing(pub Vec2);

impl Default for SwordcererFacing {
    fn default() -> Self {
        // Default facing away from the castle toward the battlefield (-X direction).
        Self(Vec2::new(-1.0, 0.0))
    }
}

/// Sword swing arc visual effect entity. Damage is dealt on the frame the
/// component is added (detected via `Added<SwordArc>`); this struct only
/// carries the lifecycle state for grow + fade.
#[derive(Component)]
pub struct SwordArc {
    pub time_alive: f32,
    pub duration: f32,
    pub direction: Vec2,
}

/// Marks a sword arc that is a visual-only ghost of the opponent's swing
/// (spawned from a replicated cast event). Excluded from `update_sword_arcs`'
/// damage pass — the real damage already crosses via CRDT.
#[derive(Component)]
pub struct GhostSwordArc;

/// Marks the host-spawned avatar that the GUEST controls (Team::Attackers). The
/// host's own local-input systems (player_movement / fire_missile / sword_swing /
/// check_avatar_death) filter this out with `Without<GuestControlledAvatar>` so a
/// Swordcerer-vs-Swordcerer match — where the host owns TWO avatars — doesn't
/// break their `single()` queries; the guest's avatar is driven by
/// `apply_guest_avatar_input` instead.
#[derive(Component)]
pub struct GuestControlledAvatar;

/// Cooldown tracker for swordcerer missile attacks.
#[derive(Component)]
pub struct SwordcererMissileCooldown {
    pub remaining: f32,
}

/// Cooldown tracker for swordcerer sword swings.
#[derive(Component)]
pub struct SwordcererSwordCooldown {
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

/// Component for the swordcerer health bar UI container.
#[derive(Component)]
pub struct SwordcererHealthBar;

/// Component for the swordcerer health bar fill node.
#[derive(Component)]
pub struct SwordcererHealthBarFill;
