use super::super::components::{EntangleRooted, EntangleTalentParams, ThornyVines};
use super::super::constants;
use crate::game::achievements::messages::EntangleHitDefenderMessage;
use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::components::{
    Corpse, Health, RootedModifier, SlowMovementModifier, Team, TemporaryHitPoints,
};
use crate::game::units::wizard::components::{LocalWizard, Mana};
use bevy::prelude::*;

/// Applies entangle root/sanctuary to a single unit based on talent params.
/// Returns true if the unit is an enemy (for hit counting).
pub(crate) fn apply_entangle_to_unit(
    commands: &mut Commands,
    entity: Entity,
    team: &Team,
    duration: f32,
    talent_params: &EntangleTalentParams,
    defender_hit_msg: &mut MessageWriter<EntangleHitDefenderMessage>,
) -> bool {
    let is_defender = *team == Team::Defenders;

    // Nature's Sanctuary: defenders get temp HP instead of root
    if talent_params.sanctuary && is_defender {
        commands.entity(entity).insert(TemporaryHitPoints::new(
            constants::SANCTUARY_TEMP_HP,
            duration,
        ));
    } else {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((
            RootedModifier::new(duration),
            EntangleRooted {
                total_root_duration: duration,
                is_defender,
                talent_params: *talent_params,
            },
        ));
        // Thorny Vines: only apply to non-defenders
        if talent_params.thorny_vines && !is_defender {
            entity_commands.insert(ThornyVines { tick_timer: 0.0 });
        }
    }

    if is_defender {
        defender_hit_msg.write(EntangleHitDefenderMessage);
    }

    !is_defender
}

/// Thorny Vines: deals periodic damage to rooted enemies (not defenders).
pub fn thorny_vines_tick(
    time: Res<Time>,
    mut commands: Commands,
    // `Without<GhostEntity>`: thorny vines mutate `Health` directly (not via the
    // CRDT/`PendingDamageEffect` forward path), so ticking it on a guest ghost
    // would corrupt host-authoritative HP. Host-authoritative only.
    mut rooted_units: Query<
        (
            Entity,
            &mut ThornyVines,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<GhostEntity>,
    >,
) {
    let delta = time.delta_secs();
    for (entity, mut thorny, mut health, mut temp_hp) in &mut rooted_units {
        thorny.tick_timer += delta;
        if thorny.tick_timer >= constants::THORNY_VINES_TICK_INTERVAL {
            thorny.tick_timer -= constants::THORNY_VINES_TICK_INTERVAL;
            let damage = constants::THORNY_VINES_DPS * constants::THORNY_VINES_TICK_INTERVAL;
            crate::game::units::components::apply_damage_to_unit(
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
            );
            if health.current <= 0.0 {
                commands.entity(entity).insert(Corpse);
            }
        }
    }
}

/// Handles effects when EntangleRooted units lose their RootedModifier (root expired).
/// Applies Clinging Roots slow and Stranglehold burst damage.
pub fn handle_entangle_root_expire(
    mut commands: Commands,
    mut rooted_units: Query<
        (
            Entity,
            &EntangleRooted,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        // Host-authoritative: don't apply Clinging Roots / Stranglehold to ghosts.
        (Without<RootedModifier>, Without<GhostEntity>),
    >,
) {
    for (entity, entangle, mut health, mut temp_hp) in &mut rooted_units {
        // Clinging Roots: slow enemies after root expires
        if entangle.talent_params.clinging_roots && !entangle.is_defender {
            commands.entity(entity).insert(SlowMovementModifier::new(
                constants::CLINGING_ROOTS_SLOW,
                constants::CLINGING_ROOTS_SLOW_DURATION,
            ));
        }

        // Stranglehold: burst damage if rooted long enough
        if entangle.talent_params.stranglehold
            && !entangle.is_defender
            && entangle.total_root_duration >= constants::STRANGLEHOLD_THRESHOLD
        {
            crate::game::units::components::apply_damage_to_unit(
                &mut health,
                temp_hp.as_deref_mut(),
                constants::STRANGLEHOLD_DAMAGE,
            );
            if health.current <= 0.0 {
                // Stranglehold kills don't leave corpses — despawn entirely
                commands.entity(entity).try_despawn();
                continue;
            }
        }

        commands
            .entity(entity)
            .remove::<(EntangleRooted, ThornyVines)>();
    }
}

/// Nourishing Roots: regenerates wizard mana based on number of rooted enemies.
pub fn nourishing_roots_mana_regen(
    time: Res<Time>,
    mut wizard_query: Query<&mut Mana, With<LocalWizard>>,
    rooted_units: Query<&EntangleRooted>,
) {
    let Ok(mut mana) = wizard_query.single_mut() else {
        return;
    };

    let enemy_count = rooted_units
        .iter()
        .filter(|e| e.talent_params.nourishing_roots && !e.is_defender)
        .count();

    if enemy_count > 0 {
        let regen =
            constants::NOURISHING_ROOTS_MANA_PER_SEC * enemy_count as f32 * time.delta_secs();
        mana.regenerate(regen);
    }
}
