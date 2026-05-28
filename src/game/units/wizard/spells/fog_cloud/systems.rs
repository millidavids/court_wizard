use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{
    BlindingMistDebuff, BlindingMistZone, ChokingFogZone, ConcealingVeilZone,
    DisorientingVaporsZone, FogCloudTalentParams, FogCloudZone, PhantomFogZone, PhantomUnit,
    RollingFogZone,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{
    AttackTiming, Corpse, Effectiveness, FogEvasionModifier, Health, Hitbox, Stunned, Team,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, handle_spell_release, try_start_cast_with_indicator,
    update_indicator_position, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> FogCloudTalentParams {
    let mut params = FogCloudTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    // Tier 1
    match talents.get_selection(Spell::FogCloud, 0) {
        Some(0) => params.evasion_chance = constants::DENSE_FOG_EVASION,
        Some(1) => params.radius_mult = constants::EXPANDING_MISTS_RADIUS_MULT,
        Some(2) => params.linger_duration = constants::CLINGING_HAZE_LINGER,
        _ => {}
    }

    // Tier 2
    match talents.get_selection(Spell::FogCloud, 1) {
        Some(0) => params.blinding_mist = true,
        Some(1) => params.concealing_veil = true,
        Some(2) => params.disorienting_vapors = true,
        _ => {}
    }

    // Tier 3
    match talents.get_selection(Spell::FogCloud, 2) {
        Some(0) => params.phantom_fog = true,
        Some(1) => params.choking_fog = true,
        Some(2) => params.rolling_fog = true,
        _ => {}
    }

    params
}

/// Local wizard fog cloud casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_fog_cloud_casting(
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
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    active_toggles: Option<Res<ActiveToggles>>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::FogCloud {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = fog_cloud_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &mut commands,
        &visual_assets,
        &mut meshes,
        &sfx,
        &game_config,
        &talent_params,
        scorched_mult,
        local_origin.0,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core fog cloud casting logic. Returns true if the spell completed.
#[allow(clippy::too_many_arguments)]
fn fog_cloud_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: &FogCloudTalentParams,
    scorched_mult: f32,
    local_origin: Vec3,
) -> bool {
    let mut completed = false;

    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = local_origin;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let circle_radius = constants::CIRCLE_RADIUS * scale * talent_params.radius_mult;
    let max_center_distance = (max_ground_radius - circle_radius).max(0.0);
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                try_start_cast_with_indicator(
                    commands,
                    meshes,
                    assets.fog_cloud_indicator.clone(),
                    wizard_entity,
                    casting_state,
                    mana,
                    constants::MANA_COST,
                    cursor_world_pos,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment * talent_params.radius_mult,
                    caster_query,
                );
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            update_indicator_position(
                wizard_entity,
                cursor_world_pos,
                caster_query,
                indicator_query,
            );
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS
                                * primed_spell.empowerment
                                * talent_params.radius_mult;
                            audio::play_sfx(
                                commands,
                                &sfx.fog_cloud_cast,
                                indicator.position,
                                game_config,
                                sfx,
                            );
                            spawn_fog_cloud_zone(
                                commands,
                                indicator.position,
                                radius,
                                primed_spell.empowerment,
                                talent_params,
                                scorched_mult,
                            );
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    completed = true;
                } else {
                    cleanup_spell_caster(commands, wizard_entity, caster_query);
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}

pub fn apply_fog_cloud_evasion(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut FogCloudZone>,
    mut targets: Query<(Entity, &Transform, Option<&mut FogEvasionModifier>), Without<Corpse>>,
) {
    let delta = time.delta_secs();
    for mut zone in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;
            for (entity, transform, existing_evasion) in &mut targets {
                let dist = xz_distance(zone.origin, transform.translation);
                if dist <= zone.radius {
                    if let Some(mut evasion) = existing_evasion {
                        evasion.refresh(zone.evasion_refresh_duration);
                    } else {
                        commands.entity(entity).insert(FogEvasionModifier::new(
                            zone.evasion_chance,
                            zone.evasion_refresh_duration,
                        ));
                    }
                }
            }
        }
    }
}

