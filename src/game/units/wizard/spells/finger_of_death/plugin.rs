use bevy::prelude::*;

use super::super::super::components::Spell;
use super::super::run_conditions::*;
use super::components::{
    AwaitingFingerOfDeathRelease, DeathmarkDebuff, FingerOfDeathBeam, FingerOfDeathCooldown,
    FingerOfDeathFlare, FingerOfDeathGlow, NecroticExplosionBurst, NecroticPulse, NecroticVein,
    PendingUndeadRaise, ReapersScytheSweep,
};
use super::systems;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::spells::utils::{PendingDefenderHeal, apply_pending_defender_heal};

pub struct FingerOfDeathPlugin;

impl Plugin for FingerOfDeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Cooldown tick
                systems::tick_finger_of_death_cooldown
                    .run_if(any_exist::<FingerOfDeathCooldown>())
                    .run_if(is_spell_effects_active),
                // Local wizard casting (mouse input)
                handle_finger_of_death_casting
                    .run_if(spell_is_primed(Spell::FingerOfDeath))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::apply_finger_of_death_damage,
                    systems::update_finger_of_death_beam_visuals,
                    systems::cleanup_finger_of_death_beams,
                )
                    .chain()
                    .run_if(any_exist::<FingerOfDeathBeam>()),
                // Release tracking (runs independently of casting run conditions)
                systems::clear_awaiting_fod_release
                    .run_if(any_exist::<AwaitingFingerOfDeathRelease>()),
                // Talent systems
                systems::process_pending_undead_raises
                    .run_if(resource_exists::<PendingUndeadRaise>),
                apply_pending_defender_heal
                    .run_if(resource_exists::<PendingDefenderHeal>),
                systems::apply_necrotic_explosion_damage
                    .run_if(any_exist::<NecroticExplosionBurst>()),
                systems::update_necrotic_explosion_bursts
                    .run_if(any_exist::<NecroticExplosionBurst>()),
                systems::update_deathmark_debuffs
                    .run_if(any_exist::<DeathmarkDebuff>()),
                systems::update_reapers_scythe
                    .run_if(any_exist::<ReapersScytheSweep>()),
                // Existing visual systems
                systems::update_necrotic_veins.run_if(any_exist::<NecroticVein>()),
                (
                    systems::update_finger_of_death_glow,
                    systems::cleanup_finger_of_death_glow,
                )
                    .chain()
                    .run_if(any_exist::<FingerOfDeathGlow>()),
                (
                    systems::update_finger_of_death_flare,
                    systems::cleanup_finger_of_death_flare,
                )
                    .chain()
                    .run_if(any_exist::<FingerOfDeathFlare>()),
                systems::update_necrotic_pulse.run_if(any_exist::<NecroticPulse>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}

// Re-export the casting system with a shorter name for use in plugin registration
use systems::handle_finger_of_death_casting;
