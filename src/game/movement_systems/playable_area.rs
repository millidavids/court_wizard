//! Invisible walls around the playable area.
//!
//! The battlefield's visual perimeter (right wall, left wall, wall floor) is
//! decorative — it has no collider and the flow field does not treat it as an
//! obstacle. Without a boundary, anything that drives a unit in a straight line
//! (fear, sleepwalking, polymorph wander, the ogre's charge, knockback, boss
//! leaps and teleports) can walk it clean off the map, where the wizard can
//! neither see it nor reach it with spells.
//!
//! Rather than patch each of those behaviours, this enforces a single
//! position-based bound every frame. That is self-correcting: a displacement
//! from any system, in any schedule order, is pulled back within one frame.

use bevy::prelude::*;

use crate::game::components::{Acceleration, Velocity};
use crate::game::multiplayer::components::GhostEntity;
use crate::game::pathfinding::StagingAttacker;
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{Corpse, Hitbox, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::archetypes::swordcerer::components::SwordcererAvatar;

/// The playable area, authored in world XZ against the fixed game camera.
/// Convex, wound so the interior lies to the left of each directed edge.
const PLAYABLE_AREA: [Vec2; 6] = [
    Vec2::new(3000.0, -3000.0),  // A
    Vec2::new(3000.0, 600.0),    // B
    Vec2::new(1100.0, 2100.0),   // C
    Vec2::new(-1200.0, 1100.0),  // D
    Vec2::new(-2400.0, -600.0),  // E
    Vec2::new(-2600.0, -3000.0), // F
];

/// Bosses and the King are held exactly inside the area — they are the units a
/// player must be able to see and target at all times.
const BOSS_KING_SLACK: f32 = 0.0;

/// Every other unit may stray this far past an edge, so ordinary skirmishing
/// along the perimeter doesn't read as units glued to an invisible line.
const UNIT_SLACK: f32 = 200.0;

/// Distance from a unit's own bound at which the steering force starts building.
///
/// Deliberately narrow. Anything a unit can legitimately stand still on — a
/// defender's rally post, the King's spawn — has to sit *outside* this band, or
/// the constant inward force would slowly walk it off its post: the
/// at-destination branch of `calculate_weighted_movement` damps velocity but
/// never clears `Acceleration`, so a steering force applied to a stationary unit
/// still integrates into movement. It also has to stay under the 200-unit rally
/// satisfaction radius (`DEFENDER_SPAWN_RALLY_RADIUS * CELL_SIZE`), otherwise a
/// rallying defender would be pushed out of its own satisfaction radius and
/// oscillate.
const REPULSION_MARGIN: f32 = 80.0;

/// Peak repulsion force at the bound. Same peak as the wall-of-stone repulsion
/// (`wall_collision::REPULSION_FORCE_MAX`), though that one applies it over a
/// much narrower band.
const REPULSION_FORCE_MAX: f32 = 600.0;

/// Ceiling on how far the hard clamp may move a unit in one second.
///
/// Without this, a unit that is a long way out — a wave laggard still at its
/// spawn offset when `StagingAttacker` is stripped, up to 600 units past the
/// bound — would be teleported in a single frame, reading as a pop through the
/// right wall. Capping the correction makes it walk back in over a few frames
/// instead. The bound still holds; it is just reached smoothly.
const MAX_CORRECTION_SPEED: f32 = 1200.0;

/// Sequential-projection passes. Mirrors `enforce_wall_collision`'s iteration
/// count so corners — where pushing off one edge crosses another — resolve.
const CLAMP_PASSES: u32 = 4;

/// A single boundary edge. The interior is `normal.dot(p) >= offset`.
struct HalfPlane {
    normal: Vec2,
    offset: f32,
}

/// The playable area as an intersection of inward half-planes.
///
/// Precomputed once so the per-frame systems only do dot products.
#[derive(Resource)]
pub(in crate::game) struct PlayableArea {
    planes: [HalfPlane; 6],
}

impl Default for PlayableArea {
    fn default() -> Self {
        let centroid =
            PLAYABLE_AREA.iter().fold(Vec2::ZERO, |acc, v| acc + *v) / PLAYABLE_AREA.len() as f32;

        let planes = std::array::from_fn(|i| {
            let a = PLAYABLE_AREA[i];
            let b = PLAYABLE_AREA[(i + 1) % PLAYABLE_AREA.len()];
            let edge = b - a;

            // Left normal of the directed edge.
            let mut normal = Vec2::new(-edge.y, edge.x).normalize();
            let mut offset = normal.dot(a);

            // Flip any normal that would exclude the centroid, so the resource
            // stays correct if the vertex list is ever re-wound or re-authored.
            if normal.dot(centroid) < offset {
                normal = -normal;
                offset = -offset;
            }

            HalfPlane { normal, offset }
        });

        Self { planes }
    }
}

impl PlayableArea {
    /// Returns true when `p` is inside the area, allowing `slack` units of
    /// overshoot past every edge.
    ///
    /// Only the tests need this — the runtime systems always clamp, and clamping
    /// already short-circuits when the point is inside.
    #[cfg(test)]
    fn is_inside(&self, p: Vec2, slack: f32) -> bool {
        self.planes
            .iter()
            .all(|plane| plane.normal.dot(p) >= plane.offset - slack)
    }

    /// Pulls `p` back inside the area, allowing `slack` units of overshoot.
    ///
    /// Projects onto each violated half-plane in turn. Convexity makes this
    /// converge; the repeated passes handle corners where pushing off one edge
    /// pushes the point across another.
    pub(in crate::game) fn clamp(&self, p: Vec2, slack: f32) -> Vec2 {
        let mut p = p;

        for _ in 0..CLAMP_PASSES {
            let mut moved = false;

            for plane in &self.planes {
                let depth = plane.normal.dot(p) - (plane.offset - slack);
                if depth < 0.0 {
                    p -= plane.normal * depth;
                    moved = true;
                }
            }

            if !moved {
                break;
            }
        }

        p
    }
}

/// Picks the slack a unit is entitled to. Bosses and the King get none.
fn slack_for(is_boss: bool, is_king: bool) -> f32 {
    if is_boss || is_king {
        BOSS_KING_SLACK
    } else {
        UNIT_SLACK
    }
}

/// Pulls a resting post — a spawn position or a rally target — far enough inside
/// the playable area that a unit standing on it feels no steering force.
///
/// Clamping to the bound itself would put the post at the point of *maximum*
/// repulsion, so units would be pushed straight back off it. Backing the post
/// off by `REPULSION_MARGIN` makes it a stable equilibrium instead.
fn clamp_to_resting_post(pos: Vec2, slack: f32) -> Vec2 {
    PlayableArea::default().clamp(pos, slack - REPULSION_MARGIN)
}

/// Pulls a defender's spawn position and rally point inside the playable area.
///
/// The back rows of the defender spawn grid sit outside the area. Without this
/// they would spawn out of bounds (and be yanked in on the first frame) and then
/// rally toward a point they can never rest on.
///
/// Builds the half-planes on demand rather than taking the resource, so the
/// spawn helpers don't have to thread it through every call site — this runs
/// once per unit at load time, where six normalizes are free.
pub(in crate::game) fn clamp_defender_post(pos: Vec2) -> Vec2 {
    clamp_to_resting_post(pos, UNIT_SLACK)
}

/// Pulls the King's spawn position and rally point inside the playable area.
/// The King is bound with no slack, so his post must satisfy the tighter bound.
pub(in crate::game) fn clamp_king_post(pos: Vec2) -> Vec2 {
    clamp_to_resting_post(pos, BOSS_KING_SLACK)
}

/// Query filters shared by both systems.
///
/// `With<Velocity>` is the load-bearing exclusion: the wizard carries both
/// `Hitbox` and `Team::Defenders` but no `Velocity`, and it sits well outside
/// the playable area on its castle wall, so it must never be moved. Arrows keep
/// their velocity inside the `Arrow` component and have no `Hitbox`/`Team`, so
/// projectiles still fly past the perimeter and despawn normally.
///
/// `StagingAttacker` covers every unit walking in from the spawn tunnels —
/// including all five bosses — which spawn well outside the area by design.
/// The swordcerer avatar is player-driven and always on screen, and its deploy
/// point can legitimately be next to the castle, so it keeps its own bound.
///
/// Corpses fall out too, since `convert_dead_to_corpses` strips `Hitbox`. That
/// is fine: a corpse dragged off the map by a black hole is invisible clutter
/// that gets despawned anyway, and if something resurrects it the revived unit
/// regains `Hitbox` and is pulled back inside on the next frame.
type BoundedUnit = (
    With<Velocity>,
    With<Hitbox>,
    With<Team>,
    Without<StagingAttacker>,
    Without<GhostEntity>,
    Without<SwordcererAvatar>,
);

/// Steers units away from the perimeter before they reach it, so they turn
/// around naturally instead of grinding along the hard bound.
pub(in crate::game) fn apply_playable_area_avoidance(
    area: Res<PlayableArea>,
    mut units: Query<
        (&Transform, &mut Acceleration, Has<Boss>, Has<King>),
        (BoundedUnit, Without<Corpse>),
    >,
) {
    for (transform, mut acceleration, is_boss, is_king) in &mut units {
        let slack = slack_for(is_boss, is_king);
        let pos = Vec2::new(transform.translation.x, transform.translation.z);

        for plane in &area.planes {
            let depth = plane.normal.dot(pos) - (plane.offset - slack);
            if depth < REPULSION_MARGIN {
                // Linear falloff: full strength at (or past) the bound, zero at
                // the outer edge of the margin.
                let strength = 1.0 - (depth / REPULSION_MARGIN).clamp(0.0, 1.0);
                acceleration.add_force(
                    Vec3::new(plane.normal.x, 0.0, plane.normal.y) * REPULSION_FORCE_MAX * strength,
                );
            }
        }
    }
}

/// Hard backstop. Runs after every system that writes a unit transform, so it
/// catches teleports, leaps, charges and knockback as well as ordinary movement.
/// Position-based enforcement means a displacement from any system, in any
/// schedule order, is corrected within at most one frame.
pub(in crate::game) fn enforce_playable_area(
    time: Res<Time>,
    area: Res<PlayableArea>,
    mut units: Query<(&mut Transform, &mut Velocity, Has<Boss>, Has<King>), BoundedUnit>,
) {
    let max_step = MAX_CORRECTION_SPEED * time.delta_secs();

    for (mut transform, mut velocity, is_boss, is_king) in &mut units {
        let slack = slack_for(is_boss, is_king);
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let target = area.clamp(pos, slack);

        if target == pos {
            continue;
        }

        // Walk a badly out-of-bounds unit back in rather than teleporting it.
        let correction = target - pos;
        let corrected = if correction.length() > max_step {
            pos + correction.normalize() * max_step
        } else {
            target
        };

        transform.translation.x = corrected.x;
        transform.translation.z = corrected.y;

        // Strip the outward component of velocity so the unit slides along the
        // boundary instead of pressing into it every frame.
        let inward = (corrected - pos).normalize_or_zero();
        let vel = Vec2::new(velocity.x, velocity.z);
        let outward = vel.dot(inward);
        if outward < 0.0 {
            let tangential = vel - inward * outward;
            velocity.x = tangential.x;
            velocity.z = tangential.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::constants::{
        DEFENDER_GRID_CENTER_ANGLE, DEFENDER_GRID_COLS, DEFENDER_GRID_GROUND_RANGE,
        DEFENDER_GRID_ROWS, STAGING_POINTS, WIZARD_POSITION, calculate_defender_grid_position,
        defender_spawn_center,
    };

    /// The half-plane representation is only valid for a convex polygon.
    #[test]
    fn polygon_is_convex() {
        let n = PLAYABLE_AREA.len();
        for i in 0..n {
            let a = PLAYABLE_AREA[i];
            let b = PLAYABLE_AREA[(i + 1) % n];
            let c = PLAYABLE_AREA[(i + 2) % n];
            let cross = (b - a).perp_dot(c - b);
            assert!(
                cross > 0.0,
                "vertex {} breaks convexity or winding (cross = {cross})",
                (i + 1) % n
            );
        }
    }

    #[test]
    fn every_vertex_lies_on_the_boundary() {
        let area = PlayableArea::default();
        for (i, vertex) in PLAYABLE_AREA.iter().enumerate() {
            assert!(
                area.is_inside(*vertex, 1.0),
                "vertex {i} is not on the boundary"
            );
        }
    }

    /// Attackers hold here between waves; if a staging point were outside, every
    /// wave would be shoved the moment `StagingAttacker` is removed.
    #[test]
    fn staging_points_are_inside() {
        let area = PlayableArea::default();
        for (i, (x, z)) in STAGING_POINTS.iter().enumerate() {
            assert!(
                area.is_inside(Vec2::new(*x, *z), BOSS_KING_SLACK),
                "staging point {i} at ({x}, {z}) is outside the playable area"
            );
        }
    }

    /// Defenders return here between waves, so it has to stay reachable.
    #[test]
    fn defender_rally_centre_is_inside() {
        let area = PlayableArea::default();
        let (x, z) = defender_spawn_center();
        assert!(area.is_inside(Vec2::new(x, z), UNIT_SLACK));
    }

    /// Every cell of the defender grid — including the back rows that start
    /// outside the area — must end up on a post the perimeter steering force
    /// leaves alone, or defenders would drift off formation every battle.
    #[test]
    fn clamped_defender_posts_are_clear_of_the_repulsion_band() {
        let area = PlayableArea::default();

        for row in 0..DEFENDER_GRID_ROWS {
            for col in 0..DEFENDER_GRID_COLS {
                let (x, z) = calculate_defender_grid_position(row, col);
                let post = clamp_defender_post(Vec2::new(x, z));
                assert!(
                    area.is_inside(post, UNIT_SLACK - REPULSION_MARGIN + 1.0),
                    "defender post row {row} col {col} sits inside the repulsion band"
                );
            }
        }
    }

    /// Same for the King, who is bound with no slack and would otherwise be
    /// slowly walked off his post by the steering force.
    #[test]
    fn clamped_king_post_is_clear_of_the_repulsion_band() {
        let area = PlayableArea::default();
        let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
        let post = clamp_king_post(Vec2::new(
            WIZARD_POSITION.x + radius * DEFENDER_GRID_CENTER_ANGLE.cos(),
            WIZARD_POSITION.z + radius * DEFENDER_GRID_CENTER_ANGLE.sin(),
        ));
        assert!(area.is_inside(post, BOSS_KING_SLACK - REPULSION_MARGIN + 1.0));
    }

    /// The King is bound with no slack, so its rally point must be inside.
    #[test]
    fn king_spawn_is_inside() {
        let area = PlayableArea::default();
        let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
        let x = WIZARD_POSITION.x + radius * DEFENDER_GRID_CENTER_ANGLE.cos();
        let z = WIZARD_POSITION.z + radius * DEFENDER_GRID_CENTER_ANGLE.sin();
        assert!(area.is_inside(Vec2::new(x, z), BOSS_KING_SLACK));
    }

    #[test]
    fn clamp_pulls_outside_points_in_and_leaves_inside_points_alone() {
        let area = PlayableArea::default();

        let inside = Vec2::new(0.0, 0.0);
        assert_eq!(area.clamp(inside, BOSS_KING_SLACK), inside);

        // Attacker spawn, well beyond the +X edge.
        let outside = Vec2::new(3800.0, -375.0);
        let clamped = area.clamp(outside, BOSS_KING_SLACK);
        assert!(area.is_inside(clamped, 1.0));

        // Far off the near corner, which violates two edges at once.
        let corner = Vec2::new(-3000.0, 3000.0);
        assert!(area.is_inside(area.clamp(corner, BOSS_KING_SLACK), 1.0));
    }

    #[test]
    fn slack_widens_the_bound() {
        let area = PlayableArea::default();
        // 100 units past the X = 3000 edge.
        let point = Vec2::new(3100.0, -1000.0);
        assert!(!area.is_inside(point, BOSS_KING_SLACK));
        assert!(area.is_inside(point, UNIT_SLACK));
    }
}
