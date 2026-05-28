use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{
    Bloodlust, ContagiousRage, FinalStand, FinalStandExplosionVfx, Frenzy, FrenzyActive,
    UndyingFury, UndyingFuryActive,
};
use super::messages::ContagiousRageKillMessage;
use super::systems;
use crate::game::plugin::PostCombatSet;
use crate::game::run_conditions::{any_exist, is_gameplay_running, is_spell_effects_active};

pub struct BerserkerRagePlugin;

impl Plugin for BerserkerRagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ContagiousRageKillMessage>()
            .add_systems(
                Update,
                (
                    // Local wizard casting (mouse input)
                    systems::handle_berserker_rage_casting
                        .run_if(spell_is_primed(Spell::BerserkerRage))
                        .run_if(spell_input_not_blocked)
                        .run_if(mouse_left_not_consumed)
                        .run_if(mouse_held_or_wizard_casting)
                        .run_if(is_spell_effects_active),
                    // Frenzy: toggle attack speed based on HP threshold
                    systems::frenzy_check_system
                        .run_if(any_exist::<Frenzy>())
                        .run_if(is_gameplay_running),
                    // Undying Fury Active: tick timer and enforce min 1 HP
                    systems::tick_undying_fury_active
                        .run_if(any_exist::<UndyingFuryActive>())
                        .run_if(is_gameplay_running),
                    // Cleanup talent components when base modifier expires
                    systems::cleanup_berserker_rage_talents
                        .run_if(
                            any_exist::<Bloodlust>()
                                .or(any_exist::<Frenzy>())
                                .or(any_exist::<FrenzyActive>())
                                .or(any_exist::<UndyingFury>())
                                .or(any_exist::<UndyingFuryActive>())
                                .or(any_exist::<ContagiousRage>())
                                .or(any_exist::<FinalStand>()),
                        )
                        .run_if(is_gameplay_running),
                ),
            )
            // Post-combat systems (after damage and corpse conversion)
            .add_systems(
                Update,
                (
                    // Contagious Rage: spread rage on kill
                    systems::contagious_rage_spread
                        .after(PostCombatSet)
                        .run_if(is_gameplay_running)
                        .run_if(on_message::<ContagiousRageKillMessage>),
                    // Final Stand: death explosion
                    systems::final_stand_explosion
                        .after(PostCombatSet)
                        .run_if(is_gameplay_running)
                        .run_if(any_exist::<FinalStand>()),
                    // Final Stand: expand explosion visual then despawn.
                    // Visual-only — runs on both MP peers.
                    systems::update_final_stand_vfx
                        .run_if(any_exist::<FinalStandExplosionVfx>())
                        .run_if(is_spell_effects_active),
                ),
            );
    }
}
