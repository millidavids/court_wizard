use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{
    BattleHymnModifier, BerserkerRageModifier, Corpse, FogEvasionModifier, HasteModifier, Health,
    MindControlled, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::{SpellShield, SpellShieldVisual};
use crate::game::units::shielder::components::ShielderDamageReduction;
use crate::game::units::wizard::components::{LocalWizard, Mana, PrimedSpell, Spell, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::grease::components::{GreaseIgnited, GreaseZone};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::utils::{get_cursor_world_position, xz_distance};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::vfx::constants as vfx_constants;
use crate::game::units::wizard::spells::vfx::systems::spawn_explosion_smoke;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

// ===== Talent Params =====

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> DispelTalentParams {
    let mut params = DispelTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    // Tier 1
    match talents.get_selection(Spell::Dispel, 0) {
        Some(0) => params.broad_spectrum = true,
        Some(1) => params.cooldown_mult = constants::SWIFT_CANCELLATION_COOLDOWN_MULT,
        Some(2) => params.mana_cost = constants::EFFICIENT_NULLIFICATION_MANA_COST,
        _ => {}
    }

    // Tier 2
    match talents.get_selection(Spell::Dispel, 1) {
        Some(0) => params.mana_drain = true,
        Some(1) => params.explosive_nullification = true,
        Some(2) => {
            params.counterspell_speed_mult = constants::COUNTERSPELL_SPEED_MULT;
            params.counterspell_expand_mult = constants::COUNTERSPELL_EXPAND_MULT;
        }
        _ => {}
    }

    // Tier 3
    match talents.get_selection(Spell::Dispel, 2) {
        Some(0) => params.antimagic_pulse = true,
        Some(1) => params.spell_reflection = true,
        Some(2) => params.null_zone = true,
        _ => {}
    }

    params
}

// ===== Wizard Casting =====

/// Instant-cast dispel on click — fires projectile at cursor position.
/// With Antimagic Pulse talent, skips the projectile and creates a wizard-centered pulse.
#[allow(clippy::too_many_arguments)]
pub fn handle_dispel_casting(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &mut Mana,
            &PrimedSpell,
            &Wizard,
            Option<&DispelCooldown>,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    visual_assets: Res<SpellVisualAssets>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((wizard_entity, mut mana, primed_spell, wizard, cooldown)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Dispel {
        return;
    }

    // Check cooldown
    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let mana_cost = talent_params.mana_cost * wizard.mana_cost_multiplier;
    if !mana.consume(mana_cost) {
        return;
    }

    let origin = SPELL_ORIGIN;
    vfx::systems::spawn_school_flare(
        &mut commands,
        &visual_assets,
        vfx::systems::SpellSchool::Arcane,
        time.elapsed_secs(),
    );
    audio::play_sfx(&mut commands, &sfx.dispel_cast, origin, &game_config, &sfx);

    let cooldown_time = constants::COOLDOWN * talent_params.cooldown_mult;
    commands.entity(wizard_entity).insert(DispelCooldown {
        remaining: cooldown_time,
    });

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some(target_pos) = cursor_pos else {
        return;
    };

    if talent_params.antimagic_pulse {
        // Antimagic Pulse: skip projectile, spawn a large impact sphere at the cursor
        let impact_pos = Vec3::new(target_pos.x, 5.0, target_pos.z);
        let mut impact_entity = commands.spawn((
            Mesh3d(visual_assets.explosion_sphere.clone()),
            MeshMaterial3d(visual_assets.guardian_aura_sphere.clone()),
            Transform::from_translation(impact_pos).with_scale(Vec3::ZERO),
            DispelImpact {
                time_alive: 0.0,
                duration: constants::ANTIMAGIC_PULSE_DURATION,
                expand_speed: constants::ANTIMAGIC_PULSE_RADIUS
                    / constants::ANTIMAGIC_PULSE_DURATION,
            },
            OnGameplayScreen,
        ));
        insert_talent_markers(&mut impact_entity, &talent_params);
    } else {
        // Normal projectile path
        spawn_dispel_projectile_with_talents(
            &mut commands,
            &mut meshes,
            &mut materials,
            origin,
            target_pos,
            constants::SPAWN_HEIGHT_OFFSET,
            &talent_params,
        );
    }
}

/// Inserts talent marker components onto a dispel impact entity.
fn insert_talent_markers(entity_commands: &mut EntityCommands, params: &DispelTalentParams) {
    if params.broad_spectrum {
        entity_commands.insert(BroadSpectrum);
    }
    if params.mana_drain {
        entity_commands.insert(ManaDrain);
    }
    if params.explosive_nullification {
        entity_commands.insert(ExplosiveNullification);
    }
    if params.spell_reflection {
        entity_commands.insert(SpellReflection);
    }
    if params.null_zone {
        entity_commands.insert(NullZoneOnImpact);
    }
}

/// Ticks down the dispel cooldown timer each frame.
pub fn tick_dispel_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut DispelCooldown)>,
) {
    for (entity, mut cooldown) in &mut cooldowns {
        cooldown.remaining -= time.delta_secs();
        if cooldown.remaining <= 0.0 {
            commands.entity(entity).remove::<DispelCooldown>();
        }
    }
}

// ===== Shared Spawn Helper =====

/// Spawns a dispel projectile from `origin` toward `target_pos`.
///
/// `height_offset` controls how high above the origin the projectile spawns.
/// Wizard uses `SPAWN_HEIGHT_OFFSET` (arcs down to ground), dispellers use `0.0`.
/// The projectile travels in 3D and detonates when it hits the battlefield (y<=0).
pub(crate) fn spawn_dispel_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    target_pos: Vec3,
    height_offset: f32,
) {
    let params = DispelTalentParams::default();
    spawn_dispel_projectile_with_talents(
        commands,
        meshes,
        materials,
        origin,
        target_pos,
        height_offset,
        &params,
    );
}

