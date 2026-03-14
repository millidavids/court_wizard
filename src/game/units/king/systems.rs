use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::KingAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::resources::InitialDefenderCount;
use crate::game::units::commander::{AuraDamageBuff, AuraSpeedBuff, Commander, TeamFilter};
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FacingDirection, FlockingModifier, FlockingVelocity,
    HasteModifier, Health, Hitbox, KingsGuard, MovementSpeed, PolymorphedModifier, RootedModifier,
    FrozenSolidModifier, RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

/// Spawns the King unit at the center of the defender grid.
///
/// King spawns in the center of the radial defender formation,
/// positioned between the wizard and battlefield center.
pub fn spawn_king(
    mut commands: Commands,
    king_assets: Res<KingAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut king_spawned: ResMut<KingSpawned>,
) {
    // King spawns at exact center of defender grid
    // Use center angle and base range (no row/col offsets)
    let angle = DEFENDER_GRID_CENTER_ANGLE;
    let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
    let spawn_x = WIZARD_POSITION.x + radius * angle.cos();
    let spawn_z = WIZARD_POSITION.z + radius * angle.sin();

    // Define King hitbox (larger than standard units)
    let hitbox = Hitbox::new(KING_RADIUS, KING_HITBOX_HEIGHT);

    // Position unit so bottom edge is 1 unit above battlefield (Y=0)
    let spawn_y = hitbox.height / 2.0 + 1.0;

    // Store spawn position for rallying when not activated
    let spawn_pos = Vec2::new(spawn_x, spawn_z);

    let anim = WalkingAnimation::default();
    let king_material = create_default_sprite_material(
        &mut materials,
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
                visual_color: KING_AURA_COLOR,
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

    // Spawn visual aura circle on the ground beneath the King
    let aura_circle = Circle::new(KING_AURA_RADIUS);
    let y_offset = 5.0 - spawn_y;
    let aura_entity = commands
        .spawn((
            Mesh3d(meshes.add(aura_circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: KING_AURA_COLOR,
                unlit: true,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, y_offset, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            OnGameplayScreen,
        ))
        .id();
    commands.entity(king_entity).add_child(aura_entity);

    // Mark that King has been spawned
    king_spawned.0 = true;
}

/// Updates King targeting velocity toward nearest enemy.
///
/// The King always moves directly toward the nearest enemy.
/// Also sets InMelee component if an enemy is within melee range.
/// King is gated by the DefendersActivated resource.
pub fn update_king_targeting(
    defenders_activated: Res<crate::game::units::infantry::components::DefendersActivated>,
    mut commands: Commands,
    mut king: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            Option<&crate::game::units::components::RetaliationTarget>,
        ),
        (With<King>, Without<Corpse>),
    >,
    all_units: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    // Collect snapshot of all unit positions
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Update King's targeting velocity
    for (entity, transform, team, mut targeting_velocity, retaliation) in &mut king {
        // Skip inactive King (wait for defenders to activate)
        if !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Use shared melee targeting function
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            transform,
            *team,
            &mut targeting_velocity,
            &mut commands,
            retaliation.map(|r| r.0),
        );
    }
}

/// King-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// King slows down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn king_movement(
    time: Res<Time>,
    mut king_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
            ),
        ),
        With<King>,
    >,
) {
    // Process King unit
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, _polymorphed, sickened, frozen),
    ) in &mut king_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(rooted, sleeping, sleepwalking, banished, sickened, frozen) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Use shared weighted movement function
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            effectiveness,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// King cohesion force system.
///
/// Applies a dynamic cohesion force to defenders, pulling them toward the King.
/// The force strength increases when enemies are near (threatened) and decreases when safe.
/// This is King-specific behavior separate from the generic commander aura system.
///
/// Note: Damage and speed buffs are now handled by the generic commander system.
pub fn king_cohesion_force(
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
    mut defenders: Query<
        (&Transform, &Team, &mut FlockingVelocity),
        (Without<King>, Without<Corpse>),
    >,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    // Process each King and apply cohesion to their team's units
    for (king_transform, king_team) in &king_query {
        let king_pos = king_transform.translation;

        // Find nearest enemy to this King
        let nearest_enemy_distance = all_units
            .iter()
            .filter(|(_, team)| *team != king_team)
            .map(|(transform, _)| transform.translation.distance(king_pos))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(f32::MAX);

        // Calculate threat level: interpolate between BASE and THREATENED
        let threat_factor = if nearest_enemy_distance > KING_AURA_RADIUS {
            0.0
        } else {
            1.0 - (nearest_enemy_distance / KING_AURA_RADIUS)
        };

        let cohesion_strength =
            KING_COHESION_BASE + (KING_COHESION_THREATENED - KING_COHESION_BASE) * threat_factor;

        // Apply cohesion force to same-team defenders within aura radius
        for (unit_transform, team, mut flocking_velocity) in &mut defenders {
            if team != king_team {
                continue;
            }

            let unit_pos = unit_transform.translation;
            let distance_to_king = unit_pos.distance(king_pos);

            // Check if unit is within aura radius
            if distance_to_king < KING_AURA_RADIUS && distance_to_king > 0.1 {
                // Calculate direction toward King
                let to_king = (king_pos - unit_pos).normalize_or_zero();

                // Add cohesion force to flocking velocity
                // Scale by distance (stronger pull when closer to edge of aura)
                let distance_factor = distance_to_king / KING_AURA_RADIUS;
                let cohesion_force = to_king * cohesion_strength * distance_factor;

                flocking_velocity.velocity += Vec3::new(cohesion_force.x, 0.0, cohesion_force.z);

                // Re-normalize to maintain consistent influence
                flocking_velocity.velocity = flocking_velocity.velocity.normalize_or_zero();
            }
        }
    }
}

