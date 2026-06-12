use bevy::prelude::*;

use crate::game::battlefield::constants::*;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Periodically spawns floating motes across the battlefield for atmosphere.
pub fn emit_ambient_motes(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < AMBIENT_MOTE_INTERVAL {
        return;
    }
    *timer -= AMBIENT_MOTE_INTERVAL;

    let t = time.elapsed_secs();

    // Spawn motes at pseudo-random positions across the playable area
    for i in 0..AMBIENT_MOTE_COUNT {
        let seed = t * 3.7 + i as f32 * 7.13;
        let x = AMBIENT_MOTE_MIN_X
            + ((seed * 17.3).sin() * 0.5 + 0.5) * (AMBIENT_MOTE_MAX_X - AMBIENT_MOTE_MIN_X);
        let z = AMBIENT_MOTE_MIN_Z
            + ((seed * 23.1).cos() * 0.5 + 0.5) * (AMBIENT_MOTE_MAX_Z - AMBIENT_MOTE_MIN_Z);
        let y = 10.0 + ((seed * 41.7).sin() * 0.5 + 0.5) * 25.0;

        vfx::systems::spawn_floating_motes(
            &mut commands,
            &visual_assets,
            &visual_assets.ambient_mote,
            Vec3::new(x, y, z),
            50.0,
            1,
            t + seed,
        );
    }
}
