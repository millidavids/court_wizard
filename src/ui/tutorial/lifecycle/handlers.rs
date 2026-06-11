use bevy::prelude::*;

use crate::game::input::gamepad::resources::ActiveInputDevice;
use crate::game::input::messages::MouseClicked;
use crate::state::InGameState;
use crate::ui::gamepad_glyphs::{CurrentControllerGlyphStyle, GamepadGlyphFonts};

use super::super::components::{
    HighlightOverlay, TutorialNextButton, TutorialOverlay, TutorialSkipButton, TutorialStepCounter,
    TutorialText,
};
use super::super::constants::TEXT_COLOR;
use super::super::constants::TEXT_FONT_SIZE;
use super::super::resources::{ActiveTutorial, TutorialProgress};
use super::highlight::remove_all_highlights;
use super::overlay::despawn_overlay;
use super::persistence::save_tutorial_progress;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_next_button(
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
pub(crate) fn handle_skip_button(
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

pub(crate) fn cleanup_tutorial(
    mut commands: Commands,
    overlay_query: Query<Entity, With<TutorialOverlay>>,
    mut highlighted: Query<(Entity, &HighlightOverlay)>,
    pending: Option<ResMut<super::super::resources::PendingTutorials>>,
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
pub(crate) fn update_tutorial_content(
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
            super::super::text_glyphs::spawn_text_spans(
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
