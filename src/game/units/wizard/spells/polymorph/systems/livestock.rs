use super::super::super::super::components::Spell;
use super::super::components::{ContagiousBaas, DireSheep, ExplosiveSheep, PermanentLivestock};
use super::super::sheep_visual::SheepBounce;
use super::core::apply_polymorph_to_target;
use super::shared::strip_polymorph_components;
use crate::config::GameConfig;
use crate::game::units::DamageType;
use crate::game::units::components::{
    AttackTiming, Corpse, Health, PolymorphedModifier, Team, TemporaryHitPoints,
    apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;
use std::cmp::Ordering;

pub use super::shared::apply_sheep_visual;

/// Ticks polymorphed unit timers and restores them when expired.
/// Handles Permanent Livestock (stay forever), Dire Sheep (kill on expiry),
/// and Contagious Baas (spread to nearest unit on expiry).
/// Explosive Sheep detonation on death is handled by `check_explosive_sheep_deaths`.
#[allow(clippy::too_many_arguments)]
pub fn tick_polymorphed_units(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    // `Without<GhostEntity>`: this mutates Health / re-inserts AttackTiming /
    // restores meshes — all host-authoritative. Host-gated by is_gameplay_running
    // today, so this guard is defensive against ghost entities.
    mut polymorphed: Query<
        (
            Entity,
            &mut Transform,
            &mut PolymorphedModifier,
            &mut Health,
            Option<&ContagiousBaas>,
            Option<&SheepBounce>,
            Has<PermanentLivestock>,
            Has<DireSheep>,
        ),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
    targets_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
            &Mesh3d,
            &Team,
        ),
        (
            Without<Corpse>,
            Without<PolymorphedModifier>,
            Without<Wizard>,
        ),
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let delta = time.delta_secs();

    for (
        entity,
        mut transform,
        mut modifier,
        mut health,
        contagious,
        sheep_bounce,
        is_permanent,
        is_dire,
    ) in &mut polymorphed
    {
        if modifier.update(delta) {
            if is_permanent {
                // Permanent Livestock: sheep survives full duration, stays polymorphed forever.
                // Set timer to infinity so it never reverts, and remove the marker.
                modifier.time_remaining = f32::MAX;
                commands.entity(entity).remove::<PermanentLivestock>();
                continue;
            }

            // Contagious Baas: spread polymorph to nearest unit on expiry (any unit, magic is indiscriminate)
            if let Some(contagious) = contagious {
                let nearest = targets_query
                    .iter()
                    .map(|(e, t, h, m, mesh, team)| {
                        let dist = t.translation.distance(transform.translation);
                        (e, dist, t.translation, h, m, mesh, *team)
                    })
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

                if let Some((
                    target_entity,
                    _,
                    target_pos,
                    target_health,
                    target_material,
                    target_mesh,
                    target_team,
                )) = nearest
                {
                    let empowerment = contagious.empowerment;
                    let talent_params = contagious.talent_params;
                    let duration = talent_params.duration * empowerment;

                    apply_polymorph_to_target(
                        &mut commands,
                        &mut materials,
                        target_entity,
                        target_health,
                        target_material,
                        target_mesh,
                        target_team,
                        duration,
                        &talent_params,
                        empowerment,
                        target_pos,
                        &visual_assets,
                        time.elapsed_secs(),
                        &mut pending_cast_events,
                    );

                    audio::play_sfx_synced(
                        &mut commands,
                        &mut pending_cast_events,
                        SpellSoundId::PolymorphCast,
                        transform.translation,
                        &game_config,
                        &sfx,
                    );

                    if let Some(ref mut progress) = talent_progress {
                        progress.increment(Spell::Polymorph, 1);
                    }
                }
            }

            if let Some(bounce) = sheep_bounce {
                transform.translation.y = bounce.base_y;
            }

            let mut e = commands.entity(entity);
            e.insert((
                MeshMaterial3d(modifier.original_material.clone()),
                Mesh3d(modifier.original_mesh.clone()),
            ));
            if is_dire {
                health.current = 0.0;
                e.insert(modifier.original_team);
            } else {
                health.current = modifier.original_health_current;
                health.max = modifier.original_health_max;
                e.insert(AttackTiming::new());
            }
            strip_polymorph_components(&mut commands, entity);
        }
    }
}

/// Checks for sheep that die (health depleted) and triggers explosive sheep if applicable.
/// This runs after tick_polymorphed_units to catch deaths from combat and timer expiry.
pub fn check_explosive_sheep_deaths(
    mut commands: Commands,
    sheep_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &PolymorphedModifier,
            Has<ExplosiveSheep>,
        ),
        (Without<Corpse>, Without<DireSheep>),
    >,
    mut damage_targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    use super::super::constants;
    let caster_team = local_player_team(session.as_deref());

    for (sheep_entity, sheep_transform, sheep_health, modifier, is_explosive) in &sheep_query {
        if !sheep_health.is_dead() {
            continue;
        }

        // Restore original team so corpse/kill tracking uses the correct team
        commands.entity(sheep_entity).insert(modifier.original_team);
        strip_polymorph_components(&mut commands, sheep_entity);

        if is_explosive {
            // Deal AoE damage to nearby enemies
            for (
                target_entity,
                target_transform,
                mut target_health,
                mut temp_hp,
                has_spell_shield,
                team,
            ) in &mut damage_targets
            {
                let dist = target_transform
                    .translation
                    .distance(sheep_transform.translation);
                if dist <= constants::EXPLOSIVE_SHEEP_RADIUS {
                    apply_spell_damage_with_team(
                        &mut commands,
                        target_entity,
                        &mut target_health,
                        temp_hp.as_deref_mut(),
                        constants::EXPLOSIVE_SHEEP_DAMAGE,
                        DamageType::Nature,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                }
            }
        }
    }
}
