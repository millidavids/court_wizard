//! Shared utility functions for spell systems.
//!
//! These functions are used across many spell implementations to avoid duplication.

use std::collections::HashSet;

use bevy::prelude::*;
use rand::Rng;

use bevy::prelude::Annulus;

use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Health, Team};
use crate::game::units::wizard::components::{CastingState, SpellCaster, Wizard, WizardInput};
use crate::networking::resources::PeerRole;
use crate::networking::session::MultiplayerSession;

/// World-space position the **local** player's spells originate from. Single-
/// player and the multiplayer host use `SPELL_ORIGIN`; the multiplayer guest
/// uses `SPELL_2_ORIGIN`. Set at app startup to the SP default and overridden
/// by `init_mp_game` based on the peer's role.
#[derive(Resource, Debug, Clone, Copy)]
pub struct LocalSpellOrigin(pub Vec3);

impl Default for LocalSpellOrigin {
    fn default() -> Self {
        Self(SPELL_ORIGIN)
    }
}

/// The team the **local** player commands: `Defenders` in single-player and as
/// the multiplayer host, `Attackers` as the guest. Enemy-targeting spells should
/// filter with `unit_team.is_enemy(&local_player_team(session.as_deref()))`
/// instead of hardcoding `Team::Attackers` (which only resolves correctly for the
/// host, making the guest target its own army).
pub fn local_player_team(session: Option<&MultiplayerSession>) -> Team {
    match session.map(|s| s.role) {
        Some(PeerRole::Guest) => Team::Attackers,
        _ => Team::Defenders,
    }
}

// ----- Lock-free snapshot of the local spell origin for non-system callers -----
//
// Some helper code paths (audio attenuation, const-derived gunslinger spawn
// positions) cannot easily take `Res<LocalSpellOrigin>` as a system param
// because they are either plain functions or const-context expressions. They
// read this snapshot instead. `LocalSpellOriginSyncPlugin` keeps it updated
// from the `LocalSpellOrigin` resource so the multiplayer guest sees their
// own wizard's value.

use std::sync::atomic::{AtomicU32, Ordering};

// Initialised directly to `SPELL_ORIGIN`'s bit pattern so the snapshot is
// already correct for SP / MP host before any system runs. The previous
// `Once`-based lazy init had a race: if `update_local_spell_origin_snapshot`
// wrote SPELL_2_ORIGIN first (guest), a later reader's `call_once` would
// stomp it back to SPELL_ORIGIN — permanently, for the rest of the process.
static LOCAL_ORIGIN_X: AtomicU32 = AtomicU32::new(SPELL_ORIGIN.x.to_bits());
static LOCAL_ORIGIN_Y: AtomicU32 = AtomicU32::new(SPELL_ORIGIN.y.to_bits());
static LOCAL_ORIGIN_Z: AtomicU32 = AtomicU32::new(SPELL_ORIGIN.z.to_bits());

pub(crate) fn set_local_spell_origin_snapshot(pos: Vec3) {
    LOCAL_ORIGIN_X.store(pos.x.to_bits(), Ordering::Relaxed);
    LOCAL_ORIGIN_Y.store(pos.y.to_bits(), Ordering::Relaxed);
    LOCAL_ORIGIN_Z.store(pos.z.to_bits(), Ordering::Relaxed);
}

pub(crate) fn local_spell_origin_snapshot() -> Vec3 {
    Vec3::new(
        f32::from_bits(LOCAL_ORIGIN_X.load(Ordering::Relaxed)),
        f32::from_bits(LOCAL_ORIGIN_Y.load(Ordering::Relaxed)),
        f32::from_bits(LOCAL_ORIGIN_Z.load(Ordering::Relaxed)),
    )
}

/// Bevy system: pushes the current `LocalSpellOrigin` resource into the
/// lock-free snapshot whenever it changes.
pub(crate) fn update_local_spell_origin_snapshot(local_origin: Res<LocalSpellOrigin>) {
    set_local_spell_origin_snapshot(local_origin.0);
}