/// Spawns a dispel projectile with talent modifications applied.
fn spawn_dispel_projectile_with_talents(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    origin: Vec3,
    target_pos: Vec3,
    height_offset: f32,
    params: &DispelTalentParams,
) {
    let spawn_pos = origin + Vec3::Y * height_offset;
    // Target is on the ground (y=0)
    let ground_target = Vec3::new(target_pos.x, 0.0, target_pos.z);
    let diff = ground_target - spawn_pos;
    let direction = diff.normalize_or_zero();
    let speed = constants::PROJECTILE_SPEED * params.counterspell_speed_mult;
    let velocity = direction * speed;

    let expand_speed = constants::IMPACT_EXPAND_SPEED * params.counterspell_expand_mult;
    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(Circle::new(constants::PROJECTILE_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: constants::PROJECTILE_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(spawn_pos),
        DispelProjectile {
            velocity,
            lifetime: constants::PROJECTILE_LIFETIME,
            expand_speed,
        },
        Billboard,
        OnGameplayScreen,
    ));

    // Store talent markers on projectile so they transfer to impact
    insert_talent_markers(&mut entity_commands, params);
}

// ===== Projectile + Impact Systems =====

/// Moves dispel projectiles. Detonates on ground impact (y<=0) or lifetime expiry.
/// Transfers talent markers from projectile to impact entity.
#[allow(clippy::type_complexity)]
pub fn move_dispel_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut projectiles: Query<(
        Entity,
        &mut Transform,
        &mut DispelProjectile,
        Has<BroadSpectrum>,
        Has<ManaDrain>,
        Has<ExplosiveNullification>,
        Has<SpellReflection>,
        Has<NullZoneOnImpact>,
    )>,
) {
    let delta = time.delta_secs();
    for (
        entity,
        mut transform,
        mut projectile,
        has_broad_spectrum,
        has_mana_drain,
        has_explosive,
        has_reflection,
        has_null_zone,
    ) in &mut projectiles
    {
        // Move projectile
        transform.translation += projectile.velocity * delta;
        projectile.lifetime -= delta;

        // Detonate when hitting the battlefield (y<=0) or lifetime expired
        let hit_ground = transform.translation.y <= 0.0;
        if hit_ground || projectile.lifetime <= 0.0 {
            // Impact position slightly above ground so cross-plane sphere is visible
            let impact_pos = Vec3::new(transform.translation.x, 5.0, transform.translation.z);

            let mut impact_entity = commands.spawn((
                Mesh3d(visual_assets.explosion_sphere.clone()),
                MeshMaterial3d(visual_assets.guardian_aura_sphere.clone()),
                Transform::from_translation(impact_pos).with_scale(Vec3::ZERO),
                DispelImpact {
                    time_alive: 0.0,
                    duration: constants::IMPACT_DURATION,
                    expand_speed: projectile.expand_speed,
                },
                OnGameplayScreen,
            ));

            // Transfer talent markers from projectile to impact
            let params = DispelTalentParams {
                broad_spectrum: has_broad_spectrum,
                mana_drain: has_mana_drain,
                explosive_nullification: has_explosive,
                spell_reflection: has_reflection,
                null_zone: has_null_zone,
                ..Default::default()
            };
            insert_talent_markers(&mut impact_entity, &params);

            commands.entity(entity).try_despawn();
        }
    }
}

