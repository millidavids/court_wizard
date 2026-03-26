//! Tutorial systems: triggering, advancing, skipping, highlight, overlay, glow animation.

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::config::save_data::{load_unified_save, new_unified_save, save_unified};
use crate::game::input::messages::MouseClicked;
use crate::state::InGameState;
use crate::ui::action_bar::ActionBarRoot;
use crate::ui::cauldron_menu::CauldronMenuButtonAction;
use crate::ui::components::ButtonStyle;
use crate::ui::in_game::{HudButtonAction, KingHealthBarFill, ManaBarFill, WaveDisplay};
use crate::ui::spell_book::{DetailName, HotkeySlotButton, ScrollableSpellList};
use crate::ui::systems::spawn_button;
use crate::ui::wizard_tower::{
    InsightDisplay, LevelDisplay, SpellGraphArea, StudyButtonAction, StudyDetailPanel,
    TimeTravelContainer, WizardTowerButtonAction,
};

use super::components::{
    GlowAnimation, OriginalBorder, TutorialHighlightable, TutorialNextButton, TutorialOverlay,
    TutorialPanel, TutorialSkipButton, TutorialStepCounter, TutorialText,
};
use super::constants::*;
use super::definitions::{HighlightTarget, PanelAnchor, TutorialId};
use super::resources::{ActiveTutorial, TutorialProgress};

// ---------------------------------------------------------------------------
// Trigger systems
// ---------------------------------------------------------------------------

/// Starts a tutorial if it hasn't been completed and tutorials are enabled.
fn try_start_tutorial(
    commands: &mut Commands,
    tutorial: TutorialId,
    progress: &TutorialProgress,
    config: &GameConfig,
    active: Option<&ActiveTutorial>,
    paused_gameplay: bool,
) -> bool {
    if !config.tutorials_enabled {
        return false;
    }
    if progress.is_completed(&tutorial) {
        return false;
    }
    if active.is_some() {
        return false;
    }

    commands.insert_resource(ActiveTutorial {
        tutorial,
        step: 0,
        paused_gameplay,
    });
    true
}

pub(super) fn trigger_wizard_tower_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::WizardTowerIntro,
        &progress,
        &config,
        active.as_deref(),
        false,
    );
}

/// Trigger Time Travel tutorial when entering WizardTower with time travel unlocked.
pub(super) fn trigger_time_travel_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
) {
    // Only show if time travel is available (beaten at least level 1)
    if config.highest_level_achieved <= 1 {
        return;
    }
    try_start_tutorial(
        &mut commands,
        TutorialId::TimeTravelIntro,
        &progress,
        &config,
        active.as_deref(),
        false,
    );
}

pub(super) fn trigger_study_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::StudyIntro,
        &progress,
        &config,
        active.as_deref(),
        false,
    );
}

pub(super) fn trigger_in_game_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if try_start_tutorial(
        &mut commands,
        TutorialId::InGameIntro,
        &progress,
        &config,
        active.as_deref(),
        true,
    ) {
        next_in_game_state.set(InGameState::Tutorial);
    }
}

pub(super) fn trigger_spell_book_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::SpellBookIntro,
        &progress,
        &config,
        active.as_deref(),
        false,
    );
}

pub(super) fn trigger_cauldron_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::CauldronIntro,
        &progress,
        &config,
        active.as_deref(),
        false,
    );
}

// ---------------------------------------------------------------------------
// Overlay spawn/despawn
// ---------------------------------------------------------------------------

