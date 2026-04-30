//! Archer targeting, movement, and spawn helpers.

use super::combat::wall_near_approach_path;
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::ArcherAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_defender_grid_position, cells_needed, distribute_units_to_cells, *,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, Effectiveness,
    EliteSpeedBonus, FacingDirection, FlockingModifier, FlockingVelocity, FrozenSolidModifier,
    HasteModifier, Health, Hitbox, MovementSpeed, PolymorphedModifier, RootedModifier,
    RoughTerrainModifier, SickenedModifier, SleepModifier, Sleepwalking, SlowMovementModifier,
    TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Updates archer movement timers to track time since stopped moving.
pub fn update_archer_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut archers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &AttackRange,
            &mut crate::game::units::components::TargetingVelocity,
        ),
        (
            With<Archer>,
            Without<Corpse>,
            Without<crate::game::units::components::MindControlled>,
        ),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
    walls: Query<&WallOfStone>,
    rocks_query2: Query<&crate::game::terrain::boulder::components::Boulder>,
    trees_query2: Query<&crate::game::terrain::tree::components::Tree>,
) {
    // Collect snapshot of all unit positions (excludes staging attackers)
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Collect wall, rock, and tree snapshots for line-of-sight checks
    let wall_snapshot: Vec<_> = walls.iter().collect();
    let rock_snapshot: Vec<_> = rocks_query2.iter().filter(|r| !r.sinking).collect();
    let tree_snapshot: Vec<_> = trees_query2.iter().collect();

    // Update each archer's targeting velocity
    for (entity, transform, team, attack_range, mut targeting_velocity) in &mut archers {
        // Skip inactive defender archers (but always process attackers)
        if *team == Team::Defenders && !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        let pos = transform.translation;

        // Find nearest enemy in seek zone [min_range, seek_range]
        // Archers advance until enemies are within seek range, then stop.
        // They can still shoot up to max_range, but won't stop that far out.
        // Only count targets with clear line-of-sight (no walls blocking).
        let ranged_target = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
            })
            .filter_map(|&(_, target_pos, _)| {
                let dx = pos.x - target_pos.x;
                let dz = pos.z - target_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist >= attack_range.min_range && dist <= ARCHER_SEEK_RANGE {
                    // Check line-of-sight: skip if any wall, rock, or tree blocks the shot
                    if WallOfStone::any_blocks_los(&wall_snapshot, pos, target_pos)
                        || crate::game::terrain::boulder::components::Boulder::any_blocks_los(
                            &rock_snapshot,
                            pos,
                            target_pos,
                        )
                        || crate::game::terrain::tree::components::Tree::any_blocks_los(
                            &tree_snapshot,
                            pos,
                            target_pos,
                        )
                    {
                        None
                    } else {
                        Some((dist, target_pos))
                    }
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Find nearest enemy overall (fallback for melee or advancing)
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
            })
            .min_by(|a, b| {
                let dist_a = (pos.x - a.1.x).powi(2) + (pos.z - a.1.z).powi(2);
                let dist_b = (pos.x - b.1.x).powi(2) + (pos.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        // Prefer ranged targets — only fall back to melee/advance if none in range
        if let Some((ranged_dist, target_pos)) = ranged_target {
            // If a wall is near the approach path (even if it doesn't block direct
            // LOS), keep advancing so the flow field routes the archer around it.
            // Otherwise enter shooting stance.
            targeting_velocity.velocity =
                if wall_near_approach_path(&wall_snapshot, pos, target_pos) {
                    Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z).normalize_or_zero()
                } else {
                    Vec3::ZERO
                };
            targeting_velocity.distance_to_target = ranged_dist;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
        } else if let Some(&(_, target_pos, enemy_team)) = nearest_enemy {
            let diff = target_pos - pos;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            let in_melee_range = distance < MELEE_SLOWDOWN_DISTANCE;
            if in_melee_range {
                commands
                    .entity(entity)
                    .insert(crate::game::units::components::InMelee(enemy_team));
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            } else {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
                // Beyond max range — advance toward enemy
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
        }
    }
}

/// Archer-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// Units slow down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn archer_movement(
    time: Res<Time>,
    mut archer_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &crate::game::units::components::FlockingVelocity,
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
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
                &Team,
                Has<StagingAttacker>,
                Has<WaveGroup>,
            ),
        ),
        With<Archer>,
    >,
) {
    // Process each archer unit
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (
            sleeping,
            sleepwalking,
            banished,
            polymorphed,
            sickened,
            frozen,
            stunned,
            petrified,
            team,
            has_staging,
            has_wave_group,
        ),
    ) in &mut archer_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Polymorphed units wander randomly
        if polymorphed.is_some() {
            let angle = (time.elapsed_secs() * 0.5 + velocity.x.to_bits() as f32).sin()
                * std::f32::consts::TAU;
            velocity.x = angle.cos() * 20.0;
            velocity.z = angle.sin() * 20.0;
            continue;
        }

        // Use shared weighted movement function
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
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

        // Archer-specific: Stop completely when in optimal shooting range (not in melee).
        // But keep moving if:
        //  - staging (needs to follow flow field to staging point)
        //  - standing on hazardous terrain (fire, spikes)
        //  - no target in range (needs to follow flow field back to spawn)
        //  - path is fully blocked (wall-attack system needs velocity)
        let is_staging =
            crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);
        if !is_staging
            && in_melee.is_none()
            && flow_field_velocity.terrain_cost <= 1.0
            && !flow_field_velocity.pathfinding_distance.is_infinite()
            && targeting_velocity.distance_to_target < f32::MAX
        {
            let targeting_is_zero = targeting_velocity.velocity.length_squared() < 0.01;
            if targeting_is_zero {
                // Override velocity and acceleration to completely stop archer when in shooting stance
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.x = 0.0;
                acceleration.z = 0.0;
            }
        }
    }
}