/// Expands impact spheres, checks overlap with spell effects, and despawns expired impacts.
/// Handles talent effects: Broad Spectrum, Mana Drain, Explosive Nullification, Spell Reflection, Null Zone.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_dispel_impacts(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    visual_assets: Res<SpellVisualAssets>,
    mut impacts: Query<(
        Entity,
        &mut DispelImpact,
        &mut Transform,
        Has<BroadSpectrum>,
        Has<ManaDrain>,
        Has<ExplosiveNullification>,
        Has<SpellReflection>,
        Has<NullZoneOnImpact>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect), Without<DispelImpact>>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut wizard_mana: Query<&mut Mana, With<LocalWizard>>,
    progress_and_toggles: (ResMut<BattleTalentProgress>, Option<Res<ActiveToggles>>),
    // Combined query for buff removal, damage, enemy finding, and mind control removal
    mut unit_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Has<HasteModifier>,
            Has<BerserkerRageModifier>,
            Has<BattleHymnModifier>,
            Has<FogEvasionModifier>,
            Has<MindControlled>,
            Has<crate::game::units::components::Petrified>,
        ),
        (Without<Corpse>, Without<DispelImpact>),
    >,
) {
    let (mut progress, active_toggles) = progress_and_toggles;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let time_secs = time.elapsed_secs();
    let mut damage_targets: Vec<(Entity, f32, bool)> = Vec::new();

    for (
        entity,
        mut impact,
        mut transform,
        has_broad_spectrum,
        has_mana_drain,
        has_explosive,
        has_reflection,
        has_null_zone,
    ) in &mut impacts
    {
        impact.time_alive += time.delta_secs();

        if impact.time_alive >= impact.duration {
            // Null Zone: spawn persistent anti-magic zone at impact point before despawning
            if has_null_zone {
                spawn_null_zone(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    transform.translation,
                    scorched_mult,
                );
            }
            commands.entity(entity).try_despawn();
            continue;
        }

        // Expand at constant speed (Counterspell talent makes this faster via expand_speed)
        let radius = impact.expand_speed * impact.time_alive;
        transform.scale = Vec3::splat(radius);

        let impact_center = transform.translation;

        // Collect dispellable spell effects once for this frame
        let all_dispellable: Vec<_> = collect_dispellable_effects(
            spell_effects
                .iter()
                .map(|(e, tf, nse)| (e, tf.translation, nse.kind)),
        );

        // Suppress all dispellable spell effects within radius
        let dispelled = suppress_spell_effects_in_radius(
            &mut commands,
            impact_center,
            radius,
            &all_dispellable,
            &wall_of_fire_query,
            &wall_of_stone_query,
            &spike_growth_query,
            &grease_query,
            &meteor_fire_query,
            &mut obstacle_events,
        );
        let mut dispelled_count = dispelled.len() as u32;

        // Talent effects on each dispelled spell effect
        for &(_spell_entity, effect_pos, effect_kind) in &dispelled {
            // Mana Drain: refund mana
            if has_mana_drain {
                let refund = constants::spell_effect_mana_cost(effect_kind)
                    * constants::MANA_DRAIN_REFUND_FRACTION;
                if refund > 0.0
                    && let Ok(mut mana) = wizard_mana.single_mut()
                {
                    mana.regenerate(refund);
                }
            }

            // Spell Reflection: find nearest enemy target for reflected damage
            let reflection_target = if has_reflection && is_offensive_effect(effect_kind) {
                let mut best: Option<(f32, Vec3)> = None;
                for (_, tf, team, _, _, _, _, _, _, _, _, _) in unit_query.iter() {
                    if !Team::Defenders.is_enemy(team) {
                        continue;
                    }
                    let d = xz_distance(tf.translation, effect_pos);
                    if best.is_none_or(|(bd, _)| d < bd) {
                        best = Some((d, tf.translation));
                    }
                }
                best.map(|(_, p)| p)
            } else {
                None
            };

            damage_targets.clear();

            // Explosive Nullification: damage enemies near the dispelled effect + VFX
            if has_explosive {
                spawn_dispel_explosion(&mut commands, &visual_assets, effect_pos, time_secs);
                for (entity, tf, team, _, _, has_shield, _, _, _, _, _, _) in unit_query.iter() {
                    if !Team::Defenders.is_enemy(team) {
                        continue;
                    }
                    if xz_distance(tf.translation, effect_pos)
                        <= constants::EXPLOSIVE_NULLIFICATION_RADIUS
                    {
                        damage_targets.push((
                            entity,
                            constants::EXPLOSIVE_NULLIFICATION_DAMAGE,
                            has_shield,
                        ));
                    }
                }
            }

            // Spell Reflection: damage enemies near the reflected target
            if let Some(target_pos) = reflection_target {
                for (entity, tf, team, _, _, has_shield, _, _, _, _, _, _) in unit_query.iter() {
                    if !Team::Defenders.is_enemy(team) {
                        continue;
                    }
                    if xz_distance(tf.translation, target_pos) <= constants::SPELL_REFLECTION_RADIUS
                    {
                        damage_targets.push((
                            entity,
                            constants::SPELL_REFLECTION_DAMAGE,
                            has_shield,
                        ));
                    }
                }
            }

            // Apply collected damage
            for &(target_entity, damage, has_shield) in &damage_targets {
                if let Ok((_, _, _, mut health, mut temp_hp, _, _, _, _, _, _, _)) =
                    unit_query.get_mut(target_entity)
                {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Force,
                        has_shield,
                    );
                }
            }
        }

        // Remove mind control from units in range
        let mc_freed = remove_mind_control_in_radius(
            &mut commands,
            impact_center,
            radius,
            unit_query
                .iter()
                .filter_map(|(entity, tf, _, _, _, _, _, _, _, _, has_mc, _)| {
                    has_mc.then_some((entity, tf.translation))
                }),
        );
        dispelled_count += mc_freed;

        // Strip spell shields from enemy units in range (shielder-applied shields)
        let shields_stripped = strip_spell_shields_in_radius(
            &mut commands,
            impact_center,
            radius,
            unit_query.iter().filter_map(
                |(entity, tf, team, _, _, has_shield, _, _, _, _, _, _)| {
                    (has_shield && Team::Defenders.is_enemy(team))
                        .then_some((entity, tf.translation))
                },
            ),
        );
        dispelled_count += shields_stripped;

        // Broad Spectrum: strip buffs from enemies in range
        if has_broad_spectrum {
            for (
                unit_entity,
                unit_tf,
                team,
                _health,
                _temp_hp,
                _has_shield,
                has_haste,
                has_rage,
                has_hymn,
                has_fog,
                _has_mind_control,
                has_petrified,
            ) in &unit_query
            {
                if xz_distance(unit_tf.translation, impact_center) > radius {
                    continue;
                }

                // Dispel cures petrified allies
                if *team == Team::Defenders && has_petrified {
                    commands
                        .entity(unit_entity)
                        .remove::<crate::game::units::components::Petrified>();
                }

                if Team::Defenders.is_enemy(team) {
                    let mut stripped = false;
                    commands.entity(unit_entity).remove::<TemporaryHitPoints>();
                    if has_haste {
                        commands.entity(unit_entity).remove::<HasteModifier>();
                        stripped = true;
                    }
                    if has_rage {
                        commands
                            .entity(unit_entity)
                            .remove::<BerserkerRageModifier>();
                        stripped = true;
                    }
                    if has_hymn {
                        commands.entity(unit_entity).remove::<BattleHymnModifier>();
                        stripped = true;
                    }
                    if has_fog {
                        commands.entity(unit_entity).remove::<FogEvasionModifier>();
                        stripped = true;
                    }
                    if stripped {
                        dispelled_count += 1;
                    }
                }
            }
        }

        // Track talent progress
        if dispelled_count > 0 {
            progress.increment(Spell::Dispel, dispelled_count);
        }
    }
}

