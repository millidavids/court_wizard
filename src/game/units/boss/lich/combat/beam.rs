use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::spawn::resolve_raise_dead;
use crate::game::constants::*;
use crate::game::resources::KillStats;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    Corpse, Health, Hitbox, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::components::Wizard;

type LichBeamTargetData = (
    Entity,
    &'static Transform,
    &'static Team,
    &'static mut Health,
    &'static Hitbox,
    Option<&'static mut TemporaryHitPoints>,
    Has<crate::game::units::king::components::King>,
);
type LichBeamTargetFilter = (
    Without<Corpse>,
    Without<Lich>,
    Without<Boss>,
    Without<Wizard>,
);

/// Phase 2: Ticks the Finger of Death cooldown and starts a beam cast wind-up
/// when ready. The actual beam fire is deferred to `tick_lich_casting`. The
/// Lich will not fire until he has closed within `FOD_KING_RANGE` of the King.
pub(crate) fn lich_fire_beam(
    time: Res<Time>,
    mut commands: Commands,
    mut lich_query: Query<
        (Entity, &Transform, &mut LichFingerOfDeath, &LichPhase),
        (With<Lich>, Without<Corpse>, Without<LichCasting>),
    >,
    king_query: Query<
        &Transform,
        (
            With<crate::game::units::king::components::King>,
            Without<Lich>,
            Without<Corpse>,
        ),
    >,
) {
    let king_pos = king_query.iter().next().map(|t| t.translation);

    for (entity, lich_transform, mut fod, phase) in &mut lich_query {
        if *phase != LichPhase::Combat {
            continue;
        }

        fod.tick(time.delta_secs());

        if !fod.is_ready() {
            continue;
        }

        // Hold fire until the Lich is within range of the King.
        let Some(king_pos) = king_pos else { continue };
        if lich_transform.translation.distance(king_pos) > FOD_KING_RANGE {
            continue;
        }

        let Some(target_entity) = fod.target else {
            continue;
        };

        commands.entity(entity).insert(LichCasting {
            remaining: LICH_FINGER_OF_DEATH_CAST_DURATION,
            kind: LichCastKind::FingerOfDeath {
                target: target_entity,
            },
        });
        fod.reset(BEAM_COOLDOWN);
    }
}

