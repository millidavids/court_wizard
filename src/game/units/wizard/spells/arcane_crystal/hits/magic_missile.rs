//! Hit detection for magic missile spells absorbed by crystals.

use super::super::auto::spawn_crystal_mini_missile;
use super::super::setup::{
    crystal_target_teams, find_random_enemies_in_range, increment_resonance, scaled_count,
    spell_echo_multiplier,
};
use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::magic_missile::components::MagicMissile;
use crate::game::units::wizard::spells::magic_missile_constants;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;

/// Detects magic missiles hitting crystals and emits mini homing missiles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_magic_missile_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    // Each peer drives its own real crystal only — the ghost copy of the
    // remote peer's crystal is excluded so the same absorption never fires
    // twice across the network.
    mut crystals: Query<
        (&mut ArcaneCrystal, Option<&mut ResonanceCascade>),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    missiles: Query<(Entity, &Transform, &MagicMissile), Without<CrystalSpawn>>,
    enemies: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    let target_teams = crystal_target_teams(session.as_deref());
    for (missile_entity, missile_transform, _missile) in &missiles {
        for (mut crystal, mut resonance) in &mut crystals {
            if crystal.permanent {
                continue;
            }
            let distance = missile_transform.translation.distance(crystal.position);

            if distance <= crystal.collision_radius {
                // Absorb the missile
                commands.entity(missile_entity).try_despawn();
                crystal.mark_absorption();
                crystal.remembered_spell = Some(RememberedSpell::MagicMissile);
                crystal.auto_cast_timer = 0.0;

                let rng = &mut game_rng.0;
                let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
                let count = scaled_count(MINI_MISSILE_COUNT, crystal.count_mult) * echo_mult;

                // Emit mini missiles at random enemy targets — peer-aware.
                let targets = find_random_enemies_in_range(
                    rng,
                    crystal.position,
                    crystal.range,
                    count,
                    &enemies,
                    target_teams,
                );

                let mini_radius = magic_missile_constants::COLLISION_RADIUS * SIZE_SCALE;

                for (target_entity, target_pos) in &targets {
                    let direction = (*target_pos - crystal.position).normalize();
                    let speed = magic_missile_constants::BASE_SPEED * SPEED_SCALE;
                    let initial_velocity = direction * speed;

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
                }

                // Track progress
                progress.increment(Spell::ArcaneCrystal, count as u32);

                // Resonance cascade
                increment_resonance(&mut resonance);

                break;
            }
        }
    }
}
