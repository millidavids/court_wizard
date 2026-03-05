use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, WizardInput,
};
use super::components::{HarvestFlash, PsychicShockwave, TelekinesisIndicator, TransmutationStacks};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::drops::components::{FlyingToWizard, IngredientDrop};
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::messages::IngredientCollectedMessage;
use crate::game::units::components::{Health, Knockback, Team, TemporaryHitPoints};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};

/// Computed talent configuration for a single Telekinesis cast.
struct TelekinesisConfig {
    pickup_radius: f32,
    cast_time: f32,
    mana_cost: f32,
    is_storm: bool,
    has_harvest: bool,
    has_shockwave: bool,
}

/// Builds a TelekinesisConfig from the active talent selections.
fn compute_telekinesis_config(active_talents: Option<&ActiveTalents>) -> TelekinesisConfig {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::Telekinesis, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::Telekinesis, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::Telekinesis, 2));

    // T1: Auto-Target removes the pickup radius constraint
    let pickup_radius = match t1 {
        Some(0) => f32::MAX,
        _ => constants::PICKUP_RADIUS,
    };

    let cast_time = match t1 {
        Some(1) => constants::QUICK_GRAB_CAST_TIME,
        _ => constants::CAST_TIME,
    };

    let mana_cost = match t1 {
        Some(2) => constants::MANA_COST * constants::MANA_EFFICIENCY_COST_MULT,
        _ => constants::MANA_COST,
    };

    let has_harvest = t2 == Some(1);

    let is_storm = t3 == Some(0);
    let has_shockwave = t3 == Some(2);

    TelekinesisConfig {
        pickup_radius,
        cast_time,
        mana_cost,
        is_storm,
        has_harvest,
        has_shockwave,
    }
}

/// Local wizard Telekinesis casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_telekinesis_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (
            Entity,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    drops_query: Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    indicator_query: Query<&TelekinesisIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut enemies_query: Query<
        (Entity, &Transform, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<IngredientDrop>,
    >,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true, // Run conditions ensure mouse is held
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Telekinesis {
        return;
    }

    let config = compute_telekinesis_config(active_talents.as_deref());

    // Spawn indicator on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(config.mana_cost)
        && let Some(cursor_world_pos) = input.cursor_pos
        && let Some((drop_entity, drop_transform, _drop)) =
            find_nearest_drop(&cursor_world_pos, &drops_query, config.pickup_radius)
    {
        // Telekinesis has infinite range — no distance check needed
        let indicator_entity = spawn_indicator(
            &mut commands,
            &visual_assets,
            drop_transform.translation,
            drop_entity,
        );
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(indicator_entity));
    }

    let completed = telekinesis_casting_logic(
        &input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &drops_query,
        &indicator_query,
        &mut commands,
        &sfx,
        &game_config,
        &config,
        &mut enemies_query,
        &visual_assets,
    );

    if completed {
        mouse_state.left_consumed = true;
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::Telekinesis, 1);
        }
    }
}

/// Core Telekinesis casting logic -- called by the local casting system.
#[allow(clippy::too_many_arguments)]
fn telekinesis_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    drops_query: &Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    indicator_query: &Query<&TelekinesisIndicator>,
    commands: &mut Commands,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    config: &TelekinesisConfig,
    enemies_query: &mut Query<
        (Entity, &Transform, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<IngredientDrop>,
    >,
    visual_assets: &SpellVisualAssets,
) -> bool {
    // Check for release event
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).try_despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(config.mana_cost)
                && let Some(cursor_world_pos) = input.cursor_pos
                && let Some((_drop_entity, _drop_transform, _drop)) =
                    find_nearest_drop(&cursor_world_pos, drops_query, config.pickup_radius)
            {
                // Telekinesis has infinite range — no distance check needed
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(config.cast_time.min(primed_spell.cast_time)) {
                if config.is_storm {
                    // Telekinetic Storm: grab ALL drops
                    completed = execute_storm_pickup(
                        mana,
                        config,
                        drops_query,
                        commands,
                        sfx,
                        game_config,
                        enemies_query,
                        visual_assets,
                    );
                } else {
                    // Normal pickup: grab targeted drop
                    let target_drop = caster_query
                        .get(wizard_entity)
                        .ok()
                        .and_then(|caster| caster.indicator_entity)
                        .and_then(|indicator_entity| indicator_query.get(indicator_entity).ok())
                        .map(|indicator| indicator.target_drop);

                    if let Some(drop_entity) = target_drop
                        && mana.consume(config.mana_cost)
                    {
                        if let Ok((_entity, drop_transform, drop_component)) =
                            drops_query.get(drop_entity)
                        {
                            let pickup_pos = drop_transform.translation;
                            convert_drop_to_flying(commands, drop_entity, drop_component.ingredient, pickup_pos);
                            audio::play_sfx(commands, &sfx.telekinesis_cast, pickup_pos, game_config);

                            // T2: Harvest — damage nearby enemies
                            if config.has_harvest {
                                apply_harvest_damage(commands, pickup_pos, visual_assets, enemies_query);
                            }

                            // T3: Psychic Shockwave — spawn expanding ring from pickup
                            if config.has_shockwave {
                                spawn_shockwave(commands, visual_assets, pickup_pos);
                            }
                        }
                        completed = true;
                    }
                }

                // Cleanup indicator and caster
                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            // Telekinesis doesn't channel
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }

    completed
}

