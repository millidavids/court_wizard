use super::super::components::{
    BlindingMistZone, ChokingFogZone, ConcealingVeilZone, DisorientingVaporsZone,
    FogCloudTalentParams, FogCloudZone, PhantomFogZone, RollingFogZone,
};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

pub(crate) fn spawn_fog_cloud_zone(
    commands: &mut Commands,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    talent_params: &FogCloudTalentParams,
    scorched_mult: f32,
) -> Entity {
    let duration = constants::ZONE_DURATION * empowerment * scorched_mult;
    let evasion = talent_params.evasion_chance;
    let refresh_dur = talent_params.linger_duration * empowerment;

    let zone_entity = commands
        .spawn((
            Transform::from_translation(Vec3::new(position.x, 0.0, position.z)),
            FogCloudZone::new(
                Vec3::new(position.x, 0.0, position.z),
                radius,
                evasion,
                refresh_dur,
                constants::TICK_INTERVAL,
                duration,
            ),
            NetworkedSpellEffect {
                kind: SpellEffectKind::FogCloudZone,
            },
            OnGameplayScreen,
        ))
        .id();

    // Insert talent-specific zone marker components
    if talent_params.blinding_mist {
        commands.entity(zone_entity).insert(BlindingMistZone);
    }
    if talent_params.concealing_veil {
        commands.entity(zone_entity).insert(ConcealingVeilZone);
    }
    if talent_params.disorienting_vapors {
        commands.entity(zone_entity).insert(DisorientingVaporsZone);
    }
    if talent_params.phantom_fog {
        commands
            .entity(zone_entity)
            .insert(PhantomFogZone { spawn_timer: 0.0 });
    }
    if talent_params.choking_fog {
        commands.entity(zone_entity).insert(ChokingFogZone::new(
            constants::CHOKING_FOG_DPS,
            constants::CHOKING_FOG_TICK_INTERVAL,
        ));
    }
    if talent_params.rolling_fog {
        commands.entity(zone_entity).insert(RollingFogZone {
            speed: constants::ROLLING_FOG_SPEED,
        });
    }
    zone_entity
}

/// Returns true if the given position is inside any fog cloud zone (given as origin/radius pairs).
pub(crate) fn is_in_fog_zone(pos: Vec3, zones: &[(Vec3, f32)]) -> bool {
    for &(origin, radius) in zones {
        if xz_distance(pos, origin) <= radius {
            return true;
        }
    }
    false
}
