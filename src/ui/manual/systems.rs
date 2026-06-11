//! Systems for the manual screen.

use bevy::prelude::*;

use super::components::{
    BackButton, ManualContentPanel, ManualTab, ManualTabButton, OnManualScreen,
    ScrollableManualContainer,
};
use super::constants::*;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::state::{MenuState, PauseMenuState};
use crate::ui::components::{ButtonActive, ButtonColors};
use crate::ui::constants::{
    ACTIVE_TAB_BG, ACTIVE_TAB_BORDER, BACK_BUTTON_STYLE, INACTIVE_TAB_BG, TAB_BORDER,
    TAB_FONT_SIZE, TAB_HEIGHT, TAB_PADDING_H, TEXT_BODY,
};
use crate::ui::link_button::LinkButton;
use crate::ui::markdown::{MarkdownBlock, parse_markdown, spawn_markdown};
use crate::ui::systems::{spawn_button, spawn_page_header};

const INSTRUCTIONS_TEXT: &str = include_str!("../../../docs/INSTRUCTIONS.md");
const CHANGELOG_TEXT: &str = include_str!("../../../docs/CHANGELOG.md");
const CREDITS_TEXT: &str = include_str!("../../../docs/CREDITS.md");
const LICENSE_TEXT: &str = include_str!("../../../LICENSE");
const HEALTH_WARNING_TEXT: &str = include_str!("../../../docs/HEALTH_WARNING.md");
const PRIVACY_POLICY_TEXT: &str = include_str!("../../../docs/PRIVACY_POLICY.md");

/// Pre-parsed markdown blocks for each tab, computed once at setup.
#[derive(Resource)]
pub(super) struct ParsedManualContent {
    instructions: Vec<MarkdownBlock>,
    changelog: Vec<MarkdownBlock>,
    credits: Vec<MarkdownBlock>,
    license: Vec<MarkdownBlock>,
    health: Vec<MarkdownBlock>,
    privacy: Vec<MarkdownBlock>,
}

impl ParsedManualContent {
    fn blocks_for(&self, tab: ManualTab) -> &[MarkdownBlock] {
        match tab {
            ManualTab::Instructions => &self.instructions,
            ManualTab::Changelog => &self.changelog,
            ManualTab::Credits => &self.credits,
            ManualTab::License => &self.license,
            ManualTab::Health => &self.health,
            ManualTab::Privacy => &self.privacy,
        }
    }
}

// Skips blocks before the first H2 (drops the H1 title and intro paragraph),
// then keeps up to `count` H2-bounded sections.
fn truncate_changelog_to_recent(blocks: Vec<MarkdownBlock>, count: usize) -> Vec<MarkdownBlock> {
    if count == 0 {
        return Vec::new();
    }

    let mut h2_indices = blocks.iter().enumerate().filter_map(|(i, b)| match b {
        MarkdownBlock::Heading { level: 2, .. } => Some(i),
        _ => None,
    });

    let Some(start) = h2_indices.next() else {
        return blocks;
    };
    let end = h2_indices.nth(count - 1).unwrap_or(blocks.len());

    blocks.into_iter().skip(start).take(end - start).collect()
}

