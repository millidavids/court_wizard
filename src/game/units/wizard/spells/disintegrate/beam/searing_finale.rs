use super::super::components::SearingFinaleDetonation;
use super::super::constants;
use crate::game::units::components::{
    Health, Hitbox, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// System that processes searing finale detonations.
/// Applies burst damage once along the detonation line, then fades and despawns.
#[allow(clippy::type_complexity)]
pub fn update_searing_finale_detonations(
    mut commands: Commands,
    mut detonation_query: Query<(Entity, &mut SearingFinaleDetonation, &mut Transform)>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (
            Without<Wizard>,
            Without<SearingFinaleDetonation>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    time: Res<Time>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let dt = time.delta_secs();

    for (det_entity, mut detonation, mut transform) in detonation_query.iter_mut() {
        detonation.time_alive += dt;

        if detonation.time_alive >= constants::SEARING_FINALE_DURATION {
            commands.entity(det_entity).try_despawn();
            continue;
        }

        // Apply damage once
        if !detonation.damage_applied {
            detonation.damage_applied = true;

            for (
                entity,
                target_transform,
                hitbox,
                mut health,
                mut temp_hp,
                has_spell_shield,
                team,
            ) in target_query.iter_mut()
            {
                let pos = target_transform.translation;
                let to_point = pos - detonation.origin;
                let proj = to_point.dot(detonation.direction);

                if proj < -hitbox.radius || proj > detonation.length + hitbox.radius {
                    continue;
                }

                let closest =
                    detonation.origin + detonation.direction * proj.clamp(0.0, detonation.length);
                let dist = pos.distance(closest);

                if dist <= detonation.half_width + hitbox.radius {
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        detonation.damage,
                        constants::DAMAGE_TYPE,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                }
            }
        }

        // Visual: expand width over duration
        let progress = detonation.time_alive / constants::SEARING_FINALE_DURATION;
        let visual_width = detonation.half_width * 2.0 * (1.0 + progress * 0.5);
        let alpha = 1.0 - progress;
        transform.scale = Vec3::new(
            visual_width * alpha,
            detonation.length,
            visual_width * alpha,
        );
    }
}
