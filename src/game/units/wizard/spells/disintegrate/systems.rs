use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    BeamGlow, BeamOriginFlare, BeamSmoke, DisintegrateBeam, DisintegrateParticle,
};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Health, Hitbox, TemporaryHitPoints, apply_spell_damage};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Action the shared logic requests the wrapper to perform on beams.
enum BeamAction {
    /// Update existing beam with new origin, direction, length.
    UpdateBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
    },
    /// Spawn a new beam.
    SpawnBeam {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        empowerment: f32,
    },
    /// Despawn all beams for this wizard.
    DespawnAll,
    /// No beam action needed.
    None,
}

/// Result from the shared casting logic.
struct CastingResult {
    beam_action: BeamAction,
}

/// Local wizard disintegrate casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_disintegrate_casting(
    time: Res<Time>,
    mut left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
            &Wizard,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut beams: Query<(Entity, &mut DisintegrateBeam)>,
    visual_assets: Res<SpellVisualAssets>,
    glow_query: Query<Entity, With<BeamGlow>>,
    flare_query: Query<Entity, With<BeamOriginFlare>>,
    particle_query: Query<Entity, With<DisintegrateParticle>>,
) {
    let released = left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((mut casting_state, mut mana, primed_spell, wizard)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Disintegrate {
        return;
    }

    let has_existing_beam = beams.iter().next().is_some();

    let result = disintegrate_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        wizard,
        has_existing_beam,
    );

    match result.beam_action {
        BeamAction::UpdateBeam {
            origin,
            direction,
            length,
        } => {
            if let Some((_, mut beam)) = beams.iter_mut().next() {
                beam.origin = origin;
                beam.direction = direction;
                beam.length = length;
            }
        }
        BeamAction::SpawnBeam {
            origin,
            direction,
            length,
            empowerment,
        } => {
            spawn_beam(
                &mut commands,
                &visual_assets,
                origin,
                direction,
                length,
                empowerment,
            );
        }
        BeamAction::DespawnAll => {
            despawn_all_beam_visuals(
                &mut commands,
                &beams,
                &glow_query,
                &flare_query,
                &particle_query,
            );
        }
        BeamAction::None => {}
    }
}

/// Core disintegrate casting logic.
///
/// Takes extracted data from queries and returns actions for the wrapper to apply.
#[allow(clippy::too_many_arguments)]
fn disintegrate_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    wizard: &Wizard,
    has_existing_beam: bool,
) -> CastingResult {
    let mut result = CastingResult {
        beam_action: BeamAction::None,
    };

    let wizard_pos = SPELL_ORIGIN;

    // Check for release
    if input.just_released {
        casting_state.cancel();
        result.beam_action = BeamAction::DespawnAll;
        return result;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            let mana_cost = constants::MANA_COST_PER_SECOND * time.delta_secs();

            if mana.consume(mana_cost) {
                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    if has_existing_beam {
                        result.beam_action = BeamAction::UpdateBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                        };
                    } else {
                        result.beam_action = BeamAction::SpawnBeam {
                            origin: beam_origin,
                            direction,
                            length: beam_length,
                            empowerment: primed_spell.empowerment,
                        };
                    }
                }
            } else {
                casting_state.cancel();
                result.beam_action = BeamAction::DespawnAll;
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                casting_state.start_channeling();

                if let Some(target_pos) = input.cursor_pos {
                    let beam_origin =
                        wizard_pos + Vec3::new(0.0, constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);

                    let to_target = target_pos - beam_origin;
                    let distance = to_target.length();
                    let clamped_target = if distance > wizard.spell_range {
                        beam_origin + to_target.normalize() * wizard.spell_range
                    } else {
                        target_pos
                    };

                    let direction = (clamped_target - beam_origin).normalize();
                    let beam_length = (clamped_target - beam_origin)
                        .length()
                        .min(constants::BEAM_LENGTH);

                    result.beam_action = BeamAction::SpawnBeam {
                        origin: beam_origin,
                        direction,
                        length: beam_length,
                        empowerment: primed_spell.empowerment,
                    };
                }
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && mana.can_afford(constants::MANA_COST_PER_SECOND * 0.1)
            {
                casting_state.start_cast();
            }
        }
    }

    result
}

/// Gets the cursor position projected onto the battlefield surface (Y=0 plane).
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    // Create a ray from the camera through the cursor position
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    // Intersect ray with Y=0 plane (battlefield surface)
    // Ray equation: origin + direction * t
    // Plane equation: y = 0
    // Solve for t: origin.y + direction.y * t = 0
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}