/// Returns the XZ-plane distance between two points (ignoring Y).
pub(crate) fn xz_distance(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length()
}

/// Tests whether a sphere intersects a vertical cylinder (unit hitbox).
///
/// The cylinder extends from ground (Y=0) to `cylinder_height` at `cylinder_pos`,
/// with the given `cylinder_radius`. The sphere is centered at `sphere_center`
/// with `sphere_radius`.
///
/// Algorithm: clamp the sphere center's Y to the cylinder's Y range, then check
/// whether the 2D (XZ) distance is less than the sum of both radii.
pub(crate) fn sphere_intersects_cylinder(
    sphere_center: Vec3,
    sphere_radius: f32,
    cylinder_pos: Vec3,
    cylinder_radius: f32,
    cylinder_height: f32,
) -> bool {
    // Clamp sphere Y to cylinder vertical range
    let clamped_y = sphere_center
        .y
        .clamp(cylinder_pos.y, cylinder_pos.y + cylinder_height);
    let dy = sphere_center.y - clamped_y;

    // XZ distance between centers
    let dx = sphere_center.x - cylinder_pos.x;
    let dz = sphere_center.z - cylinder_pos.z;
    let xz_dist_sq = dx * dx + dz * dz;

    // Combined horizontal reach
    let combined_radius = sphere_radius + cylinder_radius;

    // Full distance check: horizontal + vertical
    xz_dist_sq + dy * dy <= combined_radius * combined_radius
}

/// Returns the shortest XZ-plane distance from a point to a line segment defined by start/end.
pub(crate) fn distance_to_line_segment_xz(point: Vec3, start: Vec3, end: Vec3) -> f32 {
    let p = Vec2::new(point.x, point.z);
    let a = Vec2::new(start.x, start.z);
    let b = Vec2::new(end.x, end.z);
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_squared();
    if ab_len_sq < 0.0001 {
        return ap.length();
    }
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}

/// Resource representing a pending heal to be applied to the nearest injured defender.
///
/// Used by spells that deal damage and heal defenders as a side-effect (e.g., Void Siphon,
/// Siphon Life). The heal is deferred to a separate system to avoid query conflicts when
/// both the damage system and healing system need mutable access to Health components.
#[derive(Resource)]
pub(crate) struct PendingDefenderHeal {
    /// Total heal amount to apply.
    pub amount: f32,
    /// World-space origin position (for finding the nearest injured defender).
    pub origin: Vec3,
}

/// Applies wizard spell healing to a unit and records it for the multiplayer
/// score screen's per-wizard heal tally. Returns the HP actually restored
/// (after `healing_reduction` and clamping to `health.max`).
///
/// Call this from wizard heal spells instead of `health.heal(amount)`. Non-spell
/// heal sources (healer units, cauldron regen, melee lifesteal) keep calling
/// `health.heal` directly so they are excluded from the wizard's stat. Like the
/// damage tally, this only fires on the peer running the spell, so it captures
/// exactly the local wizard's healing output.
pub(crate) fn apply_spell_heal(
    commands: &mut Commands,
    entity: Entity,
    health: &mut Health,
    amount: f32,
) -> f32 {
    let before = health.current;
    health.heal(amount);
    let actual = health.current - before;
    if actual > 0.0 {
        // `entry` + `and_modify` accumulates multiple same-frame heals on one
        // unit instead of overwriting; the score-screen consumer clears it each frame.
        commands
            .entity(entity)
            .entry::<crate::game::units::spell_stats::SpellHealTally>()
            .and_modify(move |mut tally| tally.0 += actual)
            .or_insert(crate::game::units::spell_stats::SpellHealTally(actual));
    }
    actual
}

