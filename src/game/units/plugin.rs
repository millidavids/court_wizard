use bevy::prelude::*;

use crate::state::InGameState;

use super::archer::ArcherPlugin;
use super::behemoth::BehemothPlugin;
use super::commander::CommanderPlugin;
use super::elite::ElitePlugin;
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
/// - Temporary hit points expiration
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
            BehemothPlugin,
            KingPlugin,
        ))
        .configure_sets(
            Update,
            (MovementCalculationSet, ApplyTransformsSet)
                .chain()
                .run_if(in_state(InGameState::Running)),
        )
        .add_systems(
            Update,
            (
                systems::process_pending_damage_effects,
                systems::update_temporary_hit_points,
                systems::update_frost_slow_modifiers,
                systems::update_rooted_modifiers,
                systems::update_haste_modifiers,
                systems::update_spike_growth_slow_modifiers,
                systems::update_fire_dot,
                systems::update_electric_charge,
                systems::update_electric_arc_visuals,
                systems::update_persistent_effect_visuals,
                movement::apply_unit_movement.in_set(ApplyTransformsSet),
            )
                .run_if(in_state(InGameState::Running)),
        );
    }
}
