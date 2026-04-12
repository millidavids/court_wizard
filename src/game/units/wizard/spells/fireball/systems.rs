use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, MarkedForDeathModifier, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::arcane_crystal::components::CrystalSpawn;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, handle_spell_release, try_start_cast_with_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, SpellVisualAssets, clone_sphere_material, explosion_fade_opacity,
};
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Local wizard fireball casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_fireball_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    target_assist: Res<TargetAssistWorldPos>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Fireball {
        return;
    }

    let completed = fireball_casting_logic(
        &input,
        &time,
        wizard_entity,
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
        &active_talents,
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

/// Core fireball casting logic. Returns true if the spell completed.
#[allow(clippy::too_many_arguments)]
fn fireball_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
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
    active_talents: &Option<Res<ActiveTalents>>,
) -> bool {
    let mut completed = false;

    // Check for release event
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    // Determine talent-modified cast time
    let talents = active_talents.as_deref();
    let t2 = talents.and_then(|t| t.get_selection(Spell::Fireball, 1));
    let cast_time = match t2 {
        Some(2) => 2.0, // Quick Ignition (tier 2, choice 2)
        _ => primed_spell.cast_time,
    };

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            // Update indicator position to follow cursor
            if let Some(cursor_pos) = input.cursor_pos {
                update_indicator_position(wizard_entity, cursor_pos, caster_query, indicator_query);
            }

            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(cast_time) {
                if mana.consume(constants::MANA_COST)
                    && let Some(target_pos) = input.cursor_pos
                {
                    let spawn_origin =
                        SPELL_ORIGIN + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);
                    spawn_fireball_with_talents(
                        commands,
                        assets,
                        spawn_origin,
                        target_pos,
                        primed_spell,
                        active_talents,
                    );
                    audio::play_sfx(commands, &sfx.fireball_cast, spawn_origin, game_config, sfx);
                    completed = true;
                }
                cleanup_spell_caster(commands, wizard_entity, caster_query);
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                let indicator_pos = input.cursor_pos.unwrap_or(SPELL_ORIGIN);
                let indicator_radius = constants::EXPLOSION_RADIUS * primed_spell.empowerment;
                try_start_cast_with_indicator(
                    commands,
                    meshes,
                    assets.fireball_indicator.clone(),
                    wizard_entity,
                    casting_state,
                    mana,
                    constants::MANA_COST,
                    indicator_pos,
                    indicator_radius,
                    caster_query,
                );
            }
        }
    }

    completed
}

/// Spawns a fireball with talent modifications applied.
fn spawn_fireball_with_talents(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target: Vec3,
    primed_spell: &PrimedSpell,
    active_talents: &Option<Res<ActiveTalents>>,
) {
    let talents = active_talents.as_deref();
    let t1 = talents.and_then(|t| t.get_selection(Spell::Fireball, 0));
    let t2 = talents.and_then(|t| t.get_selection(Spell::Fireball, 1));
    let t3 = talents.and_then(|t| t.get_selection(Spell::Fireball, 2));

    // Tier 1 modifications
    let radius_mult = match t1 {
        Some(0) => 1.5, // Wider Blast
        Some(2) => 0.5, // Focused Blast: half radius
        _ => 1.0,
    };
    let duration_mult = match t1 {
        Some(1) => 1.8, // Lingering Flames: +80% duration
        _ => 1.0,
    };
    let damage_mult = match t1 {
        Some(2) => 2.0, // Focused Blast: double damage
        _ => 1.0,
    };

    // Tier 2 modifications
    let cluster_bomb = t2 == Some(0);
    let napalm = t2 == Some(1);

    // Tier 3 modifications
    let scorched_earth = t3 == Some(1);
    let chain_ignition = t3 == Some(2);
    let is_meteor = t3 == Some(0);
    let radius_mult = if is_meteor {
        radius_mult * 1.3
    } else {
        radius_mult
    };

    // Calculate spawn position and velocity
    let (spawn_origin, velocity) = if is_meteor {
        // Meteor: spawn high above target, drop straight down
        let meteor_origin = Vec3::new(target.x, 800.0, target.z);
        let speed = primed_spell.scale(constants::PROJECTILE_SPEED) * 1.5;
        (meteor_origin, Vec3::new(0.0, -speed, 0.0))
    } else {
        let direction = (target - origin).normalize();
        let speed = primed_spell.scale(constants::PROJECTILE_SPEED);
        (origin, direction * speed)
    };

    let explosion_duration = constants::EXPLOSION_DURATION * duration_mult;
    let damage = primed_spell.scale(constants::DAMAGE_PER_TICK) * damage_mult / duration_mult;
    let explosion_radius = primed_spell.scale(constants::EXPLOSION_RADIUS) * radius_mult;
    let collision_radius = primed_spell.scale(constants::PROJECTILE_COLLISION_RADIUS);
    let visual_radius = primed_spell.scale(constants::FIREBALL_RADIUS);

    // Build fireball with talent flags
    let mut fireball = Fireball::new(
        velocity,
        damage,
        constants::DAMAGE_TYPE,
        explosion_radius,
        collision_radius,
        primed_spell.empowerment,
    );
    fireball.cluster_bomb = cluster_bomb;
    fireball.napalm = napalm;
    fireball.scorched_earth = scorched_earth;
    fireball.chain_ignition = chain_ignition;
    fireball.explosion_duration = explosion_duration;

    let entity = commands
        .spawn((
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.fireball_projectile.clone()),
            Transform::from_translation(spawn_origin).with_scale(Vec3::splat(visual_radius)),
            fireball,
            OnGameplayScreen,
        ))
        .id();

    vfx::systems::spawn_fire_glow(commands, assets, entity, spawn_origin, visual_radius);
}

