use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::components::{Acceleration, Velocity};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::{Corpse, FlockingVelocity, MovementSpeed, TargetingVelocity};

/// Dark Mage movement: always follows flow field pathfinding.
/// During Approaching, transitions to Idle once reaching the battlefield.
/// During Telegraphing/Casting, stands still.
pub fn dark_mage_movement(
    time: Res<Time>,
    mut bosses: Query<
        (
            &Transform,
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &mut DarkMageState,
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
) {
    for (
        transform,
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        mut state,
    ) in &mut bosses
    {
        // Freeze movement during telegraphing and casting
        if matches!(
            *state,
            DarkMageState::Telegraphing { .. } | DarkMageState::Casting { .. }
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            acceleration.x = 0.0;
            acceleration.z = 0.0;
            continue;
        }

        // Follow flow field at all times (approach and idle)
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            false,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        // Transition from approaching to idle once on the battlefield
        if matches!(*state, DarkMageState::Approaching)
            && transform.translation.x <= DARK_MAGE_APPROACH_TARGET_X
        {
            *state = DarkMageState::Idle;
        }
    }
}

/// Ticks spell cooldowns and enqueues spells that come off cooldown.
pub fn dark_mage_spell_queue(
    time: Res<Time>,
    mut bosses: Query<
        (
            &mut DarkMageSpellCooldowns,
            &mut DarkMageSpellQueue,
            &DarkMageState,
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
) {
    let delta = time.delta_secs();

    for (mut cooldowns, mut queue, state) in &mut bosses {
        // Don't tick cooldowns while approaching
        if matches!(state, DarkMageState::Approaching) {
            continue;
        }
        cooldowns.tick(delta);

        // Enqueue spells as they come off cooldown (order: lightning, meteor, plague)
        let spell_order = [
            DarkMageSpellType::ShadowLightning,
            DarkMageSpellType::DarkMeteor,
            DarkMageSpellType::PlagueCloud,
        ];

        for spell in &spell_order {
            if cooldowns.is_ready(*spell) && !queue.queue.contains(spell) {
                queue.queue.push_back(*spell);
            }
        }
    }
}