/// System that applies damage to all units hit by disintegrate beams.
///
/// This is a high-risk spell that damages both attackers and defenders,
/// but not the wizard.
pub fn apply_disintegrate_damage(
    mut commands: Commands,
    mut beam_query: Query<&mut DisintegrateBeam>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        Without<Wizard>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    time: Res<Time>,
) {
    for mut beam in beam_query.iter_mut() {
        beam.update_damage_timer(time.delta_secs());
        beam.update_time_alive(time.delta_secs());

        // Find the nearest wall intersection to limit beam reach
        let beam_end = beam.origin + beam.direction * beam.current_length();
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(beam.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        let effective_length = beam.current_length() * max_t;

        if beam.should_damage() {
            for (entity, transform, hitbox, mut health, mut temp_hp, has_spell_shield) in
                target_query.iter_mut()
            {
                let position = transform.translation;
                // Check if unit's hitbox intersects beam AND is before the wall
                if beam.contains_point_with_radius(position, hitbox.radius) {
                    let proj = (position - beam.origin).dot(beam.direction);
                    if proj <= effective_length + hitbox.radius {
                        apply_spell_damage(
                            &mut commands,
                            entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            beam.damage_per_tick(),
                            constants::DAMAGE_TYPE,
                            has_spell_shield,
                        );
                    }
                }
            }

            beam.reset_damage_timer();
        }
    }
}

/// System that despawns beams when wizard is not actively casting/channeling disintegrate.
///
/// Checks CastingState directly to avoid deferred command timing issues.
/// Excludes crystal-spawned beams (those with CrystalSpawn) — they're managed by the crystal.
pub fn cleanup_beams_on_cancel(
    mut commands: Commands,
    wizard_query: Query<&CastingState, With<LocalWizard>>,
    beam_query: Query<Entity, (With<DisintegrateBeam>, Without<CrystalSpawn>)>,
    glow_query: Query<Entity, With<BeamGlow>>,
    flare_query: Query<Entity, With<BeamOriginFlare>>,
    particle_query: Query<Entity, With<DisintegrateParticle>>,
) {
    if let Ok(casting_state) = wizard_query.single()
        && matches!(casting_state, CastingState::Resting)
    {
        for entity in beam_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in glow_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in flare_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in particle_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns a beam entity with a cylinder mesh visible from all angles,
/// plus glow and flare sibling entities.
pub(crate) fn spawn_beam(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
) -> Entity {
    let beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    spawn_beam_visuals(commands, assets, beam)
}

/// Spawns a beam entity with custom damage per tick (for crystal use),
/// plus glow and flare sibling entities.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_beam_with_damage(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    direction: Vec3,
    length: f32,
    empowerment: f32,
    damage_per_tick: f32,
) -> Entity {
    let mut beam = DisintegrateBeam::new(origin, direction, length, empowerment);
    beam.damage_per_tick_override = Some(damage_per_tick);
    spawn_beam_visuals(commands, assets, beam)
}

/// Shared helper that spawns the core beam entity plus glow and flare siblings.
fn spawn_beam_visuals(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    beam: DisintegrateBeam,
) -> Entity {
    let midpoint = beam.origin + beam.direction * (beam.length / 2.0);

    let beam_entity = commands
        .spawn((
            beam,
            Mesh3d(assets.cross_plane_cylinder.clone()),
            MeshMaterial3d(assets.disintegrate_beam.clone()),
            Transform::from_translation(midpoint),
            OnGameplayScreen,
        ))
        .id();

    // Glow cylinder sibling (wider, semi-transparent)
    commands.spawn((
        BeamGlow {
            beam_entity,
        },
        Mesh3d(assets.cross_plane_cylinder.clone()),
        MeshMaterial3d(assets.disintegrate_glow.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    // Origin flare circle sibling (uses cross-plane sphere for visibility from all angles)
    commands.spawn((
        BeamOriginFlare {
            beam_entity,
        },
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.disintegrate_flare.clone()),
        Transform::from_translation(midpoint),
        OnGameplayScreen,
    ));

    beam_entity
}

/// Helper to despawn all beam-related visual entities.
fn despawn_all_beam_visuals(
    commands: &mut Commands,
    beams: &Query<(Entity, &mut DisintegrateBeam)>,
    glow_query: &Query<Entity, With<BeamGlow>>,
    flare_query: &Query<Entity, With<BeamOriginFlare>>,
    particle_query: &Query<Entity, With<DisintegrateParticle>>,
) {
    for (entity, _) in beams.iter() {
        commands.entity(entity).despawn();
    }
    for entity in glow_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in flare_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in particle_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// System that updates beam cylinder transform to match beam data,
/// with pulsing width and color cycling.
pub fn update_beam_visuals(
    mut beam_query: Query<(&DisintegrateBeam, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (beam, mut transform, material_handle) in beam_query.iter_mut() {
        let current_len = beam.current_length();
        let midpoint = beam.origin + beam.direction * (current_len / 2.0);
        transform.translation = midpoint;

        let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.rotation = rotation;

        // Pulsing width
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let beam_width = constants::BEAM_WIDTH * beam.empowerment * pulse;
        transform.scale = Vec3::new(beam_width, current_len, beam_width);

        // Color cycling: orange -> yellow -> white -> yellow -> orange
        if let Some(mat) = materials.get_mut(material_handle) {
            let cycle = (t * constants::COLOR_CYCLE_SPEED).sin() * 0.5 + 0.5; // 0..1
            // Interpolate emissive: orange(3,1.5,0.2) -> white(5,4.5,4)
            let r = 3.0 + cycle * 2.0;
            let g = 1.5 + cycle * 3.0;
            let b = 0.2 + cycle * 3.8;
            mat.emissive = bevy::color::LinearRgba::new(r, g, b, 1.0);

            // Also shift base color slightly
            let base_r = 1.0;
            let base_g = 0.6 + cycle * 0.35;
            let base_b = 0.1 + cycle * 0.6;
            mat.base_color = Color::srgb(base_r, base_g, base_b);
        }
    }
}

/// System that positions and animates the outer glow cylinder to follow its beam.
pub fn update_beam_glow(
    mut glow_query: Query<(&BeamGlow, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(beam) = beam_query.get(glow.beam_entity) else {
            continue;
        };

        let current_len = beam.current_length();
        let midpoint = beam.origin + beam.direction * (current_len / 2.0);
        transform.translation = midpoint;

        let rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);
        transform.rotation = rotation;

        // Glow pulse + shimmer jitter from incommensurate frequencies
        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let shimmer = constants::SHIMMER_AMPLITUDE
            * ((t * constants::SHIMMER_FREQ_A).sin() + (t * constants::SHIMMER_FREQ_B).cos());
        let glow_width =
            constants::BEAM_WIDTH * beam.empowerment * constants::GLOW_WIDTH_MULTIPLIER
                * (pulse + shimmer);
        transform.scale = Vec3::new(glow_width, current_len, glow_width);
    }
}

/// System that positions and animates the origin flare sphere.
pub fn update_beam_origin_flare(
    mut flare_query: Query<(&BeamOriginFlare, &mut Transform)>,
    beam_query: Query<&DisintegrateBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (flare, mut transform) in flare_query.iter_mut() {
        let Ok(beam) = beam_query.get(flare.beam_entity) else {
            continue;
        };

        transform.translation = beam.origin;

        // Pulsing scale
        let pulse = 1.0
            + constants::FLARE_PULSE_AMPLITUDE
                * (t * constants::FLARE_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let radius = constants::FLARE_RADIUS * pulse;
        transform.scale = Vec3::splat(radius);
    }
}

/// System that spawns impact particles at the beam tip.
pub fn spawn_impact_particles(
    mut commands: Commands,
    beam_query: Query<&DisintegrateBeam>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < constants::PARTICLE_SPAWN_INTERVAL {
        return;
    }
    *timer -= constants::PARTICLE_SPAWN_INTERVAL;

    for beam in beam_query.iter() {
        let current_len = beam.current_length();
        if current_len < 1.0 {
            continue;
        }

        let tip = beam.origin + beam.direction * current_len;

        // Build a perpendicular basis for random spread
        let up = if beam.direction.y.abs() > 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = beam.direction.cross(up).normalize();
        let forward = right.cross(beam.direction).normalize();

        for i in 0..constants::PARTICLE_COUNT_PER_SPAWN {
            // Spread particles in a circle perpendicular to the beam
            let angle = (i as f32 / constants::PARTICLE_COUNT_PER_SPAWN as f32)
                * std::f32::consts::TAU
                + time.elapsed_secs() * 17.3; // rotating offset
            let spread = right * angle.cos() + forward * angle.sin();
            let velocity = spread * constants::PARTICLE_SPEED;

            commands.spawn((
                DisintegrateParticle {
                    velocity,
                    time_alive: 0.0,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.disintegrate_particle.clone()),
                Transform::from_translation(tip)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(constants::PARTICLE_SIZE)),
                OnGameplayScreen,
            ));
        }
    }
}

/// System that moves, shrinks, and despawns impact particles.
pub fn update_impact_particles(
    mut commands: Commands,
    mut particle_query: Query<(Entity, &mut DisintegrateParticle, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform) in particle_query.iter_mut() {
        particle.time_alive += dt;

        if particle.time_alive >= constants::PARTICLE_LIFETIME {
            commands.entity(entity).despawn();
            continue;
        }

        // Move by velocity
        transform.translation += particle.velocity * dt;

        // Scale down linearly over lifetime
        let remaining = 1.0 - (particle.time_alive / constants::PARTICLE_LIFETIME);
        let size = constants::PARTICLE_SIZE * remaining;
        transform.scale = Vec3::splat(size);
    }
}

/// System that spawns dark smoke wisps along the beam.
/// Smoke is independent of the beam and self-dissipates after its lifetime.
pub fn spawn_beam_smoke(
    mut commands: Commands,
    beam_query: Query<&DisintegrateBeam>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < constants::SMOKE_SPAWN_INTERVAL {
        return;
    }
    *timer -= constants::SMOKE_SPAWN_INTERVAL;

    let t = time.elapsed_secs();

    for beam in beam_query.iter() {
        let current_len = beam.current_length();
        if current_len < 1.0 {
            continue;
        }

        // Build perpendicular basis
        let up = if beam.direction.y.abs() > 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = beam.direction.cross(up).normalize();
        let forward = right.cross(beam.direction).normalize();

        for i in 0..constants::SMOKE_COUNT_PER_SPAWN {
            // Spawn at a random-ish point along the beam length
            let frac = ((i as f32 + 0.5) / constants::SMOKE_COUNT_PER_SPAWN as f32
                + (t * 13.7).fract())
                % 1.0;
            let pos = beam.origin + beam.direction * (current_len * frac);

            // Mostly upward drift with slight lateral spread
            let angle = (i as f32 * 2.39 + t * 7.1).sin(); // pseudo-random lateral
            let lateral = (right * angle.cos() + forward * angle.sin())
                * constants::SMOKE_SPREAD_SPEED;
            let velocity = Vec3::Y * constants::SMOKE_RISE_SPEED + lateral;

            commands.spawn((
                BeamSmoke {
                    velocity,
                    time_alive: 0.0,
                },
                Mesh3d(visual_assets.particle_quad.clone()),
                MeshMaterial3d(visual_assets.disintegrate_smoke.clone()),
                Transform::from_translation(pos)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::splat(constants::SMOKE_SIZE)),
                OnGameplayScreen,
            ));
        }

        // Heat shimmer along the beam
        let shimmer_frac = (t * 5.3 + 0.37).fract();
        let shimmer_pos = beam.origin + beam.direction * (current_len * shimmer_frac);
        vfx::systems::spawn_heat_shimmer(
            &mut commands,
            &visual_assets,
            shimmer_pos,
            1,
            t,
        );
    }
}

/// System that drifts, scale-fades, and despawns smoke wisps.
/// Runs independently of beam existence so smoke lingers after casting stops.
pub fn update_beam_smoke(
    mut commands: Commands,
    mut smoke_query: Query<(Entity, &mut BeamSmoke, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut smoke, mut transform) in smoke_query.iter_mut() {
        smoke.time_alive += dt;

        if smoke.time_alive >= constants::SMOKE_LIFETIME {
            commands.entity(entity).despawn();
            continue;
        }

        // Drift by velocity
        transform.translation += smoke.velocity * dt;

        // Grow then shrink: peak at 60% of lifetime, shrink to zero by end
        let progress = smoke.time_alive / constants::SMOKE_LIFETIME;
        let size = if progress < 0.6 {
            constants::SMOKE_SIZE * (1.0 + progress * 0.83)
        } else {
            let shrink = 1.0 - (progress - 0.6) / 0.4;
            constants::SMOKE_SIZE * 1.5 * shrink
        };
        transform.scale = Vec3::splat(size);
    }
}
