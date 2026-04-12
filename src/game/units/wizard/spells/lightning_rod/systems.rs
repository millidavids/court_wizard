//! Lightning Rod spell systems.

use std::cmp::Ordering;

use super::components::{LightningRod, LightningRodArc, LightningRodTalentParams, LightningStrike};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range_ground, cleanup_spell_caster, handle_spell_release,
    spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Compute talent parameters from active talent selections.
fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> LightningRodTalentParams {
    let mut params = LightningRodTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::LightningRod, 0);
    let t2 = talents.get_selection(Spell::LightningRod, 1);
    let t3 = talents.get_selection(Spell::LightningRod, 2);

    // Tier 1
    match t1 {
        Some(0) => params.duration_mult *= TALLER_ROD_DURATION_MULT,
        Some(1) => params.strike_interval_mult = RAPID_STRIKES_INTERVAL_MULT,
        Some(2) => {
            params.arc_radius_mult = WIDER_ARC_RADIUS_MULT;
            params.extra_targets = WIDER_ARC_EXTRA_TARGETS;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => params.chain_reaction = true,
        Some(1) => params.magnetic_field = true,
        Some(2) => params.overcharge = true,
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            params.storm_spire = true;
            params.damage_mult = STORM_SPIRE_DAMAGE_MULT;
            params.duration_mult *= STORM_SPIRE_DURATION_MULT;
        }
        Some(1) => params.tesla_coil = true,
        Some(2) => params.lightning_nexus = true,
        _ => {}
    }

    params
}

/// Spawns a descending lightning bolt entity targeting the rod.
fn spawn_lightning_bolt(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    strike: LightningStrike,
) {
    let strike_start = Vec3::new(
        strike.target_pos.x,
        STRIKE_SPAWN_HEIGHT,
        strike.target_pos.z,
    );
    let bolt_length = STRIKE_SPAWN_HEIGHT - TOWER_HEIGHT;
    let bolt_width = STRIKE_BOLT_WIDTH * strike.empowerment;
    let midpoint = (strike_start + strike.target_pos) / 2.0;

    commands.spawn((
        strike,
        Mesh3d(assets.unit_rect.clone()),
        MeshMaterial3d(assets.lightning_strike.clone()),
        Transform::from_translation(midpoint).with_scale(Vec3::new(
            bolt_width,
            bolt_length,
            bolt_width,
        )),
        OnGameplayScreen,
    ));
}

/// Local wizard Lightning Rod casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_lightning_rod_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    target_assist: Res<TargetAssistWorldPos>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::LightningRod {
        return;
    }

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range_ground(pos, SPELL_ORIGIN, wizard.spell_range, 0.0));

    // Spawn indicator on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(MANA_COST)
        && let Some(pos) = clamped_pos
    {
        let circle_entity = spawn_circle_indicator(
            &mut commands,
            &mut meshes,
            visual_assets.lightning_rod_indicator.clone(),
            pos,
            ARC_RADIUS * primed_spell.empowerment,
        )
        .id();
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(circle_entity));
    }

    // Update indicator position during casting
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(pos) = clamped_pos
    {
        update_indicator_position(wizard_entity, pos, &caster_query, &mut indicator_query);
    }

    // Get the final spawn position from indicator if available
    let indicator_pos = caster_query
        .get(wizard_entity)
        .ok()
        .and_then(|caster| caster.indicator_entity)
        .and_then(|ie| indicator_query.get(ie).ok())
        .map(|indicator| indicator.position);

    // Override cursor_pos with indicator position for shared logic
    let effective_input = WizardInput {
        cursor_pos: indicator_pos.or(clamped_pos),
        ..input
    };

    let completed = lightning_rod_casting_logic(
        &effective_input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut commands,
        &visual_assets,
        &sfx,
        &game_config,
        active_talents.as_deref(),
    );

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Lightning,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core Lightning Rod casting logic.
#[allow(clippy::too_many_arguments)]
fn lightning_rod_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    _wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    active_talents: Option<&ActiveTalents>,
) -> bool {
    let wizard_pos = SPELL_ORIGIN;

    // Check for release event - cancel cast
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(MANA_COST)
                && input.cursor_pos.is_some()
            {
                // SpellCaster insertion handled by the wrapper
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST) {
                    let spawn_pos = input.cursor_pos.unwrap_or(wizard_pos);
                    let talent_params = compute_talent_params(active_talents);
                    let duration =
                        TOWER_DURATION * primed_spell.empowerment * talent_params.duration_mult;

                    if talent_params.storm_spire {
                        let offset = Vec3::new(STORM_SPIRE_OFFSET / 2.0, 0.0, 0.0);
                        spawn_lightning_rod(
                            commands,
                            assets,
                            spawn_pos + offset,
                            primed_spell.empowerment,
                            duration,
                            talent_params,
                        );
                        spawn_lightning_rod(
                            commands,
                            assets,
                            spawn_pos - offset,
                            primed_spell.empowerment,
                            duration,
                            talent_params,
                        );
                    } else {
                        spawn_lightning_rod(
                            commands,
                            assets,
                            spawn_pos,
                            primed_spell.empowerment,
                            duration,
                            talent_params,
                        );
                    }

                    audio::play_impact_sfx(
                        commands,
                        &sfx.lightning_rod_impact,
                        spawn_pos,
                        game_config,
                        sfx,
                    );
                    completed = true;
                }

                // Clean up indicator and caster
                cleanup_spell_caster(commands, wizard_entity, caster_query);
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}

