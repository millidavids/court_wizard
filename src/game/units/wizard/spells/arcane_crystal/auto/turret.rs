use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::setup::{
    crystal_target_teams, find_random_enemies_in_range, increment_resonance,
};
use super::spawn_helpers::spawn_crystal_mini_missile;

use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::magic_missile_constants;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;

/// Fires homing magic missiles at nearby enemies on a timer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn auto_crystal_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (
            &ArcaneCrystal,
            &mut AutoCrystalTimer,
            Option<&mut ResonanceCascade>,
        ),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    let delta = time.delta_secs();
    let target_teams = crystal_target_teams(session.as_deref());

    for (crystal, mut timer, mut resonance) in &mut crystals {
        timer.timer += delta;

        // Overcharged Matrix speeds up the fire rate
        let interval = AUTO_CRYSTAL_INTERVAL / crystal.count_mult;
        if timer.timer < interval {
            continue;
        }
        timer.timer -= interval;

        // Find a random enemy in range
        let targets = find_random_enemies_in_range(
            &mut game_rng.0,
            crystal.position,
            crystal.range,
            1,
            &enemies,
            target_teams,
        );

        let Some((target_entity, target_pos)) = targets.first() else {
            continue;
        };

        let direction = (*target_pos - crystal.position).normalize();
        let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
        let initial_velocity = direction * speed;
        let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

        let rng = &mut game_rng.0;
        let wobble_offset = rng.random_range(0.0..std::f32::consts::TAU);

        spawn_crystal_mini_missile(
            &mut commands,
            &visual_assets,
            crystal.position,
            crystal.range,
            initial_velocity,
            wobble_offset,
            Some(*target_entity),
            mini_radius,
            crystal.damage_mult,
            target_teams,
        );

        // Increment resonance cascade counter on each turret shot
        increment_resonance(&mut resonance);

        progress.increment(Spell::ArcaneCrystal, 1);
    }
}