/// Spawns a persistent Null Zone at the given position.
fn spawn_null_zone(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    scorched_mult: f32,
) {
    let origin = Vec3::new(position.x, 0.0, position.z);
    let radius = constants::NULL_ZONE_RADIUS;

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius, constants::NULL_ZONE_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: constants::NULL_ZONE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_translation(origin + Vec3::Y * (constants::NULL_ZONE_HEIGHT / 2.0)),
        NullZone {
            time_remaining: constants::NULL_ZONE_DURATION * scorched_mult,
            radius,
            origin,
        },
        OnGameplayScreen,
    ));
}

/// Spawns white sparks and smoke at a dispelled effect's position (Explosive Nullification VFX).
fn spawn_dispel_explosion(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    time_secs: f32,
) {
    use crate::game::units::wizard::spells::vfx::systems::spawn_sparks_with_material;
    spawn_sparks_with_material(
        commands,
        assets,
        position,
        vfx_constants::SPARK_COUNT,
        time_secs,
        assets.dispel_spark.clone(),
    );
    spawn_explosion_smoke(commands, assets, position, time_secs);
}

/// Ticks Null Zone timers. Despawns expired zones.
/// Active null zones suppress (despawn) spell effects that enter them.
#[allow(clippy::too_many_arguments)]
pub fn update_null_zones(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(Entity, &mut NullZone, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect), Without<NullZone>>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mind_controlled_query: Query<(Entity, &Transform), (With<MindControlled>, Without<NullZone>)>,
) {
    let delta = time.delta_secs();
    for (zone_entity, mut zone, material_handle) in &mut zones {
        zone.time_remaining -= delta;
        if zone.time_remaining <= 0.0 {
            commands.entity(zone_entity).try_despawn();
            continue;
        }

        // Fade alpha as zone expires
        let life_frac = zone.time_remaining / constants::NULL_ZONE_DURATION;
        let alpha = constants::NULL_ZONE_COLOR.alpha() * life_frac;
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = constants::NULL_ZONE_COLOR.with_alpha(alpha);
        }

        // Collect dispellable spell effects once for this frame
        let all_dispellable: Vec<_> = collect_dispellable_effects(
            spell_effects
                .iter()
                .map(|(e, tf, nse)| (e, tf.translation, nse.kind)),
        );

        // Suppress spell effects inside the zone
        suppress_spell_effects_in_radius(
            &mut commands,
            zone.origin,
            zone.radius,
            &all_dispellable,
            &wall_of_fire_query,
            &wall_of_stone_query,
            &spike_growth_query,
            &grease_query,
            &meteor_fire_query,
            &mut obstacle_events,
        );

        // Remove mind control from units in zone
        remove_mind_control_in_radius(
            &mut commands,
            zone.origin,
            zone.radius,
            mind_controlled_query
                .iter()
                .map(|(e, tf)| (e, tf.translation)),
        );
    }
}