/// Applies a pending defender heal to the nearest injured defender, then removes the resource.
///
/// Finds the closest defender (by world-space distance to `origin`) that is alive but not
/// at full health, and heals them for the pending amount.
pub(crate) fn apply_pending_defender_heal(
    mut commands: Commands,
    pending: Option<Res<PendingDefenderHeal>>,
    mut defenders: Query<(Entity, &Transform, &mut Health, &Team), Without<Wizard>>,
) {
    let Some(heal) = pending else {
        return;
    };

    let mut best: Option<(Entity, f32)> = None;
    for (entity, transform, health, team) in defenders.iter() {
        if *team != Team::Defenders || health.current <= 0.0 || health.current >= health.max {
            continue;
        }
        let dist = transform.translation.distance(heal.origin);
        match best {
            None => best = Some((entity, dist)),
            Some((_, best_dist)) if dist < best_dist => best = Some((entity, dist)),
            _ => {}
        }
    }

    if let Some((entity, _)) = best
        && let Ok((_, _, mut health, _)) = defenders.get_mut(entity)
    {
        apply_spell_heal(&mut commands, entity, &mut health, heal.amount);
    }

    commands.remove_resource::<PendingDefenderHeal>();
}

/// Projects the cursor position onto the Y=0 ground plane via raycasting.
///
/// Returns the world-space intersection point of the camera ray through the cursor
/// with the horizontal ground plane (Y=0), or `None` if the cursor is not over the window,
/// the ray is parallel to the ground, or the intersection is behind the camera.
///
/// Uses the barrel-distortion-corrected cursor position so that raycasting
/// matches the visually distorted CRT output.
pub(crate) fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: &Res<CorrectedCursorPosition>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let cursor_pos = corrected_cursor.0?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    // Check if ray is parallel to ground plane
    if ray.direction.y.abs() < 0.0001 {
        return None;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return None; // Intersection is behind camera
    }

    Some(ray.origin + ray.direction * t)
}

/// Clamps a target position to be within the wizard's spell range using 3D distance.
///
/// If the target is beyond `spell_range` from `wizard_pos`, it is moved along the
/// direction vector to sit exactly at `spell_range` distance.
pub(crate) fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32) -> Vec3 {
    let diff = target - wizard_pos;
    let distance = diff.length();

    if distance > spell_range {
        wizard_pos + diff.normalize() * spell_range
    } else {
        target
    }
}

/// Computes the ground-plane radius of a spell range circle, accounting for wizard height.
///
/// The wizard sits at height `wizard_height` above the ground. A spell with 3D range
/// `spell_range` can reach a ground circle of radius `sqrt(spell_range² - wizard_height²)`.
/// Returns 0.0 if the wizard is higher than the spell range.
pub(crate) fn ground_projected_range(spell_range: f32, wizard_height: f32) -> f32 {
    if wizard_height < spell_range {
        (spell_range * spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    }
}

/// Clamps a target position to be within the wizard's spell range on the ground plane,
/// accounting for the wizard's height above ground and an optional effect radius.
///
/// Uses the Pythagorean theorem to compute the maximum ground-plane radius from the
/// wizard's XZ position. If `effect_radius` is non-zero, the clamp ensures the entire
/// effect circle stays within range.
pub(crate) fn clamp_to_spell_range_ground(
    target: Vec3,
    wizard_pos: Vec3,
    spell_range: f32,
    effect_radius: f32,
) -> Vec3 {
    let max_ground_radius = ground_projected_range(spell_range, wizard_pos.y);

    // Account for effect radius so entire circle stays within range
    let max_center_distance = (max_ground_radius - effect_radius).max(0.0);

    // Calculate XZ plane distance from wizard to target
    let direction = target - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        wizard_pos + normalized_direction * max_center_distance
    } else {
        target
    }
}

/// Like the (deprecated) un-suffixed variant but accepts an explicit local origin —
/// callers should pass the `LocalSpellOrigin` resource so the clamp is computed
/// from the correct wizard position on both the host and the guest.
pub(crate) fn clamp_cursor_to_spell_range_with_origin(
    cursor_pos: Option<Vec3>,
    local_origin: Vec3,
    spell_range: f32,
    effect_radius: f32,
) -> Option<Vec3> {
    let pos = cursor_pos?;
    Some(clamp_to_spell_range_ground(
        pos,
        local_origin,
        spell_range,
        effect_radius,
    ))
}