/// Spawns the manual screen UI.
pub(super) fn setup(mut commands: Commands, pause_menu: bool) {
    commands.insert_resource(ManualTab::default());
    commands.insert_resource(ParsedManualContent {
        instructions: parse_markdown(INSTRUCTIONS_TEXT),
        changelog: truncate_changelog_to_recent(
            parse_markdown(CHANGELOG_TEXT),
            RECENT_RELEASES_COUNT,
        ),
        credits: parse_markdown(CREDITS_TEXT),
        license: parse_markdown(LICENSE_TEXT),
        health: parse_markdown(HEALTH_WARNING_TEXT),
        privacy: parse_markdown(PRIVACY_POLICY_TEXT),
    });

    let content = crate::ui::systems::spawn_page_container(
        &mut commands,
        OnManualScreen,
        pause_menu,
        crate::ui::systems::default_content_node(),
    );

    commands.entity(content).with_children(|parent| {
        spawn_page_header(
            parent,
            "Manual",
            TITLE_FONT_SIZE,
            TEXT_COLOR,
            BackButton,
            &BACK_BUTTON_STYLE,
        );

        // Tab bar
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            })
            .with_children(|tab_row| {
                let initial_tab = ManualTab::default();
                for tab in ManualTab::all() {
                    let is_active = *tab == initial_tab;

                    let (bg, border) = if is_active {
                        (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
                    } else {
                        (INACTIVE_TAB_BG, TAB_BORDER)
                    };

                    let mut tab_btn = tab_row.spawn((
                        Button,
                        Node {
                            height: Val::Px(TAB_HEIGHT),
                            padding: UiRect::horizontal(Val::Px(TAB_PADDING_H)),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderColor::all(border),
                        ButtonColors {
                            background: bg,
                            border,
                        },
                        ManualTabButton(*tab),
                        crate::ui::focus::Focusable,
                        crate::ui::focus::TabFocusable,
                    ));

                    if is_active {
                        tab_btn.insert(ButtonActive);
                    }

                    tab_btn.with_children(|btn| {
                        btn.spawn((
                            Text::new(tab.label()),
                            TextFont::from_font_size(TAB_FONT_SIZE),
                            TextColor(TEXT_BODY),
                        ));
                    });
                }
            });

        // Scrollable content area
        parent
            .spawn((
                crate::ui::systems::scroll_area_style(Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                }),
                ScrollPosition::default(),
                ScrollableManualContainer,
                crate::ui::focus::GamepadScrollTarget,
            ))
            .with_children(|scroll| {
                scroll.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    ManualContentPanel,
                ));
            });
    });
}

/// Spawns manual for main menu.
pub(super) fn setup_main_menu(commands: Commands) {
    setup(commands, false);
}

/// Spawns manual for pause menu.
pub(super) fn setup_pause_menu(commands: Commands) {
    setup(commands, true);
}

/// Rebuilds content panel when tab changes, using pre-parsed markdown blocks.
pub(super) fn rebuild_content_on_tab_change(
    mut commands: Commands,
    tab: Res<ManualTab>,
    parsed: Res<ParsedManualContent>,
    content_panel: Query<Entity, With<ManualContentPanel>>,
    mut scroll_query: Query<&mut ScrollPosition, With<ScrollableManualContainer>>,
) {
    let Ok(panel_entity) = content_panel.single() else {
        return;
    };

    if let Ok(mut scroll_pos) = scroll_query.single_mut() {
        *scroll_pos = ScrollPosition::default();
    }

    commands.entity(panel_entity).despawn_related::<Children>();

    commands.entity(panel_entity).with_children(|content| {
        spawn_markdown(content, parsed.blocks_for(*tab));

        if *tab == ManualTab::Changelog {
            content
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                })
                .with_children(|wrapper| {
                    spawn_button(
                        wrapper,
                        "View full changelog on the web",
                        LinkButton(CHANGELOG_WEBSITE_URL),
                        &CHANGELOG_LINK_BUTTON_STYLE,
                    );
                });
        }
    });
}

/// Handles tab button clicks — updates the active tab resource.
pub(super) fn handle_tab_click(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    tab_query: Query<&ManualTabButton>,
    mut tab_resource: ResMut<ManualTab>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && *tab_resource != tab_btn.0
        {
            *tab_resource = tab_btn.0;
        }
    }
}

