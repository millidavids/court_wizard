use bevy::prelude::*;

use super::components::*;
use super::constants;
use super::messages::*;
use super::spells::magic_missile_constants;
use super::styles::*;
use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::MouseButtonState;
use crate::game::units::components::{Health, Hitbox, MovementSpeed};

/// Sets up the wizard when entering the InGame state.
///
/// Spawns the wizard entity as a triangle on the castle platform in 3D space.
pub fn setup_wizard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Define wizard hitbox (cylinder) - this determines sprite size
    let hitbox = Hitbox::new(constants::HITBOX_RADIUS, constants::HITBOX_HEIGHT);

    // Spawn wizard as a triangle billboard sized to match the hitbox
    let wizard_width = hitbox.sprite_width();
    let wizard_height = hitbox.sprite_height();
    let wizard_triangle = Triangle2d::new(
        Vec2::new(0.0, wizard_height / 2.0), // Top vertex
        Vec2::new(-wizard_width / 2.0, -wizard_height / 2.0), // Bottom-left
        Vec2::new(wizard_width / 2.0, -wizard_height / 2.0), // Bottom-right
    );

    commands.spawn((
        Mesh3d(meshes.add(wizard_triangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WIZARD_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(WIZARD_POSITION),
        hitbox,
        Health::new(constants::HEALTH),
        MovementSpeed(0.0), // Wizard doesn't move
        Mana::new(constants::MANA),
        ManaRegen::new(constants::MANA_REGEN),
        CastingState::new(),
        Wizard::new(constants::DEFAULT_SPELL_RANGE),
        magic_missile_constants::PRIMED_MAGIC_MISSILE,
        Billboard,
        OnGameplayScreen,
    ));
}

/// Regenerates wizard mana over time, scaled by cauldron buffs.
pub fn regenerate_mana(
    time: Res<Time>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut wizards: Query<(&mut Mana, &ManaRegen), With<Wizard>>,
) {
    for (mut mana, regen) in &mut wizards {
        let rate = regen.rate * cauldron_buffs.mana_regen_multiplier();
        mana.regenerate(rate * time.delta_secs());
    }
}

/// Handles PrimeSpellMessage to update the wizard's primed spell.
/// This allows UI systems to request spell changes without directly accessing components.
/// Applies cauldron spell power buff as a base empowerment multiplier.
pub fn handle_prime_spell_messages(
    mut messages: MessageReader<PrimeSpellMessage>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut wizard_query: Query<&mut PrimedSpell, With<Wizard>>,
) {
    for message in messages.read() {
        if let Ok(mut primed_spell) = wizard_query.single_mut() {
            let mut spell = message.spell;
            if cauldron_buffs.spell_power_multiplier() > 1.0 {
                spell = spell
                    .with_empowerment(spell.empowerment * cauldron_buffs.spell_power_multiplier());
            }
            *primed_spell = spell;
        }
    }
}

/// Manages empowerment consumption and reset based on casting state transitions.
///
/// When a cast starts (transition to Casting), marks empowerment as consumed.
/// When returning to Resting, resets empowerment if it was previously consumed.
/// This ensures empowerment only applies to a single cast (including full channel duration).
pub fn reset_empowerment_after_cast(
    mut wizard_query: Query<
        (&CastingState, &mut PrimedSpell),
        (With<Wizard>, Changed<CastingState>),
    >,
) {
    for (casting_state, mut primed_spell) in &mut wizard_query {
        match casting_state {
            // When starting a cast, mark empowerment as consumed
            CastingState::Casting { .. } => {
                primed_spell.consume_empowerment();
            }
            // When returning to rest, reset empowerment if it was consumed
            CastingState::Resting => {
                if primed_spell.should_reset_empowerment() {
                    primed_spell.reset_empowerment();
                }
            }
            // Channeling state doesn't affect empowerment
            CastingState::Channeling { .. } => {}
        }
    }
}

/// Cancels any active casting when leaving the Running state.
///
/// Prevents spells from continuing to cast when entering menus or paused state.
/// Also resets the mouse button state to prevent lingering input.
pub fn cancel_active_casts(
    mut wizard_query: Query<&mut CastingState, With<Wizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    if let Ok(mut casting_state) = wizard_query.single_mut()
        && !matches!(*casting_state, CastingState::Resting)
    {
        casting_state.cancel();
    }
    // Reset mouse state when exiting running state
    mouse_state.left_consumed = false;
}