/// Spawns a raw fireball entity with explicit parameters.
///
/// Used by both wizard casting (via `spawn_fireball`) and crystal absorption.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_fireball_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    velocity: Vec3,
    damage: f32,
    damage_type: DamageType,
    explosion_radius: f32,
    collision_radius: f32,
    empowerment: f32,
    visual_radius: f32,
) -> Entity {
    let entity = commands
        .spawn((
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.fireball_projectile.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(visual_radius)),
            Fireball::new(
                velocity,
                damage,
                damage_type,
                explosion_radius,
                collision_radius,
                empowerment,
            ),
            OnGameplayScreen,
        ))
        .id();

    // Spawn glow halo sibling
    vfx::systems::spawn_fire_glow(commands, assets, entity, origin, visual_radius);

    entity
}

/// Updates fireball projectile positions based on velocity.
pub fn move_fireballs(time: Res<Time>, mut fireballs: Query<(&mut Transform, &Fireball)>) {
    for (mut transform, fireball) in &mut fireballs {
        transform.translation += fireball.velocity * time.delta_secs();
    }
}

/// Spawns smoke trail wisps behind flying fireballs.
pub fn spawn_fireball_smoke_trail(
    mut commands: Commands,
    fireballs: Query<&Transform, With<Fireball>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < vfx::constants::SMOKE_SPAWN_INTERVAL {
        return;
    }
    *timer -= vfx::constants::SMOKE_SPAWN_INTERVAL;

    let t = time.elapsed_secs();

    for transform in fireballs.iter() {
        vfx::systems::spawn_fire_smoke_wisps(
            &mut commands,
            &visual_assets,
            transform.translation,
            vfx::constants::SMOKE_COUNT_PER_SPAWN,
            t,
            vfx::constants::SMOKE_LIFETIME,
            vfx::constants::SMOKE_SIZE,
            vfx::constants::SMOKE_RISE_SPEED,
            vfx::constants::SMOKE_SPREAD_SPEED,
        );

        vfx::systems::spawn_heat_shimmer(
            &mut commands,
            &visual_assets,
            transform.translation,
            1,
            t,
        );
    }
}

