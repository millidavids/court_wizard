use bevy::prelude::*;

use super::components::{BurningBush, Bush};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::{
    MeteorExplosion, MeteorGroundFire,
};
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;

pub struct BushPlugin;

impl Plugin for BushPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_bush_assets)
            .add_systems(
                Update,
                (
                    // Bush slow (walking through a bush is slower) and the
                    // burning-bush DoT both write to authoritative unit
                    // state, so they stay host-only — CRDTs propagate the
                    // results to the guest.
                    apply_bush_slow
                        .run_if(any_with_component::<Bush>)
                        .run_if(is_gameplay_running),
                    apply_burning_bush_damage
                        .run_if(any_with_component::<BurningBush>)
                        .run_if(is_gameplay_running),
                    // Ignition + VFX run on both peers so each side sees
                    // bushes catch fire. Each peer adds `BurningBush` to
                    // its own local bushes from the spell visuals it can
                    // already see (own + mirrored ghost from the other
                    // peer's casts via `spell_sync.rs`).
                    ignite_bushes_from_fire
                        .run_if(
                            any_with_component::<Bush>.and(
                                any_with_component::<FireballExplosion>
                                    .or(any_with_component::<MeteorExplosion>)
                                    .or(any_with_component::<DisintegrateBeam>)
                                    .or(any_with_component::<WallOfFireEffect>)
                                    .or(any_with_component::<MeteorGroundFire>),
                            ),
                        )
                        .run_if(is_spell_effects_active),
                    emit_burning_bush_vfx
                        .run_if(any_with_component::<BurningBush>)
                        .run_if(is_spell_effects_active),
                ),
            );
    }
}
