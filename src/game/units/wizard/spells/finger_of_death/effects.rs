//! Finger of Death post-cast effects: undead raises, necrotic explosions, awaiting release.

use super::super::super::components::{CastingState, LocalWizard, Spell, Wizard};
use super::casting::spawn_beam;
use super::components::*;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{
    Corpse, Health, PermanentCorpse, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::constants::{EXCREMAGE_BROWN, EXCREMAGE_BROWN_DARK};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// Computes talent-modified parameters for Finger of Death.
pub fn process_pending_undead_raises(
    mut commands: Commands,
    pending: Option<Res<PendingUndeadRaise>>,
    corpse_query: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: Option<Res<crate::game::units::undead::resources::UndeadAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(pending) = pending else {
        return;
    };
    let Some(undead_assets) = undead_assets else {
        commands.remove_resource::<PendingUndeadRaise>();
        return;
    };

    use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
    use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;

    let mut raised_entities = Vec::new();
    for kill_pos in &pending.kill_positions {
        let mut best: Option<(Entity, f32)> = None;
        for (entity, transform) in corpse_query.iter() {
            if raised_entities.contains(&entity) {
                continue;
            }
            let dist = transform.translation.distance(*kill_pos);
            if dist < 50.0
                && (best.is_none() || dist < best.as_ref().map(|(_, d)| *d).unwrap_or(f32::MAX))
            {
                best = Some((entity, dist));
            }
        }
        if let Some((corpse_entity, _)) = best {
            raised_entities.push(corpse_entity);
            crate::game::units::systems::resurrect_corpse_as_infantry(
                &mut commands,
                corpse_entity,
                *kill_pos,
                Team::Undead,
                UNIT_HEALTH,
                UNIT_MOVEMENT_SPEED * 0.5,
                UNDEAD_SPRITE_TINT,
                undead_assets.sprite_texture.clone(),
                undead_assets.sprite_mesh.clone(),
                &mut materials,
                Some(undead_assets.death_texture.clone()),
            );
        }
    }

    commands.remove_resource::<PendingUndeadRaise>();
}

/// Removes AwaitingFingerOfDeathRelease when the mouse is no longer held.
/// This runs independently of the casting system's run conditions.
pub fn clear_awaiting_fod_release(
    mut commands: Commands,
    mouse_held: Res<crate::game::input::components::MouseLeftHeldThisFrame>,
    query: Query<Entity, With<AwaitingFingerOfDeathRelease>>,
) {
    if mouse_held.held {
        return;
    }
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<AwaitingFingerOfDeathRelease>();
    }
}

/// Spawns a necrotic explosion AoE visual and applies damage at a position.
pub(super) fn spawn_necrotic_explosion(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    visual_assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let pulse_material = materials
        .get(&visual_assets.necrotic_pulse)
        .cloned()
        .unwrap_or_default();
    let instance = materials.add(pulse_material);

    commands.spawn((
        NecroticExplosionBurst {
            time_alive: 0.0,
            lifetime: constants::NECROTIC_EXPLOSION_PULSE_LIFETIME,
            max_radius: constants::NECROTIC_EXPLOSION_RADIUS,
            damage,
            damage_applied: false,
        },
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(instance),
        Transform::from_translation(Vec3::new(
            position.x,
            constants::PULSE_Y_POSITION,
            position.z,
        ))
        .with_rotation(UPWARD_ROTATION)
        .with_scale(Vec3::splat(1.0)),
        OnGameplayScreen,
    ));
}

/// Applies necrotic explosion AoE damage to enemies near explosion positions (one-shot).
pub fn apply_necrotic_explosion_damage(
    mut commands: Commands,
    mut explosions: Query<(&Transform, &mut NecroticExplosionBurst)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        Without<Wizard>,
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());

    // Collect positions and damage of explosions that haven't applied damage yet
    let mut pending: Vec<(Vec3, f32)> = Vec::new();
    for (transform, mut burst) in explosions.iter_mut() {
        if !burst.damage_applied {
            burst.damage_applied = true;
            pending.push((transform.translation, burst.damage));
        }
    }

    for (explosion_pos, explosion_damage) in &pending {
        for (entity, transform, mut health, mut temp_hp, has_spell_shield, team) in
            targets.iter_mut()
        {
            let dist = crate::game::units::wizard::spells::utils::xz_distance(
                transform.translation,
                *explosion_pos,
            );
            if dist <= constants::NECROTIC_EXPLOSION_RADIUS {
                apply_spell_damage_with_team(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    *explosion_damage,
                    DamageType::Necrotic,
                    has_spell_shield,
                    caster_team,
                    *team,
                );
            }
        }
    }
}