/// Returns true if this spell effect kind is an offensive (damage-dealing) effect
/// for Spell Reflection purposes.
fn is_offensive_effect(kind: SpellEffectKind) -> bool {
    matches!(
        kind,
        SpellEffectKind::SpikeGrowthZone
            | SpellEffectKind::WallOfFire
            | SpellEffectKind::MeteorGroundFire
            | SpellEffectKind::PlagueWindCloud
            | SpellEffectKind::GreaseFire
            | SpellEffectKind::BlackHole
    )
}

// ===== Shared Suppress/Dispel Helpers =====

/// Suppresses (despawns) all dispellable spell effects within `radius` of `center`.
/// Returns the list of (entity, position, kind) for each dispelled effect so callers
/// can apply additional talent logic (e.g. Mana Drain, Explosive Nullification).
#[allow(clippy::too_many_arguments)]
fn suppress_spell_effects_in_radius(
    commands: &mut Commands,
    center: Vec3,
    radius: f32,
    spell_effects: &[(Entity, Vec3, SpellEffectKind)],
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    wall_of_stone_query: &Query<&WallOfStone>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) -> Vec<(Entity, Vec3, SpellEffectKind)> {
    let mut dispelled = Vec::new();

    for &(spell_entity, spell_pos, kind) in spell_effects {
        let edge_dist = spell_edge_distance(
            center,
            spell_entity,
            spell_pos,
            wall_of_fire_query,
            wall_of_stone_query,
            spike_growth_query,
            grease_query,
            meteor_fire_query,
        );

        if edge_dist <= radius {
            dispelled.push((spell_entity, spell_pos, kind));

            despawn_spell_effect(
                commands,
                spell_entity,
                wall_of_stone_query,
                wall_of_fire_query,
                spike_growth_query,
                grease_query,
                meteor_fire_query,
                obstacle_events,
            );
        }
    }

    dispelled
}