/// Returns the flexbox alignment values for a given panel anchor.
fn anchor_to_alignment(anchor: PanelAnchor) -> (JustifyContent, AlignItems) {
    match anchor {
        PanelAnchor::Center => (JustifyContent::Center, AlignItems::Center),
        PanelAnchor::TopLeft => (JustifyContent::FlexStart, AlignItems::FlexStart),
        PanelAnchor::TopRight => (JustifyContent::FlexStart, AlignItems::FlexEnd),
        PanelAnchor::BottomLeft => (JustifyContent::FlexEnd, AlignItems::FlexStart),
        PanelAnchor::BottomRight => (JustifyContent::FlexEnd, AlignItems::FlexEnd),
        PanelAnchor::TopCenter => (JustifyContent::FlexStart, AlignItems::Center),
        PanelAnchor::BottomCenter => (JustifyContent::FlexEnd, AlignItems::Center),
        PanelAnchor::CenterLeft => (JustifyContent::Center, AlignItems::FlexStart),
        PanelAnchor::CenterRight => (JustifyContent::Center, AlignItems::FlexEnd),
    }
}

/// Spawns the tutorial overlay UI when ActiveTutorial is inserted.
pub(super) fn spawn_tutorial_overlay(
    mut commands: Commands,
    active: Res<ActiveTutorial>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
) {
    if !overlay_query.is_empty() {
        return;
    }

    let steps = active.tutorial.steps();
    let step = &steps[active.step];
    let total = steps.len();

    let next_text = if active.step + 1 >= total {
        "Got it"
    } else {
        "Next"
    };

    let (justify, align) = anchor_to_alignment(step.anchor);

    commands
        .spawn((
            TutorialOverlay,
            GlobalZIndex(TUTORIAL_Z_INDEX),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: justify,
                align_items: align,
                padding: UiRect::all(Val::Px(PANEL_MARGIN)),
                ..default()
            },
            BackgroundColor(OVERLAY_BG),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    TutorialPanel,
                    Node {
                        max_width: Val::Px(PANEL_MAX_WIDTH),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(PANEL_PADDING)),
                        border: UiRect::all(Val::Px(PANEL_BORDER_WIDTH)),
                        row_gap: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                    BorderColor::all(PANEL_BORDER),
                    BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        TutorialText,
                        Text::new(step.text),
                        TextFont::from_font_size(TEXT_FONT_SIZE),
                        TextColor(TEXT_COLOR),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            max_width: Val::Px(PANEL_MAX_WIDTH - PANEL_PADDING * 2.0),
                            ..default()
                        },
                    ));

                    panel.spawn((
                        TutorialStepCounter,
                        Text::new(format!("{} of {}", active.step + 1, total)),
                        TextFont::from_font_size(STEP_FONT_SIZE),
                        TextColor(MUTED_TEXT_COLOR),
                    ));

                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            spawn_button(
                                buttons,
                                "Skip Tutorial",
                                TutorialSkipButton,
                                &ButtonStyle {
                                    width: BUTTON_WIDTH,
                                    height: BUTTON_HEIGHT,
                                    border_width: BUTTON_BORDER_WIDTH,
                                    font_size: BUTTON_FONT_SIZE,
                                    background: SKIP_BUTTON_BG,
                                    border: SKIP_BUTTON_BORDER,
                                    text_color: TEXT_COLOR,
                                    text_shadow: true,
                                },
                            );

                            spawn_button(
                                buttons,
                                next_text,
                                TutorialNextButton,
                                &ButtonStyle {
                                    width: BUTTON_WIDTH,
                                    height: BUTTON_HEIGHT,
                                    border_width: BUTTON_BORDER_WIDTH,
                                    font_size: BUTTON_FONT_SIZE,
                                    background: NEXT_BUTTON_BG,
                                    border: NEXT_BUTTON_BORDER,
                                    text_color: TEXT_COLOR,
                                    text_shadow: true,
                                },
                            );
                        });
                });
        });
}

/// Updates the overlay's flexbox alignment when the step changes.
pub(super) fn position_tutorial_panel(
    active: Res<ActiveTutorial>,
    mut overlay_query: Query<&mut Node, With<TutorialOverlay>>,
) {
    if !active.is_changed() {
        return;
    }

    let Ok(mut overlay_node) = overlay_query.single_mut() else {
        return;
    };

    let steps = active.tutorial.steps();
    let (justify, align) = anchor_to_alignment(steps[active.step].anchor);
    overlay_node.justify_content = justify;
    overlay_node.align_items = align;
}