/// Updates tab button visuals when active tab changes.
pub(super) fn update_tab_active_state(
    mut commands: Commands,
    tab: Res<ManualTab>,
    tab_buttons: Query<(Entity, &ManualTabButton, &Children, Has<ButtonActive>)>,
    mut tab_bg: Query<(&mut BackgroundColor, &mut BorderColor, &mut ButtonColors)>,
    mut text_query: Query<&mut TextColor>,
) {
    for (entity, tab_btn, children, has_active) in &tab_buttons {
        let is_active = tab_btn.0 == *tab;

        if is_active && !has_active {
            commands.entity(entity).insert(ButtonActive);
        } else if !is_active && has_active {
            commands.entity(entity).remove::<ButtonActive>();
        }

        let (bg, border) = if is_active {
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
        } else {
            (INACTIVE_TAB_BG, TAB_BORDER)
        };

        if let Ok((mut bg_color, mut border_color, mut colors)) = tab_bg.get_mut(entity) {
            *bg_color = BackgroundColor(bg);
            *border_color = BorderColor::all(border);
            colors.background = bg;
            colors.border = border;
        }

        for child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                *text_color = if is_active {
                    TextColor(TEXT_COLOR)
                } else {
                    TextColor(TEXT_BODY)
                };
            }
        }
    }
}

/// Cleans up manual resources on exit.
pub(super) fn cleanup_resources(mut commands: Commands) {
    commands.remove_resource::<ManualTab>();
    commands.remove_resource::<ParsedManualContent>();
}

/// Handles back button for main menu manual.
pub(super) fn handle_main_menu_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            channel_change.write(ChannelChangeMessage);
            next_state.set(MenuState::Landing);
        }
    }
}

/// Handles back button for pause menu manual.
pub(super) fn handle_pause_menu_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&BackButton>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            next_state.set(PauseMenuState::Main);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h2(text: &str) -> MarkdownBlock {
        MarkdownBlock::Heading {
            level: 2,
            text: text.to_string(),
        }
    }

    fn h3(text: &str) -> MarkdownBlock {
        MarkdownBlock::Heading {
            level: 3,
            text: text.to_string(),
        }
    }

    fn h1(text: &str) -> MarkdownBlock {
        MarkdownBlock::Heading {
            level: 1,
            text: text.to_string(),
        }
    }

    fn paragraph(text: &str) -> MarkdownBlock {
        MarkdownBlock::Paragraph {
            spans: vec![crate::ui::markdown::MarkdownSpan::Plain(text.to_string())],
        }
    }

    #[test]
    fn truncate_drops_h1_and_intro_paragraph() {
        let blocks = vec![
            h1("Changelog"),
            paragraph("All notable changes..."),
            h2("[v3] - 2026-04-29"),
            h3("Added"),
            h2("[v2] - 2026-04-28"),
            h2("[v1] - 2026-04-27"),
        ];
        let result = truncate_changelog_to_recent(blocks, 5);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], h2("[v3] - 2026-04-29"));
    }

    #[test]
    fn truncate_keeps_only_first_count_releases() {
        let blocks = vec![
            h1("Changelog"),
            h2("[v6]"),
            h3("Added"),
            h2("[v5]"),
            h2("[v4]"),
            h2("[v3]"),
            h2("[v2]"),
            h2("[v1]"),
        ];
        let result = truncate_changelog_to_recent(blocks, 5);
        let h2_count = result
            .iter()
            .filter(|b| matches!(b, MarkdownBlock::Heading { level: 2, .. }))
            .count();
        assert_eq!(h2_count, 5);
        assert_eq!(result[0], h2("[v6]"));
        assert!(!result.contains(&h2("[v1]")));
    }

    #[test]
    fn truncate_handles_exactly_count_releases() {
        let blocks = vec![h2("[v3]"), h3("Added"), h2("[v2]"), h2("[v1]")];
        let result = truncate_changelog_to_recent(blocks, 3);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], h2("[v3]"));
    }

    #[test]
    fn truncate_returns_all_when_fewer_than_count() {
        let blocks = vec![h2("[v2]"), h2("[v1]")];
        let result = truncate_changelog_to_recent(blocks, 5);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn truncate_returns_input_when_no_h2() {
        let input = vec![h1("Changelog"), paragraph("Nothing released yet.")];
        let expected = vec![h1("Changelog"), paragraph("Nothing released yet.")];
        let result = truncate_changelog_to_recent(input, 5);
        assert_eq!(result, expected);
    }

    #[test]
    fn truncate_count_zero_returns_empty() {
        let blocks = vec![h2("[v1]")];
        let result = truncate_changelog_to_recent(blocks, 0);
        assert!(result.is_empty());
    }
}