/// Collects dispellable spell effects from a query into a Vec for use with `suppress_spell_effects_in_radius`.
fn collect_dispellable_effects(
    spell_effects: impl Iterator<Item = (Entity, Vec3, SpellEffectKind)>,
) -> Vec<(Entity, Vec3, SpellEffectKind)> {
    spell_effects
        .filter(|&(_, _, kind)| is_dispellable(kind))
        .collect()
}

/// Removes `MindControlled` from all units within `radius` of `center`.
/// Returns the number of mind-controlled units that were freed.
fn remove_mind_control_in_radius(
    commands: &mut Commands,
    center: Vec3,
    radius: f32,
    mind_controlled_iter: impl Iterator<Item = (Entity, Vec3)>,
) -> u32 {
    let mut count = 0;
    for (entity, position) in mind_controlled_iter {
        if xz_distance(position, center) <= radius {
            commands.entity(entity).remove::<MindControlled>();
            count += 1;
        }
    }
    count
}

// ===== Shared Helpers (moved from dispeller) =====

/// Removes `SpellShield` and `ShielderDamageReduction` from all units
/// within `radius` of `center`. The orphaned `SpellShieldVisual` children
/// are cleaned up by `cleanup_orphaned_shield_visuals`.
/// Returns the number of shields stripped.
fn strip_spell_shields_in_radius(
    commands: &mut Commands,
    center: Vec3,
    radius: f32,
    shielded_units: impl Iterator<Item = (Entity, Vec3)>,
) -> u32 {
    let mut count = 0;
    for (entity, position) in shielded_units {
        if xz_distance(position, center) <= radius {
            commands.entity(entity).remove::<SpellShield>();
            commands.entity(entity).remove::<ShielderDamageReduction>();
            count += 1;
        }
    }
    count
}

/// Despawns `SpellShieldVisual` entities whose parent no longer has `SpellShield`.
/// This cleans up shield visuals after shields are dispelled.
pub fn cleanup_orphaned_shield_visuals(
    mut commands: Commands,
    shield_visuals: Query<(Entity, &ChildOf), With<SpellShieldVisual>>,
    shielded_units: Query<(), With<SpellShield>>,
) {
    for (visual_entity, child_of) in &shield_visuals {
        if shielded_units.get(child_of.parent()).is_err() {
            commands.entity(visual_entity).try_despawn();
        }
    }
}

/// Returns true if the spell effect kind is dispellable.
pub(crate) fn is_dispellable(kind: SpellEffectKind) -> bool {
    !matches!(
        kind,
        SpellEffectKind::FireballExplosion
            | SpellEffectKind::MeteorExplosion
            | SpellEffectKind::IceExplosion
            | SpellEffectKind::HealingPlumeZone
    )
}

/// Computes the XZ distance from a point to the nearest edge of a spell effect's volume.
///
/// For volumetric effects (wall of fire, wall of stone, circular zones), returns the
/// distance to the closest edge of the area rather than the center. Returns 0 if
/// the point is inside the volume. Falls back to center-point distance for unknown types.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spell_edge_distance(
    point: Vec3,
    spell_entity: Entity,
    spell_center: Vec3,
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    wall_of_stone_query: &Query<&WallOfStone>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
) -> f32 {
    // Wall of Fire: line segment with half_width
    if let Ok(wall) = wall_of_fire_query.get(spell_entity) {
        let dist_to_line = wall.distance_to_point(point);
        return (dist_to_line - wall.half_width).max(0.0);
    }

    // Wall of Stone: oriented bounding box
    if let Ok(wall) = wall_of_stone_query.get(spell_entity) {
        if wall.contains_point_xz(point) {
            return 0.0;
        }
        let diff = Vec3::new(point.x - wall.center.x, 0.0, point.z - wall.center.z);
        let forward_proj = diff
            .dot(wall.forward)
            .clamp(-wall.half_length, wall.half_length);
        let right_proj = diff
            .dot(wall.right)
            .clamp(-wall.half_width, wall.half_width);
        let closest = wall.center + wall.forward * forward_proj + wall.right * right_proj;
        return xz_distance(point, closest);
    }

    // Spike Growth: circular zone
    if let Ok(zone) = spike_growth_query.get(spell_entity) {
        return (xz_distance(point, zone.origin) - zone.effective_radius()).max(0.0);
    }

    // Grease: circular zone
    if let Ok((zone, _)) = grease_query.get(spell_entity) {
        return (xz_distance(point, zone.origin) - zone.radius).max(0.0);
    }

    // Meteor Ground Fire: circular zone
    if let Ok(fire) = meteor_fire_query.get(spell_entity) {
        return (xz_distance(point, fire.origin) - fire.radius).max(0.0);
    }

    // Fallback: center-point distance
    xz_distance(point, spell_center)
}