/// Spawns a single defender archer unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_defender_archer(
    rng: &mut impl Rng,
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
) {
    // Calculate where infantry spawned to determine archer row
    let infantry_cells = cells_needed(INITIAL_DEFENDER_COUNT);
    let infantry_rows = infantry_cells.div_ceil(DEFENDER_GRID_COLS);
    // Infantry start at row (ROWS-1) and fill `infantry_rows` rows, ending at (ROWS-1-infantry_rows+1)
    // Archers go one row lower than that
    let last_infantry_row = DEFENDER_GRID_ROWS.saturating_sub(infantry_rows);
    let archer_row = last_infantry_row.saturating_sub(1);

    let archer_cells_needed = cells_needed(INITIAL_ARCHER_DEFENDER_COUNT);
    let units_per_cell = distribute_units_to_cells(INITIAL_ARCHER_DEFENDER_COUNT);

    // Calculate which cell this unit belongs to
    let mut units_counted = 0;
    for cell_idx in 0..archer_cells_needed.min(DEFENDER_GRID_COLS) {
        let units_in_this_cell = units_per_cell[cell_idx as usize];
        if unit_index < units_counted + units_in_this_cell {
            // This unit goes in this cell
            let (spawn_x, spawn_z) = calculate_defender_grid_position(archer_row, cell_idx);
            let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

            let hitbox = Hitbox::new(ARCHER_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            let anim = WalkingAnimation::new_staggered(rng);
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                archer_assets.sprite_texture.clone(),
                DEFENDER_SPRITE_TINT,
            );

            commands
                .spawn((
                    Mesh3d(archer_assets.sprite_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(ARCHER_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Defenders,
                    Archer,
                ))
                .insert((
                    anim,
                    FacingDirection::default(),
                    AttackRange {
                        min_range: ARCHER_MIN_RANGE,
                        max_range: ARCHER_MAX_RANGE,
                    },
                    ArcherMovementTimer::new(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Defender {
                        spawn_pos: Vec2::new(spawn_x, spawn_z),
                    },
                    FlockingModifier::new(1.0, 1.0, 0.0),
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single attacker archer unit at a specific index.
/// Used for progressive loading.
pub(in crate::game) fn spawn_single_attacker_archer(
    rng: &mut impl Rng,
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    _level: u32,
) -> Entity {
    let (spawn_x, spawn_z) = attacker_spawn_position(unit_index, ARCHER_SPAWN_DEPTH_OFFSET);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(ARCHER_RADIUS, ATTACKER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let anim = WalkingAnimation::new_staggered(rng);
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        archer_assets.sprite_texture.clone(),
        ATTACKER_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(archer_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(UNIT_HEALTH),
            MovementSpeed(ARCHER_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Archer,
        ))
        .insert((
            anim,
            FacingDirection::default(),
            AttackRange {
                min_range: ARCHER_MIN_RANGE,
                max_range: ARCHER_MAX_RANGE,
            },
            ArcherMovementTimer::new(),
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ))
        .id()
}
