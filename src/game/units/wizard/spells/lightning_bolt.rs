//! Shared jagged lightning bolt visual.
//!
//! Spawns a parent entity per bolt. Each frame `update_lightning_bolts` despawns
//! the bolt's children and respawns a freshly randomized jagged path (plus a few
//! short fork branches). This produces a crackling, meandering look used by
//! Chain Lightning, Lightning Rod (sky strike + ground arcs), and the hag's
//! lightning ability.
//!
//! Callers spawn bolts via `spawn_lightning_bolt` and may move bolt endpoints
//! by mutating `LightningBolt::start` / `LightningBolt::end` (e.g. the
//! descending Lightning Rod sky strike). The crackle system picks up the new
//! endpoints automatically the next frame.
//!
//! The pulsing material formula `0.7 + 0.3 * sin(time_alive * 20)` is centralized
//! here, replacing the per-spell duplication that previously lived in
//! `chain_lightning::chain` and `lightning_rod::lifecycle`.

use bevy::prelude::*;
use rand::Rng;

use crate::game::components::OnGameplayScreen;

/// Tuning for a single bolt visual.
#[derive(Clone, Copy)]
pub(crate) struct LightningBoltConfig {
    /// World-space width of the main bolt segments.
    pub width: f32,
    /// Total visible duration in seconds (active crackling phase).
    pub lifetime: f32,
    /// Optional parabolic arch height at the midpoint (0.0 for a straight axis).
    pub peak_height: f32,
    /// Maximum perpendicular jitter offset applied to interior path points.
    pub jitter_amplitude: f32,
    /// Number of straight quad segments along the main bolt.
    pub segments: u32,
    /// Number of short fork branches that split off the main path each frame.
    pub fork_count: u32,
    /// Number of straight quad segments per fork.
    pub fork_segments: u32,
    /// Approximate length of each fork in world units.
    pub fork_length: f32,
    /// How long the bolt lingers as a fading "shadow" after `lifetime` expires.
    /// During this phase the path stops crackling and the bolt's cloned
    /// material fades alpha + dims toward black, mimicking the retinal
    /// afterimage you see after a real lightning flash. 0.0 disables.
    pub afterimage_duration: f32,
}

/// Parent component for a jagged lightning bolt visual. Children (segment quads)
/// are regenerated every frame during the active phase, then frozen and faded
/// during the afterimage phase.
#[derive(Component)]
pub(crate) struct LightningBolt {
    pub start: Vec3,
    pub end: Vec3,
    pub time_alive: f32,
    pub lifetime: f32,
    pub afterimage_remaining: f32,
    pub in_afterimage: bool,
    pub config: LightningBoltConfig,
    pub mesh: Handle<Mesh>,
    /// Per-bolt cloned material handle (so each bolt fades independently
    /// during its afterimage). Cloned lazily on the first update tick from
    /// the source handle that the caller passed in.
    pub material: Handle<StandardMaterial>,
    /// Cached base color of the cloned material, used as the reference value
    /// the afterimage fade scales against.
    pub base_color: Color,
    /// Whether `material` has been replaced with a per-bolt clone yet.
    pub material_cloned: bool,
}

/// Spawns a jagged lightning bolt parent entity. Children (quad segments)
/// will be created and refreshed every frame by `update_lightning_bolts`.
///
/// The parent has an identity Transform and inherited Visibility so child
/// world-space transforms render unmodified.
pub(crate) fn spawn_lightning_bolt(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    start: Vec3,
    end: Vec3,
    config: LightningBoltConfig,
) -> Entity {
    commands
        .spawn((
            LightningBolt {
                start,
                end,
                time_alive: 0.0,
                lifetime: config.lifetime,
                afterimage_remaining: config.afterimage_duration,
                in_afterimage: false,
                config,
                mesh,
                material,
                base_color: Color::WHITE,
                material_cloned: false,
            },
            Transform::default(),
            Visibility::Inherited,
            OnGameplayScreen,
        ))
        .id()
}

