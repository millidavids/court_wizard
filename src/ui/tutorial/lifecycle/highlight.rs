use bevy::prelude::*;

use super::super::components::{
    GlowAnimation, HighlightOverlay, HighlightOverlayGlow, TutorialHighlightable,
};
use super::super::constants::{
    GLOW_ANIMATION_SPEED, GLOW_BORDER_WIDTH, GLOW_COLOR, GLOW_HUE, TUTORIAL_Z_INDEX,
};
use super::super::definitions::HighlightTarget;
use super::super::resources::ActiveTutorial;

/// Spawns a golden-glow overlay child centered on the entity matching the
/// current step's target. The overlay sits on top via absolute positioning
/// extended slightly outside the parent so it doesn't change the parent's
/// own border or content layout.
pub(crate) fn apply_highlight(
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
                    GlobalZIndex(TUTORIAL_Z_INDEX - 1),
                ))
                .id();
            child_id = Some(id);
        });
        if let Some(child) = child_id {
            commands.entity(entity).insert(HighlightOverlay { child });
        }
    }
}

pub(crate) fn remove_all_highlights(
    commands: &mut Commands,
    highlighted: &mut Query<(Entity, &HighlightOverlay)>,
) {
    for (entity, overlay) in highlighted.iter() {
        commands.entity(overlay.child).try_despawn();
        commands.entity(entity).try_remove::<HighlightOverlay>();
    }
}

/// Animates the glow effect with a sine wave oscillation.
pub(crate) fn animate_glow(
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
