use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::components::{
    AttackTiming, Corpse, DamageMultiplier, Effectiveness, FlockingModifier, FlockingVelocity,
    FrostSlowModifier, Health, Hitbox, KingAuraSpeedModifier, KingsGuard, MovementSpeed,
    RoughTerrainModifier, TargetingVelocity, Team, Teleportable,
};

/// Spawns the King unit at the center of the defender grid.
///
/// King spawns in the center of the radial defender formation,
/// positioned between the wizard and battlefield center.
pub fn spawn_king(
    mut commands: Commands,
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

    // Spawn King as a circle billboard sized to match the hitbox
    let circle = Circle::new(hitbox.radius);

    // Position unit so bottom edge is 1 unit above battlefield (Y=0)
    let spawn_y = hitbox.height / 2.0 + 1.0;

    // Store spawn position for rallying when not activated
    let spawn_pos = Vec2::new(spawn_x, spawn_z);

    // Spawn the King unit
    let king_entity = commands
        .spawn((
            Mesh3d(meshes.add(circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: KING_COLOR,
                unlit: true,
                ..default()
            })),
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
            King,
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

    // Spawn visual aura sphere as a child entity centered on the King
    // The sphere's radius exactly represents the 3D distance check used by the aura system
    let aura_sphere = Sphere::new(KING_AURA_RADIUS);
    commands
        .spawn((
            Mesh3d(meshes.add(aura_sphere)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.6, 0.0, 0.05), // Very transparent orange sphere
                unlit: true,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                cull_mode: None, // Visible from both sides
                ..default()
            })),
            // Center sphere on King (relative position 0,0,0 since it's a child entity)
            // This accurately represents the 3D spherical distance check
            Transform::from_xyz(0.0, 0.0, 0.0),
            OnGameplayScreen,
        ))
        .set_parent_in_place(king_entity);

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
        (Entity, &Transform, &Team, &mut TargetingVelocity),
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
    for (entity, transform, team, mut targeting_velocity) in &mut king {
        // Skip inactive King (wait for defenders to activate)
        if !defenders_activated.active {
            *targeting_velocity = TargetingVelocity::default();
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
            Option<&KingAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&FrostSlowModifier>,
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
        frost_modifier,
    ) in &mut king_units
    {
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
            frost_modifier.map(|m| m.modifier),
        );
    }
}

/// King cohesion aura system.
///
/// Applies a dynamic cohesion force to all nearby units, pulling them toward the King.
/// The force strength increases when enemies are near (threatened) and decreases when safe.
/// Defenders are drawn to protect the King, attackers are drawn to kill the King.
/// Also applies/removes damage and speed buffs to defenders within aura range.
/// The King himself also receives the aura buffs.
pub fn king_cohesion_aura(
    mut commands: Commands,
    king_query: Query<(Entity, &Transform), (With<King>, Without<Corpse>)>,
    mut all_affected_units: Query<
        (Entity, &Transform, &Team, &mut FlockingVelocity),
        (Without<King>, Without<Corpse>),
    >,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    // Get King entity and position (should only be one)
    let Ok((king_entity, king_transform)) = king_query.single() else {
        return;
    };

    let king_pos = king_transform.translation;

    // Find nearest enemy to King
    let nearest_enemy_distance = all_units
        .iter()
        .filter(|(_, team)| **team != Team::Defenders)
        .map(|(transform, _)| transform.translation.distance(king_pos))
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(f32::MAX);

    // Calculate threat level: interpolate between BASE and THREATENED
    // If enemy is far (> AURA_RADIUS), use BASE
    // If enemy is close (< AURA_RADIUS), interpolate to THREATENED
    let threat_factor = if nearest_enemy_distance > KING_AURA_RADIUS {
        0.0
    } else {
        1.0 - (nearest_enemy_distance / KING_AURA_RADIUS)
    };

    let cohesion_strength =
        KING_COHESION_BASE + (KING_COHESION_THREATENED - KING_COHESION_BASE) * threat_factor;

    // Apply cohesion force to all units within aura radius, damage and speed buffs only to defenders
    for (entity, unit_transform, team, mut flocking_velocity) in &mut all_affected_units {
        let unit_pos = unit_transform.translation;
        let distance_to_king = unit_pos.distance(king_pos);

        // Check if unit is within aura radius
        if distance_to_king < KING_AURA_RADIUS && distance_to_king > 0.1 {
            // Apply cohesion force only to defenders (they protect the King)
            // Attackers use their normal targeting behavior to attack the King
            if *team == Team::Defenders {
                // Calculate direction toward King
                let to_king = (king_pos - unit_pos).normalize_or_zero();

                // Add cohesion force to flocking velocity
                // Scale by distance (stronger pull when closer to edge of aura)
                let distance_factor = distance_to_king / KING_AURA_RADIUS;
                let cohesion_force = to_king * cohesion_strength * distance_factor;

                flocking_velocity.velocity += Vec3::new(cohesion_force.x, 0.0, cohesion_force.z);

                // Re-normalize to maintain consistent influence
                flocking_velocity.velocity = flocking_velocity.velocity.normalize_or_zero();

                // Apply damage and speed buffs to defenders (just set to fixed value)
                commands
                    .entity(entity)
                    .insert(DamageMultiplier(KING_AURA_DAMAGE_PERCENTAGE));
                commands
                    .entity(entity)
                    .insert(KingAuraSpeedModifier(KING_AURA_SPEED_PERCENTAGE));
            }
        } else if *team == Team::Defenders {
            // Remove aura buffs if defender is outside aura
            commands.entity(entity).remove::<DamageMultiplier>();
            commands.entity(entity).remove::<KingAuraSpeedModifier>();
        }
    }

    // Apply aura buffs to the King himself (he's always in his own aura)
    // The King gets speed buff but not damage buff (he already has base damage multiplier)
    commands
        .entity(king_entity)
        .insert(KingAuraSpeedModifier(KING_AURA_SPEED_PERCENTAGE));
}

/// Snaps King's Guard units to fixed positions around the King each frame.
///
/// Guards orbit the King at a fixed radius. Their positions are set directly
/// rather than using velocity/acceleration, so they stay locked to the King.
pub fn snap_kings_guard_to_king(
    king_query: Query<&Transform, (With<King>, Without<Corpse>)>,
    mut guards: Query<(&KingsGuard, &mut Transform), (Without<King>, Without<Corpse>)>,
) {
    let Ok(king_transform) = king_query.single() else {
        return;
    };
    let king_pos = king_transform.translation;

    for (guard, mut transform) in &mut guards {
        let angle = guard.0 as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
        transform.translation.x = king_pos.x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
        transform.translation.z = king_pos.z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();
    }
}
