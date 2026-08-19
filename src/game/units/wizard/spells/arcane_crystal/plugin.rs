//! Arcane Crystal spell plugin.

use bevy::prelude::*;

use super::components::{
    ArcaneCrystal, AutoCrystalTimer, CrystalNetwork, CrystalOwned, CrystalRangeIndicator,
    CrystalSpawn, CrystalTint, ResonanceCascade,
};
use super::infusions::{self, CrystalEnraged, CrystalInfusion, crystal_infused_with};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::MovementCalculationSet;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::run_conditions::{
    mouse_held_or_wizard_casting, mouse_left_not_consumed, spell_input_not_blocked, spell_is_primed,
};

/// Plugin for the Arcane Crystal spell.
pub struct ArcaneCrystalPlugin;

impl Plugin for ArcaneCrystalPlugin {
    fn build(&self, app: &mut App) {
        // Visual / lifetime systems — safe on both MP peers. The ghost
        // crystal exists on the guest and needs its visuals updated +
        // cleanup when the host's authoritative entity expires.
        app.add_systems(
            Update,
            (
                systems::clear_absorption_flags.run_if(any_with_component::<ArcaneCrystal>),
                systems::handle_arcane_crystal_casting
                    .run_if(spell_is_primed(Spell::ArcaneCrystal))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                systems::update_crystal_visuals.run_if(any_with_component::<ArcaneCrystal>),
                systems::update_crystal_tint.run_if(any_with_component::<CrystalTint>),
                systems::cleanup_expired_crystals.run_if(
                    any_with_component::<ArcaneCrystal>
                        .or_else(any_with_component::<CrystalRangeIndicator>),
                ),
                systems::despawn_out_of_range_crystal_spawns
                    .run_if(any_with_component::<CrystalSpawn>),
                systems::cleanup_expired_crystal_visuals.run_if(any_with_component::<CrystalSpawn>),
                systems::cleanup_expired_crystal_beams.run_if(any_with_component::<CrystalSpawn>),
                systems::cleanup_orphaned_infusion_spawns
                    .run_if(any_with_component::<CrystalOwned>),
            )
                .chain()
                .run_if(is_spell_effects_active),
        );

        // Gameplay / hit-detection / talent systems — run on BOTH peers so
        // each peer drives propagation for the crystal it cast. Every
        // crystal query is gated `Without<GhostSpellEffect>`, so each peer
        // only processes its own real crystal and never the snapshot ghost
        // of the other peer's. `BattleTalentProgress` is initialized on
        // both host and guest at `MultiplayerGameState::Running`.
        // Propagated mini-projectiles spawn locally and reach the remote
        // peer through their normal sync paths (magic missile position
        // snapshots, `SpellHitUnit` damage forwarding).
        app.add_systems(
            Update,
            (
                systems::crystal_black_hole_interaction.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_fireball_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_beam_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_meteor_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_magic_missile_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_chain_lightning_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::detect_area_cast_hits.run_if(any_with_component::<ArcaneCrystal>),
                systems::resonance_cascade_burst.run_if(any_with_component::<ResonanceCascade>),
                systems::crystal_network_chain.run_if(any_with_component::<CrystalNetwork>),
                systems::auto_cast_remembered_spell.run_if(any_with_component::<ArcaneCrystal>),
                systems::auto_crystal_fire.run_if(any_with_component::<AutoCrystalTimer>),
                infusions::tick_enraged_lifetime.run_if(any_with_component::<CrystalEnraged>),
            )
                .run_if(is_spell_effects_active),
        );

        // Per-infusion behaviour. Each gate is cheap — "does any crystal hold
        // this?" — and each system re-checks per crystal, because Crystal
        // Network allows several crystals with different infusions at once.
        app.add_systems(
            Update,
            (
                infusions::tick_lightning_rod_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::LightningRod)),
                infusions::tick_squall_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::Squall)),
                infusions::tick_grease_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::Grease)),
                infusions::tick_spike_growth_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::SpikeGrowth)),
                infusions::tick_raise_the_dead_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::RaiseTheDead)),
                infusions::tick_telekinesis_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::Telekinesis)),
                infusions::tick_battle_hymn_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::BattleHymn)),
                infusions::tick_berserker_rage_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::BerserkerRage)),
                infusions::tick_guardian_circle_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::GuardianCircle)),
                infusions::tick_healing_plume_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::HealingPlume)),
                infusions::tick_mark_of_death_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::MarkOfDeath)),
            )
                .run_if(is_spell_effects_active),
        );

        app.add_systems(
            Update,
            (
                infusions::tick_entangle_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::Entangle)),
                infusions::tick_sleep_infusion.run_if(crystal_infused_with(CrystalInfusion::Sleep)),
                infusions::tick_plague_wind_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::PlagueWind)),
                infusions::tick_fog_cloud_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::FogCloud)),
                infusions::tick_banishment_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::Banishment)),
                infusions::tick_teleport_infusion
                    .run_if(crystal_infused_with(CrystalInfusion::Teleport)),
                // Must share the real black hole's set: it writes `Acceleration`,
                // which `apply_unit_movement` integrates and resets inside
                // `MovementCalculationSet`. Outside it the ordering is ambiguous,
                // and on a guest — where that set is host-gated — the force would
                // pile up every frame with nothing ever clearing it.
                infusions::tick_black_hole_infusion
                    .in_set(MovementCalculationSet)
                    .run_if(crystal_infused_with(CrystalInfusion::BlackHole)),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