/// Y position for all spell aiming reticles (hovering just above the ground).
pub(crate) const RETICLE_Y: f32 = 2.0;

/// Fixed world-space width of the reticle ring (in world units).
/// The annulus inner radius is computed per-spawn so this width stays constant
/// regardless of the spell's radius.
const RETICLE_RING_WIDTH: f32 = 3.0;

/// Resolution (number of segments) for the reticle annulus mesh.
const RETICLE_RING_RESOLUTION: u32 = 48;

/// Creates a reticle annulus mesh with a fixed world-space ring width for the given radius.
pub(crate) fn make_reticle_mesh(radius: f32) -> Mesh {
    let inner = (1.0 - RETICLE_RING_WIDTH / radius).max(0.0);
    Annulus::new(inner, 1.0)
        .mesh()
        .resolution(RETICLE_RING_RESOLUTION)
        .into()
}

/// Computes a standard pulse scale factor for circle indicators.
///
/// Returns a value oscillating around 1.0 with a small amplitude, creating
/// a subtle breathing/pulsing effect on the indicator circle.
pub(crate) fn indicator_pulse_scale(time_alive: f32) -> f32 {
    let pulse_freq = 2.0;
    let pulse_amplitude = 0.05;
    1.0 + (time_alive * pulse_freq * std::f32::consts::TAU).sin() * pulse_amplitude
}

/// Shared aiming reticle component for all spells.
///
/// Attached to the annulus ring entity spawned during casting. Tracks position
/// and effective radius; the shared [`update_spell_indicators`] system handles
/// pulse animation and transform updates.
#[derive(Component)]
pub(crate) struct SpellCircleIndicator {
    /// World-space position (XZ plane). Updated by spell casting systems.
    pub position: Vec3,
    /// Time this indicator has been alive (drives pulse animation).
    pub time_alive: f32,
    /// Pre-computed radius including empowerment and talent multipliers.
    pub effective_radius: f32,
}

impl SpellCircleIndicator {
    pub fn new(position: Vec3, effective_radius: f32) -> Self {
        Self {
            position,
            time_alive: 0.0,
            effective_radius,
        }
    }
}

/// Spawns a spell aiming reticle (annulus ring) on the ground plane.
///
/// The annulus mesh is generated per-spawn so that the ring width is constant
/// in world space regardless of the spell radius.
///
/// Inserts the shared [`SpellCircleIndicator`] component. The caller can chain
/// `.insert(...)` to add spell-specific marker components if needed.
pub(crate) fn spawn_circle_indicator<'a>(
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    position: Vec3,
    radius: f32,
) -> EntityCommands<'a> {
    let mesh = meshes.add(make_reticle_mesh(radius));
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(position.x, RETICLE_Y, position.z))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(radius)),
        SpellCircleIndicator::new(position, radius),
        OnGameplayScreen,
    ))
}

