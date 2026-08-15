//! Per-entity Excremage theming for the OPPONENT's spell ghosts.
//!
//! `refresh_spell_visuals_for_wizard` browns the SHARED spell material handles
//! based on the LOCAL wizard type. So when the *opponent* is the Excremage,
//! their spell ghosts render in normal colors on this machine — and we cannot
//! just brown the shared handles here without also browning our OWN spells.
//!
//! Instead we swap each remote-Excremage ghost's material for a browned CLONE,
//! cached by source material so the per-frame-respawned ephemeral ghosts
//! (projectiles, missiles, beams) reuse one clone instead of allocating.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::config::WizardType;
use crate::game::units::wizard::spells::visual_assets::{excremage_color, excremage_emissive};
use crate::networking::session::MultiplayerSession;

use super::components::{
    GhostBeam, GhostMagicMissile, GhostSpellArc, GhostSpellEffect, GhostSpellProjectile,
};

/// True when the REMOTE player (opponent) is an Excremage.
pub(crate) fn is_remote_excremage(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.remote_wizard() == WizardType::Excremage)
}

/// Cache of browned material clones, keyed by the source material's id, so the
/// per-frame ghost respawn reuses one clone per source handle.
#[derive(Resource, Default)]
pub(crate) struct ExcremageGhostMaterials(
    HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
);

/// Marks a ghost whose material has already been swapped to the browned clone.
#[derive(Component)]
pub(crate) struct ExcremageThemed;

/// Swaps remote-Excremage spell ghosts to a browned material clone. Runs only
/// when the opponent is an Excremage. The local player's own spells are never
/// touched (this only queries ghost markers).
#[allow(clippy::type_complexity)]
pub(crate) fn theme_remote_excremage_ghosts(
    mut commands: Commands,
    mut cache: ResMut<ExcremageGhostMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ghosts: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        (
            Without<ExcremageThemed>,
            Or<(
                With<GhostSpellEffect>,
                With<GhostSpellProjectile>,
                With<GhostMagicMissile>,
                With<GhostBeam>,
                With<GhostSpellArc>,
            )>,
        ),
    >,
) {
    for (entity, mat) in &ghosts {
        let source_id = mat.0.id();
        let cached = cache.0.get(&source_id).cloned();
        let browned = match cached {
            Some(handle) => handle,
            None => {
                let Some(src) = materials.get(source_id) else {
                    continue;
                };
                let mut clone = src.clone();
                clone.base_color = excremage_color(clone.base_color);
                clone.emissive = excremage_emissive(clone.emissive);
                let handle = materials.add(clone);
                cache.0.insert(source_id, handle.clone());
                handle
            }
        };
        commands
            .entity(entity)
            .insert((MeshMaterial3d(browned), ExcremageThemed));
    }
}

/// Clears the browned-material cache on MP exit so the cloned handles are freed.
///
/// Registered on `OnExit(MultiplayerGame)` *and* `OnExit(InGame)` — the co-op host
/// fills this cache from `AppState::InGame` and never sees the former. The
/// already-empty check keeps the single-player exit from touching `ResMut` and
/// marking the resource changed for no reason.
pub(crate) fn clear_excremage_ghost_materials(mut cache: ResMut<ExcremageGhostMaterials>) {
    if cache.0.is_empty() {
        return;
    }
    cache.0.clear();
}
