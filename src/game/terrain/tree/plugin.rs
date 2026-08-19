use bevy::prelude::*;

use super::components::{BurningTree, Tree};
use super::resources;
use super::systems::*;
use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::{
    MeteorExplosion, MeteorGroundFire,
};
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_tree_assets)
            .add_systems(
                Update,
                (
                    // Ignition + VFX run on BOTH MP peers so each side
                    // sees the same trees catch fire (locally driven by
                    // the spell visuals — own + ghosts of the other
                    // peer's — which are mirrored via `spell_sync.rs`).
                    // Each peer maintains its own `BurningTree` marker
                    // on its own local trees; no network message is
                    // needed for the visual.
                    ignite_trees_from_fire
                        .run_if(
                            any_with_component::<Tree>.and_then(
                                any_with_component::<FireballExplosion>
                                    .or_else(any_with_component::<MeteorExplosion>)
                                    .or_else(any_with_component::<DisintegrateBeam>)
                                    .or_else(any_with_component::<WallOfFireEffect>)
                                    .or_else(any_with_component::<MeteorGroundFire>),
                            ),
                        )
                        .run_if(is_spell_effects_active),
                    emit_burning_tree_vfx
                        .run_if(any_with_component::<BurningTree>)
                        .run_if(is_spell_effects_active),
                    // Damage application stays host-authoritative —
                    // gameplay-side `FireDoT` insertion on units happens
                    // once on the host and propagates to the guest via
                    // the existing CRDT health pipeline.
                    apply_burning_tree_damage
                        .run_if(any_with_component::<BurningTree>)
                        .run_if(is_gameplay_running),
                ),
            );
    }
}
