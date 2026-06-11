use bevy::prelude::*;

use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::components::{CombatAnimation, Corpse, Hitbox};
use crate::game::units::shielder::components::Shielder;
use crate::game::units::shielder::constants::{
    SHIELDER_CAST_DURATION, SHIELDER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    SHIELDER_CHANNEL_PARTICLE_LIFETIME, SHIELDER_CHANNEL_PARTICLE_MAX_RADIUS,
    SHIELDER_CHANNEL_PARTICLE_SIZE, SHIELDER_CHANNEL_PARTICLE_SPAWN_INTERVAL,
    SHIELDER_CHANNEL_PARTICLE_START_RADIUS,
};
use crate::game::units::wizard::spells::vfx::channel::{
    ChannelParticleSpec, ChannelingCast, spawn_channel_particle_batch,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Re-inserts the casting animation on channeling shielders so the animation loops.
pub fn shielder_refresh_casting_animation(
    mut commands: Commands,
    shielders: Query<
        Entity,
        (
            With<Shielder>,
            With<ChannelingCast>,
            Without<CombatAnimation>,
            Without<Corpse>,
        ),
    >,
    shielder_assets: Res<super::super::resources::ShielderAssets>,
) {
    for entity in &shielders {
        commands.entity(entity).insert(CombatAnimation::new_casting(
            shielder_assets.casting_texture.clone(),
            shielder_assets.sprite_texture.clone(),
        ));
    }
}

/// Spawns inward-imploding yellow particles around each channeling shielder.
pub fn shielder_spawn_channel_particles(
    mut commands: Commands,
    shielders: Query<(&Transform, &ChannelingCast, &Hitbox), (With<Shielder>, Without<Corpse>)>,
    shielder_assets: Res<super::super::resources::ShielderAssets>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut game_rng: ResMut<GameRng>,
) {
    *timer += time.delta_secs();
    if *timer < SHIELDER_CHANNEL_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= SHIELDER_CHANNEL_PARTICLE_SPAWN_INTERVAL;

    let spec = ChannelParticleSpec {
        start_radius: SHIELDER_CHANNEL_PARTICLE_START_RADIUS,
        max_radius: SHIELDER_CHANNEL_PARTICLE_MAX_RADIUS,
        size: SHIELDER_CHANNEL_PARTICLE_SIZE,
        lifetime: SHIELDER_CHANNEL_PARTICLE_LIFETIME,
        count_per_spawn: SHIELDER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    };

    for (transform, channel, hitbox) in &shielders {
        let progress = channel.elapsed / SHIELDER_CAST_DURATION;
        let center = transform.translation + Vec3::Y * (hitbox.height * 0.5);
        spawn_channel_particle_batch(
            &mut commands,
            center,
            progress,
            &visual_assets.particle_quad,
            &shielder_assets.channel_particle_material,
            &spec,
            &mut game_rng.0,
        );
    }
}
