//! Tutorial triggers and lifecycle.

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::config::save_data::{load_unified_save, new_unified_save, save_unified};
use crate::game::input::messages::MouseClicked;
use crate::state::InGameState;
use crate::ui::components::ButtonStyle;
use crate::ui::systems::spawn_button;

use super::components::{
    GlowAnimation, HighlightOverlay, HighlightOverlayGlow, TutorialHighlightable,
    TutorialNextButton, TutorialOverlay, TutorialPanel, TutorialSkipButton, TutorialStepCounter,
    TutorialText,
};
use super::constants::*;
use super::definitions::{HighlightTarget, PanelAnchor, TutorialId};
use super::resources::{ActiveTutorial, TutorialProgress};
use super::text_glyphs::spawn_segmented_text;
use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::ui::gamepad_glyphs::{CurrentControllerGlyphStyle, GamepadGlyphFonts};

// ---------------------------------------------------------------------------
// Trigger systems
// ---------------------------------------------------------------------------

/// Starts a tutorial if it hasn't been completed and tutorials are enabled.
/// If another tutorial is already active, appends this one to
/// `PendingTutorials` so it plays as soon as the current one finishes.
/// `paused_gameplay` is honored only for the immediately-started case;
/// queued tutorials run un-paused (they're already deep enough into a
/// session that pausing again would be jarring).
fn try_start_tutorial(
    commands: &mut Commands,
    tutorial: TutorialId,
    progress: &TutorialProgress,
    config: &GameConfig,
    active: Option<&ActiveTutorial>,
    pending: Option<&super::resources::PendingTutorials>,
    paused_gameplay: bool,
) -> bool {
    if !config.tutorials_enabled {
        return false;
    }
    if progress.is_completed(&tutorial) {
        return false;
    }
    if let Some(active) = active {
        if active.tutorial == tutorial {
            return false;
        }
        let _ = pending;
        commands.queue(move |world: &mut bevy::prelude::World| {
            let mut q = world.get_resource_or_insert_with::<super::resources::PendingTutorials>(
                Default::default,
            );
            if !q.queue.contains(&tutorial) {
                q.queue.push_back(tutorial);
            }
        });
        return false;
    }

    commands.insert_resource(ActiveTutorial {
        tutorial,
        step: 0,
        paused_gameplay,
    });
    true
}

// Old `trigger_wizard_tower_tutorial`, `trigger_time_travel_tutorial`, and
// `trigger_study_tutorial` were removed when the tower walkthrough was split
// into per-tab tutorials. Their `TutorialId` variants stay in the enum so
// existing saves that completed those overlays don't get them re-shown under
// any future revival.

