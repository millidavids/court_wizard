use bevy::prelude::*;

use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};

use super::archer::ArcherPlugin;
use super::brute::BrutePlugin;
use super::boss::BossPlugin;
use super::commander::CommanderPlugin;
use super::components::{
    BattleHymnModifier, BerserkerRageModifier, FacingDirection, FogEvasionModifier,
    FrostSlowModifier, GreaseSlipModifier, HasteModifier, MarkedForDeathModifier, RootedModifier,
    SleepModifier, SpikeGrowthSlowModifier, TemporaryHitPoints,
    WalkingAnimation,
};
use super::dispeller::DispellerPlugin;
use super::elite::ElitePlugin;
use super::healer::HealerPlugin;
use super::infantry::InfantryPlugin;
use super::king::KingPlugin;
use super::movement;
use super::systems;
use super::wizard::WizardPlugin;
use super::{ApplyTransformsSet, MovementCalculationSet};

/// Plugin that coordinates all unit-related sub-plugins.
///
/// Registers sub-plugins for:
/// - Wizard entity (WizardPlugin)
/// - Infantry units on both teams (InfantryPlugin)
/// - Archer units on both teams (ArcherPlugin)
/// - King unit (defender only) (KingPlugin)
///
/// Also registers global unit systems for:
/// - Timed modifier expiration
/// - Movement application (transforms)
pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CommanderPlugin,
            ElitePlugin,
            WizardPlugin,
            InfantryPlugin,
            ArcherPlugin,
            DispellerPlugin,
            HealerPlugin,
            BrutePlugin,
            BossPlugin,
            KingPlugin,
        ))
        .configure_sets(
            Update,
            (MovementCalculationSet, ApplyTransformsSet)
                .chain()
                .run_if(is_gameplay_running),
        )
        .add_systems(
            Update,
            (
                systems::update_timed_modifier::<TemporaryHitPoints>,
                systems::update_timed_modifier::<FrostSlowModifier>,
                systems::update_timed_modifier::<RootedModifier>,
                systems::update_timed_modifier::<HasteModifier>,
                systems::update_timed_modifier::<SpikeGrowthSlowModifier>,
                movement::apply_unit_movement.in_set(ApplyTransformsSet),
                systems::update_walking_animation
                    .after(ApplyTransformsSet)
                    .run_if(any_with_component::<WalkingAnimation>),
                systems::update_facing_direction
                    .after(ApplyTransformsSet)
                    .run_if(any_with_component::<FacingDirection>),
            )
                .run_if(is_gameplay_running),
        )
        // Spell damage effects must run on both host AND guest so that
        // guest spell DoTs tick and feed damage into the CRDT.
        .add_systems(
            Update,
            (
                systems::process_pending_damage_effects,
                systems::update_fire_dot,
                systems::update_electric_charge,
                systems::update_electric_arc_visuals,
                systems::update_persistent_effect_visuals,
            )
                .run_if(is_spell_effects_active),
        )
        .add_systems(
            Update,
            (
                systems::update_timed_modifier::<MarkedForDeathModifier>
                    .run_if(any_with_component::<MarkedForDeathModifier>),
                systems::update_timed_modifier::<SleepModifier>
                    .run_if(any_with_component::<SleepModifier>),
                systems::update_timed_modifier::<BattleHymnModifier>
                    .run_if(any_with_component::<BattleHymnModifier>),
                systems::update_timed_modifier::<BerserkerRageModifier>
                    .run_if(any_with_component::<BerserkerRageModifier>),
                systems::update_timed_modifier::<FogEvasionModifier>
                    .run_if(any_with_component::<FogEvasionModifier>),
                systems::update_timed_modifier::<GreaseSlipModifier>
                    .run_if(any_with_component::<GreaseSlipModifier>),
            )
                .run_if(is_gameplay_running),
        );
    }
}
