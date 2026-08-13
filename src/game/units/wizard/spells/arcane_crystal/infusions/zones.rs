//! Infusions that scatter scaled-down hazard zones inside the crystal's range.
//!
//! The burst lays down a spread of patches; each ongoing tick adds one more, so a
//! zone crystal slowly carpets its own radius. Every patch is registered with the
//! crystal so re-infusing or destroying it clears the field.

use bevy::prelude::*;
use rand::Rng;

use super::super::constants::*;
use super::driver::{InfusedCrystals, begin_infusion_tick};
use super::kinds::CrystalInfusion;
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::wizard::spells::grease::components::GreaseTalentParams;
use crate::game::units::wizard::spells::grease::constants as grease_constants;
use crate::game::units::wizard::spells::grease::ignite::spawn_grease_zone;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthTalentParams;
use crate::game::units::wizard::spells::spike_growth::constants as spike_constants;
use crate::game::units::wizard::spells::spike_growth::systems::spawn_spike_growth_zone;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Picks `count` random ground points inside `range` of `origin`, clamped to the
/// battlefield.
///
/// Deliberately not `teleport::random_position_in_circle`: that samples the
/// radius uniformly, which clusters points near the centre. Scattered hazards
/// want an even spread across the disc, hence the sqrt. The battlefield clamp is
/// shared with it, and matters — a crystal placed near the edge would otherwise
/// drop slicks and thorn patches off the map where nothing can walk into them.
pub(super) fn scatter_points(
    rng: &mut impl Rng,
    origin: Vec3,
    range: f32,
    count: usize,
) -> Vec<Vec3> {
    let half_field = BATTLEFIELD_SIZE / 2.0;
    (0..count)
        .map(|_| {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = range * rng.random::<f32>().sqrt();
            Vec3::new(
                (origin.x + angle.cos() * distance).clamp(-half_field, half_field),
                0.0,
                (origin.z + angle.sin() * distance).clamp(-half_field, half_field),
            )
        })
        .collect()
}

/// Lays oil slicks around the crystal.
///
/// These are ordinary grease zones, so the existing ignition path applies
/// unchanged: any fire that reaches one lights it exactly as it would a
/// hand-cast slick — including the crystal's own mini-fireballs if it is later
/// re-infused with Fireball.
pub(crate) fn tick_grease_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut crystals: InfusedCrystals,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::Grease,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        let count = params.pick_count(INFUSION_BURST_COUNT, 1);
        // Scaled off the *spell's* own radius, not the crystal's range. Scaling
        // the range gave 175-unit patches against a real cast's 150 — the
        // "scaled-down copy" was bigger than the thing it copied, and worse with
        // Wider Prism.
        let radius = grease_constants::CIRCLE_RADIUS * SIZE_SCALE;

        for point in scatter_points(&mut game_rng.0, params.position, params.range, count) {
            // Deliberately NOT registered for crystal teardown. These zones
            // register a pathfinding obstacle, and the only code that emits the
            // matching `ObstacleType::Removed` is the zone's own expiry system.
            // Despawning one early would leave its slow-terrain region in the
            // flow field for the rest of the level. Their durations are already
            // halved, so letting them burn out is both correct and cheap.
            spawn_grease_zone(
                &mut commands,
                &visual_assets,
                &mut materials,
                point,
                radius,
                params.empowerment * INFUSION_DURATION_SCALE,
                &mut obstacle_events,
                GreaseTalentParams::default(),
                1.0,
            );
        }
    }
}

/// Grows thorn patches around the crystal.
pub(crate) fn tick_spike_growth_infusion(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: InfusedCrystals,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();
    for (_entity, mut crystal, hastened, enraged) in &mut crystals {
        let Some(params) = begin_infusion_tick(
            &mut crystal,
            CrystalInfusion::SpikeGrowth,
            hastened,
            enraged,
            delta,
        ) else {
            continue;
        };
        let count = params.pick_count(INFUSION_BURST_COUNT, 1);
        // See the note in the grease tick: scale the spell's radius, not the range.
        let radius = spike_constants::CIRCLE_RADIUS * SIZE_SCALE;

        for point in scatter_points(&mut game_rng.0, params.position, params.range, count) {
            // Not registered for teardown — see the note in the grease tick.
            spawn_spike_growth_zone(
                &mut commands,
                &visual_assets,
                point,
                radius,
                params.empowerment * INFUSION_DURATION_SCALE,
                &mut obstacle_events,
                &SpikeGrowthTalentParams::default(),
                1.0,
            );
        }
    }
}
