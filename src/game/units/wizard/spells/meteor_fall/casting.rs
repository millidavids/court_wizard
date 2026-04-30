//! Meteor fall casting and projectile spawn.

use super::meteor::find_nearest_non_defender_xz;
use bevy::prelude::*;
use rand::Rng;

use super::components::{MeteorExplosion, MeteorFallStorm, MeteorProjectile};
use super::constants::*;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::Team;
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range_ground, cleanup_spell_caster, try_start_cast_with_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material,
};
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellEffectKind;

/// Talent configuration computed once from ActiveTalents.
struct MeteorTalentConfig {
    spawn_interval: f32,
    damage_mult: f32,
    explosion_radius_mult: f32,
    ground_fire_duration_mult: f32,
    ground_fire_damage_mult: f32,
    ground_fire_radius_mult: f32,
    storm_radius_mult: f32,
    tracking: bool,
    aftershock: bool,
    extinction_event: bool,
    volcanic_eruption: bool,
    mana_cost_mult: f32,
    mesh_radius_mult: f32,
}

impl Default for MeteorTalentConfig {
    fn default() -> Self {
        Self {
            spawn_interval: METEOR_SPAWN_INTERVAL,
            damage_mult: 1.0,
            explosion_radius_mult: 1.0,
            ground_fire_duration_mult: 1.0,
            ground_fire_damage_mult: 1.0,
            ground_fire_radius_mult: 1.0,
            storm_radius_mult: 1.0,
            tracking: false,
            aftershock: false,
            extinction_event: false,
            volcanic_eruption: false,
            mana_cost_mult: 1.0,
            mesh_radius_mult: 1.0,
        }
    }
}

fn compute_meteor_talent_config(active_talents: Option<&ActiveTalents>) -> MeteorTalentConfig {
    let t1 = active_talents.and_then(|t| t.get_selection(Spell::MeteorFall, 0));
    let t2 = active_talents.and_then(|t| t.get_selection(Spell::MeteorFall, 1));
    let t3 = active_talents.and_then(|t| t.get_selection(Spell::MeteorFall, 2));

    let mut cfg = MeteorTalentConfig::default();

    // Tier 1
    match t1 {
        Some(0) => {
            // Dense Barrage: spawn rate +30%
            cfg.spawn_interval = METEOR_SPAWN_INTERVAL / DENSE_BARRAGE_SPAWN_RATE_MULT;
        }
        Some(1) => {
            // Scorching Impact: explosion + ground fire damage +30%
            cfg.damage_mult *= SCORCHING_IMPACT_DAMAGE_MULT;
            cfg.ground_fire_damage_mult *= SCORCHING_IMPACT_DAMAGE_MULT;
        }
        Some(2) => {
            // Wide Devastation: storm radius +30%
            cfg.storm_radius_mult *= WIDE_DEVASTATION_RADIUS_MULT;
        }
        _ => {}
    }

    // Tier 2
    match t2 {
        Some(0) => {
            // Molten Core: ground fire duration 2x, damage +50%
            cfg.ground_fire_duration_mult *= MOLTEN_CORE_DURATION_MULT;
            cfg.ground_fire_damage_mult *= MOLTEN_CORE_DAMAGE_MULT;
        }
        Some(1) => {
            // Tracking Meteors
            cfg.tracking = true;
        }
        Some(2) => {
            // Aftershock
            cfg.aftershock = true;
        }
        _ => {}
    }

    // Tier 3
    match t3 {
        Some(0) => {
            // Extinction Event
            cfg.extinction_event = true;
        }
        Some(1) => {
            // Volcanic Eruption
            cfg.volcanic_eruption = true;
        }
        Some(2) => {
            // Meteor Shower: 3x spawn rate, reduced damage/radius, half mana
            cfg.spawn_interval /= METEOR_SHOWER_SPAWN_RATE_MULT;
            cfg.damage_mult *= METEOR_SHOWER_DAMAGE_MULT;
            cfg.explosion_radius_mult *= METEOR_SHOWER_RADIUS_MULT;
            cfg.ground_fire_radius_mult *= METEOR_SHOWER_RADIUS_MULT;
            cfg.mana_cost_mult *= METEOR_SHOWER_MANA_MULT;
            cfg.mesh_radius_mult *= METEOR_SHOWER_MESH_MULT;
        }
        _ => {}
    }

    cfg
}

