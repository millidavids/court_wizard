use bevy::prelude::*;
use rand::Rng;

use super::components::Assassin;
use super::constants::*;
use super::resources::AssassinAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    ASSASSIN_SPAWN_DEPTH_OFFSET, ATTACKER_HITBOX_HEIGHT, attacker_spawn_position,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::archer::Archer;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, Effectiveness, FacingDirection, FlockingVelocity,
    Health, Hitbox, MindControlled, MovementSpeed, TargetingVelocity, Team, Teleportable,
    WalkingAnimation,
};
use crate::game::units::king::components::King;
use crate::game::units::random_position_in_cell;

/// Updates assassin targeting velocity.
///
/// Assassins specifically target archers. If no archers are alive, they fall back
/// to targeting the King. Direct targeting takes over from the flow field when
/// the assassin gets close enough (within TARGETING_CROSSOVER_DISTANCE).
pub fn update_assassin_targeting(
    mut assassins: Query<
        (&Transform, &Team, &mut TargetingVelocity),
        (With<Assassin>, Without<Corpse>, Without<MindControlled>),
    >,
    archers: Query<
        (Entity, &Transform, &Team),
        (With<Archer>, Without<Corpse>, Without<BanishedModifier>),
    >,
    kings: Query<(Entity, &Transform, &Team), (With<King>, Without<Corpse>)>,
) {
    for (transform, team, mut targeting_velocity) in &mut assassins {
        let pos = transform.translation;

        // Find nearest enemy archer
        let nearest_archer = archers
            .iter()
            .filter(|(_, _, archer_team)| team.is_enemy(archer_team))
            .map(|(_, archer_transform, _)| {
                let diff = archer_transform.translation - pos;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                (dist, archer_transform.translation)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Find nearest enemy king as fallback
        let nearest_king = kings
            .iter()
            .filter(|(_, _, king_team)| team.is_enemy(king_team))
            .map(|(_, king_transform, _)| {
                let diff = king_transform.translation - pos;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                (dist, king_transform.translation)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Prefer archers, fall back to king
        let target = nearest_archer.or(nearest_king);

        if let Some((distance, target_pos)) = target {
            targeting_velocity.distance_to_target = distance;

            if distance < TARGETING_CROSSOVER_DISTANCE {
                // Close enough — direct targeting takes over
                let direction = (target_pos - pos).normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            } else {
                // Far away — let flow field guide, minimal targeting
                targeting_velocity.velocity = Vec3::ZERO;
            }
        } else {
            // No targets at all
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }
    }
}

/// Assassin movement system.
///
/// Uses shared weighted movement but with the assassin's fast speed.
#[allow(clippy::type_complexity)]
pub fn assassin_movement(
    time: Res<Time>,
    mut assassin_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::CommanderAuraSpeedModifier>,
            Option<&crate::game::units::components::RoughTerrainModifier>,
            Option<&crate::game::units::components::SlowMovementModifier>,
            (
                Option<&crate::game::cauldron::components::CauldronSpeedModifier>,
                Option<&crate::game::units::components::RootedModifier>,
                Option<&crate::game::units::components::HasteModifier>,
                Option<&crate::game::units::components::EliteSpeedBonus>,
            ),
            (
                Has<crate::game::units::components::SleepModifier>,
                Has<crate::game::units::components::Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&crate::game::units::components::PolymorphedModifier>,
                Option<&crate::game::units::components::SickenedModifier>,
                Option<&crate::game::units::components::FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        With<Assassin>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut assassin_units
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

        // Use shared weighted movement
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            false, // Assassins never slow down in melee
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// Spawns a single attacker assassin at a specific index.
/// Assassins spawn behind infantry rows (further from defenders).
pub(in crate::game) fn spawn_single_attacker_assassin(
    rng: &mut impl Rng,
    commands: &mut Commands,
    assassin_assets: &AssassinAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    _level: u32,
) {
    let (spawn_x, spawn_z) = attacker_spawn_position(unit_index, ASSASSIN_SPAWN_DEPTH_OFFSET);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(ASSASSIN_RADIUS, ATTACKER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let anim = WalkingAnimation::new_staggered(rng);

    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        assassin_assets.sprite_texture.clone(),
        ASSASSIN_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(assassin_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(ASSASSIN_HEALTH),
            MovementSpeed(ASSASSIN_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Assassin,
        ))
        .insert((
            anim,
            FacingDirection::default(),
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Assassin,
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
}
