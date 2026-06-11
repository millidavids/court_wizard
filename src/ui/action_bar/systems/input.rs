use crate::ui::components::ButtonColors;
use bevy::prelude::*;

use super::super::components::*;
use super::super::messages::AssignSpellToSlot;
use super::spawn::effective_slot;
use crate::config::{ConfigChanged, GameConfig, WizardType};
use crate::game::input::messages::{ActionBarKeyPressed, MouseClicked};
use crate::game::units::wizard::archetypes::gunslinger::GunType;
use crate::game::units::wizard::archetypes::gunslinger::messages::SelectGunMessage;
use crate::game::units::wizard::messages::PrimeSpellMessage;

/// Handles action bar slot clicks by priming the assigned spell (or selecting a gun for gunslinger).
pub(crate) fn handle_slot_click(
    mut button_clicked: MessageReader<MouseClicked>,
    slot_query: Query<&ActionBarSlot>,
    config: Res<GameConfig>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut select_gun: MessageWriter<SelectGunMessage>,
    mp_session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    let is_gunslinger = config.wizard_type == WizardType::Warglock;
    let guns = GunType::all();

    for event in button_clicked.read() {
        if let Ok(slot) = slot_query.get(event.button) {
            let slot_idx = slot.slot as usize;
            if is_gunslinger {
                if slot_idx < 5 {
                    select_gun.write(SelectGunMessage {
                        gun: guns[slot_idx],
                    });
                }
            } else if config.wizard_type.uses_exclusive_casting() {
                // RuneCaster/Randomancer can only cast via their own mechanics
            } else if let Some(spell) = effective_slot(&config, slot_idx, mp_session.as_deref()) {
                prime_spell.write(PrimeSpellMessage {
                    spell: spell.primed_config(),
                });
            }
        }
    }
}

/// Handles keyboard input for action bar slots by priming the assigned spell (or selecting a gun).
pub(crate) fn handle_keyboard_input(
    mut action_bar_key: MessageReader<ActionBarKeyPressed>,
    config: Res<GameConfig>,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut select_gun: MessageWriter<SelectGunMessage>,
    mp_session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    let is_gunslinger = config.wizard_type == WizardType::Warglock;
    let guns = GunType::all();

    for event in action_bar_key.read() {
        let slot_idx = event.slot as usize;
        if is_gunslinger {
            if slot_idx < 5 {
                select_gun.write(SelectGunMessage {
                    gun: guns[slot_idx],
                });
            }
        } else if matches!(
            config.wizard_type,
            WizardType::RuneCaster | WizardType::Randomancer
        ) {
            // RuneCaster/Randomancer can only cast via their own mechanics
        } else if let Some(spell) = effective_slot(&config, slot_idx, mp_session.as_deref()) {
            prime_spell.write(PrimeSpellMessage {
                spell: spell.primed_config(),
            });
        }
    }
}

/// Handles clicks on the debug infinite mana button.
#[cfg(debug_assertions)]
pub(crate) fn handle_debug_mana_click(
    mut button_clicked: MessageReader<MouseClicked>,
    debug_button_query: Query<Entity, With<DebugManaButton>>,
    mut infinite_mana: ResMut<InfiniteMana>,
    mut bg_query: Query<(&mut BackgroundColor, &mut ButtonColors), With<DebugManaButton>>,
) {
    use super::spawn::{DEBUG_BUTTON_BG_OFF, DEBUG_BUTTON_BG_ON};
    for event in button_clicked.read() {
        if debug_button_query.get(event.button).is_ok() {
            infinite_mana.0 = !infinite_mana.0;
            let new_bg = if infinite_mana.0 {
                DEBUG_BUTTON_BG_ON
            } else {
                DEBUG_BUTTON_BG_OFF
            };
            for (mut bg, mut colors) in bg_query.iter_mut() {
                bg.0 = new_bg;
                colors.background = new_bg;
            }
        }
    }
}

/// Handles spell assignment to action bar slots.
pub(crate) fn handle_spell_assignment(
    mut assign_spell: MessageReader<AssignSpellToSlot>,
    mut config: ResMut<GameConfig>,
    mut config_changed: MessageWriter<ConfigChanged>,
) {
    for event in assign_spell.read() {
        // Shepherd cannot assign damage-dealing spells
        if config.wizard_type == WizardType::Shepherd && !event.spell.is_shepherd_allowed() {
            continue;
        }
        let slot_idx = event.slot as usize;
        if let Some(slot) = config.action_bar_slots.get_mut(slot_idx) {
            *slot = Some(event.spell);
            config_changed.write(ConfigChanged);
        }
    }
}
