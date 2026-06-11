use bevy::prelude::*;

use super::super::components::{
    DimensionalRift, DisorientingHaste, TeleportCaster, TeleportDestinationCircle,
    TeleportSourceCircle, TeleportTalentParams,
};
use super::super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::Stunned;
use crate::game::units::wizard::components::{CastingState, LocalWizard, PrimedSpell, Spell};

/// Applies post-teleport talent effects (stun, haste, dimensional rift).
/// Effects are applied directly to the teleported entities rather than querying by position,
/// since teleport uses deferred commands (transforms haven't been applied yet).
/// Returns the rift entity if Dimensional Rift was spawned.
pub(crate) fn apply_post_teleport_effects(
    commands: &mut Commands,
    talent_params: &TeleportTalentParams,
    source_pos: Vec3,
    dest_pos: Vec3,
    teleported_entities: &[Entity],
) -> Option<Entity> {
    // Disorienting Arrival: stun AND haste all teleported units
    if talent_params.disorienting_arrival {
        for &entity in teleported_entities {
            commands
                .entity(entity)
                .insert(Stunned::new(DISORIENTING_STUN_DURATION));
            commands.entity(entity).insert(DisorientingHaste::new(
                DISORIENTING_ATTACK_SPEED,
                DISORIENTING_ATTACK_SPEED_DURATION,
            ));
        }
    }

    // Dimensional Rift: spawn persistent two-way portal
    if talent_params.dimensional_rift {
        let rift_entity = commands
            .spawn((
                DimensionalRift {
                    source_pos,
                    dest_pos,
                    walk_radius: DIMENSIONAL_RIFT_WALK_RADIUS,
                    time_remaining: DIMENSIONAL_RIFT_DURATION,
                    two_way: talent_params.swap_mode,
                },
                OnGameplayScreen,
            ))
            .id();
        return Some(rift_entity);
    }

    None
}

/// Updates pulse animations for both destination and source circles.
pub fn update_circle_animations(
    time: Res<Time>,
    mut destination_query: Query<
        (&mut Transform, &mut TeleportDestinationCircle),
        Without<TeleportSourceCircle>,
    >,
    mut source_query: Query<(&mut Transform, &mut TeleportSourceCircle)>,
) {
    // Update destination circles
    for (mut transform, mut indicator) in &mut destination_query {
        indicator.time_alive += time.delta_secs();

        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= indicator.base_radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(indicator.base_radius * pulse);
        }
    }

    // Update source circles
    for (mut transform, mut indicator) in &mut source_query {
        indicator.time_alive += time.delta_secs();

        let radius = CIRCLE_RADIUS * indicator.empowerment;
        // Only apply pulse animation after growth is mostly complete
        if transform.scale.x >= radius * PULSE_THRESHOLD {
            let pulse = indicator.pulse_scale();
            transform.scale = Vec3::splat(radius * pulse);
        }
    }
}

/// Cleans up teleport circles and caster state when the player switches away
/// from the Teleport spell while a teleport is in progress.
pub fn cleanup_teleport_on_spell_switch(
    mut commands: Commands,
    mut wizard_query: Query<
        (&PrimedSpell, &mut CastingState),
        (With<LocalWizard>, Changed<PrimedSpell>),
    >,
    mut caster_query: Query<&mut TeleportCaster, With<LocalWizard>>,
) {
    let Ok((primed_spell, mut casting_state)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell == Spell::Teleport {
        return;
    }

    let Ok(mut caster) = caster_query.single_mut() else {
        return;
    };

    // Only clean up if there's actually an in-progress teleport (circles exist).
    // TeleportCaster persists on the wizard, so we must check for active state.
    let has_circles = caster.destination_circle.is_some() || caster.source_circle.is_some();
    if !has_circles {
        return;
    }

    if let Some(dest_entity) = caster.destination_circle {
        commands.entity(dest_entity).try_despawn();
    }
    if let Some(source_entity) = caster.source_circle {
        commands.entity(source_entity).try_despawn();
    }
    caster.destination_circle = None;
    caster.destination_position = None;
    caster.source_circle = None;
    caster.lingering_gate_active = false;
    if !matches!(*casting_state, CastingState::Resting) {
        casting_state.cancel();
    }
}