/// Spawns the lightning rod tower entity.
pub(crate) fn spawn_lightning_rod(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    empowerment: f32,
    duration: f32,
    talent_params: LightningRodTalentParams,
) {
    let tower_height = TOWER_HEIGHT;
    let tower_radius = TOWER_RADIUS;

    // Cylinder sits centered, so position at half height
    let spawn_pos = Vec3::new(position.x, tower_height / 2.0, position.z);

    // cross_plane_cylinder has radius 0.5 and height 1.0, scale to tower dimensions
    // radius scale = tower_radius / 0.5, height scale = tower_height / 1.0
    let radius_scale = tower_radius / 0.5;

    commands.spawn((
        LightningRod::new(
            Vec3::new(position.x, 0.0, position.z),
            duration,
            empowerment,
            talent_params,
        ),
        Mesh3d(assets.cross_plane_cylinder.clone()),
        MeshMaterial3d(assets.lightning_rod.clone()),
        Transform::from_translation(spawn_pos).with_scale(Vec3::new(
            radius_scale,
            tower_height,
            radius_scale,
        )),
        NetworkedSpellEffect {
            kind: SpellEffectKind::LightningRod,
        },
        OnGameplayScreen,
    ));
}

/// Updates lightning rod timers, spawns lightning strikes, and despawns expired rods.
pub(super) fn update_lightning_rod(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut rods: Query<(Entity, &mut LightningRod)>,
) {
    let delta = time.delta_secs();

    for (entity, mut rod) in rods.iter_mut() {
        rod.time_alive += delta;
        rod.time_since_strike += delta;

        // Despawn if expired
        if rod.is_expired() {
            commands.entity(entity).try_despawn();
            continue;
        }

        // T1-1 Rapid Strikes: reduce interval
        let effective_interval = STRIKE_INTERVAL * rod.talent_params.strike_interval_mult;

        // Spawn lightning strike on interval
        if rod.time_since_strike >= effective_interval {
            rod.time_since_strike = 0.0;
            rod.strike_count += 1;

            // Calculate effective damage
            let mut arc_damage = ARC_DAMAGE * rod.empowerment * rod.talent_params.damage_mult;

            // T3-1 Tesla Coil: apply accumulated ramp, then increment for next strike
            arc_damage *= 1.0 + rod.damage_ramp;
            if rod.talent_params.tesla_coil {
                rod.damage_ramp += TESLA_COIL_RAMP_PER_STRIKE;
            }

            // T2-2 Overcharge: every Nth strike deals bonus damage
            if rod.talent_params.overcharge && rod.strike_count % OVERCHARGE_EVERY_N == 0 {
                arc_damage *= OVERCHARGE_DAMAGE_MULT;
            }

            // T1-2 Wider Arc: radius and targets
            let arc_radius = ARC_RADIUS * rod.empowerment * rod.talent_params.arc_radius_mult;
            let max_targets = ARC_MAX_TARGETS + rod.talent_params.extra_targets;

            let target_pos = Vec3::new(rod.position.x, TOWER_HEIGHT, rod.position.z);

            spawn_lightning_bolt(
                &mut commands,
                &visual_assets,
                LightningStrike {
                    target_pos,
                    speed: STRIKE_SPEED,
                    arc_damage,
                    arc_radius,
                    empowerment: rod.empowerment,
                    max_targets,
                    nexus_damage_mult: 1.0,
                    talent_params: rod.talent_params,
                },
            );
        }
    }
}