pub(super) fn trigger_in_game_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::resources::PendingTutorials>>,
    mut next_in_game_state: ResMut<NextState<InGameState>>,
) {
    if try_start_tutorial(
        &mut commands,
        TutorialId::InGameIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
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
    pending: Option<Res<super::resources::PendingTutorials>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::SpellBookIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

pub(super) fn trigger_cauldron_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::resources::PendingTutorials>>,
) {
    try_start_tutorial(
        &mut commands,
        TutorialId::CauldronIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Pops the next tutorial off `PendingTutorials` and inserts it as the new
/// `ActiveTutorial` whenever no tutorial is currently active. Skips entries
/// the player has already completed (e.g. via Skip while another was queued).
pub(super) fn drain_pending_tutorials(
    mut commands: Commands,
    active: Option<Res<ActiveTutorial>>,
    mut pending: Option<ResMut<super::resources::PendingTutorials>>,
    progress: Res<TutorialProgress>,
) {
    if active.is_some() {
        return;
    }
    let Some(pending) = pending.as_deref_mut() else {
        return;
    };
    while let Some(next) = pending.queue.pop_front() {
        if progress.is_completed(&next) {
            continue;
        }
        commands.insert_resource(ActiveTutorial {
            tutorial: next,
            step: 0,
            paused_gameplay: false,
        });
        return;
    }
}

/// Mouse + keyboard menu navigation primer. Mirrors the controller variant —
/// fires once per save when the player enters the Wizard Tower while
/// mouse/keyboard is the active input device.
pub(super) fn trigger_kbm_menus_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::resources::PendingTutorials>>,
    active_input: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
) {
    if active_input.is_gamepad() {
        return;
    }
    try_start_tutorial(
        &mut commands,
        TutorialId::KbmMenusIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Dismisses any active tutorial whose `modality` doesn't match the current
/// input device, so a controller-only or KBM-only walkthrough disappears the
/// moment the player switches inputs. Does NOT mark the tutorial complete —
/// the matching counterpart (or the same tutorial when the player switches
/// back) is still allowed to fire.
pub(super) fn enforce_tutorial_modality(
    mut commands: Commands,
    active: Option<Res<ActiveTutorial>>,
    active_input: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    mut highlighted: Query<(Entity, &HighlightOverlay)>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
) {
    let Some(active) = active else { return };
    use super::definitions::TutorialModality;
    let modality = active.tutorial.modality();
    let mismatch = match modality {
        TutorialModality::Any => false,
        TutorialModality::MouseKeyboard => active_input.is_gamepad(),
    };
    if !mismatch {
        return;
    }

    remove_all_highlights(&mut commands, &mut highlighted);
    despawn_overlay(&mut commands, &overlay_query);
    if active.paused_gameplay
        && let Some(next_state) = next_in_game_state.as_mut()
    {
        next_state.set(InGameState::Running);
    }
    commands.remove_resource::<ActiveTutorial>();
}

/// Study spell-selected walkthrough. Fires when a Study spell becomes the
/// selected one (detail panel populated), so the +/− and talent walkthrough
/// only shows after the relevant UI is on screen.
pub(super) fn trigger_study_spell_selected_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::resources::PendingTutorials>>,
    selected: Res<crate::ui::wizard_tower::SelectedStudySpell>,
) {
    let Some(spell) = selected.0 else {
        return;
    };
    // Only meaningful for a *locked* spell — the +/− and talents don't apply
    // to spells that come unlocked by default (e.g. Magic Missile).
    if crate::ui::wizard_tower::is_spell_unlocked(spell) {
        return;
    }
    try_start_tutorial(
        &mut commands,
        TutorialId::StudySpellSelectedIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Wizard-select walkthrough. Fires the first time the player opens the
/// "Switch Wizard" panel (RightPanelView::WizardSelect).
pub(super) fn trigger_wizard_select_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::resources::PendingTutorials>>,
    view: Res<crate::ui::wizard_tower::RightPanelView>,
) {
    if *view != crate::ui::wizard_tower::RightPanelView::WizardSelect {
        return;
    }
    try_start_tutorial(
        &mut commands,
        TutorialId::WizardSelectIntro,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
        false,
    );
}

/// Per-tab tutorial trigger. Watches the `WizardTowerTab` resource and fires
/// the matching walkthrough the first time the player opens each tab. Runs
/// only when no tutorial is currently active so it queues nicely behind any
/// in-flight overlay (e.g. controller menus primer).
pub(super) fn trigger_wizard_tower_tab_tutorial(
    mut commands: Commands,
    progress: Res<TutorialProgress>,
    config: Res<GameConfig>,
    active: Option<Res<ActiveTutorial>>,
    pending: Option<Res<super::resources::PendingTutorials>>,
    tab: Res<crate::ui::wizard_tower::WizardTowerTab>,
) {
    use crate::ui::wizard_tower::WizardTowerTab;
    let tutorial = match *tab {
        WizardTowerTab::Roguelite => TutorialId::RogueliteTabIntro,
        WizardTowerTab::Endless => TutorialId::EndlessTabIntro,
        WizardTowerTab::Study => TutorialId::StudyTabIntro,
        WizardTowerTab::Multiplayer | WizardTowerTab::Vs => return,
    };
    try_start_tutorial(
        &mut commands,
        tutorial,
        &progress,
        &config,
        active.as_deref(),
        pending.as_deref(),
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
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_tutorial_overlay(
    mut commands: Commands,
    active: Res<ActiveTutorial>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    active_input: Res<ActiveInputDevice>,
    glyph_style: Res<CurrentControllerGlyphStyle>,
    glyph_fonts: Option<Res<GamepadGlyphFonts>>,
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
                // Column direction so `anchor_to_alignment`'s
                // (justify, align) pairs match their semantic names: justify
                // controls vertical (Top/Bottom/Center) and align controls
                // horizontal (Left/Right/Center).
                flex_direction: FlexDirection::Column,
                justify_content: justify,
                align_items: align,
                padding: UiRect::all(Val::Px(PANEL_MARGIN)),
                ..default()
            },
            BackgroundColor(OVERLAY_BG),
            crate::ui::focus::ModalOverlay,
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
                        border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                    BorderColor::all(PANEL_BORDER),
                ))
                .with_children(|panel| {
                    let display_text = if active_input.is_gamepad() {
                        step.text
                    } else {
                        step.text_kbm.unwrap_or(step.text)
                    };
                    let text_id = spawn_segmented_text(
                        panel,
                        display_text,
                        TEXT_FONT_SIZE,
                        TEXT_COLOR,
                        PANEL_MAX_WIDTH - PANEL_PADDING * 2.0,
                        active_input.is_gamepad(),
                        glyph_style.0,
                        glyph_fonts.as_deref(),
                    );
                    panel.commands().entity(text_id).insert(TutorialText);

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

/// Spawns a golden-glow overlay child centered on the entity matching the
/// current step's target. The overlay sits on top via absolute positioning
/// extended slightly outside the parent so it doesn't change the parent's
/// own border or content layout.
pub(super) fn apply_highlight(
    mut commands: Commands,
    active: Res<ActiveTutorial>,
    highlightables: Query<(Entity, &TutorialHighlightable), Without<HighlightOverlay>>,
) {
    let steps = active.tutorial.steps();
    let target = steps[active.step].target;

    if target == HighlightTarget::None {
        return;
    }

    for (entity, highlightable) in &highlightables {
        if highlightable.target != target {
            continue;
        }
        // Spawn the overlay child slightly outside the parent so the gold
        // border sits on top and doesn't shrink the parent's content.
        let mut child_id = None;
        commands.entity(entity).with_children(|p| {
            let id = p
                .spawn((
                    HighlightOverlayGlow,
                    GlowAnimation { elapsed: 0.0 },
                    // Sits ON the parent's bounds (inset 0) and grows
                    // INWARD via animated padding so it never gets clipped
                    // by an ancestor's overflow or bounds.
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        border: UiRect::all(Val::Px(GLOW_BORDER_WIDTH)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BorderColor::all(GLOW_COLOR),
                    // Fully transparent fill so we don't tint the parent.
                    BackgroundColor(Color::NONE),
                    GlobalZIndex(super::constants::TUTORIAL_Z_INDEX - 1),
                ))
                .id();
            child_id = Some(id);
        });
        if let Some(child) = child_id {
            commands.entity(entity).insert(HighlightOverlay { child });
        }
    }
}

fn remove_all_highlights(
    commands: &mut Commands,
    highlighted: &mut Query<(Entity, &HighlightOverlay)>,
) {
    for (entity, overlay) in highlighted.iter() {
        commands.entity(overlay.child).try_despawn();
        commands.entity(entity).try_remove::<HighlightOverlay>();
    }
}

/// Animates the glow effect with a sine wave oscillation.
pub(super) fn animate_glow(
    time: Res<Time>,
    mut glowing: Query<(&mut BorderColor, &mut GlowAnimation), With<HighlightOverlayGlow>>,
) {
    for (mut border_color, mut anim) in &mut glowing {
        anim.elapsed += time.delta_secs();
        let phase = (anim.elapsed * GLOW_ANIMATION_SPEED).sin();
        // Brightness pulses between ~0.55 and 1.0, lightness between
        // ~0.50 and 0.70 so the gold visibly brightens and dims. Size is
        // intentionally fixed.
        let alpha = phase * 0.225 + 0.775;
        let lightness = phase * 0.10 + 0.60;
        *border_color = BorderColor::all(Color::hsla(GLOW_HUE, 0.95, lightness, alpha));
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
    mut highlighted: Query<(Entity, &HighlightOverlay)>,
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
    mut highlighted: Query<(Entity, &HighlightOverlay)>,
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
    mut highlighted: Query<(Entity, &HighlightOverlay)>,
    pending: Option<ResMut<super::resources::PendingTutorials>>,
) {
    remove_all_highlights(&mut commands, &mut highlighted);
    despawn_overlay(&mut commands, &overlay_query);
    commands.remove_resource::<ActiveTutorial>();
    if let Some(mut pending) = pending {
        pending.queue.clear();
    }
}

/// Updates tutorial text, step counter, and button label when the step or
/// input device changes. Rebuilds the segmented text entity so embedded
/// `{token}` glyphs swap between controller-font icons and keyboard labels.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_tutorial_content(
    mut commands: Commands,
    active: Res<ActiveTutorial>,
    active_input: Res<ActiveInputDevice>,
    glyph_style: Res<CurrentControllerGlyphStyle>,
    glyph_fonts: Option<Res<GamepadGlyphFonts>>,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    text_query: Query<Entity, With<TutorialText>>,
    mut counter_query: Query<&mut Text, With<TutorialStepCounter>>,
    next_buttons: Query<Entity, With<TutorialNextButton>>,
    children_query: Query<&Children>,
    mut all_text: Query<&mut Text, (Without<TutorialStepCounter>,)>,
) {
    if !active.is_changed() && !active_input.is_changed() && !glyph_style.is_changed() {
        return;
    }
    // Skip the rebuild entirely if the overlay is gone (despawn queued by
    // handle_next_button / cleanup / modality enforcer). Any commands we
    // queued against the text entity would panic when applied.
    if overlay_query.is_empty() {
        return;
    }

    let steps = active.tutorial.steps();
    let step = &steps[active.step];
    let total = steps.len();

    if let Ok(text_entity) = text_query.single() {
        if let Ok(children) = children_query.get(text_entity) {
            for child in children.iter() {
                commands.entity(child).try_despawn();
            }
        }
        // try_insert: the overlay may have been despawned earlier in this
        // tick (e.g. by the modality enforcer or a state cleanup); without
        // try_*, the deferred command panics when applied to the gone entity.
        commands.entity(text_entity).try_insert(Text::default());
        let gp = active_input.is_gamepad();
        let text = if gp {
            step.text
        } else {
            step.text_kbm.unwrap_or(step.text)
        };
        let style = glyph_style.0;
        let fonts = glyph_fonts.as_deref().cloned();
        commands.entity(text_entity).with_children(|p| {
            super::text_glyphs::spawn_text_spans(
                p,
                text,
                TEXT_FONT_SIZE,
                TEXT_COLOR,
                gp,
                style,
                fonts.as_ref(),
            );
        });
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
