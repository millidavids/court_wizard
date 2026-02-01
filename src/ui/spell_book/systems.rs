use bevy::ecs::relationship::Relationship;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::ComputedNode;

use super::components::*;
use super::constants::*;
use crate::game::units::wizard::components::{PrimeSpellMessage, Spell};
use crate::state::InGameState;
use crate::ui::components::{ButtonColors, ButtonStyle};
use crate::ui::systems::spawn_button;

/// Resource to track when we just entered the spell book.
/// Prevents spell casting on the same frame as opening the spell book.
#[derive(Resource, Default)]
pub struct JustEnteredSpellBook(pub bool);

/// Marker component to track that a button was pressed down.
#[derive(Component)]
pub(super) struct ButtonPressedDown;

/// Spawns the spell book UI when entering the SpellBook state.
pub fn spawn_spell_book_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(MARGIN),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnSpellBookScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Select Spell"),
                TextFont {
                    font_size: TITLE_FONT_SIZE,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));

            // Scrollable horizontal container
            parent
                .spawn((
                    Node {
                        width: Val::Percent(SCROLL_CONTAINER_WIDTH_PCT),
                        height: Val::Percent(SCROLL_CONTAINER_HEIGHT_PCT),
                        overflow: Overflow::scroll_x(),
                        border: UiRect::all(Val::Px(FRAME_BORDER_WIDTH)),
                        padding: UiRect::all(Val::Px(FRAME_PADDING)),
                        ..default()
                    },
                    BorderColor::all(FRAME_BORDER_COLOR),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(FRAME_BACKGROUND),
                    ScrollPosition::default(),
                    ScrollableSpellContainer,
                ))
                .with_children(|scroll| {
                    // Column of three aligned rows
                    scroll
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|col| {
                            let spells = Spell::all();

                            // Buttons row
                            col.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(SPELL_COLUMN_GAP),
                                ..default()
                            })
                            .with_children(|row| {
                                for spell in spells {
                                    let name = spell.name();
                                    let min_chars = 6.0;
                                    let max_chars = 16.0;
                                    let min_scale = 0.7;
                                    let t = ((name.len() as f32 - min_chars)
                                        / (max_chars - min_chars))
                                        .clamp(0.0, 1.0);
                                    let font_size =
                                        BUTTON_FONT_SIZE * (1.0 - t * (1.0 - min_scale));
                                    spawn_spell_button(
                                        row,
                                        name,
                                        SpellBookButtonAction::SelectSpell(*spell),
                                        &BUTTON_STYLE,
                                        font_size,
                                    );
                                }
                            });

                            // Instructions row
                            col.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(SPELL_COLUMN_GAP),
                                ..default()
                            })
                            .with_children(|row| {
                                for spell in spells {
                                    row.spawn(Node {
                                        width: Val::Px(SPELL_COLUMN_WIDTH),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::horizontal(Val::Px(COLUMN_PADDING)),
                                        ..default()
                                    })
                                    .with_children(|cell| {
                                        cell.spawn((
                                            Text::new(spell.instructions()),
                                            TextFont {
                                                font_size: INSTRUCTIONS_FONT_SIZE,
                                                ..default()
                                            },
                                            TextColor(INSTRUCTIONS_COLOR),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });
                                }
                            });

                            // Descriptions row
                            col.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(SPELL_COLUMN_GAP),
                                ..default()
                            })
                            .with_children(|row| {
                                for spell in spells {
                                    row.spawn((
                                        Text::new(spell.description()),
                                        TextFont {
                                            font_size: DESCRIPTION_FONT_SIZE,
                                            ..default()
                                        },
                                        TextColor(TEXT_COLOR),
                                        TextLayout::new_with_justify(Justify::Center),
                                        Node {
                                            width: Val::Px(SPELL_COLUMN_WIDTH),
                                            padding: UiRect::horizontal(Val::Px(COLUMN_PADDING)),
                                            ..default()
                                        },
                                    ));
                                }
                            });
                        });
                });

            // Close button
            spawn_button(
                parent,
                "Close",
                SpellBookButtonAction::Close,
                &CLOSE_BUTTON_STYLE,
            );
        });
}

/// Spawns a spell button with a custom font size override.
fn spawn_spell_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: impl Component,
    style: &ButtonStyle,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(style.width),
                height: Val::Px(style.height),
                border: UiRect::all(Val::Px(style.border_width)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(style.border),
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(style.background),
            ButtonColors {
                background: style.background,
                border: style.border,
            },
            action,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(text),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(style.text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

/// Handles mouse wheel scrolling for the spell book container.
pub fn handle_spell_scroll(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<
        (&mut ScrollPosition, &ComputedNode),
        With<ScrollableSpellContainer>,
    >,
    parent_query: Query<&ChildOf>,
) {
    const LINE_HEIGHT: f32 = 10.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    for event in mouse_wheel_events.read() {
        let dx = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            bevy::input::mouse::MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.x - visible_size.x).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.x = (scroll_position.x + dx).clamp(0.0, max_scroll);
                        break;
                    }

                    if let Ok(parent) = parent_query.get(current_entity) {
                        current_entity = parent.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

/// Handles button click actions and sends prime spell messages.
/// Uses a marker component to ensure buttons only trigger on release after being pressed.
pub fn button_action(
    mut commands: Commands,
    interaction_query: Query<
        (
            Entity,
            &Interaction,
            &SpellBookButtonAction,
            Option<&ButtonPressedDown>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut prime_spell: MessageWriter<PrimeSpellMessage>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    for (entity, interaction, action, pressed_down) in &interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Mark button as pressed down
                commands.entity(entity).insert(ButtonPressedDown);
            }
            Interaction::Hovered => {
                // Only trigger action if button was previously pressed
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    match action {
                        SpellBookButtonAction::SelectSpell(spell) => {
                            prime_spell.write(PrimeSpellMessage {
                                spell: spell.primed_config(),
                            });
                            next_in_game_state.set(InGameState::Running);
                        }
                        SpellBookButtonAction::Close => {
                            next_in_game_state.set(InGameState::Running);
                        }
                    }
                }
            }
            Interaction::None => {
                // Trigger action on release (touch goes Pressed → None, skipping Hovered)
                if pressed_down.is_some() {
                    commands.entity(entity).remove::<ButtonPressedDown>();

                    match action {
                        SpellBookButtonAction::SelectSpell(spell) => {
                            prime_spell.write(PrimeSpellMessage {
                                spell: spell.primed_config(),
                            });
                            next_in_game_state.set(InGameState::Running);
                        }
                        SpellBookButtonAction::Close => {
                            next_in_game_state.set(InGameState::Running);
                        }
                    }
                }
            }
        }
    }
}

/// Handles keyboard input (ESC to close).
pub fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_in_game_state.set(InGameState::Running);
    }
}

/// Despawns spell book UI when exiting the SpellBook state.
pub fn despawn_spell_book_ui(
    mut commands: Commands,
    query: Query<Entity, With<OnSpellBookScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Sets the flag when entering spell book to prevent spell casting.
pub fn set_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = true;
}

/// Clears the flag after one frame in SpellBook state.
pub fn clear_just_entered_flag(mut just_entered: ResMut<JustEnteredSpellBook>) {
    just_entered.0 = false;
}
