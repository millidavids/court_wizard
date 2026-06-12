//! Multiplayer king spawn helper.

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, WaveGroup};
use crate::game::units::components::{
    Effectiveness, FacingDirection, FlockingVelocity, Health, Hitbox, MovementSpeed,
    TargetingVelocity, Team, WalkingAnimation,
};
use crate::game::units::king::components::KingSpawned;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

use super::super::components::OnMultiplayerGameScreen;
use super::utils::staggered_attack_timing;

/// Spawns a King unit at the given position origin for multiplayer.
#[allow(clippy::too_many_arguments)]
pub(in crate::game::multiplayer) fn spawn_mp_king(
    commands: &mut Commands,
    king_assets: &crate::game::units::king::resources::KingAssets,
    spell_assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    king_spawned: &mut ResMut<KingSpawned>,
    wizard_position: Vec3,
    center_angle: f32,
    team: Team,
) {
    use crate::game::units::commander::{AuraDamageBuff, AuraSpeedBuff, Commander, TeamFilter};
    use crate::game::units::components::DamageMultiplier;
    use crate::game::units::king::components::King;
    use crate::game::units::king::constants::*;

    let radius = MP_DEFENDER_GRID_GROUND_RANGE + 600.0;
    let spawn_x = wizard_position.x + radius * center_angle.cos();
    let spawn_z = wizard_position.z + radius * center_angle.sin();

    let hitbox = Hitbox::new(KING_RADIUS, KING_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;
    let spawn_pos = Vec2::new(spawn_x, spawn_z);

    let team_filter = match team {
        Team::Defenders => TeamFilter::Defenders,
        Team::Attackers => TeamFilter::Attackers,
        _ => TeamFilter::Defenders,
    };

    let anim = WalkingAnimation::default();
    let king_material = crate::game::units::systems::create_default_sprite_material(
        materials,
        king_assets.sprite_texture.clone(),
        KING_SPRITE_TINT,
    );

    let king_entity = commands
        .spawn((
            Mesh3d(king_assets.sprite_mesh.clone()),
            MeshMaterial3d(king_material),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            crate::game::components::Velocity::default(),
            crate::game::components::Acceleration::new(),
            hitbox,
            Health::new(KING_HEALTH),
            MovementSpeed(KING_MOVEMENT_SPEED),
            staggered_attack_timing(),
            Effectiveness::new(),
            DamageMultiplier(KING_DAMAGE_PERCENTAGE),
            team,
            King,
        ))
        .insert((
            anim,
            FacingDirection::default(),
            Commander {
                aura_radius: KING_AURA_RADIUS,
                team_filter,
            },
            AuraDamageBuff(KING_AURA_DAMAGE_PERCENTAGE),
            AuraSpeedBuff(KING_AURA_SPEED_PERCENTAGE),
        ))
        .insert((
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            if team == Team::Defenders {
                FlowFieldInfluence::Defender { spawn_pos }
            } else {
                FlowFieldInfluence::Attacker
            },
            // NOTE: the King is intentionally NOT `Teleportable` in multiplayer —
            // teleporting your own King out of reach was an exploit to stall the
            // match forever. (Single-player `spawn_king` keeps it teleportable.)
            crate::game::units::components::FlockingModifier::new(1.0, 0.0, 0.0),
            Billboard,
            OnMultiplayerGameScreen,
        ))
        .id();

    // MP attacker kings/guards are pre-activated — see infantry/archer
    // spawn for rationale. WaveGroup(0) prevents `is_staging_attacker`
    // from classing them as inactive due to the missing tag.
    if team == Team::Attackers {
        commands.entity(king_entity).insert(WaveGroup(0));
    }

    // Spawn the SP-style aura sphere as a child of the king. Replaces the
    // earlier flat ground-plane circle so both MP peers — and SP — show
    // the same volumetric aura halo.
    crate::game::units::king::systems::spawn_king_aura_visual(
        commands,
        king_entity,
        spell_assets,
        OnMultiplayerGameScreen,
    );

    king_spawned.0 = true;
}
