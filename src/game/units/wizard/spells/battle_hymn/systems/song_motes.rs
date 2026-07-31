//! Golden song-motes rising above units under Battle Hymn.
//!
//! One tiny mote at a time drifts up from a buffed unit's head — sparse
//! enough to be missable, present enough to answer "does that unit still
//! have my hymn?". Reuses the shared `FloatingMote` particle (updated and
//! despawned by the vfx plugin's `update_floating_motes`). Host-simulated
//! units carry the real `BattleHymnModifier`; guest-rendered ghosts carry
//! the snapshot-mirrored `RemoteBattleHymnEffect`. No `GhostEntity`
//! exclusion — visual systems run on ghosts by convention.

use bevy::prelude::*;

use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{BattleHymnModifier, Corpse, Hitbox, RemoteBattleHymnEffect};
use crate::game::units::wizard::spells::vfx::components::FloatingMote;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::vfx::unit_motes::{mote_scatter_offset, unit_effect_tick};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Seconds between motes per buffed unit (kept sparse for subtlety).
const SONG_MOTE_INTERVAL: f32 = 1.2;
/// Horizontal scatter radius around the unit's head.
const SONG_MOTE_SCATTER_RADIUS: f32 = 12.0;
/// Mote quad size (world units).
const SONG_MOTE_SIZE: f32 = 4.5;
/// Mote lifetime in seconds.
const SONG_MOTE_LIFETIME: f32 = 1.6;
/// Upward drift speed of motes (world units/second).
const SONG_MOTE_RISE_SPEED: f32 = 14.0;
/// Spawn height as a fraction of hitbox height. The 128-tall sprite's top
/// sits ~64 above the unit origin (hitbox height is 100), so 0.7 puts motes
/// just above the head instead of hidden inside the sprite.
const SONG_MOTE_HEIGHT_FRACTION: f32 = 0.7;

/// Emits sparse golden motes above units under Battle Hymn.
///
/// Cadence is per-entity via phase-offset time-boundary crossing so a whole
/// hymned army (Chorus of Valor buffs every defender) doesn't pulse in
/// unison.
#[allow(clippy::type_complexity)]
pub fn emit_battle_hymn_song_motes(
    mut commands: Commands,
    hymned_units: Query<
        (Entity, &Transform, &Hitbox),
        (
            Or<(With<BattleHymnModifier>, With<RemoteBattleHymnEffect>)>,
            Without<Corpse>,
        ),
    >,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, transform, hitbox) in &hymned_units {
        let Some(seed) = unit_effect_tick(entity, t, dt, SONG_MOTE_INTERVAL) else {
            continue;
        };

        let pos = transform.translation;
        let scatter = mote_scatter_offset(seed, SONG_MOTE_SCATTER_RADIUS, 0.2);

        commands.spawn((
            FloatingMote {
                velocity: Vec3::new(0.0, SONG_MOTE_RISE_SPEED, 0.0),
                time_alive: 0.0,
                lifetime: SONG_MOTE_LIFETIME,
                base_size: SONG_MOTE_SIZE,
                phase: seed,
            },
            Mesh3d(visual_assets.particle_quad.clone()),
            MeshMaterial3d(visual_assets.battle_hymn_song_mote.clone()),
            Transform::from_translation(Vec3::new(
                pos.x + scatter.x,
                pos.y + hitbox.height * SONG_MOTE_HEIGHT_FRACTION,
                pos.z + scatter.y,
            ))
            .with_rotation(UPWARD_ROTATION)
            .with_scale(Vec3::splat(0.0)),
            OnGameplayScreen,
        ));
    }
}