fn despawn_overlay(commands: &mut Commands, overlay_query: &Query<Entity, With<TutorialOverlay>>) {
    for entity in overlay_query.iter() {
        commands.entity(entity).try_despawn();
    }
}

// ---------------------------------------------------------------------------
// Highlight system
// ---------------------------------------------------------------------------

/// Applies golden glow border to the entity matching the current step's target.
pub(super) fn apply_highlight(
    mut commands: Commands,
    active: Res<ActiveTutorial>,
    mut highlightables: Query<
        (
            Entity,
            &TutorialHighlightable,
            Option<&BorderColor>,
            &mut Node,
        ),
        Without<OriginalBorder>,
    >,
) {
    let steps = active.tutorial.steps();
    let target = steps[active.step].target;

    if target == HighlightTarget::None {
        return;
    }

    for (entity, highlightable, border_color, mut node) in &mut highlightables {
        if highlightable.target == target {
            let original_color = border_color.map(|bc| bc.top).unwrap_or(Color::NONE);
            let original_width = node.border;

            node.border = UiRect::all(Val::Px(GLOW_BORDER_WIDTH));

            commands.entity(entity).insert((
                OriginalBorder {
                    color: original_color,
                    width: original_width,
                },
                GlowAnimation { elapsed: 0.0 },
                BorderColor::all(GLOW_COLOR),
            ));
        }
    }
}

fn remove_all_highlights(
    commands: &mut Commands,
    highlighted: &mut Query<(Entity, &OriginalBorder, &mut Node), With<GlowAnimation>>,
) {
    for (entity, original, mut node) in highlighted.iter_mut() {
        node.border = original.width;
        commands
            .entity(entity)
            .insert(BorderColor::all(original.color))
            .remove::<OriginalBorder>()
            .remove::<GlowAnimation>();
    }
}

/// Animates the glow effect with a sine wave oscillation.
pub(super) fn animate_glow(
    time: Res<Time>,
    mut glowing: Query<(&mut BorderColor, &mut GlowAnimation)>,
) {
    for (mut border_color, mut anim) in &mut glowing {
        anim.elapsed += time.delta_secs();
        let pulse = (anim.elapsed * GLOW_ANIMATION_SPEED).sin() * 0.2 + 0.8;
        let glow = Color::hsla(45.0, 1.0, 0.65, pulse);
        *border_color = BorderColor::all(glow);
    }
}