/// Shared system that updates all spell aiming reticles: pulse animation and position tracking.
pub(crate) fn update_spell_indicators(
    time: Res<Time>,
    mut indicators: Query<(&mut SpellCircleIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let pulse = indicator_pulse_scale(indicator.time_alive);
        transform.scale = Vec3::splat(indicator.effective_radius * pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = RETICLE_Y;
        transform.translation.z = indicator.position.z;
    }
}

// --- Animated ring particle VFX (shared by entangle and spike growth) ---

/// Seconds between spawning a new animated ring particle.
pub(crate) const RING_SPAWN_INTERVAL: f32 = 0.08;
/// Lifetime of each animated ring (seconds).
const RING_LIFETIME: f32 = 2.4;
/// Minimum max-scale for animated rings.
const RING_MIN_SCALE: f32 = 8.0;
/// Maximum max-scale for animated rings.
const RING_MAX_SCALE: f32 = 28.0;
/// Maximum tilt from horizontal for animated rings (radians).
const RING_MAX_TILT: f32 = 1.0;

/// Animated ring particle that grows outward and shrinks back.
/// Shared by entangle vine rings and spike growth vine/spike rings.
#[derive(Component)]
pub(crate) struct AnimatedRingParticle {
    pub time_alive: f32,
    pub lifetime: f32,
    pub max_scale: f32,
}

/// Spawns a single animated ring particle at a random position within a circle.
pub(crate) fn spawn_ring_particle(
    rng: &mut impl Rng,
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    center: Vec3,
    radius: f32,
) {
    let angle = rng.random::<f32>() * std::f32::consts::TAU;
    let dist = radius * rng.random::<f32>().sqrt() * 0.9;
    let x = center.x + angle.cos() * dist;
    let z = center.z + angle.sin() * dist;

    let yaw = rng.random::<f32>() * std::f32::consts::TAU;
    let tilt = 0.3 + rng.random::<f32>() * RING_MAX_TILT;
    let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(tilt);

    let max_scale = RING_MIN_SCALE + rng.random::<f32>() * (RING_MAX_SCALE - RING_MIN_SCALE);
    let y = -max_scale * 0.3 * tilt.sin();
    let lifetime = RING_LIFETIME * (0.7 + rng.random::<f32>() * 0.6);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(x, y, z))
            .with_rotation(rotation)
            .with_scale(Vec3::ZERO),
        AnimatedRingParticle {
            time_alive: 0.0,
            lifetime,
            max_scale,
        },
        OnGameplayScreen,
    ));
}

