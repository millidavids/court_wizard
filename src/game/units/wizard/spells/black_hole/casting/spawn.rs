//! Black hole entity spawning.

use super::super::components::{BlackHole, BlackHoleSfx, BlackHoleTalentParams};
use super::super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Spawns a black hole entity (solid black icosphere) with a looping sound.
pub(crate) fn spawn_black_hole(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: BlackHoleTalentParams,
) {
    let max_radius = MAX_RADIUS * empowerment * talent_params.radius_mult;
    let spawn_pos = Vec3::new(position.x, BLACK_HOLE_HEIGHT, position.z);

    let black_hole_entity = commands
        .spawn((
            BlackHole::new(spawn_pos, max_radius, empowerment, talent_params),
            Mesh3d(assets.black_hole_sphere.clone()),
            MeshMaterial3d(assets.black_hole.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
            NetworkedSpellEffect {
                kind: SpellEffectKind::BlackHole,
            },
            OnGameplayScreen,
        ))
        .id();

    // Looping sound effect attenuated by distance from wizard to black hole
    let sfx_entity = audio::play_looping_sfx_at(
        commands,
        &sfx.black_hole_persistent,
        spawn_pos,
        game_config,
        sfx,
    );
    commands
        .entity(sfx_entity)
        .insert(BlackHoleSfx { black_hole_entity });
}