// ---------------------------------------------------------------------------
// Advance / Skip / Complete
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_next_button(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    next_buttons: Query<Entity, With<TutorialNextButton>>,
    active: Option<ResMut<ActiveTutorial>>,
    mut progress: ResMut<TutorialProgress>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    mut highlighted: Query<(Entity, &OriginalBorder, &mut Node), With<GlowAnimation>>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
) {
    let Some(mut active) = active else { return };

    for event in button_clicked.read() {
        let is_next = next_buttons.iter().any(|e| e == event.button);
        if !is_next {
            continue;
        }

        let steps = active.tutorial.steps();
        remove_all_highlights(&mut commands, &mut highlighted);

        if active.step + 1 >= steps.len() {
            despawn_overlay(&mut commands, &overlay_query);
            complete_tutorial(
                &mut commands,
                &active,
                &mut progress,
                &mut next_in_game_state,
            );
        } else {
            active.step += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_skip_button(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    skip_buttons: Query<Entity, With<TutorialSkipButton>>,
    active: Option<Res<ActiveTutorial>>,
    mut progress: ResMut<TutorialProgress>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    mut highlighted: Query<(Entity, &OriginalBorder, &mut Node), With<GlowAnimation>>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
) {
    let Some(active) = active else { return };

    for event in button_clicked.read() {
        let is_skip = skip_buttons.iter().any(|e| e == event.button);
        if !is_skip {
            continue;
        }

        remove_all_highlights(&mut commands, &mut highlighted);
        despawn_overlay(&mut commands, &overlay_query);
        complete_tutorial(
            &mut commands,
            &active,
            &mut progress,
            &mut next_in_game_state,
        );
    }
}

fn complete_tutorial(
    commands: &mut Commands,
    active: &ActiveTutorial,
    progress: &mut TutorialProgress,
    next_in_game_state: &mut Option<ResMut<NextState<InGameState>>>,
) {
    progress.mark_completed(&active.tutorial);
    save_tutorial_progress(progress);

    if active.paused_gameplay
        && let Some(next_state) = next_in_game_state
    {
        next_state.set(InGameState::Running);
    }

    commands.remove_resource::<ActiveTutorial>();
}

pub(super) fn cleanup_tutorial(
    mut commands: Commands,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    mut highlighted: Query<(Entity, &OriginalBorder, &mut Node), With<GlowAnimation>>,
) {
    remove_all_highlights(&mut commands, &mut highlighted);
    despawn_overlay(&mut commands, &overlay_query);
    commands.remove_resource::<ActiveTutorial>();
}

/// Updates tutorial text, step counter, and button label in-place when the step changes.
pub(super) fn update_tutorial_content(
    active: Res<ActiveTutorial>,
    mut text_query: Query<&mut Text, With<TutorialText>>,
    mut counter_query: Query<&mut Text, (With<TutorialStepCounter>, Without<TutorialText>)>,
    next_buttons: Query<Entity, With<TutorialNextButton>>,
    children_query: Query<&Children>,
    mut all_text: Query<&mut Text, (Without<TutorialText>, Without<TutorialStepCounter>)>,
) {
    if !active.is_changed() {
        return;
    }

    let steps = active.tutorial.steps();
    let step = &steps[active.step];
    let total = steps.len();

    if let Ok(mut text) = text_query.single_mut() {
        **text = step.text.to_string();
    }

    if let Ok(mut counter) = counter_query.single_mut() {
        **counter = format!("{} of {}", active.step + 1, total);
    }

    let next_label = if active.step + 1 >= total {
        "Got it"
    } else {
        "Next"
    };
    for button_entity in &next_buttons {
        if let Ok(children) = children_query.get(button_entity) {
            for child in children.iter() {
                if let Ok(mut text) = all_text.get_mut(child) {
                    **text = next_label.to_string();
                }
                // Walk grandchildren for shadow-wrapped buttons
                if let Ok(grandchildren) = children_query.get(child) {
                    for gc in grandchildren.iter() {
                        if let Ok(mut text) = all_text.get_mut(gc) {
                            **text = next_label.to_string();
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Save/Load helpers
// ---------------------------------------------------------------------------

pub(crate) fn load_tutorial_progress() -> TutorialProgress {
    let save_file = load_unified_save().unwrap_or_else(new_unified_save);
    TutorialProgress {
        completed: save_file.player.completed_tutorials.clone(),
    }
}

fn save_tutorial_progress(progress: &TutorialProgress) {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    save_file.player.completed_tutorials = progress.completed.clone();
    save_unified(&save_file);
}

pub(crate) fn reset_tutorial_progress() {
    let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
    save_file.player.completed_tutorials.clear();
    save_unified(&save_file);
}

// ---------------------------------------------------------------------------
// Entity tagging systems
// ---------------------------------------------------------------------------

/// Tags Wizard Tower UI entities with TutorialHighlightable.
pub(super) fn tag_wizard_tower_entities(
    mut commands: Commands,
    level_displays: Query<Entity, (With<LevelDisplay>, Without<TutorialHighlightable>)>,
    insight_displays: Query<Entity, (With<InsightDisplay>, Without<TutorialHighlightable>)>,
    wt_buttons: Query<(Entity, &WizardTowerButtonAction), Without<TutorialHighlightable>>,
    tt_containers: Query<Entity, (With<TimeTravelContainer>, Without<TutorialHighlightable>)>,
) {
    for entity in &level_displays {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::LevelDisplay,
        });
    }
    for entity in &insight_displays {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::InsightDisplay,
        });
    }
    for (entity, action) in &wt_buttons {
        let target = match action {
            WizardTowerButtonAction::StudySpells => HighlightTarget::StudySpellsButton,
            WizardTowerButtonAction::StartNextBattle => HighlightTarget::StartBattleButton,
            WizardTowerButtonAction::StartTimeTravel => HighlightTarget::TimeTravelButton,
            _ => continue,
        };
        commands
            .entity(entity)
            .insert(TutorialHighlightable { target });
    }
    for entity in &tt_containers {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::TimeTravelList,
        });
    }
}

/// Tags Study screen UI entities with TutorialHighlightable.
pub(super) fn tag_study_entities(
    mut commands: Commands,
    spell_graph_areas: Query<Entity, (With<SpellGraphArea>, Without<TutorialHighlightable>)>,
    detail_panels: Query<Entity, (With<StudyDetailPanel>, Without<TutorialHighlightable>)>,
    study_buttons: Query<(Entity, &StudyButtonAction), Without<TutorialHighlightable>>,
) {
    for entity in &spell_graph_areas {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::SpellGraphArea,
        });
    }
    for entity in &detail_panels {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::DetailPanel,
        });
    }
    for (entity, action) in &study_buttons {
        let target = match action {
            StudyButtonAction::Commit => HighlightTarget::CommitButton,
            _ => continue,
        };
        commands
            .entity(entity)
            .insert(TutorialHighlightable { target });
    }
}

