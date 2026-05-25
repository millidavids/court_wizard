use std::cmp::Ordering;

use rand::Rng;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::super::vfx;
use super::components::{
    BanishmentTalentParams, BanishmentVfx, DimensionalShunt, Displacement, OneWayTrip,
    PainfulReturn,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    BanishedModifier, Corpse, Health, Team, TemporaryHitPoints, WasBanished, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input, clamp_cursor_to_spell_range_with_origin,
    ground_projected_range,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> BanishmentTalentParams {
    let mut params = BanishmentTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    // Tier 1
    match talents.get_selection(Spell::Banishment, 0) {
        Some(0) => {
            params.duration = constants::EXTENDED_EXILE_DURATION;
        }
        Some(1) => {
            params.cast_time_mult = constants::QUICK_DISMISSAL_CAST_TIME_MULT;
        }
        Some(2) => {
            params.mana_mult = constants::CHEAP_TICKET_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2
    match talents.get_selection(Spell::Banishment, 1) {
        Some(0) => params.painful_return = true,
        Some(1) => params.displacement = true,
        Some(2) => params.dual_banishment = true,
        _ => {}
    }

    // Tier 3
    match talents.get_selection(Spell::Banishment, 2) {
        Some(0) => params.dimensional_shunt = true,
        Some(1) => params.mass_banishment = true,
        Some(2) => params.one_way_trip = true,
        _ => {}
    }

    params
}

/// Banishes a single target entity, applying talent components as needed.
/// Also spawns the lensing VFX at the target's position.
#[allow(clippy::too_many_arguments)]
fn banish_target(
    commands: &mut Commands,
    target: Entity,
    target_pos: Vec3,
    duration: f32,
    params: &BanishmentTalentParams,
    health: &Health,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
) {
    // Spawn shrinking lensing VFX at target position
    spawn_banishment_vfx(commands, target_pos, visual_assets);

    vfx::systems::spawn_smoke_poof(
        commands,
        visual_assets,
        &visual_assets.banishment_poof,
        target_pos,
        8,
        time_secs,
    );

    // One-Way Trip: if below HP threshold, mark for death on return
    if params.one_way_trip && health.current <= health.max * constants::ONE_WAY_TRIP_HP_THRESHOLD {
        commands.entity(target).insert((
            BanishedModifier::new(0.0), // Expires immediately next tick
            Visibility::Hidden,
            OneWayTrip,
        ));
        return;
    }

    let mut entity_commands = commands.entity(target);
    entity_commands.insert((BanishedModifier::new(duration), Visibility::Hidden));

    if params.painful_return {
        entity_commands.insert(PainfulReturn {
            damage: constants::PAINFUL_RETURN_DAMAGE,
        });
    }
    if params.displacement {
        entity_commands.insert(Displacement {
            radius: constants::DISPLACEMENT_RADIUS,
        });
    }
    if params.dimensional_shunt {
        entity_commands.insert(DimensionalShunt {
            hp_fraction: constants::DIMENSIONAL_SHUNT_HP_FRACTION,
        });
    }
}

/// Spawns a shrinking lensing sphere VFX and burst of sparks at the given position.
fn spawn_banishment_vfx(
    commands: &mut Commands,
    position: Vec3,
    visual_assets: &SpellVisualAssets,
) {
    let start_radius = constants::VFX_START_RADIUS;
    commands.spawn((
        BanishmentVfx {
            time_alive: 0.0,
            lifetime: constants::VFX_LIFETIME,
            start_radius,
        },
        Mesh3d(visual_assets.cross_plane_sphere.clone()),
        MeshMaterial3d(visual_assets.banishment_lens.clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(start_radius)),
        OnGameplayScreen,
    ));

    // Spawn exploding spark particles (reuses FireSpark component + update system)
    vfx::systems::spawn_sparks_with_material(
        commands,
        visual_assets,
        position,
        constants::SPARK_COUNT,
        0.0,
        visual_assets.banishment_spark.clone(),
    );
}

/// Local wizard banishment casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_banishment_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    enemies_query: Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    mut progress: ResMut<BattleTalentProgress>,
    visual_assets: Res<SpellVisualAssets>,
    target_assist: Res<TargetAssistWorldPos>,
    local_origin: Res<LocalSpellOrigin>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((_wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Banishment {
        return;
    }

    let spell_range = ground_projected_range(wizard.spell_range, local_origin.0.y);
    input.cursor_pos = clamp_cursor_to_spell_range_with_origin(input.cursor_pos, local_origin.0, wizard.spell_range, 0.0);

    let talent_params = compute_talent_params(active_talents.as_deref());

    let banished_count = banishment_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &enemies_query,
        &talent_params,
        spell_range,
        &visual_assets,
        time.elapsed_secs(),
        local_origin.0,
    );

    if banished_count > 0 {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            local_origin.0,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        progress.increment(Spell::Banishment, banished_count);
        audio::play_sfx(
            &mut commands,
            &sfx.banishment_cast,
            local_origin.0,
            &game_config,
            &sfx,
        );
        mouse_state.left_consumed = true;
    }
}

/// Core banishment casting logic. Returns the number of units banished.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn banishment_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    params: &BanishmentTalentParams,
    spell_range: f32,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    local_origin: Vec3,
) -> u32 {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return 0;
    }

    let mana_cost = if params.mass_banishment {
        constants::MASS_BANISHMENT_MANA_COST * params.mana_mult
    } else {
        constants::MANA_COST * params.mana_mult
    };
    let cast_time = primed_spell.cast_time * params.cast_time_mult;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(mana_cost) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(cast_time) {
                if let Some(cursor_pos) = input.cursor_pos
                    && mana.consume(mana_cost)
                {
                    let banished = if params.mass_banishment {
                        cast_mass_banishment(
                            commands,
                            enemies_query,
                            cursor_pos,
                            primed_spell.empowerment,
                            params,
                            spell_range,
                            visual_assets,
                            time_secs,
                            local_origin,
                        )
                    } else {
                        cast_single_banishment(
                            commands,
                            enemies_query,
                            cursor_pos,
                            primed_spell.empowerment,
                            mana,
                            params,
                            spell_range,
                            visual_assets,
                            time_secs,
                            local_origin,
                        )
                    };
                    casting_state.cancel();
                    return banished;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    0
}

/// Returns true if the target is within the wizard's spell range.
fn is_in_spell_range(target_pos: Vec3, spell_range: f32, local_origin: Vec3) -> bool {
    let dx = target_pos.x - local_origin.x;
    let dz = target_pos.z - local_origin.z;
    (dx * dx + dz * dz) <= spell_range * spell_range
}

/// Standard single-target (or dual-target) banishment.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn cast_single_banishment(
    commands: &mut Commands,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    cursor_pos: Vec3,
    empowerment: f32,
    mana: &mut Mana,
    params: &BanishmentTalentParams,
    spell_range: f32,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    local_origin: Vec3,
) -> u32 {
    let duration = params.duration * empowerment;
    let mut banished_count = 0u32;

    // Find candidates within spell range, sorted by distance to cursor
    let mut candidates: Vec<(Entity, f32, Vec3, &Health)> = enemies_query
        .iter()
        .filter(|(_, _, team, _)| Team::Defenders.is_enemy(team))
        .filter(|(_, transform, _, _)| {
            is_in_spell_range(transform.translation, spell_range, local_origin)
        })
        .map(|(entity, transform, _, health)| {
            let xz_dist = crate::game::units::wizard::spells::utils::xz_distance(
                transform.translation,
                cursor_pos,
            );
            (entity, xz_dist * xz_dist, transform.translation, health)
        })
        .collect();
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

    // Banish first target (nearest to cursor)
    if let Some(&(target, _, pos, health)) = candidates.first() {
        banish_target(
            commands,
            target,
            pos,
            duration,
            params,
            health,
            visual_assets,
            time_secs,
        );
        banished_count += 1;
    }

    // Dual Banishment: banish second target if we can afford it
    if params.dual_banishment && candidates.len() > 1 {
        let base_mana_cost = constants::MANA_COST * params.mana_mult;
        let second_mana_cost = base_mana_cost * constants::DUAL_BANISHMENT_SECOND_MANA_MULT;
        if mana.consume(second_mana_cost) {
            let (target, _, pos, health) = candidates[1];
            banish_target(
                commands,
                target,
                pos,
                duration,
                params,
                health,
                visual_assets,
                time_secs,
            );
            banished_count += 1;
        }
    }

    banished_count
}

/// Mass Banishment: banishes all enemies in a radius. Short duration, high cost.
#[allow(clippy::too_many_arguments)]
fn cast_mass_banishment(
    commands: &mut Commands,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &Health),
        (
            Without<Corpse>,
            Without<WasBanished>,
            Without<BanishedModifier>,
        ),
    >,
    cursor_pos: Vec3,
    empowerment: f32,
    params: &BanishmentTalentParams,
    spell_range: f32,
    visual_assets: &SpellVisualAssets,
    time_secs: f32,
    local_origin: Vec3,
) -> u32 {
    let duration = constants::MASS_BANISHMENT_DURATION * empowerment;
    let mut banished_count = 0u32;

    for (entity, transform, team, health) in enemies_query.iter() {
        if !Team::Defenders.is_enemy(team) {
            continue;
        }
        if !is_in_spell_range(transform.translation, spell_range, local_origin) {
            continue;
        }
        let xz_dist = crate::game::units::wizard::spells::utils::xz_distance(
            transform.translation,
            cursor_pos,
        );
        if xz_dist * xz_dist > constants::MASS_BANISHMENT_RADIUS * constants::MASS_BANISHMENT_RADIUS
        {
            continue;
        }

        banish_target(
            commands,
            entity,
            transform.translation,
            duration,
            params,
            health,
            visual_assets,
            time_secs,
        );
        banished_count += 1;
    }

    banished_count
}

/// Animates banishment lensing VFX: shrinks from start radius to zero, then despawns.
pub fn update_banishment_vfx(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BanishmentVfx, &mut Transform)>,
) {
    let delta = time.delta_secs();
    for (entity, mut vfx, mut transform) in &mut query {
        vfx.time_alive += delta;
        if vfx.time_alive >= vfx.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = vfx.time_alive / vfx.lifetime;
        // Quadratic ease-in for accelerating collapse
        let remaining = 1.0 - progress * progress;
        let radius = vfx.start_radius * remaining;
        transform.scale = Vec3::splat(radius.max(0.01));
    }
}

