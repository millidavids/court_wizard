use std::cmp::Ordering;

use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::DispellerAssets;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::components::{CombatAnimation, Corpse, Hitbox, Team};
use crate::game::units::wizard::spells::dispel::systems::{
    is_dispellable, spawn_dispel_projectile, spell_edge_distance,
};
use crate::game::units::wizard::spells::grease::components::{GreaseIgnited, GreaseZone};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::vfx::channel::{
    ChannelParticleSpec, ChannelingCast, spawn_channel_particle_batch,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;

/// Starts a 5-second dispel channel when cooldown is ready and a dispellable
/// spell effect is within range.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn dispeller_start_dispel_channel(
    mut commands: Commands,
    time: Res<Time>,
    dispeller_assets: Res<DispellerAssets>,
    mut dispellers: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&mut DispellerDispelCooldown>,
            Has<ChannelingCast>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Dispeller>, Without<Corpse>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
) {
    let delta = time.delta_secs();

    let dispellable_effects: Vec<(Entity, Vec3)> = spell_effects
        .iter()
        .filter(|(_, _, nse)| is_dispellable(nse.kind))
        .map(|(e, tf, _)| (e, tf.translation))
        .collect();

    if dispellable_effects.is_empty() {
        return;
    }

    for (entity, transform, team, cooldown, is_channeling, has_staging, has_wave_group) in
        &mut dispellers
    {
        if crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group) {
            continue;
        }
        if let Some(mut cd) = cooldown {
            cd.remaining -= delta;
            if cd.remaining > 0.0 {
                continue;
            }
            commands.entity(entity).remove::<DispellerDispelCooldown>();
        }

        if is_channeling {
            continue;
        }

        let has_in_range = dispellable_effects
            .iter()
            .any(|&(spell_entity, spell_pos)| {
                spell_edge_distance(
                    transform.translation,
                    spell_entity,
                    spell_pos,
                    &wall_of_fire_query,
                    &wall_of_stone_query,
                    &spike_growth_query,
                    &grease_query,
                    &meteor_fire_query,
                ) <= DISPEL_RANGE
            });
        if !has_in_range {
            continue;
        }

        commands.entity(entity).insert((
            ChannelingCast { elapsed: 0.0 },
            CombatAnimation::new_casting(
                dispeller_assets.casting_texture.clone(),
                dispeller_assets.sprite_texture.clone(),
            ),
        ));
    }
}

/// Ticks active dispel channels. On completion spawns a dispel projectile at
/// a freshly-picked in-range dispellable effect and starts the cooldown.
#[allow(clippy::too_many_arguments)]
pub fn dispeller_tick_dispel_channel(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dispellers: Query<
        (Entity, &Transform, &mut ChannelingCast),
        (With<Dispeller>, Without<Corpse>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
) {
    let delta = time.delta_secs();

    let mut dispellable_effects: Option<Vec<(Entity, Vec3)>> = None;

    for (entity, transform, mut channel) in &mut dispellers {
        channel.elapsed += delta;
        if channel.elapsed < DISPELLER_CAST_DURATION {
            continue;
        }

        commands
            .entity(entity)
            .remove::<ChannelingCast>()
            .insert(DispellerDispelCooldown {
                remaining: DISPEL_COOLDOWN,
            });

        let effects = dispellable_effects.get_or_insert_with(|| {
            spell_effects
                .iter()
                .filter(|(_, _, nse)| is_dispellable(nse.kind))
                .map(|(e, tf, _)| (e, tf.translation))
                .collect()
        });

        let nearest = effects
            .iter()
            .filter_map(|&(spell_entity, spell_pos)| {
                let dist = spell_edge_distance(
                    transform.translation,
                    spell_entity,
                    spell_pos,
                    &wall_of_fire_query,
                    &wall_of_stone_query,
                    &spike_growth_query,
                    &grease_query,
                    &meteor_fire_query,
                );
                if dist <= DISPEL_RANGE {
                    Some((spell_pos, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

        if let Some((target_pos, _)) = nearest {
            spawn_dispel_projectile(
                &mut commands,
                &mut meshes,
                &mut materials,
                transform.translation,
                target_pos,
                0.0,
            );
        }
    }
}

/// Re-inserts the casting animation on channeling dispellers so it loops.
pub fn dispeller_refresh_casting_animation(
    mut commands: Commands,
    dispellers: Query<
        Entity,
        (
            With<Dispeller>,
            With<ChannelingCast>,
            Without<CombatAnimation>,
            Without<Corpse>,
        ),
    >,
    dispeller_assets: Res<DispellerAssets>,
) {
    for entity in &dispellers {
        commands.entity(entity).insert(CombatAnimation::new_casting(
            dispeller_assets.casting_texture.clone(),
            dispeller_assets.sprite_texture.clone(),
        ));
    }
}

/// Spawns inward-imploding white particles around each channeling dispeller.
pub fn dispeller_spawn_channel_particles(
    mut commands: Commands,
    dispellers: Query<(&Transform, &ChannelingCast, &Hitbox), (With<Dispeller>, Without<Corpse>)>,
    dispeller_assets: Res<DispellerAssets>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut game_rng: ResMut<GameRng>,
) {
    *timer += time.delta_secs();
    if *timer < DISPELLER_CHANNEL_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= DISPELLER_CHANNEL_PARTICLE_SPAWN_INTERVAL;

    let spec = ChannelParticleSpec {
        start_radius: DISPELLER_CHANNEL_PARTICLE_START_RADIUS,
        max_radius: DISPELLER_CHANNEL_PARTICLE_MAX_RADIUS,
        size: DISPELLER_CHANNEL_PARTICLE_SIZE,
        lifetime: DISPELLER_CHANNEL_PARTICLE_LIFETIME,
        count_per_spawn: DISPELLER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    };

    for (transform, channel, hitbox) in &dispellers {
        let progress = channel.elapsed / DISPELLER_CAST_DURATION;
        let center = transform.translation + Vec3::Y * (hitbox.height * 0.5);
        spawn_channel_particle_batch(
            &mut commands,
            center,
            progress,
            &visual_assets.particle_quad,
            &dispeller_assets.channel_particle_material,
            &spec,
            &mut game_rng.0,
        );
    }
}
