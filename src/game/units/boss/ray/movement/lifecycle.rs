use bevy::prelude::*;

use super::super::beams::{despawn_fear_beam, despawn_mind_control_beam};
use super::super::components::*;
use super::super::constants::*;
use super::spawn::ray_sfx_volume;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::spells::audio::SpellSfxAssets;

/// Ray's body is a sentinel — defeat is determined by all eyes dying, not body HP.
/// Any damage dealt to the body is redistributed evenly (1/5) across all eyes, dead
/// or alive, then the body is topped back up.
pub fn ray_body_damage_to_eyes(
    mut body_query: Query<&mut Health, (With<Ray>, Without<RayEye>, Without<Corpse>)>,
    mut eyes: Query<&mut Health, (With<RayEye>, Without<Ray>, Without<RayEyeDying>)>,
) {
    let Ok(mut body_health) = body_query.single_mut() else {
        return;
    };

    let damage_taken = body_health.max - body_health.current;
    if damage_taken <= 0.0 {
        return;
    }

    let per_eye = damage_taken / RayEyeType::COUNT as f32;
    for mut eye_health in &mut eyes {
        eye_health.current = (eye_health.current - per_eye).max(0.0);
    }

    body_health.current = body_health.max;
}

/// Detects eyes that have reached 0 HP, marks them inactive, and starts their implode animation.
pub fn ray_eye_death_check(
    mut commands: Commands,
    eyes: Query<(Entity, &Health, &RayEye, &Transform), (Without<RayEyeDying>, Without<Ray>)>,
    mut body_query: Query<&mut RayEyeState, With<Ray>>,
    sfx_assets: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let Ok(mut eye_state) = body_query.single_mut() else {
        return;
    };
    for (entity, health, eye, transform) in &eyes {
        if health.current <= 0.0 {
            let idx = eye.eye_type.index();
            if eye_state.active[idx] {
                eye_state.active[idx] = false;
            }
            commands.entity(entity).insert(RayEyeDying {
                time_alive: 0.0,
                duration: RAY_EYE_IMPLODE_DURATION,
                initial_scale: transform.scale.x.max(1.0),
            });

            let volume = ray_sfx_volume(transform.translation, &game_config);
            if volume > 0.0 {
                commands.spawn((
                    bevy::audio::AudioPlayer::new(sfx_assets.ray_eye_death.clone()),
                    bevy::audio::PlaybackSettings::ONCE
                        .with_volume(bevy::audio::Volume::Linear(volume)),
                    OnGameplayScreen,
                ));
            }
        }
    }
}

/// Animates dying eyes imploding then despawns them.
pub fn update_ray_dying_eyes(
    time: Res<Time>,
    mut commands: Commands,
    mut eyes: Query<(Entity, &mut RayEyeDying, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut dying, mut transform) in &mut eyes {
        dying.time_alive += delta;
        if dying.time_alive >= dying.duration {
            commands.entity(entity).try_despawn();
            continue;
        }
        let t = (dying.time_alive / dying.duration).clamp(0.0, 1.0);
        let scale = dying.initial_scale * (1.0 - t).max(0.0).powi(2);
        transform.scale = Vec3::splat(scale);
    }
}

/// Once all eyes have died (active = false for all), kill Ray's body — triggering victory.
pub fn ray_all_eyes_dead_check(
    mut commands: Commands,
    body_query: Query<(Entity, &RayEyeState), (With<Ray>, Without<Corpse>)>,
) {
    for (entity, eye_state) in &body_query {
        if eye_state.active.iter().all(|&a| !a) {
            commands.entity(entity).insert(Corpse);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ray_death_cleanup(
    mut commands: Commands,
    dead_ray: Query<Entity, (With<Ray>, With<Corpse>)>,
    eyes: Query<Entity, With<RayEye>>,
    particles: Query<Entity, With<RayStalkParticle>>,
    mut sweep_disintegrate: Query<&mut RayDisintegrationSweep>,
    mut sweep_petrify: Query<&mut RayPetrificationSweep>,
    mut sweep_fear: Query<&mut RayFearSweep>,
    mut sweep_mind_control: Query<&mut RayMindControlSweep>,
) {
    if dead_ray.iter().next().is_none() {
        return;
    }

    for entity in &eyes {
        commands.entity(entity).try_despawn();
    }
    for entity in &particles {
        commands.entity(entity).try_despawn();
    }
    for mut sweep in &mut sweep_disintegrate {
        super::disintegration::despawn_ray_beam(&mut commands, &mut sweep);
    }
    for mut sweep in &mut sweep_petrify {
        super::petrification::despawn_petrify_beam(&mut commands, &mut sweep);
    }
    for mut sweep in &mut sweep_fear {
        despawn_fear_beam(&mut commands, &mut sweep);
    }
    for mut sweep in &mut sweep_mind_control {
        despawn_mind_control_beam(&mut commands, &mut sweep);
    }
}
