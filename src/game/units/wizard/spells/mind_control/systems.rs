use std::cmp::Ordering;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::components::*;
use super::constants;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, MindControlled, Team};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;

/// Tracked highlight target — stored as system-local state so we have a single
/// source of truth that doesn't depend on deferred command timing.
/// Stores the entity, its original material handle (to restore), and the cloned
/// tinted handle (to remove from the asset store on cleanup).
#[derive(Default)]
pub(super) struct HighlightState {
    target: Option<HighlightedUnit>,
}

struct HighlightedUnit {
    entity: Entity,
    /// The entity's original shared material handle — restored on un-highlight.
    original_handle: Handle<StandardMaterial>,
    /// The cloned tinted material we created — removed from assets on cleanup.
    tinted_handle: Handle<StandardMaterial>,
}

/// Local wizard mind control casting — hold-to-cast with dynamic target highlighting.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_mind_control_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
            Option<&MindControlCooldown>,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemies_query: Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (Without<Corpse>, Without<MindControlled>),
    >,
    existing_controlled: Query<&MindControlled>,
    mut highlight: Local<HighlightState>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell, cooldown)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::MindControl {
        return;
    }

    let controlled_count = existing_controlled.iter().count() as u32;

    // On release → cancel cast and remove highlight
    if input.just_released {
        casting_state.cancel();
        clear_highlight(&mut commands, &mut materials, &mut highlight);
        return;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && mana.can_afford(constants::MANA_COST)
                && !cooldown.is_some_and(|cd| cd.remaining > 0.0)
                && controlled_count < constants::MAX_CONTROLLED
            {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            // Find nearest enemy to cursor each frame and update highlight
            let nearest = find_nearest_enemy(&enemies_query, cursor_pos);
            update_highlight(&mut commands, &mut materials, &enemies_query, &mut highlight, nearest);

            if casting_state.is_complete(primed_spell.cast_time) {
                // Apply mind control to the currently highlighted target
                if let Some(ref highlighted) = highlight.target
                    && mana.consume(constants::MANA_COST)
                {
                    commands.entity(highlighted.entity).insert(MindControlled {
                        time_elapsed: 0.0,
                        wear_off_duration: constants::EFFECT_WEAR_OFF_DURATION,
                        original_spawn_pos: None,
                    });

                    commands
                        .entity(wizard_entity)
                        .insert(MindControlCooldown { remaining: constants::COOLDOWN });

                    mouse_state.left_consumed = true;
                }

                clear_highlight(&mut commands, &mut materials, &mut highlight);
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
            clear_highlight(&mut commands, &mut materials, &mut highlight);
        }
    }
}

/// Finds the nearest enemy to the cursor within TARGET_SEARCH_RADIUS.
fn find_nearest_enemy(
    enemies_query: &Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (Without<Corpse>, Without<MindControlled>),
    >,
    cursor_pos: Option<Vec3>,
) -> Option<Entity> {
    cursor_pos.and_then(|pos| {
        enemies_query
            .iter()
            .filter(|(_, _, team, _)| **team == Team::Attackers || **team == Team::Undead)
            .filter_map(|(entity, transform, _, _)| {
                let dist = transform.translation.distance(pos);
                if dist <= constants::TARGET_SEARCH_RADIUS {
                    Some((entity, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
            .map(|(entity, _)| entity)
    })
}

/// Updates the highlight to point at a new target (or clears if None).
/// Clones the material for the highlighted entity so only it gets tinted.
fn update_highlight(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (Without<Corpse>, Without<MindControlled>),
    >,
    highlight: &mut HighlightState,
    nearest: Option<Entity>,
) {
    let current_entity = highlight.target.as_ref().map(|h| h.entity);

    // If the target hasn't changed, nothing to do
    if nearest == current_entity {
        return;
    }

    // Restore old target's original material
    clear_highlight(commands, materials, highlight);

    // Clone + tint new target's material
    if let Some(target_entity) = nearest
        && let Ok((_, _, _, material_handle)) = enemies_query.get(target_entity)
    {
        let original_handle = material_handle.0.clone();
        if let Some(original_mat) = materials.get(&original_handle) {
            let mut tinted_mat = original_mat.clone();
            let base_linear = tinted_mat.base_color.to_linear();
            let highlight_linear = constants::HIGHLIGHT_COLOR.to_linear();
            let blended = base_linear.mix(&highlight_linear, 0.6);
            tinted_mat.base_color = Color::from(blended);
            let tinted_handle = materials.add(tinted_mat);

            // Swap the entity's material to the tinted clone
            commands
                .entity(target_entity)
                .insert(MeshMaterial3d(tinted_handle.clone()));

            highlight.target = Some(HighlightedUnit {
                entity: target_entity,
                original_handle,
                tinted_handle,
            });
        }
    }
}

/// Restores the highlighted entity's original material and cleans up the tinted clone.
fn clear_highlight(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    highlight: &mut HighlightState,
) {
    if let Some(highlighted) = highlight.target.take() {
        // Restore the entity's original shared material
        commands
            .entity(highlighted.entity)
            .insert(MeshMaterial3d(highlighted.original_handle));

        // Remove the tinted clone from assets
        materials.remove(&highlighted.tinted_handle);
    }
}

/// Ticks the mind control cooldown timer.
pub(super) fn tick_mind_control_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut MindControlCooldown)>,
) {
    let delta = time.delta_secs();

    for (entity, mut cd) in &mut query {
        cd.remaining -= delta;
        if cd.remaining <= 0.0 {
            commands.entity(entity).remove::<MindControlCooldown>();
        }
    }
}