/// Telekinetic Storm: pick up all drops on the battlefield.
/// Picks up as many as mana allows, nearest first.
#[allow(clippy::too_many_arguments)]
fn execute_storm_pickup(
    mana: &mut Mana,
    config: &TelekinesisConfig,
    drops_query: &Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    commands: &mut Commands,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    enemies_query: &mut Query<
        (Entity, &Transform, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<IngredientDrop>,
    >,
    visual_assets: &SpellVisualAssets,
) -> bool {
    let total_cost = config.mana_cost * constants::STORM_MANA_COST_MULT;

    // Collect all drops (order doesn't matter — all are picked up)
    let all_drops: Vec<(Entity, Vec3, crate::game::cauldron::brews::Ingredient)> = drops_query
        .iter()
        .map(|(e, t, d)| (e, t.translation, d.ingredient))
        .collect();

    if all_drops.is_empty() || !mana.consume(total_cost) {
        return false;
    }

    let mut picked_any = false;
    let mut played_sfx = false;

    for (drop_entity, drop_pos, ingredient) in &all_drops {
        let start_pos = *drop_pos;
        convert_drop_to_flying(commands, *drop_entity, *ingredient, start_pos);

        // Play SFX once
        if !played_sfx {
            audio::play_sfx(commands, &sfx.telekinesis_cast, start_pos, game_config);
            played_sfx = true;
        }

        // T2: Harvest — damage nearby enemies per pickup
        if config.has_harvest {
            apply_harvest_damage(commands, start_pos, visual_assets, enemies_query);
        }

        picked_any = true;
    }

    picked_any
}

/// Converts an ingredient drop entity to the flying-to-wizard state.
fn convert_drop_to_flying(
    commands: &mut Commands,
    drop_entity: Entity,
    ingredient: crate::game::cauldron::brews::Ingredient,
    position: Vec3,
) {
    let total_distance = position.distance(crate::game::constants::WIZARD_POSITION);
    commands
        .entity(drop_entity)
        .remove::<IngredientDrop>()
        .insert(FlyingToWizard {
            ingredient,
            start_pos: position,
            total_distance,
        });
}

/// T2: Harvest — deals damage to enemies near the pickup point and spawns flash overlays.
fn apply_harvest_damage(
    commands: &mut Commands,
    pickup_pos: Vec3,
    visual_assets: &SpellVisualAssets,
    enemies_query: &mut Query<
        (Entity, &Transform, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<IngredientDrop>,
    >,
) {
    let radius_sq = constants::HARVEST_RADIUS * constants::HARVEST_RADIUS;
    for (entity, transform, _team, mut health, temp_hp) in enemies_query.iter_mut() {
        let dx = transform.translation.x - pickup_pos.x;
        let dz = transform.translation.z - pickup_pos.z;
        if dx * dx + dz * dz <= radius_sq {
            crate::game::units::components::apply_damage_to_unit(
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                constants::HARVEST_DAMAGE,
            );
            commands.entity(entity).insert(crate::game::units::components::SpellDamaged);
            // Spawn a light blue circle flash at the enemy's position
            commands.spawn((
                Mesh3d(visual_assets.unit_circle.clone()),
                MeshMaterial3d(visual_assets.harvest_flash_material.clone()),
                Transform::from_translation(Vec3::new(
                    transform.translation.x,
                    2.0,
                    transform.translation.z,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(20.0)),
                HarvestFlash {
                    time_remaining: constants::HARVEST_FLASH_DURATION,
                    material_cloned: false,
                },
                OnGameplayScreen,
            ));
        }
    }
}

/// T3: Psychic Shockwave — spawns an expanding torus ring from the ingredient pickup position.
/// Material is cloned per-entity on the first update frame for independent alpha fade.
fn spawn_shockwave(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    pickup_pos: Vec3,
) {
    commands.spawn((
        Mesh3d(visual_assets.shockwave_torus.clone()),
        MeshMaterial3d(visual_assets.shockwave_material.clone()),
        Transform::from_translation(Vec3::new(pickup_pos.x, 1.0, pickup_pos.z))
            .with_scale(Vec3::splat(0.01)),
        PsychicShockwave {
            time_alive: 0.0,
            prev_radius: 0.0,
            origin: Vec3::new(pickup_pos.x, 0.0, pickup_pos.z),
            material_cloned: false,
        },
        OnGameplayScreen,
    ));
}

/// T2: Magnetic Pull — passively drifts ingredient drops toward the wizard.
pub(super) fn magnetic_pull_ingredients(
    time: Res<Time>,
    mut drops: Query<&mut Transform, (With<IngredientDrop>, Without<FlyingToWizard>)>,
) {
    let wizard_pos = crate::game::constants::WIZARD_POSITION;
    let pull_radius_sq = constants::MAGNETIC_PULL_RADIUS * constants::MAGNETIC_PULL_RADIUS;

    for mut transform in drops.iter_mut() {
        let diff = wizard_pos - transform.translation;
        let dist_sq = diff.x * diff.x + diff.z * diff.z;

        if dist_sq <= pull_radius_sq && dist_sq > 1.0 {
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize();
            let move_dist = constants::MAGNETIC_PULL_SPEED * time.delta_secs();
            transform.translation.x += direction.x * move_dist;
            transform.translation.z += direction.z * move_dist;
        }
    }
}

/// T3: Transmutation — increments stacks when ingredients are collected.
pub(super) fn track_transmutation_stacks(
    mut collected: MessageReader<IngredientCollectedMessage>,
    mut stacks: ResMut<TransmutationStacks>,
) {
    for _ in collected.read() {
        stacks.count += 1;
    }
}

/// Updates telekinesis indicator visuals during casting.
pub(super) fn update_telekinesis_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut TelekinesisIndicator, &mut Transform)>,
    drops: Query<&Transform, (With<IngredientDrop>, Without<TelekinesisIndicator>)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();

        // Follow the drop's position
        if let Ok(drop_transform) = drops.get(indicator.target_drop) {
            transform.translation.x = drop_transform.translation.x;
            transform.translation.y = constants::INDICATOR_Y;
            transform.translation.z = drop_transform.translation.z;
        }

        // Pulse animation (unit-sized mesh scaled by radius)
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(constants::INDICATOR_RADIUS * pulse);
    }
}

