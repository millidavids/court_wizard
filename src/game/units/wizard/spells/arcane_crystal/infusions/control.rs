//! Crowd-control infusions.
//!
//! Like the auras, these are indiscriminate — a Sleep crystal will put your own
//! infantry down as readily as the wave. Placement is the cost.

use bevy::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

use super::super::constants::*;
use super::super::setup::register_infusion_spawn;
use super::driver::{InfusedCrystals, begin_infusion_tick};
use super::kinds::CrystalInfusion;
use super::zones::scatter_points;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::components::{Corpse, Health, RootedModifier, Team};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::entangle::casting::apply_entangle_to_unit;
use crate::game::units::wizard::spells::entangle::components::EntangleTalentParams;
use crate::game::units::wizard::spells::entangle::constants as entangle_constants;
use crate::game::units::wizard::spells::fog_cloud::components::FogCloudTalentParams;
use crate::game::units::wizard::spells::fog_cloud::constants as fog_constants;
use crate::game::units::wizard::spells::fog_cloud::systems::spawn_fog_cloud_zone;
use crate::game::units::wizard::spells::plague_wind::cloud::spawn_plague_cloud;
use crate::game::units::wizard::spells::plague_wind::components::PlagueWindTalentParams;
use crate::game::units::wizard::spells::plague_wind::constants as plague_constants;
use crate::game::units::wizard::spells::sleep::components::SleepTalentParams;
use crate::game::units::wizard::spells::sleep::systems::apply_sleep;
use crate::game::units::wizard::spells::utils::xz_distance;

/// Snares units caught inside the crystal's range.
#[allow(clippy::type_complexity)]
pub(crate) fn tick_entangle_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut crystals: InfusedCrystals,
    targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<RootedModifier>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut defender_hit_msg: MessageWriter<EntangleHitDefenderMessage>,
) {
    let delta = time.delta_secs();
    let talent_params = EntangleTalentParams::default();

    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::Entangle,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        let duration = entangle_constants::ROOT_DURATION * INFUSION_DURATION_SCALE;

        let mut in_range: Vec<(Entity, Team)> = targets
            .iter()
            .filter(|(_, transform, _)| {
                xz_distance(params.position, transform.translation) <= params.range
            })
            .map(|(target, _, team)| (target, *team))
            .collect();

        // The burst catches everything; ongoing snares pick a few at random so a
        // parked crystal cannot permanently lock the field down.
        if !params.is_burst() {
            in_range.shuffle(&mut game_rng.0);
            in_range.truncate(params.pick_count(INFUSION_BURST_COUNT, INFUSION_ONGOING_COUNT));
        }

        for (target, team) in in_range {
            apply_entangle_to_unit(
                &mut commands,
                target,
                &team,
                duration,
                &talent_params,
                &mut defender_hit_msg,
            );
        }
    }
}

/// Pulses drowsiness across the crystal's range.
///
/// Pairs with any damage source: sleeping units take double damage from the hit
/// that wakes them, so a Sleep crystal next to a Fireball crystal is a combo.
#[allow(clippy::type_complexity)]
pub(crate) fn tick_sleep_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut crystals: InfusedCrystals,
    targets: Query<
        (Entity, &Transform, &Health, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();
    let talent_params = SleepTalentParams::default();

    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::Sleep,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        // Ongoing pulses cover a fraction of the range; the burst covers all of it.
        let radius = params.range * params.pick(1.0, INFUSION_ZONE_RADIUS_SCALE);
        apply_sleep(
            &mut commands,
            params.position,
            radius,
            params.empowerment * INFUSION_DURATION_SCALE,
            &targets,
            &talent_params,
        );
    }
}

/// Breathes toxic clouds outward from the crystal.
pub(crate) fn tick_plague_wind_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut crystals: InfusedCrystals,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::PlagueWind,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        let count = params.pick_count(INFUSION_BURST_COUNT, 1);

        for index in 0..count {
            // Fan the clouds evenly outward rather than randomly, so the burst
            // reads as a bloom from the crystal instead of a scatter.
            let angle = std::f32::consts::TAU * (index as f32 / count.max(1) as f32)
                + game_rng.0.random_range(0.0..0.4);
            let direction = Vec3::new(angle.cos(), 0.0, angle.sin());
            // Not registered for crystal teardown: the cloud owns a pathfinding
            // obstacle that only its own drift/expiry system knows how to
            // remove. See the note in `zones::tick_grease_infusion`.
            spawn_plague_cloud(
                &mut commands,
                &mut obstacle_events,
                Vec3::new(params.position.x, 0.0, params.position.z),
                plague_constants::CLOUD_RADIUS * INFUSION_ZONE_RADIUS_SCALE * 2.0,
                plague_constants::DAMAGE_PER_TICK * DAMAGE_SCALE * params.damage_mult,
                plague_constants::CLOUD_DURATION * INFUSION_DURATION_SCALE,
                plague_constants::CLOUD_SPEED * SPEED_SCALE,
                direction,
                PlagueWindTalentParams::default(),
            );
        }
    }
}

/// Veils the crystal's surroundings in evasion-granting fog.
pub(crate) fn tick_fog_cloud_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut crystals: InfusedCrystals,
) {
    let delta = time.delta_secs();
    for (entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::FogCloud,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };

        if params.is_burst() {
            // Flood the whole range once.
            let zone = spawn_fog_cloud_zone(
                &mut commands,
                Vec3::new(params.position.x, 0.0, params.position.z),
                params.range,
                params.empowerment * INFUSION_DURATION_SCALE,
                &FogCloudTalentParams::default(),
                1.0,
            );
            register_infusion_spawn(&mut commands, &mut crystal, entity, zone);
            continue;
        }

        for point in scatter_points(&mut game_rng.0, params.position, params.range, 1) {
            let zone = spawn_fog_cloud_zone(
                &mut commands,
                point,
                fog_constants::CIRCLE_RADIUS * SIZE_SCALE,
                params.empowerment * INFUSION_DURATION_SCALE,
                &FogCloudTalentParams::default(),
                1.0,
            );
            register_infusion_spawn(&mut commands, &mut crystal, entity, zone);
        }
    }
}
