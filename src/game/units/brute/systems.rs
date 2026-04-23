use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker};

use crate::game::resources::CurrentLevel;
use crate::game::terrain::boulder::constants::{
    BOULDER_SPRITE_COUNT, ROCK_THROW_COOLDOWN, ROCK_THROW_RANGE,
};
use crate::game::terrain::boulder::messages::BoulderThrownMessage;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FlockingModifier, FlockingVelocity, FrozenSolidModifier,
    HasteModifier, Health, Hitbox, InMelee, MindControlled, MovementSpeed, PolymorphedModifier,
    RetaliationTarget, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier, TargetingVelocity, Team, Teleportable, UnitTypeGlow,
};
use crate::game::units::infantry::constants::ATTACKER_SPRITE_TINT;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::random_position_in_cell;

/// Spawns a brute attacker.
/// Brutes spawn in the archer row alongside archers.
pub fn spawn_brute(
    rng: &mut impl Rng,
    mut commands: Commands,
    infantry_assets: Res<InfantryAssets>,
    materials: &mut Assets<StandardMaterial>,
    _current_level: Res<CurrentLevel>,
) {
    // Brute spawns at the front with infantry
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(BRUTE_RADIUS, BRUTE_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * BRUTE_MOVEMENT_SPEED;

    // Use infantry sprite mesh but scaled larger
    let anim = crate::game::units::components::WalkingAnimation::new_staggered(rng);
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        infantry_assets.sprite_texture.clone(),
        ATTACKER_SPRITE_TINT,
    );

    commands
        .spawn((
            // Rendering — infantry sprite, scaled up
            Mesh3d(infantry_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z).with_scale(Vec3::splat(BRUTE_SCALE)),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(BRUTE_HEALTH),
            MovementSpeed(BRUTE_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Brute,
            // Movement systems
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
        ))
        .insert((
            anim,
            crate::game::units::components::FacingDirection::default(),
            // Normal single-target melee damage
            DamageMultiplier(0.0),
            FlockingVelocity::default(),
            FlockingModifier::new(1.0, 1.0, 1.0),
            CommanderAuraSpeedModifier(0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
            UnitTypeGlow {
                color: crate::game::units::constants::BRUTE_GLOW_COLOR,
            },
            // Rock throw with initial cooldown so brute doesn't throw immediately on spawn
            RockThrowCooldown::new(5.0),
        ));
}

/// Updates brute targeting velocity toward nearest enemy.
pub fn update_brute_targeting(
    mut commands: Commands,
    mut brutes: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            Option<&RetaliationTarget>,
        ),
        (With<Brute>, Without<MindControlled>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Brute>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, brute_transform, brute_team, mut targeting, retaliation) in &mut brutes {
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            brute_transform,
            *brute_team,
            &mut targeting,
            &mut commands,
            retaliation.map(|r| r.0),
        );
    }
}

/// Brute movement system using weighted velocities.
#[allow(clippy::type_complexity)]
pub fn brute_movement(
    time: Res<Time>,
    mut brutes: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&InMelee>,
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
            ),
        ),
        With<Brute>,
    >,
) {
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
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut brutes
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
    }
}

/// Brute rock throw — picks a target enemy within range and throws a rock at them.
#[allow(clippy::type_complexity)]
pub fn brute_rock_throw(
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut rock_events: MessageWriter<BoulderThrownMessage>,
    mut brutes: Query<
        (
            &Transform,
            &Team,
            &mut RockThrowCooldown,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
                Option<&PolymorphedModifier>,
            ),
        ),
        (With<Brute>, Without<Corpse>),
    >,
    targets: Query<
        (&Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        brute_transform,
        brute_team,
        mut cooldown,
        (
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
            polymorphed,
        ),
    ) in &mut brutes
    {
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) || polymorphed.is_some()
        {
            continue;
        }

        cooldown.tick(delta);
        if !cooldown.is_ready() {
            continue;
        }

        if let Some(target_pos) = crate::game::units::systems::find_closest_enemy_in_range(
            brute_transform.translation,
            brute_team,
            ROCK_THROW_RANGE,
            &targets,
        ) {
            rock_events.write(BoulderThrownMessage {
                origin: brute_transform.translation,
                target: target_pos,
                sprite_index: game_rng.0.gen_range(0..BOULDER_SPRITE_COUNT as u8),
            });
            cooldown.reset(ROCK_THROW_COOLDOWN);
        }
    }
}
