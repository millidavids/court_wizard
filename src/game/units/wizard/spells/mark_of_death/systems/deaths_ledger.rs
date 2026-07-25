use bevy::prelude::*;

use super::super::components::DeathsLedgerBurst;
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::DamageType;
use crate::game::units::components::Team;
use crate::game::units::components::{
    Corpse, Health, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

/// Spawns a Death's Ledger explosion visual at a position.
pub(crate) fn spawn_deaths_ledger_explosion(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    visual_assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let pulse_material = if let Some(mat) = materials.get(&visual_assets.necrotic_pulse).cloned() {
        mat
    } else {
        bevy::log::warn!(
            "spawn_deaths_ledger_explosion: necrotic_pulse material handle is invalid; \
             explosion will render as default white material"
        );
        StandardMaterial::default()
    };
    let instance = materials.add(pulse_material);

    commands.spawn((
        DeathsLedgerBurst {
            time_alive: 0.0,
            lifetime: constants::DEATHS_LEDGER_PULSE_LIFETIME,
            max_radius: constants::DEATHS_LEDGER_RADIUS,
            damage,
            damage_applied: false,
        },
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(instance),
        Transform::from_translation(Vec3::new(position.x, 10.0, position.z))
            .with_rotation(UPWARD_ROTATION)
            .with_scale(Vec3::splat(1.0)),
        OnGameplayScreen,
    ));
}

/// Apply AoE damage from Death's Ledger explosions (one-shot).
#[allow(clippy::type_complexity)]
pub fn apply_deaths_ledger_damage(
    mut commands: Commands,
    mut explosions: Query<(&Transform, &mut DeathsLedgerBurst)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (
            Without<Wizard>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    for (explosion_transform, mut burst) in &mut explosions {
        if burst.damage_applied {
            continue;
        }
        burst.damage_applied = true;

        for (entity, target_transform, mut health, mut temp_hp, has_spell_shield, team) in
            &mut targets
        {
            let dx = explosion_transform.translation.x - target_transform.translation.x;
            let dz = explosion_transform.translation.z - target_transform.translation.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist <= burst.max_radius {
                apply_spell_damage_with_team(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    burst.damage,
                    DamageType::Necrotic,
                    has_spell_shield,
                    caster_team,
                    *team,
                );
            }
        }
    }
}

/// Update Death's Ledger burst visuals — expand and fade.
pub fn update_deaths_ledger_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut bursts: Query<(
        Entity,
        &mut DeathsLedgerBurst,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut burst, mut transform, material_handle) in bursts.iter_mut() {
        burst.time_alive += dt;

        if burst.time_alive >= burst.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = burst.time_alive / burst.lifetime;
        let scale = burst.max_radius * progress;
        transform.scale = Vec3::splat(scale.max(0.1));

        if let Some(mat) = materials.get_mut(material_handle) {
            let alpha = 1.0 - progress;
            mat.base_color = mat.base_color.with_alpha(alpha * 0.5);
        }
    }
}
