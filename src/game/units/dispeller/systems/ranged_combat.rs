use std::cmp::Ordering;

use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::DispellerAssets;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{StagingAttacker, WaveGroup};
use crate::game::units::components::{BanishedModifier, Corpse, SleepModifier, Team};
use crate::game::units::ranged_bolt::RangedAttackTimer;
use crate::game::units::wizard::spells::dispel::systems::is_dispellable;
use crate::game::units::wizard::spells::vfx::channel::ChannelingCast;

/// Fires weak magic bolts at enemies when no spell effects exist to dispel.
#[allow(clippy::type_complexity)]
pub fn dispeller_ranged_combat(
    mut commands: Commands,
    time: Res<Time>,
    dispeller_assets: Res<DispellerAssets>,
    spell_effects: Query<&NetworkedSpellEffect>,
    mut dispellers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut RangedAttackTimer,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
            Has<ChannelingCast>,
            Has<StagingAttacker>,
            Has<WaveGroup>,
        ),
        (With<Dispeller>, Without<Corpse>),
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<Corpse>, Without<BanishedModifier>)>,
) {
    // Only fire bolts when no dispellable spell effects exist
    let has_spell_targets = spell_effects.iter().any(|nse| is_dispellable(nse.kind));
    if has_spell_targets {
        return;
    }

    let delta = time.delta_secs();

    for (
        dispeller_entity,
        dispeller_transform,
        dispeller_team,
        mut attack_timer,
        sleeping,
        banished,
        is_channeling,
        has_staging,
        has_wave_group,
    ) in &mut dispellers
    {
        // Skip staging attackers (includes 1-frame delay before WaveGroup is added)
        if crate::game::units::systems::is_staging_attacker(
            dispeller_team,
            has_staging,
            has_wave_group,
        ) {
            continue;
        }

        attack_timer.time_since_last_attack += delta;

        if is_channeling || sleeping.is_some() || banished.is_some() {
            continue;
        }

        // Check cooldown
        if attack_timer.time_since_last_attack < ATTACK_COOLDOWN {
            continue;
        }

        // Find nearest enemy within attack range
        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team)| {
                *entity != dispeller_entity && dispeller_team.is_enemy(team)
            })
            .filter(|(_, transform, _)| {
                let distance = dispeller_transform
                    .translation
                    .distance(transform.translation);
                distance <= ATTACK_RANGE
            })
            .min_by(|a, b| {
                let dist_a = dispeller_transform.translation.distance(a.1.translation);
                let dist_b = dispeller_transform.translation.distance(b.1.translation);
                dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
            });

        if let Some((_, target_transform, _)) = nearest_enemy {
            let origin = dispeller_transform.translation + Vec3::Y * 10.0;
            let diff = target_transform.translation - origin;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

            crate::game::units::ranged_bolt::spawn_magic_bolt(
                &mut commands,
                &dispeller_assets.bolt_mesh,
                &dispeller_assets.bolt_material,
                origin,
                direction,
                BOLT_SPEED,
                BOLT_DAMAGE,
                BOLT_RADIUS,
                BOLT_LIFETIME,
                *dispeller_team,
            );

            attack_timer.time_since_last_attack = 0.0;

            commands.entity(dispeller_entity).insert(
                crate::game::units::components::CombatAnimation::new_attack(
                    dispeller_assets.attacking_texture.clone(),
                    dispeller_assets.sprite_texture.clone(),
                ),
            );
        }
    }
}