/// Moves lightning strikes downward and triggers arcs when they hit the rod.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_lightning_strikes(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut strikes: Query<(Entity, &mut Transform, &LightningStrike)>,
    mut screen_flash: MessageWriter<crate::game::crt_effect::ScreenFlashMessage>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut SlowMovementModifier>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, strike) in strikes.iter_mut() {
        // Move downward
        transform.translation.y -= strike.speed * delta;

        // Check if bolt has reached the rod
        if transform.translation.y <= strike.target_pos.y {
            screen_flash.write(crate::game::crt_effect::ScreenFlashMessage {
                color: [1.0, 1.0, 1.0],
                duration: 0.15,
                intensity: 0.04,
            });

            // Spawn arcs to nearby units
            let kills = spawn_arcs_to_nearby_units(
                &mut commands,
                &visual_assets,
                strike.target_pos,
                strike.arc_damage,
                strike.arc_radius,
                strike.empowerment,
                strike.max_targets,
                &strike.talent_params,
                &mut units,
                &mut talent_progress,
            );

            // T3-2 Lightning Nexus: kills trigger a bonus strike with compounding falloff
            if strike.talent_params.lightning_nexus && kills > 0 {
                let next_mult = strike.nexus_damage_mult * LIGHTNING_NEXUS_FALLOFF;
                // Only spawn bonus if damage is still meaningful (> 5% of original)
                if next_mult >= 0.05 {
                    spawn_lightning_bolt(
                        &mut commands,
                        &visual_assets,
                        LightningStrike {
                            target_pos: strike.target_pos,
                            speed: STRIKE_SPEED,
                            arc_damage: strike.arc_damage * LIGHTNING_NEXUS_FALLOFF,
                            arc_radius: strike.arc_radius,
                            empowerment: strike.empowerment,
                            max_targets: strike.max_targets,
                            nexus_damage_mult: next_mult,
                            talent_params: strike.talent_params,
                        },
                    );
                }
            }

            // Despawn the strike bolt
            commands.entity(entity).try_despawn();
        }
    }
}

/// Finds nearby units and spawns lightning arcs from the rod to each target.
/// Returns the number of units killed by the arcs.
#[allow(clippy::too_many_arguments)]
fn spawn_arcs_to_nearby_units(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    rod_top: Vec3,
    damage: f32,
    radius: f32,
    empowerment: f32,
    max_targets: usize,
    talent_params: &LightningRodTalentParams,
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut SlowMovementModifier>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
    talent_progress: &mut Option<ResMut<BattleTalentProgress>>,
) -> u32 {
    // Collect targets sorted by distance (closest first)
    let mut targets: Vec<(Entity, Vec3, f32)> = units
        .iter()
        .map(|(entity, transform, _, _, _, _)| {
            let pos = transform.translation;
            let dist = Vec3::new(rod_top.x, 0.0, rod_top.z).distance(Vec3::new(pos.x, 0.0, pos.z));
            (entity, pos, dist)
        })
        .filter(|(_, _, dist)| *dist <= radius)
        .collect();

    targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));
    targets.truncate(max_targets);

    let mut kills = 0u32;
    let mut hit_entities: Vec<Entity> = Vec::new();
    let hit_count = targets.len() as u32;

    // Apply damage and spawn arc visuals
    for (target_entity, target_pos, _) in &targets {
        kills += apply_arc_hit(commands, units, *target_entity, damage, talent_params);
        hit_entities.push(*target_entity);
        spawn_arc(commands, assets, rod_top, *target_pos, empowerment);
    }

    // T2-0 Chain Reaction: chain from each hit target to additional nearby enemies
    if talent_params.chain_reaction {
        let chain_damage = damage * CHAIN_REACTION_DAMAGE_MULT;

        // Collect chain targets from each primary target
        let mut chain_targets: Vec<(Entity, Vec3, Vec3)> = Vec::new(); // (entity, pos, source_pos)

        for (primary_entity, primary_pos, _) in &targets {
            // Find closest unit to the primary target that wasn't already hit
            let mut candidates: Vec<(Entity, Vec3, f32)> = units
                .iter()
                .filter_map(|(entity, transform, _, _, _, _)| {
                    if hit_entities.contains(&entity) || entity == *primary_entity {
                        return None;
                    }
                    let pos = transform.translation;
                    let dist = Vec3::new(primary_pos.x, 0.0, primary_pos.z)
                        .distance(Vec3::new(pos.x, 0.0, pos.z));
                    if dist <= radius {
                        Some((entity, pos, dist))
                    } else {
                        None
                    }
                })
                .collect();

            candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));
            candidates.truncate(CHAIN_REACTION_EXTRA_TARGETS);

            for (chain_entity, chain_pos, _) in candidates {
                if !chain_targets.iter().any(|(e, _, _)| *e == chain_entity) {
                    chain_targets.push((chain_entity, chain_pos, *primary_pos));
                    hit_entities.push(chain_entity);
                }
            }
        }

        // Apply chain damage
        for (chain_entity, chain_pos, source_pos) in &chain_targets {
            kills += apply_arc_hit(commands, units, *chain_entity, chain_damage, talent_params);
            spawn_arc(commands, assets, *source_pos, *chain_pos, empowerment);
        }
    }

    // Track talent progress: "Enemies struck by arcs"
    if let Some(progress) = talent_progress
        && hit_count > 0
    {
        progress.increment(Spell::LightningRod, hit_count);
    }

    kills
}

