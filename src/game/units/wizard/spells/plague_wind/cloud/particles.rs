use super::super::components::PlagueWindCloud;
use super::super::constants;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Continuously spawns plague smoke particles from active clouds.
pub fn emit_plague_cloud_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut clouds: Query<&mut PlagueWindCloud>,
    assets: Res<SpellVisualAssets>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for mut cloud in &mut clouds {
        // Don't emit particles during fade-out
        let remaining = cloud.duration - cloud.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        cloud.smoke_spawn_timer += dt;
        if cloud.smoke_spawn_timer >= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL {
            cloud.smoke_spawn_timer -= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL;

            vfx::systems::spawn_plague_smoke_puffs(
                &mut commands,
                &assets,
                cloud.origin,
                cloud.radius,
                vfx::constants::PLAGUE_SMOKE_COUNT_PER_SPAWN,
                t,
            );
        }
    }
}
