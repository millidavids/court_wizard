//! Spell book UI setup.

use bevy::prelude::*;

use super::components::*;
use crate::config::save_data::load_unified_save;
use crate::config::{GameConfig, WizardType};
use crate::game::units::wizard::components::{LocalWizard, PrimedSpell, Spell, SpellCategory};
use crate::networking::session::MultiplayerSession;
use crate::ui::components::SpellIconAssets;
use crate::ui::systems::{spawn_button, spawn_page_container};

/// Resource to track when we just entered the spell book.
/// Prevents spell casting on the same frame as opening the spell book.
#[derive(Resource, Default)]
pub(super) struct JustEnteredSpellBook(pub bool);

// ---------------------------------------------------------------------------
// Setup / teardown
// ---------------------------------------------------------------------------

/// Spawns the spell book UI when entering the SpellBook state.
pub(super) fn spawn_spell_book_ui(
    mut commands: Commands,
    config: Res<GameConfig>,
    mp_session: Option<Res<MultiplayerSession>>,
    icon_assets: Res<SpellIconAssets>,
    local_wizard: Query<&PrimedSpell, With<LocalWizard>>,
) {
    // In multiplayer, all spells are available regardless of single-player progression.
    let is_multiplayer = mp_session.is_some();

    let unlocked_spells: Vec<String> = if is_multiplayer {
        Vec::new() // Not needed — is_unlocked always returns true
    } else {
        load_unified_save()
            .map(|s| s.player.unlocked_content.spells)
            .unwrap_or_default()
    };

    let is_shepherd = config.wizard_type == WizardType::Shepherd;
    let is_unlocked = move |spell: &Spell| {
        if is_shepherd && !spell.is_shepherd_allowed() {
            return false;
        }
        // Hide spells that have no effect in MP (e.g. Telekinesis pulls
        // IngredientDrops which are SP-only and never synced). Without
        // this, the player can assign them to slots and see an empty
        // action-bar slot with no feedback as to why.
        if is_multiplayer && !spell.is_mp_allowed() {
            return false;
        }
        if is_multiplayer {
            return true;
        }
        let debug_name = spell.save_key().to_string();
        unlocked_spells.contains(&debug_name)
    };

    // Open on the wizard's currently primed spell; fall back to the first
    // unlocked, then MagicMissile.
    let initial_spell = local_wizard
        .single()
        .ok()
        .map(|p| p.spell)
        .filter(|s| is_unlocked(s))
        .or_else(|| {
            SpellCategory::all()
                .iter()
                .flat_map(|cat| cat.spells().iter())
                .find(|s| is_unlocked(s))
                .copied()
        })
        .unwrap_or(Spell::MagicMissile);

    commands.insert_resource(SelectedSpellPreview(initial_spell));

    // Page container with column layout (header + two-panel content).
    // `ModalOverlay` scopes gamepad focus to this overlay so the Spells /
    // Cauldron HUD buttons rendering behind it don't pick up focus.
    let content = spawn_page_container(
        &mut commands,
        OnSpellBookScreen,
        false,
        crate::ui::systems::default_content_node(),
    );
    commands
        .entity(content)
        .insert(crate::ui::focus::ModalOverlay);

    commands.entity(content).with_children(|root| {
        // Header row: title left, Back button right
        root.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|header| {
            crate::ui::systems::spawn_title_with_shadow(
                header,
                "Spells",
                36.0,
                crate::ui::constants::TEXT_PRIMARY,
                Node::default(),
            );
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            spawn_button(
                header,
                "Back",
                (
                    SpellBookButtonAction::Close,
                    crate::ui::focus::NoGamepadFocus,
                ),
                &crate::ui::main_menu::BACK_BUTTON_STYLE,
            );
        });

        // Two-panel content row. `min_height: 0` lets it shrink to page
        // height; see `spawn_right_scroll_panel` for the same flex gotcha.
        root.spawn(Node {
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            min_height: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(crate::ui::constants::TWO_PANEL_GAP),
            ..default()
        })
        .with_children(|panels| {
            // === Left panel: detail + buttons ===
            super::detail_panel::spawn_detail_panel(
                panels,
                initial_spell,
                &config,
                mp_session.as_deref(),
                &icon_assets,
            );

            // === Right panel: categorized spell list ===
            super::spell_list::spawn_spell_list(panels, initial_spell, &is_unlocked, &icon_assets);
        });
    });
}
