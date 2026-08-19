use bevy::prelude::*;

use super::super::components::MeteorExplosion;
use super::super::constants::*;
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, explosion_fade_opacity,
};
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;

/// Updates explosion visuals and applies one-time impact damage.
/// Also tracks talent progress.
/// Visual-only: grows + fades every `MeteorExplosion`, **including the ghost
/// copies on the opposing client** so the explosion is visible there. Damage and
/// despawn live in `apply_meteor_explosion_damage` (gated host-only). Chained
/// before it so `time_alive` is current for the despawn check.
pub(crate) fn animate_meteor_explosions(
    time: Res<Time>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut explosions: Query<(
        &mut MeteorExplosion,
        &mut Transform,
        Option<&MeshMaterial3d<FireExplosionSphereMaterial>>,
    )>,
) {
    for (mut explosion, mut transform, material_handle) in explosions.iter_mut() {
        explosion.time_alive += time.delta_secs();

        // Update visual scale (growth animation)
        let current_radius = explosion.current_radius(EXPLOSION_GROWTH_TIME);
        transform.scale = Vec3::splat(current_radius);

        // Fade out over the last portion of lifetime
        if let Some(handle) = material_handle
            && let Some(mut mat) = sphere_materials.get_mut(handle)
        {
            mat.opacity = explosion_fade_opacity(explosion.time_alive / EXPLOSION_LIFETIME);
        }
    }
}

/// Host-only: applies the explosion's one-shot AoE damage + terrain damage +
/// talent progress, then despawns it after its lifetime. Ghost copies on the
/// guest are excluded — they're animated by `animate_meteor_explosions` and
/// despawned by snapshot reconciliation (the host's snapshot drives their life).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_meteor_explosion_damage(
    mut commands: Commands,
    mut explosions: Query<
        (Entity, &mut MeteorExplosion),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (
            Without<MeteorExplosion>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
    session: Option<Res<MultiplayerSession>>,
) {
    for (explosion_entity, mut explosion) in explosions.iter_mut() {
        // Apply damage once when explosion spawns
        if !explosion.damage_applied {
            explosion.damage_applied = true;

            terrain_damage.write(TerrainDamageMessage {
                position: explosion.origin,
                radius: explosion.max_radius,
                damage: explosion.damage,
                damage_type: DamageType::Fire,
            });

            let caster_team = local_player_team(session.as_deref());
            let mut hit_count = 0u32;

            for (unit_entity, unit_transform, mut health, mut temp_hp, has_spell_shield, team) in
                units.iter_mut()
            {
                let distance = crate::game::units::wizard::spells::utils::xz_distance(
                    unit_transform.translation,
                    explosion.origin,
                );

                if distance <= explosion.max_radius {
                    apply_spell_damage_with_team(
                        &mut commands,
                        unit_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage,
                        DamageType::Fire,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    hit_count += 1;
                }
            }

            // Track talent progress
            if hit_count > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::MeteorFall, hit_count);
            }
        }

        // Despawn explosion after lifetime
        if explosion.time_alive >= EXPLOSION_LIFETIME {
            commands.entity(explosion_entity).try_despawn();
        }
    }
}