/// Animates ring particles: grow outward then shrink, despawn when lifetime expires.
pub(crate) fn animate_ring_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut rings: Query<(Entity, &mut AnimatedRingParticle, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut ring, mut transform) in &mut rings {
        ring.time_alive += delta;
        let progress = (ring.time_alive / ring.lifetime).clamp(0.0, 1.0);

        // Grow in first 40% (ease-out), hold 40-60%, shrink last 40% (ease-in)
        let scale_fraction = if progress < 0.4 {
            let t = progress / 0.4;
            1.0 - (1.0 - t) * (1.0 - t)
        } else if progress < 0.6 {
            1.0
        } else {
            let t = (progress - 0.6) / 0.4;
            (1.0 - t) * (1.0 - t)
        };
        transform.scale = Vec3::splat(ring.max_scale * scale_fraction);

        if ring.time_alive >= ring.lifetime {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Tracks unique entities hit by a persistent spell effect for talent progress.
///
/// Attach this component to any persistent spell entity (zone, beam, cloud, etc.)
/// that tracks talent progress. Call [`track_hit`] when a unit is damaged — it
/// returns `true` only for the first hit on each entity, preventing inflated
/// progress from repeated per-tick counting.
#[derive(Component, Default)]
pub(crate) struct UniqueHitTracker {
    hits: HashSet<Entity>,
}

impl UniqueHitTracker {
    /// Records a hit on the given entity. Returns `true` if this is the first
    /// time this entity has been hit by this spell instance.
    pub fn track_hit(&mut self, entity: Entity) -> bool {
        self.hits.insert(entity)
    }
}

// --- Shared spell casting helpers ---
// These functions extract the boilerplate duplicated across 15+ spell casting systems.

/// Pre-computed target-assist snap position, updated once per frame by
/// [`compute_target_assist`]. Spells read this via [`build_wizard_input`].
#[derive(Resource, Default)]
pub(crate) struct TargetAssistWorldPos(pub Option<Vec3>);

/// Each frame, finds the nearest living unit to the cursor and stores its position
/// if within the configured snap radius. When targeting_assistance is 0, clears the snap.
#[allow(clippy::too_many_arguments)]
pub(crate) fn compute_target_assist(
    mut assist: ResMut<TargetAssistWorldPos>,
    config: Res<crate::config::GameConfig>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    units: Query<
        &Transform,
        (
            With<Health>,
            Without<super::super::super::components::Corpse>,
        ),
    >,
) {
    let snap_radius = config.target_assist_snap_radius();
    if snap_radius <= 0.0 {
        assist.0 = None;
        return;
    }

    let Some(cursor_world) = get_cursor_world_position(&camera_query, &corrected_cursor) else {
        assist.0 = None;
        return;
    };

    let mut best_dist = snap_radius;
    let mut best_pos: Option<Vec3> = None;

    for transform in &units {
        let unit_pos = transform.translation;
        let dist = xz_distance(cursor_world, unit_pos);
        if dist < best_dist {
            best_dist = dist;
            best_pos = Some(Vec3::new(unit_pos.x, 0.0, unit_pos.z));
        }
    }

    assist.0 = best_pos;
}

/// Builds a `WizardInput` from mouse state and camera raycasting.
///
/// Every spell casting system constructs an identical `WizardInput` — this
/// centralizes that logic. When targeting assistance is active, snaps the
/// cursor to the nearest unit via [`TargetAssistWorldPos`].
pub(crate) fn build_wizard_input(
    mouse_left_released: &mut MessageReader<MouseLeftReleased>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: &Res<CorrectedCursorPosition>,
) -> WizardInput {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(camera_query, corrected_cursor);
    WizardInput {
        just_pressed: true, // Run conditions already ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    }
}

/// Applies targeting assistance snap to a `WizardInput`. Call after `build_wizard_input`.
pub(crate) fn apply_target_assist(input: &mut WizardInput, assist: &TargetAssistWorldPos) {
    if let Some(snap_pos) = assist.0
        && input.cursor_pos.is_some()
    {
        input.cursor_pos = Some(snap_pos);
    }
}

/// Cleans up the spell caster indicator and removes the `SpellCaster` component.
///
/// Called on release, completion, and channeling-cancel — identical logic every time.
pub(crate) fn cleanup_spell_caster(
    commands: &mut Commands,
    wizard_entity: Entity,
    caster_query: &Query<&SpellCaster>,
) {
    if let Ok(caster) = caster_query.get(wizard_entity) {
        if let Some(indicator_entity) = caster.indicator_entity {
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
    }
}

/// Handles the mouse-release event during casting: cleans up indicator and cancels cast.
///
/// Returns `true` if released (caller should return early), `false` otherwise.
pub(crate) fn handle_spell_release(
    input: &WizardInput,
    commands: &mut Commands,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    caster_query: &Query<&SpellCaster>,
) -> bool {
    if input.just_released {
        cleanup_spell_caster(commands, wizard_entity, caster_query);
        casting_state.cancel();
        return true;
    }
    false
}

/// Updates the spell circle indicator position to track the cursor.
///
/// Used in the `CastingState::Casting` arm of every spell.
pub(crate) fn update_indicator_position(
    wizard_entity: Entity,
    cursor_pos: Vec3,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
) {
    if let Ok(caster) = caster_query.get(wizard_entity)
        && let Some(indicator_entity) = caster.indicator_entity
        && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
    {
        indicator.position = cursor_pos;
    }
}

/// Spawns a spell circle indicator in the Resting state and starts the cast.
///
/// Returns `true` if the indicator was spawned (cast started), `false` otherwise.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_start_cast_with_indicator(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    indicator_material: Handle<StandardMaterial>,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &crate::game::units::wizard::components::Mana,
    mana_cost: f32,
    cursor_pos: Vec3,
    indicator_radius: f32,
    caster_query: &Query<&SpellCaster>,
) -> bool {
    if caster_query.get(wizard_entity).is_err() && mana.can_afford(mana_cost) {
        let circle_entity = spawn_circle_indicator(
            commands,
            meshes,
            indicator_material,
            cursor_pos,
            indicator_radius,
        )
        .id();
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
        casting_state.start_cast();
        true
    } else {
        false
    }
}

/// Trait for cooldown-style components with a `remaining: f32` countdown field.
///
/// Implementing this trait lets a component be ticked by the generic
/// [`tick_spell_cooldown`] system, eliminating the need for a per-cooldown
/// tick function.
pub(crate) trait HasCooldownRemaining {
    fn remaining_mut(&mut self) -> &mut f32;
}

/// Generic per-frame countdown system. Decrements the `remaining` field of
/// every entity's cooldown component, and removes the component when it expires.
///
/// Register one instance per cooldown type, e.g.:
/// `app.add_systems(Update, tick_spell_cooldown::<MagicMissileCooldown>)`.
pub(crate) fn tick_spell_cooldown<C>(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut C)>,
) where
    C: Component<Mutability = bevy::ecs::component::Mutable> + HasCooldownRemaining,
{
    for (entity, mut cd) in &mut cooldowns {
        let remaining = cd.remaining_mut();
        *remaining -= time.delta_secs();
        if *remaining <= 0.0 {
            commands.entity(entity).remove::<C>();
        }
    }
}

/// Ticks `GlobalCastCooldown` down each frame, removing the component when
/// it expires so spell input is unblocked.
pub(crate) fn tick_global_cast_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut crate::game::units::wizard::components::GlobalCastCooldown,
    )>,
) {
    for (entity, mut cd) in &mut query {
        cd.remaining -= time.delta_secs();
        if cd.remaining <= 0.0 {
            commands
                .entity(entity)
                .remove::<crate::game::units::wizard::components::GlobalCastCooldown>();
        }
    }
}

