use bevy::prelude::*;

use crate::game::run_conditions::{is_gameplay_running, is_spell_effects_active};

use super::archer::ArcherPlugin;
use super::boss::BossPlugin;
use super::brute::BrutePlugin;
use super::commander::CommanderPlugin;
use super::components::{
    BattleHymnModifier, BerserkerRageModifier, FacingDirection, FogEvasionModifier,
    FrostEffectMarker, FrozenSolidModifier, HasteModifier, Knockback, MarkedForDeathModifier,
    PoisonedModifier, RootedModifier, SickenedModifier, SlowMovementModifier,
    SmellyModifier, TemporaryHitPoints, WalkingAnimation,
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
                systems::update_timed_modifier::<SlowMovementModifier>,
                systems::update_timed_modifier::<FrostEffectMarker>,
                systems::update_timed_modifier::<RootedModifier>,
                systems::update_timed_modifier::<HasteModifier>,
                movement::apply_unit_movement.in_set(ApplyTransformsSet),
                movement::clear_corpse_velocity.after(movement::apply_unit_movement),
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
                systems::update_poisoned.run_if(any_with_component::<PoisonedModifier>),
                systems::update_sickened.run_if(any_with_component::<SickenedModifier>),
                systems::update_timed_modifier::<SmellyModifier>
                    .run_if(any_with_component::<SmellyModifier>),
                systems::update_persistent_effect_visuals,
            )
                .run_if(is_spell_effects_active),
        )
        .add_systems(
            Update,
            systems::apply_knockback_effects
                .after(ApplyTransformsSet)
                .run_if(is_gameplay_running)
                .run_if(any_with_component::<Knockback>),
        )
        .add_systems(
            Update,
            (
                systems::update_timed_modifier::<MarkedForDeathModifier>
                    .run_if(any_with_component::<MarkedForDeathModifier>),
                // SleepModifier timer is handled by SleepPlugin::update_sleep_modifiers
                // (combined with Night Terrors DPS to avoid query conflicts)
                systems::update_timed_modifier::<BattleHymnModifier>
                    .run_if(any_with_component::<BattleHymnModifier>),
                systems::update_timed_modifier::<BerserkerRageModifier>
                    .run_if(any_with_component::<BerserkerRageModifier>),
                systems::update_timed_modifier::<FogEvasionModifier>
                    .run_if(any_with_component::<FogEvasionModifier>),
                systems::update_timed_modifier::<FrozenSolidModifier>
                    .run_if(any_with_component::<FrozenSolidModifier>),
            )
                .run_if(is_gameplay_running),
        );
    }
}
