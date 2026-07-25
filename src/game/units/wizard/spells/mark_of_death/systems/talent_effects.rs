use std::cmp::Ordering;

use bevy::prelude::*;

use super::super::components::{ActiveMarkOfDeath, ExecutionerTriggered, MarkTalentFlags};
use super::super::constants;
use super::deaths_ledger::spawn_deaths_ledger_explosion;
use crate::game::multiplayer::components::GhostEntity;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, MarkedForDeathModifier, TargetingVelocity, Team, TemporaryHitPoints,
    apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::LocalWizard;
use crate::game::units::wizard::components::{Mana, Wizard};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Doom talent: increase damage amplification over time for doom-marked targets.
pub fn tick_doom_marks(
    time: Res<Time>,
    // Host-authoritative — never tick doom amplification on the guest's ghosts.
    mut marks: Query<(&mut MarkedForDeathModifier, &MarkTalentFlags), Without<GhostEntity>>,
) {
    let dt = time.delta_secs();
    for (mut mark, flags) in &mut marks {
        if flags.doom {
            mark.damage_amplification += constants::DOOM_AMP_PER_SECOND * dt;
            // Doom marks never expire — reset timer to keep them alive
            if mark.time_remaining < 1.0 {
                mark.time_remaining = 1.0;
            }
        }
    }
}

/// Executioner's Brand: deal burst damage when marked target falls below 30% HP.
#[allow(clippy::type_complexity)]
pub fn executioner_brand_check(
    mut commands: Commands,
    mut targets: Query<
        (
            Entity,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &MarkTalentFlags,
            Has<SpellShield>,
        ),
        (
            With<MarkedForDeathModifier>,
            Without<ExecutionerTriggered>,
            Without<Corpse>,
            // Host-authoritative — never deal Executioner burst on the guest's
            // ghost units (the host applies it and the damage syncs back).
            Without<GhostEntity>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    for (entity, mut health, mut temp_hp, flags, has_spell_shield) in &mut targets {
        if !flags.executioner_brand {
            continue;
        }
        if health.current <= health.max * constants::EXECUTIONER_HP_THRESHOLD
            && health.current > 0.0
        {
            apply_spell_damage(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                constants::EXECUTIONER_BURST_DAMAGE,
                DamageType::Necrotic,
                has_spell_shield,
            );
            commands.entity(entity).insert(ExecutionerTriggered);
        }
    }
}

/// Handles all death-triggered talent effects for marked corpses.
/// Runs when any MarkTalentFlags exists — checks for Corpse to detect death.
/// Processes spreading blight, swift hex refund, and death's ledger, then cleans up.
#[allow(clippy::too_many_arguments)]
pub fn handle_marked_corpses(
    mut commands: Commands,
    dead_marked: Query<
        (Entity, &Health, &MarkTalentFlags, &Transform),
        (With<Corpse>, Without<GhostEntity>),
    >,
    alive_enemies: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<MarkedForDeathModifier>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut wizard: Query<&mut Mana, (With<Wizard>, With<LocalWizard>)>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    let caster_team =
        crate::game::units::wizard::spells::utils::local_player_team(session.as_deref());
    for (entity, health, flags, transform) in &dead_marked {
        // Swift Hex: refund mana on death
        if flags.swift_hex_refund > 0.0
            && let Ok(mut mana) = wizard.single_mut()
        {
            mana.current = (mana.current + flags.swift_hex_refund).min(mana.max);
        }

        // Spreading Blight: jump mark to nearest unmarked enemy
        if flags.spreading_blight {
            let nearest = alive_enemies
                .iter()
                .filter(|(_, _, team)| caster_team.is_enemy(team))
                .min_by(|a, b| {
                    let dist_a = a.1.translation.distance_squared(transform.translation);
                    let dist_b = b.1.translation.distance_squared(transform.translation);
                    dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
                });

            if let Some((target_entity, _, _)) = nearest {
                let new_duration =
                    constants::MARK_DURATION * constants::SPREADING_BLIGHT_DURATION_PERCENT;
                commands.entity(target_entity).insert((
                    MarkedForDeathModifier::new(flags.amplification, new_duration),
                    ActiveMarkOfDeath,
                    flags.clone(),
                ));
            }
        }

        // Death's Ledger: explode proportional to max HP
        if flags.deaths_ledger {
            let explosion_damage = health.max * constants::DEATHS_LEDGER_DAMAGE_PER_MAX_HP;
            spawn_deaths_ledger_explosion(
                &mut commands,
                transform.translation,
                explosion_damage,
                &visual_assets,
                &mut materials,
            );
        }

        // Clean up mark components from corpse
        commands
            .entity(entity)
            .remove::<ActiveMarkOfDeath>()
            .remove::<MarkTalentFlags>()
            .remove::<ExecutionerTriggered>();
    }
}

/// Focal Point: redirect defender targeting toward marked focal-point targets.
pub fn focal_point_retarget(
    marked_targets: Query<
        (Entity, &Transform, &MarkTalentFlags),
        (
            With<MarkedForDeathModifier>,
            Without<Corpse>,
            Without<GhostEntity>,
        ),
    >,
    mut defenders: Query<
        (&Transform, &mut TargetingVelocity, &Team),
        (Without<Corpse>, Without<Wizard>),
    >,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
) {
    // Find the focal point target (if any)
    let focal_target = marked_targets
        .iter()
        .find(|(_, _, flags)| flags.focal_point);

    let Some((_, target_transform, _)) = focal_target else {
        return;
    };

    let target_pos = target_transform.translation;

    // Direct the LOCAL player's own army (Defenders for SP/host, Attackers for
    // the guest) toward the focal-point target.
    let caster_team =
        crate::game::units::wizard::spells::utils::local_player_team(session.as_deref());

    // Override own-army targeting velocity toward the focal point target
    for (defender_transform, mut targeting, team) in &mut defenders {
        if *team != caster_team {
            continue;
        }
        let direction = (target_pos - defender_transform.translation).normalize_or_zero();
        targeting.velocity = Vec3::new(direction.x, 0.0, direction.z);
        let dx = defender_transform.translation.x - target_pos.x;
        let dz = defender_transform.translation.z - target_pos.z;
        targeting.distance_to_target = (dx * dx + dz * dz).sqrt();
    }
}