/// Checks for fireball collisions with units or the ground.
///
/// When a fireball hits a unit or the ground, it explodes.
/// Talent effects: Cluster Bomb spawns mini-fireballs, Scorched Earth leaves burning ground.
#[allow(clippy::too_many_arguments)]
pub fn check_fireball_collisions(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    time: Res<Time>,
    fireballs: Query<(Entity, &Transform, &Fireball, Option<&CrystalSpawn>)>,
    targets: Query<(&Transform, &Team, &crate::game::units::components::Hitbox)>,
    walls: Query<&WallOfStone>,
    rocks: Query<&crate::game::terrain::boulder::components::Boulder>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let t = time.elapsed_secs();

    for (fireball_entity, fireball_transform, fireball, crystal_spawn) in &fireballs {
        let fireball_pos = fireball_transform.translation;

        let explode_at =
            |commands: &mut Commands,
             mats: &mut Assets<FireExplosionSphereMaterial>,
             pos: Vec3| {
                spawn_explosion_with_talents(
                    commands,
                    &visual_assets,
                    mats,
                    pos,
                    fireball,
                    t,
                    &sfx,
                    &game_config,
                    crystal_spawn,
                );
            };

        // Check collision with walls
        let mut hit_wall = false;
        for wall in &walls {
            if wall.contains_point_xz(fireball_pos) && fireball_pos.y <= wall.height {
                explode_at(&mut commands, &mut sphere_materials, fireball_pos);
                commands.entity(fireball_entity).try_despawn();
                hit_wall = true;
                break;
            }
        }
        if hit_wall {
            continue;
        }

        // Check collision with rocks
        let mut hit_rock = false;
        for rock in &rocks {
            if rock.blocks_projectile(fireball_pos) {
                explode_at(&mut commands, &mut sphere_materials, fireball_pos);
                commands.entity(fireball_entity).try_despawn();
                hit_rock = true;
                break;
            }
        }
        if hit_rock {
            continue;
        }

        // Check collision with ground (Y <= 0)
        if fireball_pos.y <= 0.0 {
            let explosion_pos = Vec3::new(fireball_pos.x, 5.0, fireball_pos.z);
            explode_at(&mut commands, &mut sphere_materials, explosion_pos);
            commands.entity(fireball_entity).try_despawn();
            continue;
        }

        // Check collision with units (cylinder hitbox from Y=0)
        for (target_transform, _team, hitbox) in &targets {
            let hit = crate::game::units::wizard::spells::utils::sphere_intersects_cylinder(
                fireball_pos,
                fireball.radius,
                Vec3::new(target_transform.translation.x, 0.0, target_transform.translation.z),
                hitbox.radius,
                hitbox.height,
            );

            if hit {
                explode_at(&mut commands, &mut sphere_materials, fireball_pos);
                commands.entity(fireball_entity).try_despawn();
                break;
            }
        }
    }
}

