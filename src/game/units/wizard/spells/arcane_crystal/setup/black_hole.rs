//! What happens when a crystal meets a black hole.

use bevy::prelude::*;

use super::super::auto::crystal_aoe_burst;
use super::super::components::*;
use super::super::constants::*;
use super::super::infusions::{CrystalAnchored, CrystalWarded};
use super::helpers::destroy_crystal;
use crate::game::units::components::{Corpse, Health, Team, TemporaryHitPoints};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::black_hole::constants::GRAVITY_RANGE;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

/// Pulls crystals toward black holes and detonates them if they cross the horizon.
#[allow(clippy::too_many_arguments)]
pub(crate) fn crystal_black_hole_interaction(
    mut commands: Commands,
    time: Res<Time>,
    black_holes: Query<&crate::game::units::wizard::spells::black_hole::components::BlackHole>,
    // `Without<GhostSpellEffect>` and the `permanent` guard below match every
    // other crystal gameplay system. Without them a black hole would eat the
    // remote peer's ghost copy and Auto-Crystal turrets, which are meant to
    // persist between levels via the save path.
    mut crystals: Query<
        (
            Entity,
            &mut ArcaneCrystal,
            &mut Transform,
            Has<CrystalAnchored>,
            Option<&mut CrystalWarded>,
            Has<PrismaticExplosion>,
        ),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
    // Excludes crystals so this `&Transform` access stays disjoint from the
    // `&mut Transform` held above.
    targets: Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<ArcaneCrystal>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    visual_assets: Res<SpellVisualAssets>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();

    for (crystal_entity, mut crystal, mut transform, anchored, mut warded, has_prismatic) in
        &mut crystals
    {
        if crystal.permanent {
            continue;
        }
        for black_hole in &black_holes {
            let to_bh = black_hole.position - crystal.position;
            let distance = to_bh.length();

            if black_hole.contains_point(crystal.position) {
                // Guardian Circle's ward spends a charge to survive the horizon.
                // Surviving is not enough on its own: the crystal is still inside
                // the sphere, so without shoving it clear it would be consumed
                // again next frame and the ward would have bought one frame.
                if let Some(ward) = warded.as_deref_mut()
                    && ward.absorb()
                {
                    let escape = (-to_bh).try_normalize().unwrap_or(Vec3::X)
                        * (black_hole.current_radius + WARD_ESCAPE_MARGIN);
                    crystal.position = black_hole.position + escape;
                    crystal.position.y = transform.translation.y;
                    transform.translation = crystal.position;
                    continue;
                }
                // Go out with a bang rather than blinking out of existence.
                // Prismatic Explosion pays full price; an untalented crystal
                // still gets the smaller shatter burst.
                let (damage, radius) = if has_prismatic {
                    (PRISMATIC_EXPLOSION_DAMAGE, PRISMATIC_EXPLOSION_RADIUS)
                } else {
                    (
                        SHATTER_BASE_DAMAGE * INFUSION_DURATION_SCALE,
                        SHATTER_RADIUS,
                    )
                };
                crystal_aoe_burst(
                    &mut commands,
                    &visual_assets,
                    crystal.position,
                    crystal.range,
                    damage * crystal.empowerment,
                    radius,
                    3.0,
                    0.5,
                    &targets,
                    &mut health_query,
                    caster_team,
                );

                destroy_crystal(&mut commands, crystal_entity, &crystal, &indicators);
                break;
            }

            // Wall of Dirt roots the crystal against gravity.
            if anchored {
                continue;
            }

            if distance > 0.01 && distance <= GRAVITY_RANGE {
                let gravity_strength = black_hole.gravitational_strength();
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(2500.0);
                let direction = to_bh.normalize();

                let displacement = direction * pull_strength * delta * 0.01; // Damped movement
                crystal.position += displacement;
                transform.translation = crystal.position;
            }
        }
    }
}