/// Ticks banished unit timers and restores them when expired.
/// Handles talent effects on return: Painful Return, Displacement, Dimensional Shunt, One-Way Trip.
#[allow(clippy::type_complexity)]
pub fn tick_banished_units(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut banished: Query<(
        Entity,
        &mut BanishedModifier,
        &mut Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&PainfulReturn>,
        Option<&Displacement>,
        Option<&DimensionalShunt>,
        Option<&OneWayTrip>,
    )>,
) {
    let delta = time.delta_secs();
    for (
        entity,
        mut modifier,
        mut transform,
        mut health,
        mut temp_hp,
        painful_return,
        displacement,
        dimensional_shunt,
        one_way_trip,
    ) in &mut banished
    {
        if !modifier.update(delta) {
            continue;
        }

        // One-Way Trip: unit doesn't return, just dies (stays hidden until corpse conversion)
        if one_way_trip.is_some() {
            health.current = 0.0;
            commands
                .entity(entity)
                .remove::<BanishedModifier>()
                .remove::<OneWayTrip>()
                .insert(WasBanished);
            continue;
        }

        // Dimensional Shunt: set HP to fraction of max
        if let Some(shunt) = dimensional_shunt {
            let target_hp = health.max * shunt.hp_fraction;
            if health.current > target_hp {
                health.current = target_hp;
            }
        }

        // Painful Return: deal damage on return
        if let Some(painful) = painful_return {
            apply_spell_damage(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                painful.damage,
                DamageType::Force,
                false,
            );
        }

        // Displacement: randomize return position, clamped to battlefield
        if let Some(displace) = displacement {
            let half = BATTLEFIELD_SIZE / 2.0;
            let angle = game_rng.0.random::<f32>() * std::f32::consts::TAU;
            let dist = displace.radius * 0.5 + game_rng.0.random::<f32>() * displace.radius * 0.5;
            transform.translation.x =
                (transform.translation.x + angle.cos() * dist).clamp(-half, half);
            transform.translation.z =
                (transform.translation.z + angle.sin() * dist).clamp(-half, half);
        }

        // Clean up talent components and restore visibility
        commands
            .entity(entity)
            .remove::<BanishedModifier>()
            .remove::<PainfulReturn>()
            .remove::<Displacement>()
            .remove::<DimensionalShunt>()
            .remove::<OneWayTrip>()
            .insert(Visibility::Visible)
            .insert(WasBanished);
    }
}