/// Updates necrotic explosion burst visuals — expand and fade.
pub fn update_necrotic_explosion_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut bursts: Query<(
        Entity,
        &mut NecroticExplosionBurst,
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
        let radius = burst.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            let color = constants::NECROTIC_EXPLOSION_COLOR.to_srgba();
            let alpha = 0.6 * (1.0 - progress);
            material.base_color = Color::srgba(color.red, color.green, color.blue, alpha);
        }
    }
}

/// Ticks deathmark debuffs and fires chain beams when marked targets die.
#[allow(clippy::too_many_arguments)]
pub fn update_deathmark_debuffs(
    mut commands: Commands,
    time: Res<Time>,
    mut marked_targets: Query<(Entity, &Transform, &Health, &mut DeathmarkDebuff)>,
    all_enemies: Query<(Entity, &Transform, &Health), Without<Wizard>>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, transform, health, mut debuff) in marked_targets.iter_mut() {
        debuff.time_remaining -= dt;

        // Target died while debuffed — fire chain beam at nearest enemy
        if health.current <= 0.0 {
            let target_pos = transform.translation;
            // Find nearest living enemy
            let mut best: Option<(Vec3, f32)> = None;
            for (other_entity, other_transform, other_health) in all_enemies.iter() {
                if other_entity == entity || other_health.current <= 0.0 {
                    continue;
                }
                let dist = other_transform.translation.distance(target_pos);
                match best {
                    None => best = Some((other_transform.translation, dist)),
                    Some((_, best_dist)) if dist < best_dist => {
                        best = Some((other_transform.translation, dist));
                    }
                    _ => {}
                }
            }

            if let Some((enemy_pos, _)) = best {
                let direction = (enemy_pos - debuff.beam_origin).normalize();
                let length = enemy_pos.distance(debuff.beam_origin) + 50.0; // extend slightly past target

                let mut chain_params = debuff.talent_params.clone();
                chain_params.is_chain_beam = true;
                // Flat 10% of base damage every hop (no falloff)
                chain_params.chain_damage_mult = constants::DEATHMARK_CHAIN_DAMAGE_PERCENT;

                let mut chain_beam = FingerOfDeathBeam::with_talents(
                    debuff.beam_origin,
                    direction,
                    length,
                    debuff.empowerment,
                    chain_params,
                );
                chain_beam.cast_progress = 1.0; // fires immediately
                spawn_beam(&mut commands, &visual_assets, &mut materials, chain_beam);
            }

            commands.entity(entity).remove::<DeathmarkDebuff>();
            continue;
        }

        // Debuff expired
        if debuff.time_remaining <= 0.0 {
            commands.entity(entity).remove::<DeathmarkDebuff>();
        }
    }
}