/// Tags InGame HUD entities with TutorialHighlightable.
pub(super) fn tag_in_game_entities(
    mut commands: Commands,
    mana_bars: Query<Entity, (With<ManaBarFill>, Without<TutorialHighlightable>)>,
    king_health_bars: Query<Entity, (With<KingHealthBarFill>, Without<TutorialHighlightable>)>,
    wave_displays: Query<Entity, (With<WaveDisplay>, Without<TutorialHighlightable>)>,
    hud_buttons: Query<(Entity, &HudButtonAction), Without<TutorialHighlightable>>,
    action_bar_roots: Query<Entity, (With<ActionBarRoot>, Without<TutorialHighlightable>)>,
) {
    for entity in &mana_bars {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::ManaBar,
        });
    }
    for entity in &king_health_bars {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::KingHealthBar,
        });
    }
    for entity in &wave_displays {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::WaveDisplay,
        });
    }
    for (entity, action) in &hud_buttons {
        let target = match action {
            HudButtonAction::OpenSpellBook => HighlightTarget::SpellBookButton,
            HudButtonAction::OpenCauldronMenu => HighlightTarget::CauldronButton,
        };
        commands
            .entity(entity)
            .insert(TutorialHighlightable { target });
    }
    for entity in &action_bar_roots {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::ActionBar,
        });
    }
}

/// Tags Spell Book UI entities with TutorialHighlightable.
pub(super) fn tag_spell_book_entities(
    mut commands: Commands,
    spell_lists: Query<Entity, (With<ScrollableSpellList>, Without<TutorialHighlightable>)>,
    detail_names: Query<Entity, (With<DetailName>, Without<TutorialHighlightable>)>,
    hotkey_buttons: Query<Entity, (With<HotkeySlotButton>, Without<TutorialHighlightable>)>,
) {
    for entity in &spell_lists {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::SpellList,
        });
    }
    for entity in &detail_names {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::SpellDetail,
        });
    }
    if let Some(entity) = hotkey_buttons.iter().next() {
        commands.entity(entity).insert(TutorialHighlightable {
            target: HighlightTarget::HotkeySlots,
        });
    }
}

/// Tags Cauldron menu UI entities with TutorialHighlightable.
pub(super) fn tag_cauldron_entities(
    mut commands: Commands,
    cauldron_buttons: Query<(Entity, &CauldronMenuButtonAction), Without<TutorialHighlightable>>,
) {
    for (entity, action) in &cauldron_buttons {
        let target = match action {
            CauldronMenuButtonAction::StartBrew => HighlightTarget::BrewButton,
            _ => continue,
        };
        commands
            .entity(entity)
            .insert(TutorialHighlightable { target });
    }
}
