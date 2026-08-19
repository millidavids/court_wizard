use bevy::math::Affine2;
use bevy::prelude::*;
use rand::Rng;

use super::super::components::{
    CORPSE_MATERIAL_VARIANTS, Corpse, FacingDirection, FlockingVelocity, RoughTerrain,
    SPRITE_FRAME_SIZE, SPRITE_SHEET_IMAGE_HEIGHT, Team, WalkingAnimation,
};
use crate::game::components::{Acceleration, Billboard, Velocity};
use crate::game::constants::DEFENDER_HITBOX_HEIGHT;

/// Creates corpse sprite materials, each using a random frame from the sprite sheet.
///
/// Each material shows one frozen animation frame with the corpse tint color applied.
/// These are shared across all deaths of a given unit type/team for efficiency.
pub fn create_corpse_sprite_materials(
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    tint: Color,
) -> [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS] {
    let mut rng = rand::rng();
    let anim = WalkingAnimation::default();
    let rows = (SPRITE_SHEET_IMAGE_HEIGHT / SPRITE_FRAME_SIZE) as usize;
    let total_frames = anim.columns * rows;

    std::array::from_fn(|_| {
        let frame = rng.random_range(0..total_frames);
        let col = frame % anim.columns;
        let row = frame / anim.columns;
        let uv_offset = Vec2::new(col as f32 * anim.frame_uv.x, row as f32 * anim.frame_uv.y);
        let handle =
            create_sprite_material(materials, texture.clone(), tint, anim.frame_uv, uv_offset);
        // Corpses use semi-transparent alpha, so override to Blend
        if let Some(mut mat) = materials.get_mut(&handle) {
            mat.alpha_mode = AlphaMode::Blend;
        }
        handle
    })
}

/// Creates a per-entity sprite material with the given texture and tint.
pub fn create_sprite_material(
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    tint: Color,
    uv_scale: Vec2,
    uv_offset: Vec2,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        base_color: tint,
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        uv_transform: Affine2::from_scale_angle_translation(uv_scale, 0.0, uv_offset),
        ..default()
    })
}

/// Creates a sprite material using the default forward-facing UV offset.
///
/// Convenience wrapper used by all unit spawn functions.
pub fn create_default_sprite_material(
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    tint: Color,
) -> Handle<StandardMaterial> {
    let anim = WalkingAnimation::default();
    create_sprite_material(
        materials,
        texture,
        tint,
        anim.frame_uv,
        anim.uv_offset(FacingDirection::default()),
    )
}

/// Picks a corpse material by team from the standard `[defender, attacker, undead]` arrays.
pub fn corpse_material_for_team(
    defender: &[Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    attacker: &[Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    undead: &[Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    team: Team,
    idx: usize,
) -> Handle<StandardMaterial> {
    match team {
        Team::Defenders => defender[idx].clone(),
        Team::Attackers => attacker[idx].clone(),
        Team::Undead => undead[idx].clone(),
    }
}

/// Returns the sprite tint color for a given team (infantry/generic).
pub fn sprite_tint_for_team(team: Team) -> Color {
    use super::super::infantry::constants::{
        ATTACKER_SPRITE_TINT, DEFENDER_SPRITE_TINT, UNDEAD_SPRITE_TINT,
    };
    match team {
        Team::Defenders => DEFENDER_SPRITE_TINT,
        Team::Attackers => ATTACKER_SPRITE_TINT,
        Team::Undead => UNDEAD_SPRITE_TINT,
    }
}

/// Returns the sprite tint color for archers (lighter attacker tint).
pub fn archer_sprite_tint_for_team(team: Team) -> Color {
    use super::super::archer::constants::ATTACKER_SPRITE_TINT as ARCHER_ATTACKER_TINT;
    use super::super::infantry::constants::{DEFENDER_SPRITE_TINT, UNDEAD_SPRITE_TINT};
    match team {
        Team::Defenders => DEFENDER_SPRITE_TINT,
        Team::Attackers => ARCHER_ATTACKER_TINT,
        Team::Undead => UNDEAD_SPRITE_TINT,
    }
}

/// Resurrects a corpse entity as an infantry unit with sprite animation.
///
/// Shared between Raise the Dead, Perpetual Unrest, and Font of Life.
/// The caller should add any extra components (RaisedUndead, PermanentCorpse, etc.) afterward.
///
/// `sprite_texture` and `sprite_mesh` allow overriding the default infantry sprites
/// (e.g. passing undead-specific textures for Team::Undead).
#[allow(clippy::too_many_arguments)]
pub fn resurrect_corpse_as_infantry(
    commands: &mut Commands,
    entity: Entity,
    position: Vec3,
    team: Team,
    health: f32,
    speed: f32,
    tint: Color,
    sprite_texture: Handle<Image>,
    sprite_mesh: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    rising_death_texture: Option<Handle<Image>>,
) {
    use super::super::components::{
        AttackTiming, Effectiveness, Hitbox, MovementSpeed, TargetingVelocity, Teleportable,
    };
    use super::super::infantry::components::Infantry;
    use super::super::infantry::constants::UNIT_RADIUS;

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;
    let upright_transform = Transform::from_xyz(position.x, spawn_y, position.z);

    let anim = WalkingAnimation::default();
    let material = create_default_sprite_material(materials, sprite_texture.clone(), tint);

    let mut entity_cmds = commands.entity(entity);
    if let Some(death_tex) = rising_death_texture {
        entity_cmds.insert(super::super::components::RisingAnimation::new(
            death_tex,
            sprite_texture,
        ));
    }

    entity_cmds
        .remove::<Corpse>()
        .remove::<RoughTerrain>()
        // Strip all non-infantry unit markers so resurrected units
        // behave purely as infantry (melee only, no special pathing).
        .remove::<super::super::archer::Archer>()
        .remove::<super::super::archer::components::ArcherMovementTimer>()
        .remove::<super::super::assassin::Assassin>()
        .remove::<super::super::dispeller::components::Dispeller>()
        .remove::<super::super::ranged_bolt::RangedAttackTimer>()
        .remove::<super::super::dispeller::components::DispellerDispelCooldown>()
        .remove::<super::super::shielder::components::Shielder>()
        .remove::<super::super::shielder::components::ShielderShieldCooldown>()
        .remove::<super::super::shielder::components::ShielderDamageReduction>()
        .remove::<super::super::healer::components::Healer>()
        .remove::<super::super::healer::components::HealerAttackTimer>()
        .remove::<super::super::commander::components::Commander>()
        .remove::<super::super::brute::components::Brute>()
        .remove::<super::super::aerialist::Aerialist>()
        .remove::<super::super::components::Flying>()
        // Strip staging/wave components so resurrected undead don't
        // trigger wave activation or get treated as staging attackers.
        .remove::<crate::game::pathfinding::StagingAttacker>()
        .remove::<crate::game::pathfinding::WaveGroup>()
        .insert(upright_transform)
        .insert(Mesh3d(sprite_mesh))
        .insert(MeshMaterial3d(material))
        .insert(anim)
        .insert(FacingDirection::default())
        .insert(team)
        .insert(super::super::components::Health::new(health))
        .insert(Velocity::default())
        .insert(Acceleration::new())
        .insert(MovementSpeed(speed))
        .insert(AttackTiming::new())
        .insert(Effectiveness::new())
        .insert(Billboard)
        .insert(hitbox)
        .insert(Teleportable)
        .insert(Infantry)
        .insert(TargetingVelocity::default())
        .insert(FlockingVelocity::default())
        // Ensure resurrected units use the standard attacker flow field
        .insert(crate::game::pathfinding::FlowFieldInfluence::Attacker);
}
