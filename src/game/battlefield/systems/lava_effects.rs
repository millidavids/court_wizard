use bevy::prelude::*;

use crate::game::battlefield::components::LavaPool;
use crate::game::battlefield::constants::*;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Emits fire smoke puffs from the lava pool on the wall floor.
pub fn emit_lava_fire_smoke(
    mut commands: Commands,
    lava_pools: Query<&Transform, With<LavaPool>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < LAVA_SMOKE_INTERVAL {
        return;
    }
    *timer -= LAVA_SMOKE_INTERVAL;

    let t = time.elapsed_secs();

    for transform in lava_pools.iter() {
        vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            transform.translation,
            LAVA_POOL_RADIUS,
            4,
            t,
        );
        vfx::systems::spawn_heat_shimmer(
            &mut commands,
            &visual_assets,
            transform.translation,
            2,
            t,
        );
    }
}

/// Emits occasional spark bursts from the lava pool.
pub fn emit_lava_sparks(
    mut commands: Commands,
    lava_pools: Query<&Transform, With<LavaPool>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < LAVA_SPARK_INTERVAL {
        return;
    }
    *timer -= LAVA_SPARK_INTERVAL;

    let t = time.elapsed_secs();

    for transform in lava_pools.iter() {
        vfx::systems::spawn_fire_sparks(&mut commands, &visual_assets, transform.translation, 6, t);
    }
}
