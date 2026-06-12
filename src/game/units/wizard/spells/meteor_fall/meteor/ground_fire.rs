use bevy::prelude::*;

use super::super::components::MeteorGroundFire;
use super::super::constants::*;
use crate::game::pathfinding::OBSTACLE_BUFFER;
use crate::game::pathfinding::resources::PathfindingGrid;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::utils::{local_player_team, xz_distance};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

/// Spawns procedural fire particles rising off meteor ground fire pools.
pub(crate) fn spawn_ground_fire_particles(
    mut commands: Commands,
    fires: Query<&MeteorGroundFire>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < GROUND_FIRE_SMOKE_INTERVAL {
        return;
    }
    *timer -= GROUND_FIRE_SMOKE_INTERVAL;

    let t = time.elapsed_secs();

    for fire in fires.iter() {
        // Don't emit smoke during the fade-out period
        let remaining = fire.duration - fire.time_alive;
        if remaining < GROUND_FIRE_FADE_DURATION {
            continue;
        }

        vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            Vec3::new(fire.origin.x, 0.0, fire.origin.z),
            fire.radius,
            GROUND_FIRE_PARTICLE_COUNT,
            t,
        );
    }
}

/// Applies periodic fire damage to units standing in ground fire zones.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_ground_fire_damage(
    mut commands: Commands,
    time: Res<Time>,
    // Host-authoritative — the ghost ground fire on the guest must not tick its
    // own lifetime/damage; reconciliation drives its lifecycle.
    mut fires: Query<
        &mut MeteorGroundFire,
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut units: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    session: Option<Res<MultiplayerSession>>,
) {
    let delta = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

    for mut fire in &mut fires {
        fire.time_alive += delta;
        fire.time_since_last_tick += delta;

        if fire.time_since_last_tick >= fire.tick_interval {
            fire.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield, team) in &mut units {
                let dist = xz_distance(fire.origin, transform.translation);

                if dist <= fire.radius {
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        fire.damage_per_tick,
                        DamageType::Fire,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                }
            }
        }
    }
}

/// Fades ground fire by scaling down as it approaches expiration.
pub(crate) fn fade_ground_fire(mut fires: Query<(&MeteorGroundFire, &mut Transform)>) {
    for (fire, mut transform) in &mut fires {
        let remaining = fire.duration - fire.time_alive;
        if remaining < GROUND_FIRE_FADE_DURATION {
            let fade = (remaining / GROUND_FIRE_FADE_DURATION).max(0.0);
            let base_radius = fire.radius;
            transform.scale = Vec3::splat(base_radius * fade);
        }
    }
}

/// Cleans up expired ground fire zones and resets pathfinding costs.
pub(crate) fn cleanup_ground_fire(
    mut commands: Commands,
    // Host-authoritative — the ghost ground fire never wrote terrain cost on the
    // guest, so it must not reset it on expiry (would clobber other obstacles).
    fires: Query<
        (Entity, &MeteorGroundFire),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut pathfinding: ResMut<PathfindingGrid>,
) {
    for (entity, fire) in &fires {
        if fire.time_alive >= fire.duration {
            // Reset terrain cost for the fire zone
            let origin_2d = Vec2::new(fire.origin.x, fire.origin.z);
            let buffered = fire.radius + OBSTACLE_BUFFER;
            let bounds = Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0));
            let shape = crate::game::pathfinding::ObstacleShape::circle(origin_2d, buffered);
            let cells = pathfinding.shape_filtered_cells(bounds, &shape);
            pathfinding.set_terrain_cost(&cells, 1.0);

            // Continuous flow field rebuilds will pick up the cost change automatically

            commands.entity(entity).try_despawn();
        }
    }
}