/// Detects charge-spell completion (CastingState just went from `Casting`
/// to `Resting` AND mana decreased on the same frame) and inserts a global
/// cooldown on the local wizard. Channel-release returns `Channeling` →
/// `Resting` (not `Casting` → `Resting`) so channeled spells don't trigger
/// the cooldown. Cancellation (release before cast_time) doesn't consume
/// mana, so the mana check also excludes that case.
pub(crate) fn insert_global_cooldown_on_cast(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &crate::game::units::wizard::components::CastingState,
            &crate::game::units::wizard::components::Mana,
        ),
        With<crate::game::units::wizard::components::LocalWizard>,
    >,
    mut prev_state: Local<Option<crate::game::units::wizard::components::CastingState>>,
    mut prev_mana: Local<Option<f32>>,
) {
    let Ok((entity, state, mana)) = query.single() else {
        return;
    };
    let mana_decreased = prev_mana.is_some_and(|p| mana.current < p - f32::EPSILON);
    let was_casting = matches!(
        *prev_state,
        Some(crate::game::units::wizard::components::CastingState::Casting { .. })
    );
    let now_resting = matches!(
        state,
        crate::game::units::wizard::components::CastingState::Resting
    );
    if was_casting && now_resting && mana_decreased {
        commands.entity(entity).insert(
            crate::game::units::wizard::components::GlobalCastCooldown {
                remaining: crate::game::units::wizard::components::GLOBAL_CAST_COOLDOWN_SECS,
            },
        );
    }
    *prev_state = Some(*state);
    *prev_mana = Some(mana.current);
}

/// Writes `BlockSpellInput` whenever the local wizard has a
/// `GlobalCastCooldown`, so every spell's `spell_input_not_blocked`
/// run_if rejects while the cooldown is active.
pub(crate) fn block_spells_during_global_cooldown(
    query: Query<
        (),
        (
            With<crate::game::units::wizard::components::LocalWizard>,
            With<crate::game::units::wizard::components::GlobalCastCooldown>,
        ),
    >,
    mut block: MessageWriter<crate::game::input::messages::BlockSpellInput>,
) {
    if !query.is_empty() {
        block.write(crate::game::input::messages::BlockSpellInput);
    }
}