/// Local wizard meteor fall casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_meteor_fall_casting(
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
    existing_storms: Query<Entity, With<MeteorFallStorm>>,
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
    if primed_spell.spell != Spell::MeteorFall {
        return;
    }

    let talent_cfg = compute_meteor_talent_config(active_talents.as_deref());

    let completed = meteor_fall_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &existing_storms,
        &mut commands,
        &visual_assets,
        &mut meshes,
        &talent_cfg,
    );

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            vfx::systems::SpellSchool::Fire,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core meteor fall casting logic.
#[allow(clippy::too_many_arguments)]
fn meteor_fall_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    existing_storms: &Query<Entity, With<MeteorFallStorm>>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    talent_cfg: &MeteorTalentConfig,
) -> bool {
    let mut completed = false;

    // Check for release event - cancel cast
    if input.just_released {
        cleanup_spell_caster(commands, wizard_entity, caster_query);
        casting_state.cancel();
        return false;
    }

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = SPELL_ORIGIN;
    let scale = primed_spell.empowerment;
    let storm_radius = STORM_RADIUS * scale * talent_cfg.storm_radius_mult;
    let effective_mana_cost = MANA_COST * talent_cfg.mana_cost_mult;

    cursor_world_pos = clamp_to_spell_range_ground(
        cursor_world_pos,
        wizard_pos,
        wizard.spell_range,
        storm_radius,
    );

    // Handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                try_start_cast_with_indicator(
                    commands,
                    meshes,
                    assets.meteor_fall_indicator.clone(),
                    wizard_entity,
                    casting_state,
                    mana,
                    effective_mana_cost,
                    cursor_world_pos,
                    storm_radius,
                    caster_query,
                );
            }
        }
        CastingState::Casting { .. } => {
            // Currently casting - advance cast time
            casting_state.advance(time.delta_secs());

            // Update circle position to follow cursor
            update_indicator_position(
                wizard_entity,
                cursor_world_pos,
                caster_query,
                indicator_query,
            );

            // Check if cast is complete
            if casting_state.is_complete(primed_spell.cast_time) {
                // Cast complete - spawn storm entity
                if mana.consume(effective_mana_cost) {
                    // Despawn any existing storms (only one storm at a time)
                    for existing_storm in existing_storms.iter() {
                        commands.entity(existing_storm).try_despawn();
                    }

                    // Get final circle position and spawn storm
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let mut storm = MeteorFallStorm::new(
                                indicator.position,
                                storm_radius,
                                primed_spell.empowerment,
                            );
                            // Apply talent config to storm
                            storm.spawn_interval = talent_cfg.spawn_interval;
                            storm.damage_mult = talent_cfg.damage_mult;
                            storm.explosion_radius_mult = talent_cfg.explosion_radius_mult;
                            storm.ground_fire_duration_mult = talent_cfg.ground_fire_duration_mult;
                            storm.ground_fire_damage_mult = talent_cfg.ground_fire_damage_mult;
                            storm.ground_fire_radius_mult = talent_cfg.ground_fire_radius_mult;
                            storm.tracking = talent_cfg.tracking;
                            storm.aftershock = talent_cfg.aftershock;
                            storm.extinction_event = talent_cfg.extinction_event;
                            storm.volcanic_eruption = talent_cfg.volcanic_eruption;
                            storm.mesh_radius_mult = talent_cfg.mesh_radius_mult;

                            // Extinction Event: fixed-duration cast, no concentration
                            if talent_cfg.extinction_event {
                                storm.duration = Some(EXTINCTION_STORM_DURATION);
                                commands.spawn((storm, OnGameplayScreen));
                            } else {
                                commands.spawn((
                                    storm,
                                    ConcentrationSpell {
                                        spell_name: "Meteor Fall",
                                        mana_cost: MANA_COST,
                                    },
                                    OnGameplayScreen,
                                ));
                            }
                        }

                        // Despawn circle indicator
                        commands.entity(indicator_entity).try_despawn();
                    }

                    // Remove caster marker immediately
                    commands.entity(wizard_entity).remove::<SpellCaster>();

                    // Return to resting state
                    casting_state.cancel();
                    completed = true;
                } else {
                    // Out of mana - cancel cast
                    cleanup_spell_caster(commands, wizard_entity, caster_query);
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            // Meteor Fall doesn't use channeling, cancel if we somehow get here
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}