/// Advances bolt timers and rebuilds the jagged child segment set every frame.
/// The jitter is reseeded each tick to produce a crackling animation; the bolt
/// also subtly thickens/thins on a sine wave for an electric flicker. When
/// the active lifetime expires, the bolt freezes its last jagged snapshot and
/// fades that snapshot's material over `afterimage_duration` for the
/// retinal-afterimage effect.
pub(crate) fn update_lightning_bolts(
    time: Res<Time>,
    mut commands: Commands,
    mut bolts: Query<(Entity, &mut LightningBolt)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    for (entity, mut bolt) in &mut bolts {
        // Lazily clone the source material so each bolt can fade independently
        // during its afterimage without affecting other live bolts that share
        // the original handle. Only flag as cloned on success so a bolt that
        // spawns before the asset is loaded retries next frame.
        if !bolt.material_cloned
            && let Some(src) = materials.get(&bolt.material)
        {
            let mut cloned = src.clone();
            cloned.alpha_mode = AlphaMode::Blend;
            bolt.base_color = cloned.base_color;
            bolt.material = materials.add(cloned);
            bolt.material_cloned = true;
        }

        if bolt.in_afterimage {
            if bolt.afterimage_remaining <= 0.0 {
                continue;
            }
            bolt.afterimage_remaining -= dt;
            // Retinal-afterimage fade: brightness drops fast, alpha lingers.
            if let Some(mat) = materials.get_mut(&bolt.material) {
                let t =
                    (bolt.afterimage_remaining / bolt.config.afterimage_duration).clamp(0.0, 1.0);
                let base = bolt.base_color.to_srgba();
                let dim = 0.3 + 0.7 * t.powf(2.0);
                mat.base_color = Color::srgba(
                    base.red * dim,
                    base.green * dim,
                    base.blue * dim,
                    base.alpha * t,
                );
            }
            continue;
        }

        bolt.time_alive += dt;
        bolt.lifetime -= dt;

        if bolt.lifetime <= 0.0 {
            if bolt.config.afterimage_duration > 0.0 {
                // Final pose at the locked-in start/end, with forks, so the
                // frozen afterimage matches the bolt's impact frame.
                commands.entity(entity).despawn_related::<Children>();
                let path = generate_jagged_path(
                    bolt.start,
                    bolt.end,
                    bolt.config.segments,
                    bolt.config.jitter_amplitude,
                    bolt.config.peak_height,
                    &mut rng,
                );
                spawn_segments(
                    &mut commands,
                    entity,
                    &bolt.mesh,
                    &bolt.material,
                    &path,
                    bolt.config.width,
                );
                spawn_forks(
                    &mut commands,
                    entity,
                    &bolt.mesh,
                    &bolt.material,
                    &path,
                    &bolt.config,
                    bolt.config.width,
                    &mut rng,
                );
                bolt.in_afterimage = true;
            }
            continue;
        }

        commands.entity(entity).despawn_related::<Children>();

        let LightningBolt {
            start,
            end,
            config,
            mesh,
            material,
            ..
        } = &*bolt;

        let path = generate_jagged_path(
            *start,
            *end,
            config.segments,
            config.jitter_amplitude,
            config.peak_height,
            &mut rng,
        );
        // Subtle width flicker keeps the bolt feeling electric even when the
        // jitter happens to land tame on a given frame.
        let flicker = 0.85 + 0.15 * (bolt.time_alive * 35.0).sin();
        let frame_width = config.width * flicker;
        spawn_segments(&mut commands, entity, mesh, material, &path, frame_width);
        spawn_forks(
            &mut commands,
            entity,
            mesh,
            material,
            &path,
            config,
            frame_width,
            &mut rng,
        );
    }
}