/// Finds the nearest ingredient drop within the given radius of the cursor position.
fn find_nearest_drop<'a>(
    cursor_pos: &Vec3,
    drops: &'a Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    pickup_radius: f32,
) -> Option<(Entity, &'a Transform, &'a IngredientDrop)> {
    let mut nearest: Option<(Entity, &Transform, &IngredientDrop, f32)> = None;

    for (entity, transform, drop) in drops.iter() {
        let dx = transform.translation.x - cursor_pos.x;
        let dz = transform.translation.z - cursor_pos.z;
        let distance = (dx * dx + dz * dz).sqrt();

        if distance <= pickup_radius
            && (nearest.is_none() || distance < nearest.as_ref().expect("checked").3)
        {
            nearest = Some((entity, transform, drop, distance));
        }
    }

    nearest.map(|(e, t, d, _)| (e, t, d))
}

/// Clones a shared material asset into a per-entity copy for independent alpha fade.
/// Returns true if the clone was performed (first call), false otherwise.
fn clone_material_if_needed(
    commands: &mut Commands,
    entity: Entity,
    materials: &mut Assets<StandardMaterial>,
    source_handle: &Handle<StandardMaterial>,
    already_cloned: &mut bool,
) {
    if !*already_cloned {
        *already_cloned = true;
        if let Some(base_mat) = materials.get(source_handle).cloned() {
            let cloned = materials.add(base_mat);
            commands.entity(entity).insert(MeshMaterial3d(cloned));
        }
    }
}

