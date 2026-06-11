use bevy::prelude::*;

use super::{
    HEALER_CAST_DURATION, HEALER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    HEALER_CHANNEL_PARTICLE_LIFETIME, HEALER_CHANNEL_PARTICLE_MAX_RADIUS,
    HEALER_CHANNEL_PARTICLE_SIZE, HEALER_CHANNEL_PARTICLE_SPAWN_INTERVAL,
    HEALER_CHANNEL_PARTICLE_START_RADIUS,
};
use crate::game::seeded_rng::resources::GameRng;
use crate::game::units::components::{CombatAnimation, Corpse, Hitbox};
use crate::game::units::healer::components::Healer;
use crate::game::units::healer::resources::HealerAssets;
use crate::game::units::wizard::spells::vfx::channel::{
    ChannelParticleSpec, ChannelingCast, spawn_channel_particle_batch,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Re-inserts the casting animation on channeling healers whose previous cycle
/// already ran to completion, producing a continuous loop.
pub fn healer_refresh_casting_animation(
    mut commands: Commands,
    healers: Query<
        Entity,
        (
            With<Healer>,
            With<ChannelingCast>,
            Without<CombatAnimation>,
            Without<Corpse>,
        ),
    >,
    healer_assets: Res<HealerAssets>,
) {
    for entity in &healers {
        commands.entity(entity).insert(CombatAnimation::new_casting(
            healer_assets.casting_texture.clone(),
            healer_assets.sprite_texture.clone(),
        ));
    }
}

/// Spawns inward-imploding green particles around each channeling healer.
pub fn healer_spawn_channel_particles(
    mut commands: Commands,
    healers: Query<(&Transform, &ChannelingCast, &Hitbox), (With<Healer>, Without<Corpse>)>,
    healer_assets: Res<HealerAssets>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut game_rng: ResMut<GameRng>,
) {
    *timer += time.delta_secs();
    if *timer < HEALER_CHANNEL_PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= HEALER_CHANNEL_PARTICLE_SPAWN_INTERVAL;

    let spec = ChannelParticleSpec {
        start_radius: HEALER_CHANNEL_PARTICLE_START_RADIUS,
        max_radius: HEALER_CHANNEL_PARTICLE_MAX_RADIUS,
        size: HEALER_CHANNEL_PARTICLE_SIZE,
        lifetime: HEALER_CHANNEL_PARTICLE_LIFETIME,
        count_per_spawn: HEALER_CHANNEL_PARTICLE_COUNT_PER_SPAWN,
    };

    for (transform, channel, hitbox) in &healers {
        let progress = channel.elapsed / HEALER_CAST_DURATION;
        let center = transform.translation + Vec3::Y * (hitbox.height * 0.5);
        spawn_channel_particle_batch(
            &mut commands,
            center,
            progress,
            &visual_assets.particle_quad,
            &healer_assets.channel_particle_material,
            &spec,
            &mut game_rng.0,
        );
    }
}
