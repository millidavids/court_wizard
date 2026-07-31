use crate::game::units::components::Corpse;
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::KingAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::commander::{AuraDamageBuff, AuraSpeedBuff, Commander, TeamFilter};
use crate::game::units::components::{
    AttackTiming, DamageMultiplier, Effectiveness, FacingDirection, FlockingModifier,
    FlockingVelocity, Health, Hitbox, MovementSpeed, TargetingVelocity, Team, Teleportable,
    WalkingAnimation,
};
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns the King unit at the center of the defender grid.
///
/// King spawns in the center of the radial defender formation,
/// positioned between the wizard and battlefield center.
pub fn spawn_king(
    commands: &mut Commands,
    king_assets: &KingAssets,
    _meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spell_assets: &SpellVisualAssets,
    king_spawned: &mut KingSpawned,
) {
    // King spawns at exact center of defender grid
    // Use center angle and base range (no row/col offsets)
    let angle = DEFENDER_GRID_CENTER_ANGLE;
    let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
    // The King is bound to the playable area with no slack, so both his post and
    // the rally point derived from it must sit far enough inside that the
    // perimeter steering force can't slowly walk him off it.
    let post = crate::game::movement_systems::clamp_king_post(Vec2::new(
        WIZARD_POSITION.x + radius * angle.cos(),
        WIZARD_POSITION.z + radius * angle.sin(),
    ));
    let (spawn_x, spawn_z) = (post.x, post.y);

    // Define King hitbox (larger than standard units)
    let hitbox = Hitbox::new(KING_RADIUS, KING_HITBOX_HEIGHT);

    // Position unit so bottom edge is 1 unit above battlefield (Y=0)
    let spawn_y = hitbox.height / 2.0 + 1.0;

    // Store spawn position for rallying when not activated
    let spawn_pos = Vec2::new(spawn_x, spawn_z);

    let anim = WalkingAnimation::default();
    let king_material = create_default_sprite_material(
        materials,
        king_assets.sprite_texture.clone(),
        KING_SPRITE_TINT,
    );

    // Spawn the King unit with Commander components
    let king_entity = commands
        .spawn((
            Mesh3d(king_assets.sprite_mesh.clone()),
            MeshMaterial3d(king_material),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(KING_HEALTH),
            MovementSpeed(KING_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            DamageMultiplier(KING_DAMAGE_PERCENTAGE),
            Team::Defenders,
            King, // Marker for game-ending logic
        ))
        .insert((
            anim,
            FacingDirection::default(),
            // Commander components
            Commander {
                aura_radius: KING_AURA_RADIUS,
                team_filter: TeamFilter::Defenders,
            },
            AuraDamageBuff(KING_AURA_DAMAGE_PERCENTAGE),
            AuraSpeedBuff(KING_AURA_SPEED_PERCENTAGE),
        ))
        .insert((
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Defender { spawn_pos },
            Teleportable,
            FlockingModifier::new(1.0, 0.0, 0.0),
            Billboard,
            OnGameplayScreen,
        ))
        .id();

    // Spawn the visual aura sphere as a child of the king.
    spawn_king_aura_visual(commands, king_entity, spell_assets, OnGameplayScreen);

    // Mark that King has been spawned
    king_spawned.0 = true;
}

/// Spawns the king's aura sphere (the SP-style halo) as a child of the given
/// king entity. Both single-player and multiplayer kings — host-local AND
/// ghost — call this so the aura looks identical on every peer. Generic over
/// the screen-cleanup marker (`OnGameplayScreen` for SP, `OnMultiplayerGameScreen`
/// for MP) so the right cleanup hook sweeps it.
///
/// The mesh + material are the same shared `explosion_sphere` and
/// `king_aura_sphere` handles used everywhere else; both are reference-counted
/// asset handles and nothing later mutates them per-entity, so sharing is safe.
pub(in crate::game) fn spawn_king_aura_visual<M: Component>(
    commands: &mut Commands,
    parent_entity: Entity,
    spell_assets: &SpellVisualAssets,
    screen_marker: M,
) {
    let aura_entity = commands
        .spawn((
            Mesh3d(spell_assets.explosion_sphere.clone()),
            MeshMaterial3d(spell_assets.king_aura_sphere.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(KING_AURA_RADIUS)),
            // `KingAuraVisual` marker lets `despawn_king_aura_on_death`
            // find and despawn the aura when the king dies — the king's
            // entity itself becomes a corpse rather than being despawned,
            // so child hierarchy cleanup does not fire.
            super::super::components::KingAuraVisual,
            screen_marker,
        ))
        .id();
    commands.entity(parent_entity).add_child(aura_entity);
}

/// Despawns the king's aura sphere when the king becomes a corpse. Without
/// this, the flat corpse sprite ends up surrounded by the still-glowing
/// aura halo — visually wrong on both SP and MP. The aura is a child of
/// the king entity, so we look it up via `Children` rather than a global
/// query (avoids tearing down the OTHER king's aura when one king dies).
pub fn despawn_king_aura_on_death(
    mut commands: Commands,
    dead_kings: Query<&Children, (With<King>, Added<Corpse>)>,
    aura_visuals: Query<(), With<super::super::components::KingAuraVisual>>,
) {
    for children in &dead_kings {
        for child in children.iter() {
            if aura_visuals.contains(child)
                && let Ok(mut ec) = commands.get_entity(child)
            {
                ec.try_despawn();
            }
        }
    }
}