/// Updates Reaper's Scythe sweep — rotates beam through arc and damages targets.
#[allow(clippy::too_many_arguments)]
pub fn update_reapers_scythe(
    mut commands: Commands,
    time: Res<Time>,
    mut sweeps: Query<(Entity, &mut ReapersScytheSweep)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        Without<Wizard>,
    >,
    walls: Query<&crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone>,
    rocks_query: Query<&crate::game::terrain::boulder::components::Boulder>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let dt = time.delta_secs();
    let caster_team = local_player_team(session.as_deref());

    for (sweep_entity, mut sweep) in sweeps.iter_mut() {
        sweep.time_elapsed += dt;

        if sweep.time_elapsed >= sweep.duration {
            commands.entity(sweep_entity).try_despawn();
            continue;
        }

        // Calculate current sweep angle
        let arc_radians = constants::REAPERS_SCYTHE_ARC_DEGREES.to_radians();
        let half_arc = arc_radians / 2.0;
        let progress = sweep.time_elapsed / sweep.duration;
        let current_angle = -half_arc + arc_radians * progress;

        // Rotate center direction by current_angle around Y axis
        let cos_a = current_angle.cos();
        let sin_a = current_angle.sin();
        let dir = sweep.center_direction;
        let rotated_dir = Vec3::new(
            dir.x * cos_a + dir.z * sin_a,
            dir.y,
            -dir.x * sin_a + dir.z * cos_a,
        )
        .normalize();

        // Create a temporary beam for hit detection
        let beam = FingerOfDeathBeam::with_talents(
            sweep.origin,
            rotated_dir,
            sweep.length,
            sweep.empowerment,
            sweep.talent_params.clone(),
        );

        // Find wall/rock intersection
        let beam_end = sweep.origin + rotated_dir * sweep.length;
        let mut max_t = 1.0_f32;
        for wall in &walls {
            if let Some(t) = wall.line_segment_intersects(sweep.origin, beam_end) {
                max_t = max_t.min(t);
            }
        }
        for rock in &rocks_query {
            if !rock.sinking
                && let Some(t) = rock.line_segment_intersects(sweep.origin, beam_end)
            {
                max_t = max_t.min(t);
            }
        }
        let effective_length = sweep.length * max_t;

        let beam_width = beam.beam_width();
        let damage = beam.damage();
        let mut kill_count = 0u32;

        // Damage targets in current sweep position (skip already-hit)
        for (entity, transform, mut health, mut temp_hp, has_spell_shield, team) in
            targets.iter_mut()
        {
            // Enemy shielded King is immune; your own King takes the sweep (friendly fire).
            if (has_spell_shield && caster_team != *team) || sweep.hit_entities.contains(&entity) {
                continue;
            }
            if beam.contains_point(transform.translation, beam_width) {
                let proj = (transform.translation - sweep.origin).dot(rotated_dir);
                if proj <= effective_length {
                    sweep.hit_entities.insert(entity);
                    let was_alive = health.current > 0.0;
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Necrotic,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );

                    if was_alive && health.current <= 0.0 {
                        kill_count += 1;

                        // Necrotic explosion on kills during sweep
                        if sweep.talent_params.necrotic_explosion {
                            let explosion_damage =
                                damage * constants::NECROTIC_EXPLOSION_DAMAGE_PERCENT;
                            spawn_necrotic_explosion(
                                &mut commands,
                                transform.translation,
                                explosion_damage,
                                &visual_assets,
                                &mut materials,
                            );
                        }
                    }
                }
            }
        }

        if kill_count > 0
            && let Some(ref mut progress) = talent_progress
        {
            progress.increment(Spell::FingerOfDeath, kill_count);
        }

        // Spawn visual beam trail at intervals (every ~0.05s) instead of every frame
        let spawn_interval = 0.05;
        let prev_step = ((sweep.time_elapsed - dt) / spawn_interval) as u32;
        let curr_step = (sweep.time_elapsed / spawn_interval) as u32;
        if curr_step > prev_step || sweep.time_elapsed < dt {
            let material = materials
                .get(&visual_assets.finger_of_death_beam)
                .cloned()
                .unwrap_or_default();
            let instance = materials.add(material);

            let mut sweep_beam = FingerOfDeathBeam::with_talents(
                sweep.origin,
                rotated_dir,
                sweep.length,
                sweep.empowerment,
                sweep.talent_params.clone(),
            );
            sweep_beam.has_fired = true;
            sweep_beam.cast_progress = 1.0;
            sweep_beam.time_since_fired = 0.0;

            commands.spawn((
                sweep_beam,
                Mesh3d(visual_assets.cross_plane_triangle.clone()),
                MeshMaterial3d(instance),
                Transform::from_translation(sweep.origin),
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates Finger of Death beam visuals based on cast progress and fire state.
pub fn update_finger_of_death_beam_visuals(
    time: Res<Time>,
    mut beam_query: Query<(
        &mut FingerOfDeathBeam,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<crate::config::GameConfig>,
) {
    let t = time.elapsed_secs();
    let is_excremage = config.wizard_type == crate::config::WizardType::Excremage;

    for (mut beam, mut transform, material_handle) in beam_query.iter_mut() {
        if beam.has_fired {
            beam.time_since_fired += time.delta_secs();
        }

        let visual_len = beam.length + constants::BEAM_VISUAL_OVERSHOOT;

        // Position at origin, rotate to point along beam direction
        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Pulsing width
        let pulse = 1.0
            + constants::BEAM_PULSE_AMPLITUDE
                * (t * constants::BEAM_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let base_width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        let beam_width = base_width * pulse;

        // During cast: scale up with cast_progress; after fire: full size
        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::new(
            beam_width * progress_scale,
            visual_len * progress_scale,
            beam_width * progress_scale,
        );

        // Color cycling and fade
        if let Some(mat) = materials.get_mut(&material_handle.0) {
            if beam.has_fired {
                let fade = (1.0 - beam.time_since_fired / constants::POST_FIRE_DURATION).max(0.0);

                if is_excremage {
                    let srgba = EXCREMAGE_BROWN.to_srgba();
                    mat.base_color = Color::srgb(srgba.red, srgba.green, srgba.blue);
                    mat.emissive = LinearRgba::new(
                        srgba.red * 2.0 * fade,
                        srgba.green * 0.5 * fade,
                        srgba.blue * 0.2 * fade,
                        1.0,
                    );
                } else {
                    // Purple color cycling: dark purple -> bright violet -> pale purple
                    let cycle = (t * constants::COLOR_CYCLE_SPEED).sin() * 0.5 + 0.5;
                    let r = (0.5 + cycle * 0.5) * fade;
                    let g = (0.0 + cycle * 0.3) * fade;
                    let b = (0.6 + cycle * 0.4) * fade;
                    mat.base_color = Color::srgb(r, g, b);
                    mat.emissive = LinearRgba::new(
                        (1.5 + cycle * 1.5) * fade,
                        (0.0 + cycle * 0.8) * fade,
                        (2.5 + cycle * 2.0) * fade,
                        1.0,
                    );
                }
            } else {
                // During cast: growing intensity with cast_progress
                let intensity = beam.cast_progress;

                if is_excremage {
                    let srgba = EXCREMAGE_BROWN_DARK.to_srgba();
                    mat.base_color = Color::srgb(srgba.red, srgba.green, srgba.blue);
                    mat.emissive = LinearRgba::new(
                        srgba.red * intensity,
                        srgba.green * 0.3 * intensity,
                        srgba.blue * 0.1 * intensity,
                        1.0,
                    );
                } else {
                    mat.base_color = Color::srgb(0.6 * intensity, 0.0, 0.8 * intensity);
                    mat.emissive = LinearRgba::new(1.5 * intensity, 0.0, 2.5 * intensity, 1.0);
                }
            }
        }
    }
}

/// Updates necrotic vein particles: meander, fade color, shrink, despawn when expired.
pub fn update_necrotic_veins(
    mut commands: Commands,
    time: Res<Time>,
    mut veins: Query<(
        Entity,
        &mut NecroticVein,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut vein, mut transform, material_handle) in veins.iter_mut() {
        vein.time_alive += dt;

        if vein.time_alive >= vein.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = vein.time_alive / vein.lifetime;

        // Meander: apply lateral sine-wave offset to velocity direction
        let wander_offset = (t * constants::VEIN_WANDER_FREQUENCY + vein.wander_phase).sin()
            * constants::VEIN_WANDER_AMPLITUDE
            * dt;
        let lateral = Vec3::new(-vein.velocity.z, 0.0, vein.velocity.x).normalize_or_zero();
        transform.translation += vein.velocity * dt + lateral * wander_offset;

        // Clamp Y to ground level
        transform.translation.y = constants::VEIN_Y_POSITION;

        // Shrink over lifetime
        let scale = vein.base_size * (1.0 - progress);
        transform.scale = Vec3::splat(scale);

        // Animate material: purple → dark purple → black, alpha fading out
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let r = 0.6 * (1.0 - progress);
            let g = 0.0;
            let b = 0.8 * (1.0 - progress * 0.5);
            let alpha = 0.8 * (1.0 - progress);
            material.base_color = Color::srgba(r, g, b, alpha);
            let em_scale = 1.0 - progress;
            material.emissive = LinearRgba::new(1.5 * em_scale, 0.0, 2.0 * em_scale, 1.0);
        }
    }
}

/// Updates the glow aura to follow its beam with shimmer and pulsing.
pub fn update_finger_of_death_glow(
    mut glow_query: Query<(&FingerOfDeathGlow, &mut Transform), Without<FingerOfDeathBeam>>,
    beam_query: Query<&FingerOfDeathBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (glow, mut transform) in glow_query.iter_mut() {
        let Ok(beam) = beam_query.get(glow.beam_entity) else {
            continue;
        };

        let visual_len = beam.length + constants::BEAM_VISUAL_OVERSHOOT;

        transform.translation = beam.origin;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, beam.direction);

        // Glow pulse + shimmer jitter
        let pulse = 1.0
            + constants::GLOW_PULSE_AMPLITUDE
                * (t * constants::GLOW_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let shimmer = constants::SHIMMER_AMPLITUDE
            * ((t * constants::SHIMMER_FREQ_A).sin() + (t * constants::SHIMMER_FREQ_B).cos());
        let base_width = if beam.has_fired {
            beam.beam_width_fired()
        } else {
            beam.beam_width()
        };
        let glow_width = base_width * constants::GLOW_WIDTH_MULTIPLIER * (pulse + shimmer);

        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::new(
            glow_width * progress_scale,
            visual_len * progress_scale,
            glow_width * progress_scale,
        );
    }
}

/// Despawns glow entities when their beam no longer exists.
pub fn cleanup_finger_of_death_glow(
    mut commands: Commands,
    glow_query: Query<(Entity, &FingerOfDeathGlow)>,
    beam_query: Query<Entity, With<FingerOfDeathBeam>>,
) {
    for (glow_entity, glow) in glow_query.iter() {
        if beam_query.get(glow.beam_entity).is_err() {
            commands.entity(glow_entity).try_despawn();
        }
    }
}

/// Updates the origin flare to pulse at the beam origin.
pub fn update_finger_of_death_flare(
    mut flare_query: Query<(&FingerOfDeathFlare, &mut Transform)>,
    beam_query: Query<&FingerOfDeathBeam>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (flare, mut transform) in flare_query.iter_mut() {
        let Ok(beam) = beam_query.get(flare.beam_entity) else {
            continue;
        };

        transform.translation = beam.origin;

        let pulse = 1.0
            + constants::FLARE_PULSE_AMPLITUDE
                * (t * constants::FLARE_PULSE_FREQUENCY * std::f32::consts::TAU).sin();
        let radius = constants::FLARE_RADIUS * pulse;

        // Scale with cast progress so flare grows during cast
        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        transform.scale = Vec3::splat(radius * progress_scale);
    }
}

/// Despawns flare entities when their beam no longer exists.
pub fn cleanup_finger_of_death_flare(
    mut commands: Commands,
    flare_query: Query<(Entity, &FingerOfDeathFlare)>,
    beam_query: Query<Entity, With<FingerOfDeathBeam>>,
) {
    for (flare_entity, flare) in flare_query.iter() {
        if beam_query.get(flare.beam_entity).is_err() {
            commands.entity(flare_entity).try_despawn();
        }
    }
}

/// Updates necrotic pulse ring: expand scale, fade alpha, despawn when expired.
pub fn update_necrotic_pulse(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(
        Entity,
        &mut NecroticPulse,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut pulse, mut transform, material_handle) in pulses.iter_mut() {
        pulse.time_alive += dt;

        if pulse.time_alive >= pulse.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = pulse.time_alive / pulse.lifetime;

        // Expand from small to max_radius
        let radius = pulse.max_radius * progress;
        transform.scale = Vec3::splat(radius);

        // Fade alpha from 0.5 to 0
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 0.5 * (1.0 - progress);
            material.base_color = Color::srgba(0.4, 0.0, 0.6, alpha);
        }
    }
}

/// Cleans up Finger of Death beams after firing or cancellation.
pub fn cleanup_finger_of_death_beams(
    mut commands: Commands,
    beams: Query<(Entity, &FingerOfDeathBeam)>,
    wizard_query: Query<&CastingState, With<LocalWizard>>,
) {
    let resting = wizard_query
        .single()
        .map(|state| matches!(state, CastingState::Resting))
        .unwrap_or(true);

    for (entity, beam) in beams.iter() {
        let should_despawn = if beam.has_fired {
            beam.time_since_fired >= constants::POST_FIRE_DURATION
        } else {
            resting
        };

        if should_despawn {
            commands.entity(entity).try_despawn();
        }
    }
}
