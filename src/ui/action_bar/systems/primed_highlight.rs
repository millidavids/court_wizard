//! Persistent highlight on the action bar slot holding the active choice —
//! the primed spell, or the Warglock's selected gun.

use bevy::prelude::*;

use super::super::components::ActionBarSlot;
use super::super::constants::{PRIMED_SLOT_COLOR, SLOT_BUTTON_STYLE};
use super::spawn::effective_slot;
use crate::config::{GameConfig, WizardType};
use crate::game::units::wizard::archetypes::gunslinger::{GunState, GunType};
use crate::game::units::wizard::components::{LocalWizard, PrimedSpell};
use crate::networking::session::MultiplayerSession;
use crate::ui::components::ButtonColors;

/// Marks the slot holding the active spell/gun by writing its `ButtonColors`.
///
/// `ButtonColors` — not the button's own `BorderColor`. The slots are
/// restructured into 3D buttons by `apply_3d_button_structure`, which zeroes
/// the wrapper's border and moves the visible surface onto a `ButtonFront`
/// child. `sync_front_face_colors` propagates `ButtonColors` there, and every
/// transient effect (press, radial hover, commit flash, device-change reset)
/// restores from `colors.border` — so parking the highlight in `ButtonColors`
/// makes those effects *restore* it instead of erasing it.
///
/// This relies on action bar slots never carrying `ButtonActive`, since
/// `sync_front_face_colors` is filtered `Without<ButtonActive>`. If slots ever
/// gain that marker, route the highlight through `enforce_active_button_state`
/// instead.
pub(crate) fn highlight_active_slot(
    config: Res<GameConfig>,
    wizard_query: Query<&PrimedSpell, With<LocalWizard>>,
    gun_state: Option<Res<GunState>>,
    mp_session: Option<Res<MultiplayerSession>>,
    mut slots: Query<(&ActionBarSlot, &mut ButtonColors)>,
) {
    let active = active_slot_index(
        &config,
        &wizard_query,
        gun_state.as_deref(),
        mp_session.as_deref(),
    );

    for (slot, mut colors) in slots.iter_mut() {
        let desired = if Some(slot.slot) == active {
            PRIMED_SLOT_COLOR
        } else {
            SLOT_BUTTON_STYLE.border
        };

        // Compare against the live value rather than caching in a `Local`:
        // slots are despawned and respawned between levels while the config
        // and primed spell stay put, so cached state would skip the re-apply
        // and leave the highlight missing from level 2 onward. Writing only on
        // a real change also keeps `Changed<ButtonColors>` from refiring the
        // front-face sync every frame.
        if colors.border != desired {
            colors.border = desired;
        }
    }
}

/// Which slot currently holds the active choice, if any.
///
/// See `primed_spell_indicator::sync` for the same archetype precedence — that
/// one also covers RuneCaster/Randomancer, which have no action bar here.
fn active_slot_index(
    config: &GameConfig,
    wizard_query: &Query<&PrimedSpell, With<LocalWizard>>,
    gun_state: Option<&GunState>,
    mp_session: Option<&MultiplayerSession>,
) -> Option<u8> {
    // Checked before `GunState` because the Endless archetype cycle can move
    // the player off Warglock while leaving the resource in place.
    if config.wizard_type == WizardType::Warglock {
        let selected = gun_state?.selected_gun;
        return GunType::all()
            .iter()
            .position(|gun| *gun == selected)
            .map(|idx| idx as u8);
    }

    // RuneCaster/Randomancer have no action bar at all.
    if config.wizard_type.uses_exclusive_casting() {
        return None;
    }

    let primed = wizard_query.single().ok()?;

    (0..5u8).find(|slot| effective_slot(config, *slot as usize, mp_session) == Some(primed.spell))
}