/// Snaps King's Guard units to fixed positions around the King each frame.
///
/// Guards orbit the King at a fixed radius. Their positions are set directly
/// rather than using velocity/acceleration, so they stay locked to the King.
pub fn snap_kings_guard_to_king(
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
    mut guards: Query<(&KingsGuard, &Team, &mut Transform), (Without<King>, Without<Corpse>)>,
) {
    // Snap each guard to their team's King
    for (king_transform, king_team) in &king_query {
        let king_pos = king_transform.translation;

        for (guard, guard_team, mut transform) in &mut guards {
            if guard_team != king_team {
                continue;
            }
            let angle = guard.0 as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
            transform.translation.x = king_pos.x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
            transform.translation.z = king_pos.z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();
        }
    }
}

/// Attaches SpellShield to the King in multiplayer games.
///
/// Runs once when a King is first spawned (via `Added<King>` filter).
/// Only activates when a `MultiplayerSession` resource exists.
/// Also spawns the translucent shield visual as a child entity.
pub fn attach_king_spell_shield(
    mut commands: Commands,
    session: Option<Res<MultiplayerSession>>,
    new_kings: Query<(Entity, &Transform), Added<King>>,
    spell_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if session.is_none() {
        return;
    }

    for (entity, _transform) in &new_kings {
        commands.entity(entity).insert(SpellShield);

        // Spawn translucent cross-plane sphere visual as child
        let shield_visual = commands
            .spawn((
                Mesh3d(spell_assets.cross_plane_sphere.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: SPELL_SHIELD_COLOR,
                    unlit: true,
                    alpha_mode: bevy::prelude::AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_scale(Vec3::splat(SPELL_SHIELD_RADIUS)),
                SpellShieldVisual,
                OnGameplayScreen,
            ))
            .id();
        commands.entity(entity).add_child(shield_visual);
    }
}

/// Removes the King's spell shield when fewer than 10% of non-King defenders remain.
pub fn update_king_spell_shield(
    mut commands: Commands,
    king_query: Query<Entity, (With<King>, With<SpellShield>, Without<Corpse>)>,
    defenders: Query<&Team, (Without<Corpse>, Without<King>)>,
    initial_count: Option<Res<InitialDefenderCount>>,
    shield_visuals: Query<Entity, With<SpellShieldVisual>>,
) {
    let Some(initial_count) = initial_count else {
        return;
    };

    let Ok(king_entity) = king_query.single() else {
        return;
    };

    let living_defenders = defenders
        .iter()
        .filter(|t| matches!(t, Team::Defenders))
        .count() as f32;

    if living_defenders / initial_count.0 as f32 <= SPELL_SHIELD_THRESHOLD {
        commands.entity(king_entity).remove::<SpellShield>();

        // Despawn shield visual
        for visual_entity in &shield_visuals {
            commands.entity(visual_entity).try_despawn();
        }
    }
}
