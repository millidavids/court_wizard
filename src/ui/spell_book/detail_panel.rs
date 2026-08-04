//! The left detail panel: the previewed spell's icon, name, type, and text.
//!
//! The hotkey boxes and the right-hand list refresh themselves — see
//! `hotkey_slots::refresh_hotkey_slots` and
//! `spell_list::refresh_spell_list_selection`.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use crate::config::GameConfig;
use crate::game::units::wizard::components::{
    Spell, effective_status_effects, spawn_status_effects_section,
};
use crate::networking::session::MultiplayerSession;
use crate::ui::components::SpellIconAssets;

/// Spawns the left detail panel showing spell info and the hotkey row.
pub(super) fn spawn_detail_panel(
    parent: &mut ChildSpawnerCommands,
    spell: Spell,
    config: &GameConfig,
    mp_session: Option<&MultiplayerSession>,
    icon_assets: &SpellIconAssets,
) {
    let detail_box = crate::ui::systems::spawn_left_detail_panel(parent);

    parent.commands().entity(detail_box).with_children(|panel| {
        // Spell icon + name
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    ImageNode::new(icon_assets.get(&spell).cloned().unwrap_or_default()),
                    Node {
                        width: Val::Px(DETAIL_ICON_SIZE),
                        height: Val::Px(DETAIL_ICON_SIZE),
                        ..default()
                    },
                    DetailSpellIcon,
                ));
                header.spawn((
                    Text::new(spell.display_name()),
                    TextFont::from_font_size(DETAIL_NAME_FONT_SIZE),
                    TextColor(DETAIL_NAME_COLOR),
                    DetailName,
                ));
            });

        // Damage type
        panel.spawn((
            Text::new(spell.damage_type().display_name()),
            TextFont::from_font_size(DETAIL_TYPE_FONT_SIZE),
            TextColor(DETAIL_TYPE_COLOR),
            DetailDamageType,
        ));

        // Description
        panel.spawn((
            Text::new(spell.description()),
            TextFont::from_font_size(DETAIL_DESC_FONT_SIZE),
            TextColor(DETAIL_DESC_COLOR),
            Node {
                max_width: Val::Px(LEFT_PANEL_WIDTH - DETAIL_PADDING * 2.0),
                ..default()
            },
            DetailDescription,
        ));

        // Status effects section (no-op when the spell applies none).
        spawn_status_effects_section(
            panel,
            effective_status_effects(spell, config.wizard_type),
            DETAIL_DESC_FONT_SIZE,
            DETAIL_DESC_COLOR,
            Some(Node {
                max_width: Val::Px(LEFT_PANEL_WIDTH - DETAIL_PADDING * 2.0),
                ..default()
            }),
        );

        // Instructions
        panel.spawn((
            Text::new(spell.instructions()),
            TextFont::from_font_size(DETAIL_INSTRUCTIONS_FONT_SIZE),
            TextColor(DETAIL_INSTRUCTIONS_COLOR),
            DetailInstructions,
        ));

        super::hotkey_slots::spawn_hotkey_row(panel, spell, config, mp_session, icon_assets);

        // Selecting a spell from the right list primes it and closes the
        // menu automatically, so no Select / Close buttons are needed here.
        // The header's Back button (B / Escape) still closes the menu.
    });
}

/// Updates the detail panel's icon and text when the previewed spell changes.
pub(super) fn update_detail_panel(
    selected: Res<SelectedSpellPreview>,
    icon_assets: Res<SpellIconAssets>,
    mut icon_query: Query<&mut ImageNode, With<DetailSpellIcon>>,
    mut name_query: Query<
        &mut Text,
        (
            With<DetailName>,
            Without<DetailDamageType>,
            Without<DetailDescription>,
            Without<DetailInstructions>,
        ),
    >,
    mut type_query: Query<
        &mut Text,
        (
            With<DetailDamageType>,
            Without<DetailName>,
            Without<DetailDescription>,
            Without<DetailInstructions>,
        ),
    >,
    mut desc_query: Query<
        &mut Text,
        (
            With<DetailDescription>,
            Without<DetailName>,
            Without<DetailDamageType>,
            Without<DetailInstructions>,
        ),
    >,
    mut instr_query: Query<
        &mut Text,
        (
            With<DetailInstructions>,
            Without<DetailName>,
            Without<DetailDamageType>,
            Without<DetailDescription>,
        ),
    >,
) {
    if !selected.is_changed() {
        return;
    }

    let spell = selected.0;

    if let Ok(mut image) = icon_query.single_mut()
        && let Some(handle) = icon_assets.get(&spell)
    {
        image.image = handle.clone();
    }
    if let Ok(mut text) = name_query.single_mut() {
        **text = spell.display_name().to_string();
    }
    if let Ok(mut text) = type_query.single_mut() {
        **text = spell.damage_type().display_name().to_string();
    }
    if let Ok(mut text) = desc_query.single_mut() {
        **text = spell.description().to_string();
    }
    if let Ok(mut text) = instr_query.single_mut() {
        **text = spell.instructions().to_string();
    }
}
