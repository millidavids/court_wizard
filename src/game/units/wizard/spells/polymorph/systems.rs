use std::cmp::Ordering;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    ContagiousBaas, DireSheep, ExplosiveSheep, PermanentLivestock, PigForm, PolymorphTalentParams,
};
use super::constants;
use super::sheep_visual::SheepBounce;
use crate::config::GameConfig;
use crate::game::components::Billboard;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::DamageType;
use crate::game::units::components::{
    AttackTiming, Corpse, Health, PolymorphedModifier, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::systems::create_sprite_material;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Removes all polymorph-related talent components from an entity.
fn strip_polymorph_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<PolymorphedModifier>()
        .remove::<ExplosiveSheep>()
        .remove::<ContagiousBaas>()
        .remove::<PigForm>()
        .remove::<PermanentLivestock>()
        .remove::<DireSheep>()
        .remove::<SheepBounce>()
        .remove::<Billboard>();
}

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> PolymorphTalentParams {
    let mut params = PolymorphTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Polymorph, 0);
    let t2 = talents.get_selection(Spell::Polymorph, 1);
    let t3 = talents.get_selection(Spell::Polymorph, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Extended Transformation
            params.duration = constants::EXTENDED_DURATION;
        }
        Some(1) => {
            // Fragile Form
            params.sheep_hp = constants::FRAGILE_SHEEP_HP;
        }
        Some(2) => {
            // Quick Shapeshift
            params.cast_time_mult = constants::QUICK_SHAPESHIFT_CAST_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.explosive = true,
        Some(1) => params.contagious = true,
        Some(2) => params.pig_form = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.permanent = true,
        Some(1) => params.mass = true,
        Some(2) => params.dire = true,
        _ => {}
    }

    params
}

/// Applies the polymorph effect to a single target entity.
#[allow(clippy::too_many_arguments)]
fn apply_polymorph_to_target(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    target_entity: Entity,
    target_health: &Health,
    target_material: &MeshMaterial3d<StandardMaterial>,
    target_mesh: &Mesh3d,
    target_team: Team,
    duration: f32,
    talent_params: &PolymorphTalentParams,
    empowerment: f32,
    position: Vec3,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) {
    vfx::systems::spawn_smoke_poof_synced(
        commands,
        visual_assets,
        pending,
        &visual_assets.polymorph_poof,
        crate::networking::snapshot::PoofVariant::Polymorph,
        position,
        8,
        time_secs,
    );
    let (sheep_hp, color) = if talent_params.dire {
        (constants::DIRE_SHEEP_HP, constants::DIRE_SHEEP_COLOR)
    } else if talent_params.pig_form {
        (talent_params.sheep_hp, constants::PIG_COLOR)
    } else {
        (talent_params.sheep_hp, constants::SHEEP_COLOR)
    };

    let sheep_material = create_sprite_material(
        materials,
        visual_assets.sheep_icon.clone(),
        color,
        Vec2::ONE,
        Vec2::ZERO,
    );

    let mut entity_cmds = commands.entity(target_entity);
    entity_cmds.insert((
        PolymorphedModifier::new(
            duration,
            target_health.current,
            target_health.max,
            target_material.0.clone(),
            target_mesh.0.clone(),
            target_team,
        ),
        MeshMaterial3d(sheep_material),
        Mesh3d(visual_assets.sheep_mesh.clone()),
        Health::new(sheep_hp),
        SheepBounce {
            base_y: position.y,
            elapsed: 0.0,
        },
        Billboard,
    ));
    entity_cmds.remove::<AttackTiming>();

    // Insert talent-specific behavioral components
    if talent_params.explosive {
        entity_cmds.insert(ExplosiveSheep);
    }
    if talent_params.contagious {
        // Spread targets also get ContagiousBaas so it keeps jumping
        entity_cmds.insert(ContagiousBaas {
            empowerment,
            talent_params: *talent_params,
        });
    }
    if talent_params.pig_form {
        entity_cmds.insert(PigForm);
    }
    if talent_params.permanent {
        entity_cmds.insert(PermanentLivestock);
    }
    if talent_params.dire {
        entity_cmds.insert((DireSheep::new(), Team::Defenders, AttackTiming::new()));
    }
}

