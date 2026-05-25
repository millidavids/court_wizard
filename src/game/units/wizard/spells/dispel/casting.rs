//! Dispel casting and projectile spawn.

use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::components::{LocalWizard, Mana, PrimedSpell, Spell, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
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
    local_origin: Res<LocalSpellOrigin>,
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

    let origin = local_origin.0;
    vfx::systems::spawn_school_flare(
        &mut commands,
        &visual_assets,
        local_origin.0,
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
