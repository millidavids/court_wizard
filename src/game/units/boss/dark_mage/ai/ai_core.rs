use bevy::prelude::*;

use super::super::components::*;
use super::super::resources::DarkMageAssets;
use super::super::spells::{
    find_spell_target, spawn_lightning_strike, spawn_meteor_explosion, spawn_plague_cloud,
    spawn_telegraph_indicators, spell_cooldown, telegraph_duration,
};
use crate::config::GameConfig;
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::utils::{animate_telegraph_material, despawn_indicators};
use crate::game::units::components::{
    BanishedModifier, Corpse, FrozenSolidModifier, Petrified, RootedModifier, SickenedModifier,
    SleepModifier, Sleepwalking, Stunned, Team,
};
use crate::game::units::wizard::spells::audio::{SpellSfxAssets, play_sfx_scaled};
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets,
};

/// Main Dark Mage AI: processes the spell queue, manages telegraph → cast transitions.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn dark_mage_ai(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    assets: Res<DarkMageAssets>,
    spell_assets: Res<SpellVisualAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut bosses: Query<
        (
            &Transform,
            &mut DarkMageState,
            &mut DarkMageSpellCooldowns,
            &mut DarkMageSpellQueue,
            &DarkMageEnrage,
            &Team,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&Stunned>,
                Option<&Petrified>,
            ),
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
    potential_targets: Query<
        (Entity, &Transform, &Team),
        (
            Without<DarkMage>,
            Without<Corpse>,
            Without<Boss>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        boss_transform,
        mut state,
        mut cooldowns,
        mut queue,
        enrage,
        boss_team,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned, petrified),
    ) in &mut bosses
    {
        // CC'd bosses can't cast
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
            // If telegraphing, cancel and go back to idle
            if let DarkMageState::Telegraphing { indicators, .. } = state.as_ref() {
                despawn_indicators(&mut commands, indicators.all());
            }
            if !matches!(state.as_ref(), &DarkMageState::Idle) {
                *state = DarkMageState::Idle;
            }
            continue;
        }

        match state.as_mut() {
            DarkMageState::Approaching => {
                // Don't cast spells while approaching
                continue;
            }
            DarkMageState::Idle => {
                // Pop next spell from queue
                if let Some(spell_type) = queue.queue.pop_front() {
                    // Find target position
                    let boss_pos = boss_transform.translation;
                    if let Some((target_pos, direction)) =
                        find_spell_target(spell_type, boss_pos, boss_team, &potential_targets)
                    {
                        let duration = telegraph_duration(spell_type);

                        // Spawn telegraph indicators
                        let indicators = spawn_telegraph_indicators(
                            &mut commands,
                            &assets,
                            &mut materials,
                            spell_type,
                            target_pos,
                            direction,
                        );

                        *state = DarkMageState::Telegraphing {
                            spell_type,
                            elapsed: 0.0,
                            duration,
                            target_pos,
                            direction,
                            indicators,
                        };

                        // Reset cooldown now so it starts ticking during telegraph
                        let base_cd = spell_cooldown(spell_type);
                        cooldowns.reset(spell_type, base_cd * enrage.cooldown_mult);
                    } else {
                        // No valid target -- push spell back and wait
                        queue.queue.push_front(spell_type);
                    }
                }
            }

            DarkMageState::Telegraphing {
                spell_type,
                elapsed,
                duration,
                target_pos,
                direction,
                indicators,
            } => {
                *elapsed += delta;
                let progress = (*elapsed / *duration).min(1.0);

                // Animate indicator emissive glow
                if let Some(mat) = materials.get_mut(&indicators.fill_material) {
                    animate_telegraph_material(mat, *elapsed, progress, 0.8);
                }

                if *elapsed >= *duration {
                    let sp = *spell_type;
                    let tp = *target_pos;
                    let dir = *direction;
                    despawn_indicators(&mut commands, indicators.all());
                    *state = DarkMageState::Casting {
                        spell_type: sp,
                        target_pos: tp,
                        direction: dir,
                    };
                }
            }

            DarkMageState::Casting {
                spell_type,
                target_pos,
                direction,
            } => {
                // Fire the spell with sound effects
                let tp = *target_pos;
                match spell_type {
                    DarkMageSpellType::DarkMeteor => {
                        spawn_meteor_explosion(
                            &mut commands,
                            &spell_assets,
                            &mut sphere_materials,
                            tp,
                        );
                        play_sfx_scaled(&mut commands, &sfx.fireball_impact, tp, &game_config, 1.0);
                    }
                    DarkMageSpellType::ShadowLightning => {
                        if let Some(dir) = direction {
                            spawn_lightning_strike(&mut commands, &assets, &spell_assets, tp, *dir);
                        }
                        play_sfx_scaled(
                            &mut commands,
                            &sfx.chain_lightning_cast,
                            tp,
                            &game_config,
                            1.0,
                        );
                    }
                    DarkMageSpellType::PlagueCloud => {
                        spawn_plague_cloud(
                            &mut game_rng.0,
                            &mut commands,
                            &assets,
                            &spell_assets,
                            tp,
                        );
                        play_sfx_scaled(
                            &mut commands,
                            &sfx.plague_wind_cast,
                            tp,
                            &game_config,
                            1.0,
                        );
                    }
                }
                *state = DarkMageState::Idle;
            }
        }
    }
}
