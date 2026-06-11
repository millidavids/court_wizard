use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;

use crate::game::components::OnGameplayScreen;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::utils::{local_player_team, xz_distance};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;

/// When a crystal's resonance counter reaches the threshold, emit a powerful
/// damage burst to all enemies in range and reset the counter.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resonance_cascade_burst(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (&mut ArcaneCrystal, &mut ResonanceCascade),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    for (mut crystal, mut resonance) in &mut crystals {
        if resonance.absorptions < RESONANCE_CASCADE_THRESHOLD {
            continue;
        }

        resonance.absorptions = 0;
        crystal.trigger_pulse();

        let hit_count = crystal_aoe_burst(
            &mut commands,
            &visual_assets,
            crystal.position,
            crystal.range,
            RESONANCE_CASCADE_DAMAGE * crystal.empowerment,
            RESONANCE_CASCADE_RADIUS,
            2.0,
            0.3,
            &targets,
            &mut health_query,
            caster_team,
        );

        if hit_count > 0 {
            progress.increment(Spell::ArcaneCrystal, hit_count);
        }
    }
}

/// Shared AoE burst: damages all enemies in radius and spawns a visual ring.
/// Returns the number of enemies hit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn crystal_aoe_burst(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    position: Vec3,
    crystal_range: f32,
    damage: f32,
    radius: f32,
    visual_height: f32,
    visual_lifetime: f32,
    targets: &Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    caster_team: Team,
) -> u32 {
    let mut hit_count: u32 = 0;

    for (target_entity, target_transform) in targets {
        let dist = xz_distance(position, target_transform.translation);
        if dist > radius {
            continue;
        }

        if let Ok((mut health, mut temp_hp, has_spell_shield, team)) =
            health_query.get_mut(target_entity)
        {
            apply_spell_damage_with_team(
                commands,
                target_entity,
                &mut health,
                temp_hp.as_deref_mut(),
                damage,
                DamageType::Force,
                has_spell_shield,
                caster_team,
                *team,
            );
            hit_count += 1;
        }
    }

    // Spawn burst visual — expanding ring
    let burst_pos = Vec3::new(position.x, visual_height, position.z);
    commands.spawn((
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(visual_assets.arcane_crystal_indicator.clone()),
        Transform::from_translation(burst_pos)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        CrystalSpawn {
            origin: position,
            max_range: crystal_range,
            lifetime: Some(visual_lifetime),
        },
        OnGameplayScreen,
    ));

    hit_count
}
