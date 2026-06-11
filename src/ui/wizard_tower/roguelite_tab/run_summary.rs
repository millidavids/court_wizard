use bevy::prelude::*;

use crate::game::game_mode::components::RogueliteModifiers;

use super::components::{PendingToggles, RunSummaryContent};
use super::panel_no_run::spawn_summary_items;

/// Rebuilds the left panel summary text when modifiers or pending toggles change.
pub(crate) fn update_run_summary(
    mut commands: Commands,
    modifiers: Option<Res<RogueliteModifiers>>,
    pending_toggles: Option<Res<PendingToggles>>,
    summary_query: Query<Entity, With<RunSummaryContent>>,
    children_query: Query<&Children>,
) {
    let (Some(modifiers), Some(pending_toggles)) = (modifiers, pending_toggles) else {
        return;
    };
    if !modifiers.is_changed() && !pending_toggles.is_changed() {
        return;
    }

    let Ok(summary_entity) = summary_query.single() else {
        return;
    };

    // Despawn all existing children
    if let Ok(children) = children_query.get(summary_entity) {
        for child in children.iter() {
            commands.entity(child).try_despawn();
        }
    }

    // Re-spawn fresh text nodes
    commands.entity(summary_entity).with_children(|summary| {
        spawn_summary_items(summary, &modifiers, &pending_toggles);
    });
}
