//! Keeps the floating icon's texture and visibility matched to what is primed.

use bevy::prelude::*;

use crate::config::{GameConfig, WizardType};
use crate::game::units::wizard::archetypes::gunslinger::GunState;
use crate::game::units::wizard::components::{LocalWizard, PrimedSpell};
use crate::ui::components::{GunIconAssets, SpellIconAssets};

use super::icon::{PrimedSpellIcon, PrimedSpellIconCorona};

/// Swaps the icon's texture when the primed spell (or selected gun) changes.
///
/// Reuses the `Startup`-loaded [`SpellIconAssets`] / [`GunIconAssets`] rather
/// than loading the PNGs a second time.
#[allow(clippy::too_many_arguments)]
pub(super) fn sync_primed_spell_icon(
    config: Res<GameConfig>,
    wizard_query: Query<&PrimedSpell, With<LocalWizard>>,
    gun_state: Option<Res<GunState>>,
    spell_icons: Res<SpellIconAssets>,
    gun_icons: Res<GunIconAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut icon_query: Query<(
        &mut PrimedSpellIcon,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
    corona_query: Query<&MeshMaterial3d<StandardMaterial>, With<PrimedSpellIconCorona>>,
) {
    let Ok((mut icon, material_handle, mut visibility)) = icon_query.single_mut() else {
        return;
    };

    let desired = resolve_texture(
        &config,
        &wizard_query,
        gun_state.as_deref(),
        &spell_icons,
        &gun_icons,
    );

    if desired == icon.0 {
        return;
    }

    *visibility = if desired.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    // The corona shares the icon's texture — its alpha is what shapes the glow.
    for handle in std::iter::once(material_handle).chain(corona_query.iter()) {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color_texture = desired.clone();
        }
    }

    icon.0 = desired;
}

/// The icon the current archetype should be showing, if any.
///
/// `wizard_type` is checked before `GunState` on purpose: the Endless archetype
/// cycle can move the player off Warglock while leaving `GunState` in place
/// (its reset is `OnExit`-gated by the archetype), so the resource's presence
/// alone is not evidence that guns are in use.
///
/// Unlike the action bar's `highlight_active_slot`, this shows an icon for the
/// RuneCaster and Randomancer too — they have no action bar, so this is their
/// only readout.
fn resolve_texture(
    config: &GameConfig,
    wizard_query: &Query<&PrimedSpell, With<LocalWizard>>,
    gun_state: Option<&GunState>,
    spell_icons: &SpellIconAssets,
    gun_icons: &GunIconAssets,
) -> Option<Handle<Image>> {
    if config.wizard_type == WizardType::Warglock {
        return gun_state.and_then(|state| gun_icons.get(&state.selected_gun).cloned());
    }

    wizard_query
        .single()
        .ok()
        .and_then(|primed| spell_icons.get(&primed.spell).cloned())
}