/// Local wizard polymorph casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_polymorph_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    targets_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
            &Mesh3d,
            &Team,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    target_assist: Res<TargetAssistWorldPos>,
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
    local_origin: Res<LocalSpellOrigin>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (active_talents, mut talent_progress) = talent_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((_wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Polymorph {
        return;
    }

    let cursor_pos = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        0.0,
    );
    input.cursor_pos = cursor_pos;

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = polymorph_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &mut materials,
        &targets_query,
        &talent_params,
        &visual_assets,
        &mut pending_cast_events,
    );

    if completed > 0 {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Transmutation,
            time.elapsed_secs(),
        );
        if let Some(pos) = cursor_pos {
            audio::play_sfx(&mut commands, &sfx.polymorph_cast, pos, &game_config, &sfx);
        }
        mouse_state.left_consumed = true;
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::Polymorph, completed);
        }
    }
}

/// Core polymorph casting logic. Returns the number of enemies polymorphed (0 if cancelled/failed).
#[allow(clippy::too_many_arguments)]
fn polymorph_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    targets_query: &Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
            &Mesh3d,
            &Team,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    talent_params: &PolymorphTalentParams,
    visual_assets: &SpellVisualAssets,
    pending: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) -> u32 {
    let time_secs = time.elapsed_secs();
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return 0;
    }

    let mut polymorphed_count = 0;
    let cast_time = primed_spell.cast_time * talent_params.cast_time_mult;
    let mana_cost = if talent_params.mass {
        constants::MANA_COST * constants::MASS_POLYMORPH_MANA_MULT
    } else {
        constants::MANA_COST
    };

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(mana_cost) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(cast_time) {
                if mana.consume(mana_cost)
                    && let Some(cursor_pos) = input.cursor_pos
                {
                    let duration = talent_params.duration * primed_spell.empowerment;

                    if talent_params.mass {
                        // Mass Polymorph: collect target entity IDs first, then apply
                        let target_entities: Vec<Entity> = targets_query
                            .iter()
                            .filter(|(_, transform, _, _, _, _)| {
                                transform.translation.distance(cursor_pos)
                                    <= constants::MASS_POLYMORPH_RADIUS
                            })
                            .map(|(entity, _, _, _, _, _)| entity)
                            .collect();

                        for entity in &target_entities {
                            if let Ok((_, transform, health, material, mesh, team)) =
                                targets_query.get(*entity)
                            {
                                apply_polymorph_to_target(
                                    commands,
                                    materials,
                                    *entity,
                                    health,
                                    material,
                                    mesh,
                                    *team,
                                    duration,
                                    talent_params,
                                    primed_spell.empowerment,
                                    transform.translation,
                                    visual_assets,
                                    time_secs,
                                    pending,
                                );
                                polymorphed_count += 1;
                            }
                        }
                    } else {
                        // Single target: find nearest enemy in radius
                        if let Some((
                            target_entity,
                            _,
                            target_transform,
                            target_health,
                            target_material,
                            target_mesh,
                            target_team,
                        )) = targets_query
                            .iter()
                            .filter_map(|(entity, transform, health, material, mesh, team)| {
                                let dist = transform.translation.distance(cursor_pos);
                                if dist <= constants::TARGET_SEARCH_RADIUS {
                                    Some((entity, dist, transform, health, material, mesh, team))
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
                        {
                            apply_polymorph_to_target(
                                commands,
                                materials,
                                target_entity,
                                target_health,
                                target_material,
                                target_mesh,
                                *target_team,
                                duration,
                                talent_params,
                                primed_spell.empowerment,
                                target_transform.translation,
                                visual_assets,
                                time_secs,
                                pending,
                            );
                            polymorphed_count += 1;
                        }
                    }
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    polymorphed_count
}

/// Ticks polymorphed unit timers and restores them when expired.
/// Handles Permanent Livestock (stay forever), Dire Sheep (kill on expiry),
/// and Contagious Baas (spread to nearest unit on expiry).
/// Explosive Sheep detonation on death is handled by `check_explosive_sheep_deaths`.
#[allow(clippy::too_many_arguments)]
pub fn tick_polymorphed_units(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    mut polymorphed: Query<(
        Entity,
        &mut Transform,
        &mut PolymorphedModifier,
        &mut Health,
        Option<&ContagiousBaas>,
        Option<&SheepBounce>,
        Has<PermanentLivestock>,
        Has<DireSheep>,
    )>,
    targets_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
            &Mesh3d,
            &Team,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let delta = time.delta_secs();

    for (
        entity,
        mut transform,
        mut modifier,
        mut health,
        contagious,
        sheep_bounce,
        is_permanent,
        is_dire,
    ) in &mut polymorphed
    {
        if modifier.update(delta) {
            if is_permanent {
                // Permanent Livestock: sheep survives full duration, stays polymorphed forever.
                // Set timer to infinity so it never reverts, and remove the marker.
                modifier.time_remaining = f32::MAX;
                commands.entity(entity).remove::<PermanentLivestock>();
                continue;
            }

            // Contagious Baas: spread polymorph to nearest unit on expiry (any unit, magic is indiscriminate)
            if let Some(contagious) = contagious {
                let nearest = targets_query
                    .iter()
                    .map(|(e, t, h, m, mesh, team)| {
                        let dist = t.translation.distance(transform.translation);
                        (e, dist, t.translation, h, m, mesh, *team)
                    })
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

                if let Some((
                    target_entity,
                    _,
                    target_pos,
                    target_health,
                    target_material,
                    target_mesh,
                    target_team,
                )) = nearest
                {
                    let empowerment = contagious.empowerment;
                    let talent_params = contagious.talent_params;
                    let duration = talent_params.duration * empowerment;

                    apply_polymorph_to_target(
                        &mut commands,
                        &mut materials,
                        target_entity,
                        target_health,
                        target_material,
                        target_mesh,
                        target_team,
                        duration,
                        &talent_params,
                        empowerment,
                        target_pos,
                        &visual_assets,
                        time.elapsed_secs(),
                        &mut pending_cast_events,
                    );

                    audio::play_sfx(
                        &mut commands,
                        &sfx.polymorph_cast,
                        transform.translation,
                        &game_config,
                        &sfx,
                    );

                    if let Some(ref mut progress) = talent_progress {
                        progress.increment(Spell::Polymorph, 1);
                    }
                }
            }

            if let Some(bounce) = sheep_bounce {
                transform.translation.y = bounce.base_y;
            }

            let mut e = commands.entity(entity);
            e.insert((
                MeshMaterial3d(modifier.original_material.clone()),
                Mesh3d(modifier.original_mesh.clone()),
            ));
            if is_dire {
                health.current = 0.0;
                e.insert(modifier.original_team);
            } else {
                health.current = modifier.original_health_current;
                health.max = modifier.original_health_max;
                e.insert(AttackTiming::new());
            }
            strip_polymorph_components(&mut commands, entity);
        }
    }
}

/// Checks for sheep that die (health depleted) and triggers explosive sheep if applicable.
/// This runs after tick_polymorphed_units to catch deaths from combat and timer expiry.
pub fn check_explosive_sheep_deaths(
    mut commands: Commands,
    sheep_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &PolymorphedModifier,
            Has<ExplosiveSheep>,
        ),
        (Without<Corpse>, Without<DireSheep>),
    >,
    mut damage_targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
) {
    for (sheep_entity, sheep_transform, sheep_health, modifier, is_explosive) in &sheep_query {
        if !sheep_health.is_dead() {
            continue;
        }

        // Restore original team so corpse/kill tracking uses the correct team
        commands.entity(sheep_entity).insert(modifier.original_team);
        strip_polymorph_components(&mut commands, sheep_entity);

        if is_explosive {
            // Deal AoE damage to nearby enemies
            for (
                target_entity,
                target_transform,
                mut target_health,
                mut temp_hp,
                has_spell_shield,
            ) in &mut damage_targets
            {
                let dist = target_transform
                    .translation
                    .distance(sheep_transform.translation);
                if dist <= constants::EXPLOSIVE_SHEEP_RADIUS {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut target_health,
                        temp_hp.as_deref_mut(),
                        constants::EXPLOSIVE_SHEEP_DAMAGE,
                        DamageType::Nature,
                        has_spell_shield,
                    );
                }
            }
        }
    }
}

/// Pig Form: polymorphed pigs flee away from the nearest unit at high speed.
pub fn handle_pig_movement(
    pig_query: Query<(Entity, &Transform), (With<PigForm>, With<PolymorphedModifier>)>,
    units_query: Query<&Transform, (Without<Corpse>, Without<PolymorphedModifier>)>,
    mut velocity_query: Query<&mut crate::game::components::Velocity>,
) {
    for (pig_entity, pig_transform) in &pig_query {
        // Find nearest living unit to flee from
        let mut nearest_dist = f32::MAX;
        let mut flee_dir = Vec3::new(0.0, 0.0, -1.0); // Default flee direction

        for unit_transform in &units_query {
            let dist = pig_transform
                .translation
                .distance(unit_transform.translation);
            if dist < nearest_dist {
                nearest_dist = dist;
                let dir = pig_transform.translation - unit_transform.translation;
                if dir.length_squared() > 0.01 {
                    flee_dir = dir.normalize();
                }
            }
        }

        if let Ok(mut velocity) = velocity_query.get_mut(pig_entity) {
            velocity.x = flee_dir.x * constants::PIG_SPEED;
            velocity.z = flee_dir.z * constants::PIG_SPEED;
        }
    }
}

/// Dire Sheep: friendly sheep that moves toward and attacks nearby enemies.
pub fn tick_dire_sheep(
    mut commands: Commands,
    time: Res<Time>,
    mut sheep_query: Query<
        (Entity, &Transform, &mut DireSheep),
        (With<PolymorphedModifier>, Without<Corpse>),
    >,
    mut enemies_query: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            &Team,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Corpse>, Without<PolymorphedModifier>),
    >,
    mut velocity_query: Query<&mut crate::game::components::Velocity>,
) {
    let delta = time.delta_secs();

    for (sheep_entity, sheep_transform, mut dire) in &mut sheep_query {
        dire.attack_timer -= delta;

        // Find nearest enemy (attackers or undead)
        let mut nearest_enemy: Option<(Entity, f32, Vec3)> = None;
        for (entity, transform, _, team, _, _) in &enemies_query {
            if Team::Defenders.is_enemy(team) {
                let dist = sheep_transform.translation.distance(transform.translation);
                if nearest_enemy.as_ref().is_none_or(|e| dist < e.1) {
                    nearest_enemy = Some((entity, dist, transform.translation));
                }
            }
        }

        if let Some((enemy_entity, dist, enemy_pos)) = nearest_enemy {
            // Move toward nearest enemy
            let dir = (enemy_pos - sheep_transform.translation).normalize_or_zero();
            if let Ok(mut velocity) = velocity_query.get_mut(sheep_entity) {
                velocity.x = dir.x * constants::DIRE_SHEEP_MOVE_SPEED;
                velocity.z = dir.z * constants::DIRE_SHEEP_MOVE_SPEED;
            }

            // Attack if in range and timer ready
            if dist <= constants::DIRE_SHEEP_ATTACK_RADIUS && dire.attack_timer <= 0.0 {
                dire.attack_timer = constants::DIRE_SHEEP_ATTACK_INTERVAL;
                if let Ok((_, _, mut enemy_health, _, mut temp_hp, has_spell_shield)) =
                    enemies_query.get_mut(enemy_entity)
                {
                    apply_spell_damage(
                        &mut commands,
                        enemy_entity,
                        &mut enemy_health,
                        temp_hp.as_deref_mut(),
                        constants::DIRE_SHEEP_DAMAGE,
                        DamageType::Nature,
                        has_spell_shield,
                    );
                }
            }
        } else {
            // No enemies found, stop moving
            if let Ok(mut velocity) = velocity_query.get_mut(sheep_entity) {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
        }
    }
}
