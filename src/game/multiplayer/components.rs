//! Multiplayer-specific components.

use bevy::prelude::*;

use crate::networking::snapshot::SpellEffectKind;

/// Marker component for entities that belong to the multiplayer game screen.
///
/// Used for bulk cleanup when exiting the multiplayer game state.
#[derive(Component, Clone)]
pub struct OnMultiplayerGameScreen;

/// Marker: this entity must NEVER be assigned a `NetworkEntityId` / included in
/// state snapshots, even though it has `Health` + `Team`.
///
/// Wizards are spawned locally on BOTH peers (their casts sync via the
/// bidirectional spell-visual path), so they must not also be ghosted from the
/// host snapshot or they'd double-render. The single-player wizard carries
/// `Team::Defenders`, so a plain `(With<Health>, With<Team>)` filter in
/// `assign_network_ids` would pick it up — this marker is the principled
/// exclusion (add it to any future local-only Health+Team entity rather than
/// growing an ad-hoc `Without<…>` list). Placed in the spawn bundle so it is
/// present on the entity's very first frame, before `assign_network_ids` runs.
#[derive(Component)]
pub struct NoSnapshot;

/// Marker for ghost entities rendered on the guest from host state snapshots.
///
/// Ghost entities are lightweight visual representations — they have a mesh,
/// material, and transform but no simulation components. Their positions are
/// updated each frame from the latest snapshot received from the host.
#[derive(Component)]
pub struct GhostEntity;

/// Marker for ghost arrow projectiles rendered on the guest.
///
/// Unlike `GhostEntity` (which uses `NetworkEntityMap` for stable identity),
/// ghost arrows are ephemeral — all are despawned and re-spawned each frame
/// from the snapshot since arrows have no stable network ID.
#[derive(Component)]
pub struct GhostArrow;

/// Marker for ghost magic missile projectiles rendered on the guest.
///
/// Ephemeral like `GhostArrow` — despawned and re-spawned each frame.
#[derive(Component)]
pub struct GhostMagicMissile;

/// Marker for ghost beam effects rendered on the guest.
///
/// Ephemeral like `GhostArrow` — despawned and re-spawned each frame.
#[derive(Component)]
pub struct GhostBeam;

/// Marker component for spell effect entities that should be synced to the guest.
///
/// Added alongside the spell-specific component when spawning persistent spell
/// effects on the host. The `assign_network_ids` system assigns `NetworkEntityId`
/// to these entities, and `collect_spell_snapshots` builds snapshot data from them.
#[derive(Component)]
pub struct NetworkedSpellEffect {
    pub kind: SpellEffectKind,
}

/// Marker for the guest's local copy of a host-owned spell effect (BlackHole
/// pull, ArcaneCrystal hit-reactions, LightningRod strikes, PlagueWind DPS,
/// etc.). SP-shared gameplay systems running on the guest must filter this
/// out — only the HOST should run authoritative spell-effect logic; the guest
/// only renders visuals. Without this filter, the guest's local copy of a
/// host-cast spell would independently re-apply gameplay effects, doubling
/// damage / status / forces (Category C "double-application" bugs).
#[derive(Component)]
pub struct GhostSpellEffect;

/// Marker for ghost spell projectiles rendered on the guest (ephemeral).
///
/// Used for fireball, ice projectile, and meteor projectile ghosts.
#[derive(Component)]
pub struct GhostSpellProjectile;

/// Marker for ghost spell arcs/beams rendered on the guest (ephemeral).
///
/// Used for chain lightning, lightning strikes, crystal beams, etc.
#[derive(Component)]
pub struct GhostSpellArc;

/// Marker for the opponent's Swordcerer battlefield avatar, rendered from the
/// unit snapshot's `SWORDCERER_AVATAR` flag. Lets the health-bar UI find it and
/// keeps it visually distinct from a generic infantry ghost.
#[derive(Component)]
pub struct GhostSwordcererAvatar;

/// Marker for entities on the multiplayer score screen.
///
/// Used for targeted cleanup when leaving the score screen sub-state.
#[derive(Component)]
pub struct OnMpScoreScreen;

/// Tracks rematch readiness for both players on the score screen.
#[derive(Resource, Default)]
pub struct MpRematchState {
    pub local_ready: bool,
    pub remote_ready: bool,
}

/// Button actions for the multiplayer score screen.
#[derive(Component, Clone)]
pub enum MpScoreButtonAction {
    Rematch,
    Disconnect,
}

/// Marker for the rematch status text on the score screen.
#[derive(Component)]
pub struct RematchStatusText;

/// Marks a stat value `Text` node on the multiplayer score screen so
/// `update_mp_stat_values` can refresh it from `MatchStats` (the numbers may
/// arrive a frame or two after the screen spawns).
#[derive(Component, Clone, Copy)]
pub(super) enum MpStatValueText {
    YourKills,
    YourDeaths,
    YourDamage,
    YourHealing,
    EnemyKills,
    EnemyDeaths,
    EnemyDamage,
    EnemyHealing,
}

/// Marker resource inserted when both players agree to rematch.
///
/// Signals that the transition back to `MainMenu` → `Multiplayer` should
/// skip the connection phase and go straight to wizard select, preserving
/// the existing WebRTC connection.
#[derive(Resource)]
pub struct PendingRematch;

/// Marker for entities on the MP escape menu overlay.
#[derive(Component)]
pub struct OnMpPauseScreen;

/// Button actions for the MP escape menu.
#[derive(Component, Clone)]
pub enum MpPauseButtonAction {
    Resume,
    Settings,
    Forfeit,
    Disconnect,
}

/// Marker for the forfeit confirmation overlay.
#[derive(Component)]
pub struct OnMpForfeitConfirm;

/// Yes/No actions for the forfeit confirmation overlay.
#[derive(Component, Clone)]
pub enum MpForfeitConfirmAction {
    Confirm,
    Cancel,
}

/// Marker for entities on the MP disconnected overlay.
#[derive(Component)]
pub struct OnMpDisconnectedScreen;

/// Button action for the disconnected overlay.
#[derive(Component, Clone)]
pub struct MpDisconnectedButtonAction;

/// Maps remote spell effect network IDs to local Bevy entities (guest only).
///
/// Separate from `NetworkEntityMap` because units and spell effects have
/// independent ID spaces (both start from the same counter, but separating
/// them prevents cross-contamination in the despawn logic).
#[derive(Resource, Default)]
pub struct SpellEffectEntityMap {
    pub remote_to_local: std::collections::HashMap<u32, Entity>,
    pub local_to_remote: std::collections::HashMap<Entity, u32>,
}

impl SpellEffectEntityMap {
    pub fn insert(&mut self, remote_id: u32, local_entity: Entity) {
        self.remote_to_local.insert(remote_id, local_entity);
        self.local_to_remote.insert(local_entity, remote_id);
    }

    pub fn remove_by_remote(&mut self, remote_id: u32) -> Option<Entity> {
        if let Some(entity) = self.remote_to_local.remove(&remote_id) {
            self.local_to_remote.remove(&entity);
            Some(entity)
        } else {
            None
        }
    }
}