/// Updates harvest flash overlay entities — clones material on first frame, fades alpha, despawns.
pub(super) fn update_harvest_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut query: Query<(
        Entity,
        &mut HarvestFlash,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let delta = time.delta_secs();

    for (entity, mut flash, material_handle) in &mut query {
        clone_material_if_needed(
            &mut commands,
            entity,
            &mut materials,
            &visual_assets.harvest_flash_material,
            &mut flash.material_cloned,
        );

        flash.time_remaining -= delta;

        if flash.time_remaining <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Fade alpha over the flash duration
        let alpha = (flash.time_remaining / constants::HARVEST_FLASH_DURATION).clamp(0.0, 1.0) * 0.7;
        if let Some(mat) = materials.get_mut(material_handle) {
            mat.base_color = constants::HARVEST_FLASH_COLOR.with_alpha(alpha);
        }
    }
}

/// Updates expanding psychic shockwave torus rings.
///
/// Expands the ring, applies knockback to enemies as the ring passes over them,
/// fades alpha, and despawns when complete.
pub(super) fn update_psychic_shockwave(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut shockwaves: Query<(
        Entity,
        &mut PsychicShockwave,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    enemies: Query<(Entity, &Transform, &Team), (Without<PsychicShockwave>, Without<IngredientDrop>)>,
) {
    let delta = time.delta_secs();

    for (entity, mut shockwave, mut transform, material_handle) in &mut shockwaves {
        clone_material_if_needed(
            &mut commands,
            entity,
            &mut materials,
            &visual_assets.shockwave_material,
            &mut shockwave.material_cloned,
        );

        shockwave.time_alive += delta;

        if shockwave.time_alive >= constants::SHOCKWAVE_EXPAND_DURATION {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = shockwave.time_alive / constants::SHOCKWAVE_EXPAND_DURATION;
        let current_radius = constants::SHOCKWAVE_MAX_RADIUS * progress;

        // Scale the torus to current radius
        transform.scale = Vec3::splat(current_radius.max(0.1));

        // Ring collision: knockback enemies between prev_radius and current_radius
        let prev_r_sq = shockwave.prev_radius * shockwave.prev_radius;
        let curr_r_sq = current_radius * current_radius;
        let origin = shockwave.origin;

        for (enemy_entity, enemy_transform, _team) in &enemies {
            let diff = enemy_transform.translation - origin;
            let dist_sq = diff.x * diff.x + diff.z * diff.z;

            if dist_sq > prev_r_sq && dist_sq <= curr_r_sq && dist_sq > 0.001 {
                let direction = Vec3::new(diff.x, 0.0, diff.z);
                commands.entity(enemy_entity).insert(Knockback::new(
                    direction,
                    constants::SHOCKWAVE_KNOCKBACK_SPEED,
                    constants::SHOCKWAVE_KNOCKBACK_DURATION,
                ));
            }
        }

        shockwave.prev_radius = current_radius;

        // Fade alpha as the ring expands
        if let Some(mat) = materials.get_mut(material_handle) {
            let alpha = (1.0 - progress) * 0.6;
            mat.base_color = constants::SHOCKWAVE_COLOR.with_alpha(alpha);
        }
    }
}

/// Spawns a visual indicator ring around a targeted drop.
fn spawn_indicator(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    target_drop: Entity,
) -> Entity {
    commands
        .spawn((
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(assets.telekinesis_indicator.clone()),
            Transform::from_translation(Vec3::new(position.x, constants::INDICATOR_Y, position.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(constants::INDICATOR_RADIUS)),
            TelekinesisIndicator::new(target_drop),
            OnGameplayScreen,
        ))
        .id()
}