/// Spawns a fireball explosion with talent effects at the given position.
#[allow(clippy::too_many_arguments)]
fn spawn_explosion_with_talents(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    sphere_materials: &mut Assets<FireExplosionSphereMaterial>,
    position: Vec3,
    fireball: &Fireball,
    _time_secs: f32,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    crystal_spawn: Option<&CrystalSpawn>,
) {
    let explosion_duration = if fireball.explosion_duration > 0.0 {
        fireball.explosion_duration
    } else {
        constants::EXPLOSION_DURATION
    };

    let mut explosion = FireballExplosion::new(
        position,
        fireball.explosion_radius,
        fireball.damage,
        constants::DAMAGE_TYPE,
        fireball.empowerment,
    );
    explosion.duration = explosion_duration;
    explosion.chain_ignition = fireball.chain_ignition;

    // Per-entity material clone so each explosion can fade independently
    let mat_handle =
        clone_sphere_material(sphere_materials, &assets.fireball_explosion_sphere);

    // Pre-generate all sub-explosion bubbles with distance-based sizes
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let max_r = fireball.explosion_radius;
    let pending: Vec<PendingBubble> = (0..constants::EXPLOSION_BUBBLE_COUNT)
        .map(|_| {
            let direction = Vec3::new(
                rng.gen_range(-1.0..1.0_f32),
                rng.gen_range(0.0..1.0_f32),
                rng.gen_range(-1.0..1.0_f32),
            )
            .normalize_or(Vec3::Y);
            let offset_frac = rng.gen_range(
                constants::BUBBLE_OFFSET_FRACTION_MIN..constants::BUBBLE_OFFSET_FRACTION_MAX,
            );
            let distance = max_r * offset_frac;
            // Size so bubble never reaches more than 10% past main explosion edge
            let radius = max_r * (constants::BUBBLE_OVERSHOOT - offset_frac);
            PendingBubble {
                direction,
                distance,
                radius,
            }
        })
        .collect();

    let entity = commands
        .spawn((
            Mesh3d(assets.explosion_sphere.clone()),
            MeshMaterial3d(mat_handle),
            Transform::from_translation(position).with_scale(Vec3::splat(0.1)),
            ExplosionBubbleSpawner { pending },
            explosion,
            NetworkedSpellEffect {
                kind: SpellEffectKind::FireballExplosion,
            },
            OnGameplayScreen,
        ))
        .id();

    if let Some(cs) = crystal_spawn {
        commands.entity(entity).insert(CrystalSpawn {
            origin: cs.origin,
            max_range: cs.max_range,
            lifetime: None,
        });
    }

    // Sparks, smoke, and heat shimmer are spawned by update_explosions on first frame

    // Impact sound effect
    audio::play_impact_sfx(commands, &sfx.fireball_impact, position, game_config, sfx);

    // Cluster Bomb: spawn 3 mini-fireballs in random directions
    if fireball.cluster_bomb {
        spawn_cluster_bombs(commands, assets, position, fireball);
    }

    // Scorched Earth: spawn persistent burning ground circle
    if fireball.scorched_earth {
        let scorched_pos = Vec3::new(position.x, 1.5, position.z);
        let mut scorched = FireballExplosion::new(
            scorched_pos,
            fireball.explosion_radius * 0.8,
            fireball.damage * 0.3,
            constants::DAMAGE_TYPE,
            fireball.empowerment,
        );
        scorched.duration = 5.0;
        scorched.skip_growth = true;
        scorched.chain_ignition = fireball.chain_ignition;

        commands.spawn((
            Transform::from_translation(scorched_pos),
            Visibility::default(),
            scorched,
            ScorchedEarthFire,
            OnGameplayScreen,
        ));
    }
}

/// Spawns 3 mini-fireballs for the Cluster Bomb talent.
fn spawn_cluster_bombs(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    parent_fireball: &Fireball,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..3 {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(100.0..300.0);
        let flight_time = 0.4;
        let horizontal_speed = distance / flight_time;
        let vy = -(origin.y.max(5.0)) / flight_time;
        let velocity = Vec3::new(
            angle.cos() * horizontal_speed,
            vy,
            angle.sin() * horizontal_speed,
        );

        let mut mini = Fireball::new(
            velocity,
            parent_fireball.damage * 0.5,
            constants::DAMAGE_TYPE,
            parent_fireball.explosion_radius * 0.5,
            parent_fireball.radius * 0.5,
            parent_fireball.empowerment,
        );
        mini.scorched_earth = parent_fireball.scorched_earth;
        mini.chain_ignition = parent_fireball.chain_ignition;

        let visual_radius = 15.0 * parent_fireball.empowerment;

        let entity = commands
            .spawn((
                Mesh3d(assets.cross_plane_sphere.clone()),
                MeshMaterial3d(assets.fireball_projectile.clone()),
                Transform::from_translation(origin).with_scale(Vec3::splat(visual_radius)),
                mini,
                OnGameplayScreen,
            ))
            .id();

        vfx::systems::spawn_fire_glow(commands, assets, entity, origin, visual_radius);
    }
}