/// Despawns a spell effect entity and cleans up its pathfinding obstacle if applicable.
///
/// Wall of Stone is special: instead of instant despawn, it enters the sinking animation
/// so it visually sinks into the ground with dust VFX before being cleaned up.
#[allow(clippy::too_many_arguments)]
pub(crate) fn despawn_spell_effect(
    commands: &mut Commands,
    spell_entity: Entity,
    wall_of_stone_query: &Query<&WallOfStone>,
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    // Wall of Stone -- trigger sink animation instead of instant despawn.
    // The obstacle is removed immediately so units can path through,
    // but the wall entity sinks visually over WALL_SINK_DURATION before cleanup.
    if let Ok(wall) = wall_of_stone_query.get(spell_entity) {
        let obs_bounds = wall.obstacle_bounds();
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(obs_bounds[0], obs_bounds[1], obs_bounds[2], obs_bounds[3]),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::obb_from_center(
                wall.center,
                wall.forward,
                wall.half_length,
                wall.half_width,
            )),
            rebuild: false,
        });

        // Trigger sinking animation — the existing tick/animate/cleanup pipeline
        // will handle the visual sink and eventual despawn.
        let sink_duration =
            crate::game::units::wizard::spells::wall_of_stone::constants::WALL_SINK_DURATION;
        commands.entity(spell_entity).insert(
            crate::game::units::wizard::spells::wall_of_stone::components::DispelledWall {
                sink_duration,
            },
        );
        // Remove the NetworkedSpellEffect so the dispel impact doesn't re-target this wall
        commands
            .entity(spell_entity)
            .remove::<NetworkedSpellEffect>();
        return;
    }

    // Wall of Fire -- hazard obstacle
    if let Ok(effect) = wall_of_fire_query.get(spell_entity) {
        let a = Vec2::new(effect.start.x, effect.start.z);
        let b = Vec2::new(effect.end.x, effect.end.z);
        let dir = b - a;
        let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero() * effect.half_width;
        let c0 = a + perp;
        let c1 = a - perp;
        let c2 = b + perp;
        let c3 = b - perp;
        let min_x = c0.x.min(c1.x).min(c2.x).min(c3.x) - OBSTACLE_BUFFER;
        let max_x = c0.x.max(c1.x).max(c2.x).max(c3.x) + OBSTACLE_BUFFER;
        let min_y = c0.y.min(c1.y).min(c2.y).min(c3.y) - OBSTACLE_BUFFER;
        let max_y = c0.y.max(c1.y).max(c2.y).max(c3.y) + OBSTACLE_BUFFER;

        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(min_x, min_y, max_x, max_y),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::obb_from_wall(
                effect.start,
                effect.end,
                effect.half_width + OBSTACLE_BUFFER,
            )),
            rebuild: false,
        });
    }

    // Spike Growth -- hazard obstacle (circular zone)
    if let Ok(zone) = spike_growth_query.get(spell_entity) {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.effective_radius() + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
            rebuild: false,
        });
    }

    // Grease -- hazard obstacle when ignited
    if let Ok((zone, is_ignited)) = grease_query.get(spell_entity)
        && is_ignited
    {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
            rebuild: false,
        });
    }

    // Meteor Ground Fire -- hazard obstacle
    if let Ok(fire) = meteor_fire_query.get(spell_entity) {
        let origin_2d = Vec2::new(fire.origin.x, fire.origin.z);
        let buffered = fire.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
            shape: Some(ObstacleShape::circle(origin_2d, buffered)),
            rebuild: false,
        });
    }

    commands.entity(spell_entity).try_despawn();
}
