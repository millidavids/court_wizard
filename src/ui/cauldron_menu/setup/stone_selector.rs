use bevy::prelude::*;

use crate::ui::cauldron_menu::components::*;
use crate::ui::cauldron_menu::constants::*;
use crate::ui::systems::spawn_button;

/// Spawns the Philosopher's Stone selector for the Alchemist — a gold toggle
/// button plus its description. Lives at the top of the left detail panel.
/// Spawns nothing when `show` is false (non-Alchemist, already used this battle,
/// or while brewing).
pub(crate) fn spawn_philosophers_stone_selector(
    parent: &mut ChildSpawnerCommands,
    has_stone: bool,
    show: bool,
) {
    if !show {
        return;
    }

    let stone_style = if has_stone {
        &STONE_SELECTED_STYLE
    } else {
        &STONE_BUTTON_STYLE
    };

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
            },
            StoneSelectorPanel,
        ))
        .with_children(|card| {
            spawn_button(
                card,
                "Philosopher's Stone",
                (
                    CauldronMenuButtonAction::TogglePhilosophersStone,
                    crate::ui::focus::CrossRowHorizontalNav,
                ),
                stone_style,
            );

            card.spawn((
                Text::new("Removes dilution (once per battle)"),
                TextFont::from_font_size(DESCRIPTION_FONT_SIZE),
                TextColor(stone_style.text_color),
                TextLayout::justify(Justify::Center),
                Node {
                    max_width: Val::Px(BUTTON_WIDTH),
                    ..default()
                },
            ));
        });
}