/// Napalm talent: fireballs with napalm leave small burning zones behind them.
pub fn update_napalm_trails(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut fireballs: Query<(&Transform, &mut Fireball)>,
) {
    for (transform, mut fireball) in &mut fireballs {
        if !fireball.napalm {
            continue;
        }
        fireball.napalm_timer += time.delta_secs();
        if fireball.napalm_timer >= 0.1 {
            fireball.napalm_timer = 0.0;

            let pos = Vec3::new(transform.translation.x, 3.0, transform.translation.z);
            let mut trail_explosion = FireballExplosion::new(
                pos,
                30.0 * fireball.empowerment,
                fireball.damage * 0.2,
                constants::DAMAGE_TYPE,
                fireball.empowerment,
            );
            trail_explosion.duration = 1.0;

            let mat = sphere_materials
                .get(&visual_assets.fireball_explosion_sphere)
                .expect("sphere material template")
                .clone();
            let mat_handle = sphere_materials.add(mat);

            commands.spawn((
                Mesh3d(visual_assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(pos)
                    .with_scale(Vec3::splat(30.0 * fireball.empowerment)),
                trail_explosion,
                OnGameplayScreen,
            ));
        }
    }
}

/// Updates explosion visuals and timing. Spawns VFX burst on first frame and
/// continuous sparks + smoke throughout the explosion's lifetime.
pub fn update_explosions(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut explosions: Query<(&mut FireballExplosion, &mut Transform)>,
) {
    use rand::Rng;
    let time_secs = time.elapsed_secs();
    let mut rng = rand::thread_rng();

    for (mut explosion, mut transform) in &mut explosions {
        explosion.time_alive += time.delta_secs();
        explosion.time_since_last_tick += time.delta_secs();

        let current_radius = explosion.current_radius();
        transform.scale = Vec3::splat(current_radius);

        // Spawn sparks + smoke + shimmer on first frame (skip for persistent ground effects)
        if !explosion.vfx_spawned && !explosion.skip_growth {
            explosion.vfx_spawned = true;
            let pos = explosion.origin;
            vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                pos,
                vfx::constants::SPARK_COUNT,
                time_secs,
            );
            vfx::systems::spawn_explosion_smoke(&mut commands, &visual_assets, pos, time_secs);
            vfx::systems::spawn_heat_shimmer(
                &mut commands,
                &visual_assets,
                pos,
                vfx::constants::EXPLOSION_SHIMMER_COUNT,
                time_secs,
            );
            vfx::systems::spawn_explosion_dark_smoke(&mut commands, &visual_assets, pos, time_secs);
        }

        // Continuous sparks and smoke from random positions on the explosion surface
        if explosion.vfx_spawned
            && !explosion.skip_growth
            && current_radius > 5.0
            && explosion.time_since_last_tick >= constants::DAMAGE_TICK_INTERVAL
        {
            let dir = Vec3::new(
                rng.gen_range(-1.0..1.0_f32),
                rng.gen_range(0.2..1.0_f32),
                rng.gen_range(-1.0..1.0_f32),
            )
            .normalize_or(Vec3::Y);
            let surface_pos = explosion.origin + dir * current_radius;

            vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                surface_pos,
                4,
                time_secs,
            );
            vfx::systems::spawn_explosion_smoke_with_material(
                &mut commands,
                &visual_assets,
                surface_pos,
                time_secs,
                visual_assets.fire_smoke.clone(),
                5,
            );
        }
    }
}

/// Applies damage to units hit by the explosion on a tick interval.
pub fn apply_explosion_damage(
    mut commands: Commands,
    mut explosions: Query<&mut FireballExplosion>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        Option<&MarkedForDeathModifier>,
    )>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
) {
    for mut explosion in &mut explosions {
        if explosion.time_since_last_tick >= constants::DAMAGE_TICK_INTERVAL {
            explosion.time_since_last_tick = 0.0;

            // Skip damage iteration for VFX-only explosions (e.g. undead detonation)
            if explosion.damage_per_tick <= 0.0 {
                continue;
            }

            let current_radius = explosion.current_radius();
            let mut hit_count = 0u32;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield, existing_mark) in
                &mut targets
            {
                let distance = crate::game::units::wizard::spells::utils::xz_distance(
                    explosion.origin,
                    transform.translation,
                );

                if distance <= current_radius {
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        explosion.damage_per_tick,
                        explosion.damage_type,
                        has_spell_shield,
                    );
                    hit_count += 1;

                    // Chain Ignition: apply damage amplification debuff
                    if explosion.chain_ignition && existing_mark.is_none() {
                        commands
                            .entity(entity)
                            .insert(MarkedForDeathModifier::new(0.5, 3.0));
                    }
                }
            }

            if hit_count > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(explosion.source_spell, hit_count);
            }
        }
    }
}