/// Spawns meteor projectiles periodically from active storms.
///
/// Projectiles spawn at random positions within the storm radius, high above the battlefield.
pub(super) fn spawn_meteor_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut storms: Query<(Entity, &mut MeteorFallStorm)>,
    enemies: Query<(&Transform, &Team)>,
) {
    let rng = &mut game_rng.0;

    for (storm_entity, mut storm) in storms.iter_mut() {
        storm.update_timers(time.delta_secs());

        // Check if fixed-duration storm has expired (Extinction Event)
        if let Some(duration) = storm.duration
            && storm.time_alive >= duration
        {
            commands.entity(storm_entity).try_despawn();
            continue;
        }

        // Check if it's time to spawn another meteor
        if storm.time_since_spawn >= storm.spawn_interval {
            storm.reset_spawn_timer();

            // Random position within storm circle (on XZ plane)
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = rng.random_range(0.0..storm.radius);
            let offset = Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance);

            let spawn_pos = Vec3::new(
                storm.position.x + offset.x,
                METEOR_SPAWN_HEIGHT,
                storm.position.z + offset.z,
            );

            // Spawn projectile with talent-modified values
            let damage = METEOR_DAMAGE * storm.empowerment * storm.damage_mult;
            let explosion_radius =
                EXPLOSION_RADIUS * storm.empowerment * storm.explosion_radius_mult;
            let mesh_radius = METEOR_MESH_RADIUS * storm.mesh_radius_mult;

            let entity = spawn_meteor_projectile_entity(
                &mut commands,
                &visual_assets,
                spawn_pos,
                Vec3::new(0.0, METEOR_INITIAL_VELOCITY, 0.0),
                damage,
                explosion_radius,
                storm.empowerment,
                mesh_radius,
                MeteorProjectileTalentFlags {
                    aftershock: storm.aftershock,
                    volcanic_eruption: storm.volcanic_eruption,
                    ground_fire_duration_mult: storm.ground_fire_duration_mult,
                    ground_fire_damage_mult: storm.ground_fire_damage_mult,
                    ground_fire_radius_mult: storm.ground_fire_radius_mult,
                    tracking: storm.tracking,
                    is_extinction: false,
                },
            );

            // For tracking meteors, bias spawn position toward nearest enemy
            if storm.tracking {
                let storm_center_xz = Vec2::new(storm.position.x, storm.position.z);
                if let Some((enemy_xz, _)) = find_nearest_non_defender_xz(
                    enemies
                        .iter()
                        .map(|(t, team)| (Vec2::new(t.translation.x, t.translation.z), *team)),
                    storm_center_xz,
                    Some(storm.radius),
                ) {
                    // Bias 50% toward nearest enemy
                    let biased_x = spawn_pos.x * 0.5 + enemy_xz.x * 0.5;
                    let biased_z = spawn_pos.z * 0.5 + enemy_xz.y * 0.5;
                    commands.entity(entity).insert(
                        Transform::from_translation(Vec3::new(
                            biased_x,
                            METEOR_SPAWN_HEIGHT,
                            biased_z,
                        ))
                        .with_scale(Vec3::splat(mesh_radius)),
                    );
                }
            }
        }
    }
}

/// Talent flags to apply to a meteor projectile at spawn time.
pub(crate) struct MeteorProjectileTalentFlags {
    pub aftershock: bool,
    pub volcanic_eruption: bool,
    pub ground_fire_duration_mult: f32,
    pub ground_fire_damage_mult: f32,
    pub ground_fire_radius_mult: f32,
    pub tracking: bool,
    pub is_extinction: bool,
}

impl Default for MeteorProjectileTalentFlags {
    fn default() -> Self {
        Self {
            aftershock: false,
            volcanic_eruption: false,
            ground_fire_duration_mult: 1.0,
            ground_fire_damage_mult: 1.0,
            ground_fire_radius_mult: 1.0,
            tracking: false,
            is_extinction: false,
        }
    }
}

/// Spawns a raw meteor projectile entity with explicit parameters.
///
/// Used by both storm spawning and crystal absorption/auto-cast.
/// Pass `MeteorProjectileTalentFlags::default()` for non-talented projectiles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_meteor_projectile_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    spawn_pos: Vec3,
    velocity: Vec3,
    damage: f32,
    explosion_radius: f32,
    empowerment: f32,
    mesh_radius: f32,
    talent_flags: MeteorProjectileTalentFlags,
) -> Entity {
    let mut projectile =
        MeteorProjectile::new(velocity, damage, explosion_radius, empowerment, mesh_radius);
    projectile.aftershock = talent_flags.aftershock;
    projectile.volcanic_eruption = talent_flags.volcanic_eruption;
    projectile.ground_fire_duration_mult = talent_flags.ground_fire_duration_mult;
    projectile.ground_fire_damage_mult = talent_flags.ground_fire_damage_mult;
    projectile.ground_fire_radius_mult = talent_flags.ground_fire_radius_mult;
    projectile.tracking = talent_flags.tracking;
    projectile.is_extinction = talent_flags.is_extinction;

    commands
        .spawn((
            projectile,
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.meteor_projectile.clone()),
            Transform::from_translation(spawn_pos).with_scale(Vec3::splat(mesh_radius)),
            OnGameplayScreen,
        ))
        .id()
}

/// Spawns a meteor explosion visual entity.
pub(super) fn spawn_explosion_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    pos: Vec3,
    radius: f32,
    damage: f32,
    networked: bool,
) {
    let mat_handle = clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);

    let mut entity = commands.spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(mat_handle),
        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)).with_scale(Vec3::splat(0.1)),
        MeteorExplosion::new(pos, radius, damage),
        OnGameplayScreen,
    ));
    if networked {
        entity.insert(NetworkedSpellEffect {
            kind: SpellEffectKind::MeteorExplosion,
        });
    }
}