/// Tier 2: Blinding Mist — apply/refresh debuff on units inside fog zones with this talent.
pub fn apply_blinding_mist(
    mut commands: Commands,
    zones: Query<(&FogCloudZone, &BlindingMistZone)>,
    mut targets: Query<(Entity, &Transform, Option<&mut BlindingMistDebuff>), Without<Corpse>>,
) {
    for (zone, _) in &zones {
        for (entity, transform, existing_debuff) in &mut targets {
            if xz_distance(zone.origin, transform.translation) <= zone.radius {
                if let Some(mut debuff) = existing_debuff {
                    debuff.refresh();
                } else {
                    commands
                        .entity(entity)
                        .insert(BlindingMistDebuff::new(constants::BLINDING_MIST_RANGE_MULT));
                }
            }
        }
    }
}

/// Tick down BlindingMistDebuff timers and remove expired ones.
pub fn tick_blinding_mist_debuff(
    mut commands: Commands,
    time: Res<Time>,
    mut debuffs: Query<(Entity, &mut BlindingMistDebuff)>,
) {
    let delta = time.delta_secs();
    for (entity, mut debuff) in &mut debuffs {
        debuff.time_remaining -= delta;
        if debuff.time_remaining <= 0.0 {
            commands.entity(entity).remove::<BlindingMistDebuff>();
        }
    }
}

/// Tier 3: Choking Fog — deal minor DPS to non-ally units inside the fog.
pub fn apply_choking_fog_damage(
    time: Res<Time>,
    mut zones: Query<(&FogCloudZone, &mut ChokingFogZone)>,
    mut targets: Query<(&Transform, &Team, &mut Health), Without<Corpse>>,
) {
    let delta = time.delta_secs();
    for (zone, mut choking) in &mut zones {
        choking.tick_accumulator += delta;
        if choking.tick_accumulator >= choking.tick_interval {
            choking.tick_accumulator -= choking.tick_interval;
            let damage = choking.dps * choking.tick_interval;
            for (transform, _team, mut health) in &mut targets {
                let dist = xz_distance(zone.origin, transform.translation);
                if dist <= zone.radius {
                    health.current = (health.current - damage).max(0.0);
                }
            }
        }
    }
}

/// Tier 3: Rolling Fog — move the fog zone toward the nearest attacker approach direction.
pub fn move_rolling_fog(
    time: Res<Time>,
    mut zones: Query<(&mut FogCloudZone, &mut Transform, &RollingFogZone)>,
    units: Query<(&Transform, &Team), (Without<Corpse>, Without<FogCloudZone>)>,
) {
    let delta = time.delta_secs();

    // Collect attacker positions
    let attacker_positions: Vec<Vec3> = units
        .iter()
        .filter(|(_, team)| **team == Team::Attackers)
        .map(|(t, _)| t.translation)
        .collect();

    for (mut zone, mut zone_transform, rolling) in &mut zones {
        // Find nearest attacker to move toward
        let mut nearest_dist = f32::MAX;
        let mut nearest_dir = Vec3::ZERO;

        for &attacker_pos in &attacker_positions {
            let diff = Vec3::new(
                attacker_pos.x - zone.origin.x,
                0.0,
                attacker_pos.z - zone.origin.z,
            );
            let dist = diff.length();
            if dist < nearest_dist && dist > 1.0 {
                nearest_dist = dist;
                nearest_dir = diff / dist;
            }
        }

        if nearest_dist < f32::MAX {
            let movement = nearest_dir * rolling.speed * delta;
            zone.origin.x += movement.x;
            zone.origin.z += movement.z;
            zone_transform.translation.x = zone.origin.x;
            zone_transform.translation.z = zone.origin.z;
        }
    }
}

