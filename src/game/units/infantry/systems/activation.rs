use crate::game::constants::*;
use bevy::prelude::*;

use super::super::components::*;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::components::{Corpse, Team};

/// Checks if any attacker is within activation range of any defender.
///
/// Once a single defender detects an enemy within DEFENDER_ACTIVATION_RANGE,
/// ALL defenders activate collectively via the DefendersActivated resource.
/// Activation persists for the entire game.
pub fn check_defender_activation(
    mut defenders_activated: ResMut<DefendersActivated>,
    retreat_state: Res<RetreatState>,
    defender_query: Query<(&Transform, &Team), (With<Infantry>, Without<Corpse>)>,
    attacker_query: Query<(&Transform, &Team), (Without<Corpse>, Without<StagingAttacker>)>,
) {
    // During retreat, defenders are force-deactivated — skip all activation logic
    if retreat_state.is_active() {
        return;
    }

    // If already activated, check whether any enemies remain on the battlefield.
    // If not, deactivate so defenders hold position until the next wave arrives.
    if defenders_activated.active {
        let enemies_exist = attacker_query
            .iter()
            .any(|(_, team)| *team == Team::Attackers || *team == Team::Undead);
        if !enemies_exist {
            defenders_activated.active = false;
            info!("Defenders deactivated — no enemies on battlefield");
        }
        return;
    }

    // Check if ANY attacker is within activation range of ANY defender
    // As soon as one defender "sees" an enemy, all defenders activate
    for (defender_transform, defender_team) in defender_query.iter() {
        // Only check defender infantry, not attacker infantry
        if *defender_team != Team::Defenders {
            continue;
        }

        for (attacker_transform, attacker_team) in attacker_query.iter() {
            // Only check against Attackers and Undead, not other Defenders
            if *attacker_team != Team::Attackers && *attacker_team != Team::Undead {
                continue;
            }

            let dx = defender_transform.translation.x - attacker_transform.translation.x;
            let dz = defender_transform.translation.z - attacker_transform.translation.z;
            let distance = (dx * dx + dz * dz).sqrt();

            if distance <= DEFENDER_ACTIVATION_RANGE {
                // Activate ALL defenders collectively
                defenders_activated.active = true;
                info!(
                    "Defenders activated! Enemy within {} units - all defenders now active",
                    DEFENDER_ACTIVATION_RANGE
                );
                return;
            }
        }
    }
}

/// Checks if the King has moved too close to the wizard's spell range limit.
///
/// When the King reaches 90% of the wizard's ground-projected spell range,
/// a retreat is triggered: defenders deactivate (stop targeting and attacking)
/// and fall back to spawn via the flow field. Retreat can only happen once per level.
#[allow(clippy::too_many_arguments)]
pub fn check_retreat_trigger(
    mut retreat_state: ResMut<RetreatState>,
    mut defenders_activated: ResMut<DefendersActivated>,
    time: Res<Time>,
    king_query: Query<
        (&Transform, &Team),
        (
            With<crate::game::units::king::components::King>,
            Without<Corpse>,
        ),
    >,
    wizard_query: Query<&crate::game::units::wizard::components::Wizard>,
    mut commands: Commands,
    defender_units: Query<(Entity, &Team), Without<Corpse>>,
    retreating_units: Query<Entity, With<Retreating>>,
    mut retreat_events: MessageWriter<crate::game::messages::RetreatMessage>,
) {
    // Tick retreat timer if active
    if retreat_state.is_active() {
        retreat_state.retreat_timer -= time.delta_secs();
        if retreat_state.retreat_timer <= 0.0 {
            retreat_state.retreat_timer = 0.0;

            // Remove Retreating component only from entities that have it
            for entity in &retreating_units {
                commands.entity(entity).remove::<Retreating>();
            }
        }
        return;
    }

    // No retreats remaining — don't trigger
    if !retreat_state.can_retreat() {
        return;
    }

    // Only check when defenders are actively fighting
    if !defenders_activated.active {
        return;
    }

    // Find the Defender King
    let Some((king_transform, _)) = king_query
        .iter()
        .find(|(_, team)| **team == Team::Defenders)
    else {
        return;
    };

    // Get wizard's spell range
    let Ok(wizard) = wizard_query.single() else {
        return;
    };

    // Calculate XZ distance from King to Wizard
    let king_xz = Vec2::new(king_transform.translation.x, king_transform.translation.z);
    let wizard_xz = Vec2::new(WIZARD_POSITION.x, WIZARD_POSITION.z);
    let distance = king_xz.distance(wizard_xz);

    let ground_range = crate::game::units::wizard::spells::utils::ground_projected_range(
        wizard.spell_range,
        WIZARD_POSITION.y,
    );
    let trigger_distance = ground_range * RETREAT_TRIGGER_DISTANCE_PERCENT;

    if distance >= trigger_distance {
        // Trigger retreat
        retreat_state.retreats_remaining = retreat_state.retreats_remaining.saturating_sub(1);
        retreat_state.retreat_timer = RETREAT_DURATION_SECS;

        // Deactivate defenders — triggers flow field rally to spawn
        defenders_activated.active = false;

        // Add Retreating marker to all defender units (suppresses attacks)
        for (entity, team) in &defender_units {
            if *team == Team::Defenders {
                commands.entity(entity).insert(Retreating);
            }
        }

        // Send message for UI popup
        retreat_events.write(crate::game::messages::RetreatMessage);
    }
}
