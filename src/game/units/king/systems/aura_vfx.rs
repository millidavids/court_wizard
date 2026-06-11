use bevy::prelude::*;
use rand::Rng;

use crate::game::components::OnGameplayScreen;
use crate::game::units::commander::{Commander, CommanderAuraParticle};
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns small particles from all commanders that travel outward to the aura edge.
pub fn spawn_commander_aura_particles(
    mut commands: Commands,
    commanders: Query<(&Transform, &Commander)>,
    spell_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < 0.12 {
        return;
    }
    *timer -= 0.12;

    for (transform, commander) in &commanders {
        let pos = transform.translation;
        let radius = commander.aura_radius;

        for _ in 0..2 {
            let dir = Vec3::new(
                game_rng.0.random_range(-1.0..1.0_f32),
                game_rng.0.random_range(0.0..0.5_f32),
                game_rng.0.random_range(-1.0..1.0_f32),
            )
            .normalize_or(Vec3::Y);

            let speed = game_rng.0.random_range(80.0..150.0_f32);
            let lifetime = radius / speed;

            commands.spawn((
                CommanderAuraParticle {
                    velocity: dir * speed,
                    time_alive: 0.0,
                    lifetime,
                },
                Mesh3d(spell_assets.particle_quad.clone()),
                MeshMaterial3d(spell_assets.buff_mote.clone()),
                Transform::from_translation(pos)
                    .with_rotation(UPWARD_ROTATION)
                    .with_scale(Vec3::splat(3.0)),
                OnGameplayScreen,
            ));
        }
    }
}

/// Moves commander aura particles outward and fades them over their lifetime.
pub fn update_commander_aura_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut CommanderAuraParticle, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (entity, mut particle, mut transform) in &mut particles {
        particle.time_alive += delta;

        if particle.time_alive >= particle.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Move outward
        transform.translation += particle.velocity * delta;

        // Fade: grow slightly then shrink at the end
        let t = particle.time_alive / particle.lifetime;
        let scale = if t < 0.2 {
            3.0 * (t / 0.2)
        } else if t > 0.7 {
            3.0 * (1.0 - (t - 0.7) / 0.3)
        } else {
            3.0
        };
        transform.scale = Vec3::splat(scale);
    }
}