/// Cleans up explosions that have finished animating.
pub fn cleanup_finished_explosions(
    mut commands: Commands,
    explosions: Query<(Entity, &FireballExplosion)>,
) {
    for (entity, explosion) in &explosions {
        if explosion.time_alive >= explosion.duration {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns wall-of-fire-style orange and black smoke puffs over Scorched Earth fire zones.
pub fn spawn_scorched_earth_fire_smoke(
    mut commands: Commands,
    zones: Query<&FireballExplosion, With<ScorchedEarthFire>>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();
    if *timer < 0.25 {
        return;
    }
    *timer -= 0.25;

    let t = time.elapsed_secs();

    for explosion in zones.iter() {
        // Don't emit smoke in the last 0.5s (fade-out)
        let remaining = explosion.duration - explosion.time_alive;
        if remaining < 0.5 {
            continue;
        }

        vfx::systems::spawn_fire_orange_smoke(
            &mut commands,
            &visual_assets,
            Vec3::new(explosion.origin.x, 0.0, explosion.origin.z),
            explosion.max_radius,
            9,
            t,
        );
    }
}

/// Spawns sub-explosion spheres when the main explosion's radius reaches their position.
///
/// Each pending bubble has a pre-computed trigger distance. When the main explosion
/// grows past that distance, the bubble spawns — giving an amorphous, erupting look.
pub(super) fn spawn_explosion_bubbles(
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut spawners: Query<(
        &FireballExplosion,
        &mut ExplosionBubbleSpawner,
        &Transform,
    )>,
) {
    for (explosion, mut spawner, transform) in &mut spawners {
        let current_radius = explosion.current_radius();

        for i in (0..spawner.pending.len()).rev() {
            if current_radius < spawner.pending[i].distance {
                continue;
            }

            let bubble = spawner.pending.swap_remove(i);
            let pos = transform.translation + bubble.direction * bubble.distance;

            // Per-entity material clone for independent fade
            let mat = sphere_materials
                .get(&visual_assets.fireball_explosion_sphere)
                .expect("sphere material template")
                .clone();
            let mat_handle = sphere_materials.add(mat);

            // Duration = remaining time so it ends with the main explosion
            let remaining = (explosion.duration - explosion.time_alive).max(0.1);

            let mut sub = FireballExplosion::new(
                pos,
                bubble.radius,
                0.0, // visual only, no damage
                constants::DAMAGE_TYPE,
                explosion.empowerment,
            );
            sub.duration = remaining;
            sub.vfx_spawned = true; // skip sparks/smoke

            commands.spawn((
                Mesh3d(visual_assets.explosion_sphere.clone()),
                MeshMaterial3d(mat_handle),
                Transform::from_translation(pos).with_scale(Vec3::splat(0.1)),
                sub,
                OnGameplayScreen,
            ));
        }
    }
}

/// Fades out explosion spheres that use FireExplosionSphereMaterial over their last portion.
pub(super) fn fade_explosion_spheres(
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    explosions: Query<(
        &FireballExplosion,
        &MeshMaterial3d<FireExplosionSphereMaterial>,
    )>,
) {
    for (explosion, material_handle) in &explosions {
        if explosion.duration <= 0.0 {
            continue;
        }
        let opacity = explosion_fade_opacity(explosion.time_alive / explosion.duration);
        if let Some(mat) = sphere_materials.get_mut(material_handle) {
            mat.opacity = opacity;
        }
    }
}

/// Despawns fireballs that travel beyond the wizard's spell range.
pub fn despawn_distant_fireballs(
    mut commands: Commands,
    fireballs: Query<(Entity, &Transform), With<Fireball>>,
    wizard_query: Query<&Wizard, (With<LocalWizard>, Without<Fireball>)>,
) {
    let Ok(wizard) = wizard_query.single() else {
        return;
    };

    let spell_range = wizard.spell_range;

    for (entity, transform) in &fireballs {
        let distance_from_wizard = transform.translation.distance(SPELL_ORIGIN);

        if distance_from_wizard > spell_range {
            commands.entity(entity).try_despawn();
        }
    }
}
