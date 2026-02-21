//! Multiplayer-specific components.

use bevy::prelude::*;

use crate::networking::snapshot::SpellEffectKind;

/// Marker component for entities that belong to the multiplayer game screen.
///
/// Used for bulk cleanup when exiting the multiplayer game state.
#[derive(Component)]
pub struct OnMultiplayerGameScreen;

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

/// Preloaded mesh and material handles for ghost spell effects on the guest.
#[derive(Resource)]
pub struct GhostSpellAssets {
    /// Small pink circle mesh for ghost magic missiles.
    pub missile_mesh: Handle<Mesh>,
    /// Unlit pink material for ghost magic missiles.
    pub missile_material: Handle<StandardMaterial>,
    /// Small rectangle mesh for ghost beams (unit size, scaled by transform).
    pub beam_mesh: Handle<Mesh>,
    /// Unlit orange material for ghost beams.
    pub beam_material: Handle<StandardMaterial>,
}

impl GhostSpellAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        Self {
            missile_mesh: meshes.add(Circle::new(8.0)),
            missile_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.4, 0.7),
                unlit: true,
                ..default()
            }),
            beam_mesh: meshes.add(Rectangle::new(1.0, 1.0)),
            beam_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.6, 0.1),
                unlit: true,
                ..default()
            }),
        }
    }
}

/// Marker component for spell effect entities that should be synced to the guest.
///
/// Added alongside the spell-specific component when spawning persistent spell
/// effects on the host. The `assign_network_ids` system assigns `NetworkEntityId`
/// to these entities, and `collect_spell_snapshots` builds snapshot data from them.
#[derive(Component)]
pub struct NetworkedSpellEffect {
    pub kind: SpellEffectKind,
}

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

/// Preloaded mesh and material handles for guest-side spell effect rendering.
///
/// Created in `init_mp_game` so there are zero asset allocations during gameplay.
#[derive(Resource)]
pub struct SpellEffectAssets {
    // Shared base meshes (unit size, scaled by Transform)
    pub circle_mesh: Handle<Mesh>,
    pub sphere_mesh: Handle<Mesh>,
    pub cuboid_mesh: Handle<Mesh>,
    pub rect_mesh: Handle<Mesh>,

    // Zone materials
    pub spike_growth_material: Handle<StandardMaterial>,
    pub healing_plume_material: Handle<StandardMaterial>,
    pub entangle_material: Handle<StandardMaterial>,
    pub fog_cloud_material: Handle<StandardMaterial>,
    pub grease_material: Handle<StandardMaterial>,
    pub grease_fire_material: Handle<StandardMaterial>,
    pub plague_wind_material: Handle<StandardMaterial>,
    pub meteor_ground_fire_material: Handle<StandardMaterial>,

    // Object materials
    pub black_hole_material: Handle<StandardMaterial>,
    pub arcane_crystal_material: Handle<StandardMaterial>,
    pub lightning_rod_material: Handle<StandardMaterial>,

    // Wall materials
    pub wall_of_stone_material: Handle<StandardMaterial>,
    pub wall_of_fire_material: Handle<StandardMaterial>,

    // Explosion materials
    pub fireball_explosion_material: Handle<StandardMaterial>,
    pub meteor_explosion_material: Handle<StandardMaterial>,
    pub ice_explosion_material: Handle<StandardMaterial>,

    // Ephemeral projectile materials
    pub fireball_projectile_material: Handle<StandardMaterial>,
    pub ice_projectile_material: Handle<StandardMaterial>,
    pub meteor_projectile_material: Handle<StandardMaterial>,

    // Ephemeral arc/beam materials
    pub chain_lightning_material: Handle<StandardMaterial>,
    pub lightning_strike_material: Handle<StandardMaterial>,
    pub lightning_rod_arc_material: Handle<StandardMaterial>,
    pub crystal_beam_material: Handle<StandardMaterial>,
    pub crystal_arc_material: Handle<StandardMaterial>,
    pub finger_of_death_material: Handle<StandardMaterial>,
}

impl SpellEffectAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        let unlit = |color: Color| StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        };

        let unlit_blend = |color: Color| StandardMaterial {
            base_color: color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        };

        Self {
            // Shared base meshes
            circle_mesh: meshes.add(Circle::new(1.0)),
            sphere_mesh: meshes.add(Sphere::new(1.0)),
            cuboid_mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            rect_mesh: meshes.add(Rectangle::new(1.0, 1.0)),

            // Zone materials (circles on ground, alpha blended)
            spike_growth_material: materials.add(unlit_blend(Color::srgba(0.15, 0.4, 0.05, 0.4))),
            healing_plume_material: materials.add(unlit_blend(Color::srgba(0.1, 0.7, 0.2, 0.4))),
            entangle_material: materials.add(unlit_blend(Color::srgba(0.1, 0.6, 0.15, 0.35))),
            fog_cloud_material: materials.add(unlit_blend(Color::srgba(0.6, 0.65, 0.7, 0.35))),
            grease_material: materials.add(unlit_blend(Color::srgba(0.45, 0.4, 0.05, 0.4))),
            grease_fire_material: materials.add(unlit_blend(Color::srgba(0.9, 0.3, 0.05, 0.55))),
            plague_wind_material: materials.add(unlit_blend(Color::srgba(0.2, 0.6, 0.1, 0.4))),
            meteor_ground_fire_material: materials.add(unlit_blend(Color::srgba(0.9, 0.25, 0.05, 0.5))),

            // Object materials (spheres)
            black_hole_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.05, 0.0, 0.1),
                emissive: bevy::color::LinearRgba::new(0.2, 0.0, 0.4, 1.0),
                ..default()
            }),
            arcane_crystal_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.1, 0.9),
                emissive: bevy::color::LinearRgba::new(0.4, 0.05, 0.6, 1.0),
                ..default()
            }),
            lightning_rod_material: materials.add(unlit(Color::srgb(0.6, 0.6, 0.65))),

            // Wall materials
            wall_of_stone_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.75, 0.6, 0.45, 1.0),
                ..default()
            }),
            wall_of_fire_material: materials.add(unlit_blend(Color::srgba(1.0, 0.5, 0.0, 0.4))),

            // Explosion materials
            fireball_explosion_material: materials.add(unlit(Color::srgb(1.0, 0.3, 0.0))),
            meteor_explosion_material: materials.add(unlit_blend(Color::srgba(1.0, 0.5, 0.1, 0.6))),
            ice_explosion_material: materials.add(unlit(Color::srgb(0.3, 0.8, 1.0))),

            // Ephemeral projectile materials
            fireball_projectile_material: materials.add(unlit(Color::srgb(1.0, 0.5, 0.0))),
            ice_projectile_material: materials.add(unlit(Color::srgb(0.7, 0.9, 1.0))),
            meteor_projectile_material: materials.add(unlit(Color::srgb(1.0, 0.4, 0.1))),

            // Ephemeral arc/beam materials
            chain_lightning_material: materials.add(unlit(Color::srgb(0.7, 0.85, 1.0))),
            lightning_strike_material: materials.add(unlit(Color::srgb(0.8, 0.9, 1.0))),
            lightning_rod_arc_material: materials.add(unlit(Color::srgb(0.7, 0.85, 1.0))),
            crystal_beam_material: materials.add(unlit(Color::srgb(1.0, 0.6, 0.1))),
            crystal_arc_material: materials.add(unlit(Color::srgba(0.6, 0.4, 1.0, 0.9))),
            finger_of_death_material: materials.add(unlit(Color::srgb(0.8, 0.0, 1.0))),
        }
    }
}

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
    Disconnect,
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