/// Spawns the Finger of Death beam, applies damage, and triggers the screen
/// darkening effect. Called from `tick_lich_casting` when a FoD cast resolves.
#[allow(clippy::too_many_arguments)]
fn resolve_finger_of_death(
    commands: &mut Commands,
    lich_transform: &Transform,
    lich_hitbox: &Hitbox,
    target: Entity,
    spell_assets: &crate::game::units::wizard::spells::visual_assets::SpellVisualAssets,
    target_query: &Query<&Transform, (Without<Lich>, Without<Wizard>)>,
    materials: &mut Assets<StandardMaterial>,
    desaturate: &mut MessageWriter<crate::game::crt_effect::ScreenDesaturateMessage>,
    target_health: &mut Query<LichBeamTargetData, LichBeamTargetFilter>,
    king_immune_to_fod: bool,
) {
    use crate::game::units::wizard::spells::finger_of_death::components::{
        FingerOfDeathBeam, FodTalentParams, PendingUndeadRaise,
    };

    let Ok(target_transform) = target_query.get(target) else {
        return;
    };

    let origin = Vec3::new(
        lich_transform.translation.x,
        lich_transform.translation.y + lich_hitbox.height * 0.5,
        lich_transform.translation.z,
    );

    let to_target = target_transform.translation - origin;
    let direction = to_target.normalize_or_zero();
    if direction.length_squared() < 0.5 {
        return;
    }

    let beam_length = BEAM_LENGTH.min(to_target.length() + 200.0);

    let talent_params = FodTalentParams {
        damage: BEAM_DAMAGE,
        beam_width: BEAM_WIDTH,
        beam_width_fired: BEAM_WIDTH,
        ..Default::default()
    };
    let mut beam =
        FingerOfDeathBeam::with_talents(origin, direction, beam_length, 1.0, talent_params);
    beam.has_fired = true;
    beam.cast_progress = 1.0;

    crate::game::units::wizard::spells::finger_of_death::systems::spawn_beam(
        commands,
        spell_assets,
        materials,
        beam,
    );

    desaturate.write(crate::game::crt_effect::ScreenDesaturateMessage);

    let mut kill_positions: Vec<Vec3> = Vec::new();
    for (entity, t_transform, team, mut health, hitbox, temp_hp, is_king) in
        target_health.iter_mut()
    {
        if *team != Team::Defenders {
            continue;
        }

        let to_point = t_transform.translation - origin;
        let proj = to_point.dot(direction);
        if proj < 0.0 || proj > beam_length {
            continue;
        }
        let closest = origin + direction * proj;
        let dist = t_transform.translation.distance(closest);
        if dist <= BEAM_WIDTH + hitbox.radius {
            // King is fully immune to FoD until enough defenders have fallen.
            if is_king && king_immune_to_fod {
                continue;
            }

            let damage = if is_king {
                BEAM_DAMAGE * KING_FOD_DAMAGE_MULTIPLIER
            } else {
                BEAM_DAMAGE
            };

            let hp_before = health.current;
            apply_spell_damage(
                commands,
                entity,
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                damage,
                DamageType::Necrotic,
                false,
            );

            if hp_before > 0.0 && health.is_dead() {
                kill_positions.push(t_transform.translation);
            }
        }
    }

    if !kill_positions.is_empty() {
        commands.insert_resource(PendingUndeadRaise { kill_positions });
    }
}

/// Decrements the active cast's wind-up timer. When it expires, dispatches to
/// the appropriate spell-resolution helper and removes `LichCasting` so the
/// trigger systems can start the next cast on subsequent frames.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tick_lich_casting(
    time: Res<Time>,
    kill_stats: Res<KillStats>,
    mut commands: Commands,
    mut lich_query: Query<
        (Entity, &Transform, &Hitbox, &mut LichCasting),
        (With<Lich>, Without<Corpse>),
    >,
    corpse_query: Query<
        (Entity, &Transform),
        (
            With<Corpse>,
            Without<crate::game::units::components::PermanentCorpse>,
            Without<Lich>,
            Without<Wizard>,
        ),
    >,
    undead_assets: Res<UndeadAssets>,
    spell_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    target_query: Query<&Transform, (Without<Lich>, Without<Wizard>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut desaturate: MessageWriter<crate::game::crt_effect::ScreenDesaturateMessage>,
    mut target_health: Query<LichBeamTargetData, LichBeamTargetFilter>,
) {
    let delta = time.delta_secs();

    for (entity, transform, hitbox, mut casting) in &mut lich_query {
        casting.remaining -= delta;
        if casting.remaining > 0.0 {
            continue;
        }

        match casting.kind {
            LichCastKind::RaiseDead => {
                resolve_raise_dead(
                    &mut commands,
                    transform.translation,
                    &corpse_query,
                    &undead_assets,
                    &mut materials,
                );
            }
            LichCastKind::FingerOfDeath { target } => {
                let defender_death_ratio = if INITIAL_DEFENDER_COUNT > 0 {
                    kill_stats.defenders_killed as f32 / INITIAL_DEFENDER_COUNT as f32
                } else {
                    0.0
                };
                let king_immune_to_fod = defender_death_ratio < KING_FOD_IMMUNITY_THRESHOLD;

                resolve_finger_of_death(
                    &mut commands,
                    transform,
                    hitbox,
                    target,
                    &spell_assets,
                    &target_query,
                    &mut materials,
                    &mut desaturate,
                    &mut target_health,
                    king_immune_to_fod,
                );
            }
        }

        commands.entity(entity).remove::<LichCasting>();
    }
}