/// Continuously spawns gray smoke particles from active fog cloud zones.
pub fn emit_fog_cloud_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut zones: Query<&mut FogCloudZone>,
    assets: Res<SpellVisualAssets>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for mut zone in &mut zones {
        // Don't emit particles during fade-out
        let remaining = zone.duration - zone.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        zone.smoke_spawn_timer += dt;
        if zone.smoke_spawn_timer >= vfx::constants::FOG_SMOKE_SPAWN_INTERVAL {
            zone.smoke_spawn_timer -= vfx::constants::FOG_SMOKE_SPAWN_INTERVAL;

            vfx::systems::spawn_fog_smoke_puffs(
                &mut commands,
                &assets,
                zone.origin,
                zone.radius,
                vfx::constants::FOG_SMOKE_COUNT_PER_SPAWN,
                t,
            );
        }
    }
}

/// Tier 3: Phantom Fog — periodically spawns phantom decoy units inside fog zones.
pub fn spawn_phantom_units(
    time: Res<Time>,
    mut commands: Commands,
    mut zones: Query<(Entity, &FogCloudZone, &mut PhantomFogZone)>,
    existing_phantoms: Query<&PhantomUnit>,
    infantry_assets: Res<InfantryAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    let mut phantom_count: Option<usize> = None;

    for (_zone_entity, zone, mut phantom_zone) in &mut zones {
        phantom_zone.spawn_timer += dt;
        if phantom_zone.spawn_timer < constants::PHANTOM_SPAWN_INTERVAL {
            continue;
        }
        phantom_zone.spawn_timer -= constants::PHANTOM_SPAWN_INTERVAL;

        // Count phantoms lazily (only when a zone is ready to spawn)
        let count = *phantom_count.get_or_insert_with(|| existing_phantoms.iter().count());
        if count >= constants::PHANTOM_MAX_TOTAL {
            continue;
        }

        // Spawn phantom at a random position within the zone
        let seed = t * 7.1 + zone.origin.x * 3.3;
        let angle = seed * 2.39 + (seed * 13.7).sin() * 1.5;
        let r_frac = 0.3 + 0.5 * ((seed * 23.1).sin() * 0.5 + 0.5);
        let r = zone.radius * r_frac;
        let x = zone.origin.x + angle.cos() * r;
        let z = zone.origin.z + angle.sin() * r;

        // Ghostly translucent infantry sprite
        let material = create_default_sprite_material(
            &mut materials,
            infantry_assets.sprite_texture.clone(),
            Color::srgba(0.7, 0.75, 0.85, 0.3),
        );

        commands.spawn((
            Mesh3d(infantry_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(x, constants::PHANTOM_HITBOX_HEIGHT * 0.5, z)),
            Billboard,
            Hitbox::new(
                constants::PHANTOM_HITBOX_RADIUS,
                constants::PHANTOM_HITBOX_HEIGHT,
            ),
            Health::new(1.0),
            Team::Defenders,
            AttackTiming::new(),
            Effectiveness::new(),
            Stunned {
                time_remaining: f32::MAX,
            },
            PhantomUnit,
            OnGameplayScreen,
        ));
    }
}

/// Despawns phantom units when they die or when no phantom fog zones remain.
pub fn cleanup_phantom_units(
    mut commands: Commands,
    phantoms: Query<(Entity, &Health), With<PhantomUnit>>,
    zones: Query<&PhantomFogZone>,
) {
    let zones_exist = !zones.is_empty();
    for (entity, health) in &phantoms {
        if health.current <= 0.0 || !zones_exist {
            commands.entity(entity).try_despawn();
        }
    }
}

pub fn cleanup_fog_cloud_zone(mut commands: Commands, zones: Query<(Entity, &FogCloudZone)>) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            commands.entity(entity).try_despawn();
        }
    }
}

pub(crate) fn spawn_fog_cloud_zone(
    commands: &mut Commands,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    talent_params: &FogCloudTalentParams,
    scorched_mult: f32,
) {
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