/// Applies arc damage to a single target, optionally slows it (Magnetic Field), and returns 1 if killed.
fn apply_arc_hit(
    commands: &mut Commands,
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut SlowMovementModifier>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
    entity: Entity,
    damage: f32,
    talent_params: &LightningRodTalentParams,
) -> u32 {
    let Ok((_, _, mut health, mut temp_hp, has_spell_shield, mut slow)) = units.get_mut(entity)
    else {
        return 0;
    };

    let was_alive = health.current > 0.0;

    apply_spell_damage(
        commands,
        entity,
        &mut health,
        temp_hp.as_deref_mut(),
        damage,
        DamageType::Electric,
        has_spell_shield,
    );

    let killed = u32::from(was_alive && health.current <= 0.0);

    // T2-1 Magnetic Field: slow hit enemies
    if talent_params.magnetic_field {
        if let Some(existing_slow) = &mut slow {
            existing_slow.apply(MAGNETIC_FIELD_SLOW, MAGNETIC_FIELD_SLOW_DURATION);
        } else {
            commands.entity(entity).insert(SlowMovementModifier::new(
                MAGNETIC_FIELD_SLOW,
                MAGNETIC_FIELD_SLOW_DURATION,
            ));
        }
    }

    killed
}

/// Spawns a lightning arc visual between two points.
fn spawn_arc(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    start: Vec3,
    end: Vec3,
    empowerment: f32,
) {
    let midpoint = (start + end) / 2.0;
    let direction = (end - start).normalize();
    let length = start.distance(end);

    let arc_width = ARC_WIDTH * empowerment;

    let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

    commands.spawn((
        LightningRodArc::new(ARC_LIFETIME),
        Mesh3d(assets.unit_rect.clone()),
        MeshMaterial3d(assets.lightning_rod_arc.clone()),
        Transform::from_translation(midpoint)
            .with_rotation(rotation)
            .with_scale(Vec3::new(arc_width, length, arc_width)),
        OnGameplayScreen,
    ));
}

/// Updates lightning arc visuals with pulsing animation and despawns expired arcs.
pub(super) fn update_lightning_rod_arcs(
    time: Res<Time>,
    mut commands: Commands,
    mut arcs: Query<(
        Entity,
        &mut LightningRodArc,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut arc, material_handle) in &mut arcs {
        arc.time_alive += time.delta_secs();
        arc.lifetime -= time.delta_secs();

        // Despawn expired arcs
        if arc.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Pulsing intensity animation
        let intensity = 0.7 + 0.3 * (arc.time_alive * 20.0).sin();

        if let Some(material) = materials.get_mut(&material_handle.0) {
            let base = ARC_COLOR;
            material.base_color = Color::srgba(
                base.to_srgba().red * intensity,
                base.to_srgba().green * intensity,
                base.to_srgba().blue * intensity,
                base.to_srgba().alpha,
            );
        }
    }
}