/// Picks `config.fork_count` random interior points on `path` and spawns a
/// short, half-width jagged branch from each. Forks bias perpendicular to the
/// local path direction with a slight outward tilt.
#[allow(clippy::too_many_arguments)]
fn spawn_forks(
    commands: &mut Commands,
    parent: Entity,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    path: &[Vec3],
    config: &LightningBoltConfig,
    main_width: f32,
    rng: &mut impl Rng,
) {
    if config.fork_count == 0 || path.len() < 3 {
        return;
    }
    for _ in 0..config.fork_count {
        let idx = rng.random_range(1..path.len() - 1);
        let origin = path[idx];
        let along = (path[idx + 1] - path[idx - 1]).normalize_or_zero();
        if along == Vec3::ZERO {
            continue;
        }
        let mut perp = along.cross(Vec3::Y);
        if perp.length_squared() < 1e-3 {
            perp = along.cross(Vec3::X);
        }
        let perp = perp.normalize_or_zero();
        if perp == Vec3::ZERO {
            continue;
        }
        let sign = if rng.random_range(0.0..1.0) < 0.5 {
            -1.0
        } else {
            1.0
        };
        let fork_dir = (perp * sign + along * rng.random_range(-0.3..0.3)).normalize_or_zero();
        let fork_end = origin + fork_dir * config.fork_length;
        let fork_path = generate_jagged_path(
            origin,
            fork_end,
            config.fork_segments,
            config.jitter_amplitude * 0.5,
            0.0,
            rng,
        );
        spawn_segments(
            commands,
            parent,
            mesh,
            material,
            &fork_path,
            main_width * 0.5,
        );
    }
}

/// Despawns expired bolt parents once their afterimage has fully faded.
/// Children are removed automatically via the hierarchy. The cloned per-bolt
/// `StandardMaterial` is freed when the last strong handle drops with the
/// despawning entity tree.
pub(crate) fn cleanup_lightning_bolts(
    mut commands: Commands,
    bolts: Query<(Entity, &LightningBolt)>,
) {
    for (entity, bolt) in &bolts {
        if bolt.lifetime <= 0.0 && bolt.afterimage_remaining <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Generates `segments + 1` points between `start` and `end`. Interior points
/// get optional parabolic Y arch (`peak_height`) plus perpendicular jitter
/// that tapers to zero at the endpoints so they stay anchored.
pub(crate) fn generate_jagged_path(
    start: Vec3,
    end: Vec3,
    segments: u32,
    jitter_amplitude: f32,
    peak_height: f32,
    rng: &mut impl Rng,
) -> Vec<Vec3> {
    let n = segments.max(1);
    let mut out = Vec::with_capacity(n as usize + 1);

    let along = end - start;
    let along_len = along.length();
    if along_len < 1e-3 {
        out.push(start);
        out.push(end);
        return out;
    }
    let along_dir = along / along_len;

    // Build an orthonormal frame around the path. Prefer perpendicular-to-Y;
    // fall back to X for nearly-vertical bolts where the Y cross collapses.
    let mut perp_a = along_dir.cross(Vec3::Y);
    if perp_a.length_squared() < 1e-3 {
        perp_a = along_dir.cross(Vec3::X);
    }
    let perp_a = perp_a.normalize_or_zero();
    let perp_b = along_dir.cross(perp_a).normalize_or_zero();

    for i in 0..=n {
        let t = i as f32 / n as f32;
        // 4*t*(1-t) is 0 at endpoints and 1 at midpoint — anchors the path.
        let taper = 4.0 * t * (1.0 - t);
        let base = start.lerp(end, t);
        let height = peak_height * taper;

        let jitter = if i == 0 || i == n {
            Vec3::ZERO
        } else {
            let amp = jitter_amplitude * taper;
            perp_a * rng.random_range(-amp..amp) + perp_b * rng.random_range(-amp..amp)
        };

        out.push(Vec3::new(base.x, base.y + height, base.z) + jitter);
    }
    out
}

/// Spawns one quad child per consecutive pair of points. Uses the parent's
/// shared mesh/material handles. Each quad is oriented along its segment.
fn spawn_segments(
    commands: &mut Commands,
    parent: Entity,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    path: &[Vec3],
    width: f32,
) {
    if path.len() < 2 {
        return;
    }
    for window in path.windows(2) {
        let p0 = window[0];
        let p1 = window[1];
        let dir = p1 - p0;
        let length = dir.length();
        if length < 1e-3 {
            continue;
        }
        let direction = dir / length;
        let midpoint = (p0 + p1) * 0.5;
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                .with_scale(Vec3::new(width, length, width)),
            ChildOf(parent),
            OnGameplayScreen,
        ));
    }
}
