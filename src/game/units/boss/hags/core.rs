//! Hag spawn, animations, movement, combat.

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::{EyeTransferTimer, HagAssets, HagDeathTracker};
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker};
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::utils::{EYE_FRAME_UV, EYE_PULSE_FRAME_DURATION, EYE_SHEET_COLUMNS};
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CombatAnimation, CommanderAuraSpeedModifier, Corpse,
    DamageMultiplier, Effectiveness, EliteSpeedBonus, FacingDirection, FacingDwell,
    FacingHysteresisBoost, FlockingModifier, FlockingVelocity, FrozenSolidModifier, HasteModifier,
    Health, Hitbox, InMelee, Invulnerable, MindControlled, MovementSpeed, PolymorphedModifier,
    PulsingAnimation, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier, SmoothedFacingVelocity, TargetingVelocity, Team,
    Teleportable, TemporaryHitPoints, WalkingAnimation, apply_damage_to_unit,
};
use crate::game::units::king::components::King;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::components::Wizard;

/// Builds a `WalkingAnimation` configured for the hag sprite sheet (4×4 frames).
fn hag_walking_animation(rng: &mut impl Rng) -> WalkingAnimation {
    let mut anim = WalkingAnimation::new_staggered(rng);
    anim.columns = HAG_SHEET_COLUMNS;
    anim.frame_uv = HAG_FRAME_UV;
    anim.direction_rows = HAG_DIRECTION_ROWS;
    anim
}

/// Builds a `CombatAnimation` for a hag using one of her sprite sheets
/// (attack or cast). All hag sheets share the same frame size; only the
/// row order, column count, and combat texture differ.
fn hag_combat_animation(
    hag_assets: &HagAssets,
    columns: usize,
    direction_rows: [usize; 4],
    combat_texture: Handle<Image>,
) -> CombatAnimation {
    CombatAnimation {
        current_frame: 0,
        elapsed: 0.0,
        columns,
        frame_uv: HAG_FRAME_UV,
        direction_rows,
        combat_texture,
        walking_texture: hag_assets.walking_texture.clone(),
        started: false,
    }
}

fn hag_attack_animation(hag_assets: &HagAssets) -> CombatAnimation {
    hag_combat_animation(
        hag_assets,
        HAG_ATTACK_COLUMNS,
        HAG_DIRECTION_ROWS,
        hag_assets.attacking_texture.clone(),
    )
}

pub(super) fn hag_casting_animation(hag_assets: &HagAssets) -> CombatAnimation {
    hag_combat_animation(
        hag_assets,
        HAG_CASTING_COLUMNS,
        HAG_CASTING_DIRECTION_ROWS,
        hag_assets.casting_texture.clone(),
    )
}

/// Pins a hag's material to a specific frame of the attacking sprite sheet
/// for her current facing direction. Used to lock Josephina's leap pose.
pub(super) fn set_hag_attack_pose_frame(
    materials: &mut Assets<StandardMaterial>,
    material_handle: &MeshMaterial3d<StandardMaterial>,
    hag_assets: &HagAssets,
    facing: FacingDirection,
    frame_idx: usize,
) {
    if let Some(mat) = materials.get_mut(material_handle) {
        let row = HAG_DIRECTION_ROWS[facing as usize] as f32;
        let offset = Vec2::new(frame_idx as f32 * HAG_FRAME_UV.x, row * HAG_FRAME_UV.y);
        mat.base_color_texture = Some(hag_assets.attacking_texture.clone());
        mat.uv_transform =
            bevy::math::Affine2::from_scale_angle_translation(HAG_FRAME_UV, 0.0, offset);
    }
}

/// Restores a hag's material to the walking sheet, frame 0 in her facing direction.
pub(super) fn restore_hag_walking_pose(
    materials: &mut Assets<StandardMaterial>,
    material_handle: &MeshMaterial3d<StandardMaterial>,
    hag_assets: &HagAssets,
    facing: FacingDirection,
) {
    if let Some(mat) = materials.get_mut(material_handle) {
        let row = HAG_DIRECTION_ROWS[facing as usize] as f32;
        let offset = Vec2::new(0.0, row * HAG_FRAME_UV.y);
        mat.base_color_texture = Some(hag_assets.walking_texture.clone());
        mat.uv_transform =
            bevy::math::Affine2::from_scale_angle_translation(HAG_FRAME_UV, 0.0, offset);
    }
}

/// Builds a `PulsingAnimation` for an eye sprite (4-frame in-place loop).
fn eye_pulsing_animation() -> PulsingAnimation {
    PulsingAnimation::new(EYE_SHEET_COLUMNS, EYE_FRAME_UV, EYE_PULSE_FRAME_DURATION)
}

/// Spawns all 3 hags at their designated grid positions.
pub fn spawn_hags(rng: &mut impl Rng, mut commands: Commands, hag_assets: Res<HagAssets>) {
    let hags = [
        (
            HagIdentity::Justina,
            JUSTINA_COL,
            &hag_assets.justina_material,
        ),
        (
            HagIdentity::Martina,
            MARTINA_COL,
            &hag_assets.martina_material,
        ),
        (
            HagIdentity::Josephina,
            JOSEPHINA_COL,
            &hag_assets.josephina_material,
        ),
    ];

    let mut spawned_entities = Vec::new();

    for (idx, (identity, _col, material)) in hags.iter().enumerate() {
        // Stagger each hag deeper behind the wall so the two that share a
        // tunnel (idx 0 and idx 2 both map to spawn point 0) don't land on
        // top of each other.
        let depth_offset = idx as f32 * 250.0;
        let (spawn_x, spawn_z) = attacker_spawn_position(idx as u32, depth_offset);
        let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

        let hitbox = Hitbox::new(HAG_RADIUS, HAG_HITBOX_HEIGHT);
        let spawn_y = hitbox.height / 2.0 + (HAG_ELLIPSE_DEPTH / 2.0) + 60.0;

        // Initial velocity toward castle
        let to_center = Vec3::new(
            WIZARD_POSITION.x - final_x,
            0.0,
            WIZARD_POSITION.z - final_z,
        );
        let initial_velocity = to_center.normalize_or_zero() * HAG_MOVEMENT_SPEED;

        let entity = commands
            .spawn((
                // Rendering
                Mesh3d(hag_assets.sprite_mesh.clone()),
                MeshMaterial3d((*material).clone()),
                Transform::from_xyz(final_x, spawn_y, final_z),
                // Physics
                Velocity {
                    x: initial_velocity.x,
                    z: initial_velocity.z,
                    ..default()
                },
                Acceleration::new(),
                // Core
                hitbox,
                Health::new(HAG_HEALTH),
                MovementSpeed(HAG_MOVEMENT_SPEED),
                AttackTiming::new(),
                Effectiveness::new(),
                Team::Attackers,
                Boss,
                Hag,
                *identity,
            ))
            .insert((
                HagEyeState::new(),
                HagAttackCooldown::new(),
                // Movement systems
                TargetingVelocity::default(),
                FlowFieldVelocity::default(),
                FlowFieldInfluence::Attacker,
                DamageMultiplier(HAG_DAMAGE_MULTIPLIER),
                FlockingVelocity::default(),
                FlockingModifier::new(0.0, 0.0, 0.0),
                CommanderAuraSpeedModifier(0.0),
                RoughTerrainModifier(0.0),
                Teleportable,
                Billboard,
                OnGameplayScreen,
            ))
            .insert((
                hag_walking_animation(rng),
                FacingDirection::default(),
                // Strong stickiness — separation/flow forces jitter the velocity
                // and would otherwise make hags flicker between facing rows.
                // Boost = 1.0 → 8° buffer past the 45° axis boundary.
                // Larger values widen the buffer further if needed.
                FacingHysteresisBoost(1.0),
                // After every facing change, lock in for 3.0s before another flip.
                FacingDwell::new(3.0),
                // Smoothed (low-pass) velocity for the facing decision so
                // tunnel/flow-field oscillations don't drive the choice.
                SmoothedFacingVelocity::new(0.4),
            ))
            .id();

        // Add identity-specific ability components
        match identity {
            HagIdentity::Justina => {
                commands
                    .entity(entity)
                    .insert((ChainLightningCooldown::new(), FireballCooldown::new()));
            }
            HagIdentity::Martina => {
                commands.entity(entity).insert(TeleportPullCooldown::new());
                // Spawn mind control aura circle on the ground beneath Martina
                let aura_y = 2.0 - spawn_y;
                let aura_entity = commands
                    .spawn((
                        Mesh3d(hag_assets.mind_control_aura_mesh.clone()),
                        MeshMaterial3d(hag_assets.mind_control_aura_material.clone()),
                        Transform::from_xyz(0.0, aura_y, 0.0)
                            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                        OnGameplayScreen,
                    ))
                    .id();
                commands.entity(entity).add_child(aura_entity);
            }
            HagIdentity::Josephina => {
                commands
                    .entity(entity)
                    .insert((LeapState::new(), MaulingState::new()));
            }
        }

        spawned_entities.push(entity);
    }

    // Initialize eye transfer timer
    let initial_interval = EYE_TRANSFER_BASE_INTERVAL
        + rng.random_range(-EYE_TRANSFER_VARIANCE..EYE_TRANSFER_VARIANCE);
    commands.insert_resource(EyeTransferTimer {
        time_remaining: initial_interval,
    });
    commands.insert_resource(HagDeathTracker::new());

    // Assign each eye to a random hag at spawn (different hags)
    if spawned_entities.len() >= 2 {
        let invuln_idx = rng.random_range(0..spawned_entities.len());
        let mut ability_idx = rng.random_range(0..spawned_entities.len());
        while ability_idx == invuln_idx {
            ability_idx = rng.random_range(0..spawned_entities.len());
        }

        let invuln_entity = spawned_entities[invuln_idx];
        let ability_entity = spawned_entities[ability_idx];

        commands.entity(invuln_entity).insert((
            HagEyeState {
                has_invulnerability_eye: true,
                has_ability_eye: false,
            },
            Invulnerable {
                health_snapshot: HAG_HEALTH,
            },
        ));
        spawn_eye_visual(
            &mut commands,
            invuln_entity,
            EyeType::Invulnerability,
            &hag_assets,
            false,
        );

        commands.entity(ability_entity).insert(HagEyeState {
            has_invulnerability_eye: false,
            has_ability_eye: true,
        });
        spawn_eye_visual(
            &mut commands,
            ability_entity,
            EyeType::Ability,
            &hag_assets,
            false,
        );
    }
}

/// Updates hag targeting velocity toward nearest enemy.
/// Blind hags skip normal targeting (handled by blind_hag_wandering).
pub fn update_hag_targeting(
    mut commands: Commands,
    mut hags: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            &HagEyeState,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<Wizard>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, hag_transform, hag_team, mut targeting, _eye_state) in &mut hags {
        // All hags use normal targeting (blind hags get noise added in hag_movement)
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            hag_transform,
            *hag_team,
            &mut targeting,
            &mut commands,
            None,
        );
    }
}

/// Hag melee combat — only hags with the invulnerability eye (or both eyes) attack.
/// Ability-only and blind hags skip combat. Consuming a corpse stops attacks.
#[allow(clippy::type_complexity)]
pub fn hag_combat(
    time: Res<Time>,
    mut commands: Commands,
    hag_assets: Res<HagAssets>,
    mut hags: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut HagAttackCooldown,
            Option<&MaulingState>,
            Option<&CorpseConsumeState>,
        ),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<Hag>,
            Without<Corpse>,
            Without<MindControlled>,
            Without<Wizard>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        hag_entity,
        hag_transform,
        hag_hitbox,
        hag_team,
        identity,
        eye_state,
        mut cooldown,
        mauling,
        consuming,
    ) in &mut hags
    {
        // Only hags with invulnerability eye do basic attacks
        if !eye_state.has_invulnerability_eye {
            continue;
        }

        // Consuming a corpse stops attacking
        if consuming.is_some() {
            continue;
        }

        cooldown.tick(delta);
        if !cooldown.is_ready() {
            continue;
        }

        let hag_pos = hag_transform.translation;
        let mut has_target = false;

        // First pass: check for any enemy in melee range
        for (entity, target_transform, target_hitbox, team, _, _) in &targets {
            if entity == hag_entity {
                continue;
            }
            if !hag_team.is_enemy(team) {
                continue;
            }

            let dx = hag_pos.x - target_transform.translation.x;
            let dz = hag_pos.z - target_transform.translation.z;
            let distance = (dx * dx + dz * dz).sqrt();
            let attack_range = (hag_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
            if distance <= attack_range {
                has_target = true;
                break;
            }
        }

        if !has_target {
            continue;
        }

        // Josephina's frenzy: 5x attack speed
        let attack_cd = if mauling.is_some_and(|m| m.is_frenzied()) {
            HAG_ATTACK_COOLDOWN / 5.0
        } else {
            HAG_ATTACK_COOLDOWN
        };
        cooldown.reset(attack_cd);

        // Hit nearest single enemy in melee range
        let mut nearest_target: Option<(Entity, f32)> = None;
        for (entity, target_transform, target_hitbox, team, _, _) in &targets {
            if entity == hag_entity {
                continue;
            }
            if !hag_team.is_enemy(team) {
                continue;
            }

            let target_pos = target_transform.translation;
            let dx = target_pos.x - hag_pos.x;
            let dz = target_pos.z - hag_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            if distance > (hag_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER {
                continue;
            }

            if let Some((_, best_dist)) = nearest_target {
                if distance < best_dist {
                    nearest_target = Some((entity, distance));
                }
            } else {
                nearest_target = Some((entity, distance));
            }
        }

        if let Some((target_entity, _)) = nearest_target
            && let Ok((_, _, _, _, mut health, mut temp_hp)) = targets.get_mut(target_entity)
        {
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), HAG_ATTACK_DAMAGE);

            // Josephina plays the melee swing animation on each landed attack.
            if *identity == HagIdentity::Josephina {
                commands
                    .entity(hag_entity)
                    .insert(hag_attack_animation(&hag_assets));
            }
        }
    }
}

/// Hag movement system using weighted velocities.
#[allow(clippy::type_complexity)]
pub fn hag_movement(
    time: Res<Time>,
    mut hags: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut hags
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Polymorphed units wander randomly
        if polymorphed.is_some() {
            let angle = (time.elapsed_secs() * 0.5 + velocity.x.to_bits() as f32).sin()
                * std::f32::consts::TAU;
            velocity.x = angle.cos() * 20.0;
            velocity.z = angle.sin() * 20.0;
            continue;
        }

        // All hags use weighted movement (flow field + targeting)
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// Applies a gentle separation force between hags to keep them spread apart.
/// Skipped during staging — all three hags share the center staging point and
/// shouldn't be shoved apart on the way there.
pub fn hag_separation(
    time: Res<Time>,
    mut hags: Query<
        (Entity, &Transform, &mut Velocity),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
) {
    let delta = time.delta_secs();
    let positions: Vec<(Entity, Vec3)> = hags.iter().map(|(e, t, _)| (e, t.translation)).collect();

    for (entity, _transform, mut velocity) in &mut hags {
        let my_pos = positions
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, p)| *p);
        let Some(my_pos) = my_pos else { continue };

        for (other_entity, other_pos) in &positions {
            if *other_entity == entity {
                continue;
            }
            let dx = my_pos.x - other_pos.x;
            let dz = my_pos.z - other_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist < HAG_SEPARATION_DISTANCE && dist > 0.1 {
                // Quadratic falloff — soft at the outer edge so flow-field
                // guidance dominates, but ramps up sharply when the sprites
                // are about to overlap. Per-second so it's frame-rate independent.
                let linear = 1.0 - (dist / HAG_SEPARATION_DISTANCE);
                let factor = linear * linear;
                let push_x = (dx / dist) * HAG_SEPARATION_STRENGTH * factor * delta;
                let push_z = (dz / dist) * HAG_SEPARATION_STRENGTH * factor * delta;
                velocity.x += push_x;
                velocity.z += push_z;
            }
        }
    }
}

/// Stops Justina advancing once she's within `JUSTINA_KITE_DISTANCE` of the king,
/// so she kites with her ranged abilities instead of charging into melee.
pub fn justina_kite_distance(
    mut justina_query: Query<
        (&Transform, &HagIdentity, &mut Velocity),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    king_query: Query<&Transform, (With<King>, Without<Hag>, Without<Corpse>)>,
) {
    let Ok(king_transform) = king_query.single() else {
        return;
    };
    let king_pos = king_transform.translation;
    let kite_dist_sq = JUSTINA_KITE_DISTANCE * JUSTINA_KITE_DISTANCE;

    for (transform, identity, mut velocity) in &mut justina_query {
        if *identity != HagIdentity::Justina {
            continue;
        }
        let dx = transform.translation.x - king_pos.x;
        let dz = transform.translation.z - king_pos.z;
        if dx * dx + dz * dz <= kite_dist_sq {
            velocity.x = 0.0;
            velocity.z = 0.0;
        }
    }
}

/// Spawns a floating eye visual as a child of the given hag entity.
fn spawn_eye_visual(
    commands: &mut Commands,
    parent: Entity,
    eye_type: EyeType,
    hag_assets: &HagAssets,
    has_other_eye: bool,
) {
    let material = match eye_type {
        EyeType::Invulnerability => hag_assets.invulnerability_eye_material.clone(),
        EyeType::Ability => hag_assets.ability_eye_material.clone(),
    };

    // Offset X if the hag has both eyes
    let x_offset = if has_other_eye {
        match eye_type {
            EyeType::Invulnerability => -EYE_VISUAL_SPACING / 2.0,
            EyeType::Ability => EYE_VISUAL_SPACING / 2.0,
        }
    } else {
        0.0
    };

    let eye_entity = commands
        .spawn((
            Mesh3d(hag_assets.eye_sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(x_offset, EYE_VISUAL_OFFSET_Y, 0.0),
            EyeVisual { eye_type },
            eye_pulsing_animation(),
        ))
        .id();

    commands.entity(parent).add_child(eye_entity);
}

/// Ticks the eye transfer timer and launches eyes in flight to new hag holders.
/// Invulnerability is removed immediately when the eye leaves the source hag.
#[allow(clippy::too_many_arguments)]
pub fn tick_eye_transfer(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut timer: ResMut<EyeTransferTimer>,
    mut hags: Query<
        (Entity, &Transform, &mut HagEyeState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    eye_visuals: Query<(Entity, &ChildOf, &EyeVisual)>,
    hag_assets: Res<HagAssets>,
    death_tracker: Res<HagDeathTracker>,
) {
    // Don't tick the timer until at least one hag has finished staging —
    // otherwise eyes would shuffle around before the fight even starts.
    if hags.is_empty() {
        return;
    }

    timer.time_remaining -= time.delta_secs();
    if timer.time_remaining > 0.0 {
        return;
    }

    // Reset timer
    timer.time_remaining = EYE_TRANSFER_BASE_INTERVAL
        + game_rng
            .0
            .random_range(-EYE_TRANSFER_VARIANCE..EYE_TRANSFER_VARIANCE);

    let living_hags: Vec<Entity> = hags.iter().map(|(e, _, _)| e).collect();
    if living_hags.len() < 2 {
        return;
    }

    // Determine how many eyes are still in play based on permanent deaths
    let has_invuln_eye = death_tracker.permanent_deaths < 1;
    let has_ability_eye = death_tracker.permanent_deaths < 2;

    // Find current holders
    let mut current_invuln_holder: Option<Entity> = None;
    let mut current_ability_holder: Option<Entity> = None;
    for (entity, _, eye_state) in &hags {
        if eye_state.has_invulnerability_eye {
            current_invuln_holder = Some(entity);
        }
        if eye_state.has_ability_eye {
            current_ability_holder = Some(entity);
        }
    }

    // Pick new holders (must be different from current holder)
    let new_invuln_holder = if has_invuln_eye {
        let candidates: Vec<Entity> = living_hags
            .iter()
            .copied()
            .filter(|e| Some(*e) != current_invuln_holder)
            .collect();
        if candidates.is_empty() {
            current_invuln_holder // Only one hag alive, keep it
        } else {
            Some(candidates[game_rng.0.random_range(0..candidates.len())])
        }
    } else {
        None
    };

    let new_ability_holder = if has_ability_eye {
        let candidates: Vec<Entity> = living_hags
            .iter()
            .copied()
            .filter(|e| Some(*e) != current_ability_holder && Some(*e) != new_invuln_holder)
            .collect();
        if candidates.is_empty() {
            current_ability_holder // No valid candidate, keep it
        } else {
            Some(candidates[game_rng.0.random_range(0..candidates.len())])
        }
    } else {
        None
    };

    // Process invulnerability eye
    if let Some(new_holder) = new_invuln_holder {
        let needs_flight = current_invuln_holder.is_some_and(|cur| cur != new_holder);

        if needs_flight {
            let source = current_invuln_holder.expect("checked above");
            // Clear source hag's eye state and invulnerability immediately
            if let Ok((_, _, mut eye_state)) = hags.get_mut(source) {
                eye_state.has_invulnerability_eye = false;
            }
            commands.entity(source).remove::<Invulnerable>();
            // Despawn eye visual from source
            for (eye_entity, child_of, eye_visual) in &eye_visuals {
                if child_of.parent() == source && eye_visual.eye_type == EyeType::Invulnerability {
                    commands.entity(eye_entity).try_despawn();
                }
            }
            // Get source position and spawn flying eye
            if let Ok((_, source_transform, _)) = hags.get(source) {
                let start_pos =
                    source_transform.translation + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0);
                commands.spawn((
                    Mesh3d(hag_assets.eye_sprite_mesh.clone()),
                    MeshMaterial3d(hag_assets.invulnerability_eye_material.clone()),
                    Transform::from_translation(start_pos),
                    Billboard,
                    OnGameplayScreen,
                    eye_pulsing_animation(),
                    EyeInFlight {
                        eye_type: EyeType::Invulnerability,
                        target: new_holder,
                        start_pos,
                        progress: 0.0,
                    },
                ));
            }
        } else if current_invuln_holder.is_none() {
            // Eye needs to appear fresh (shouldn't happen after initialize_eyes, but handle it)
            if let Ok((_, _, mut eye_state)) = hags.get_mut(new_holder) {
                eye_state.has_invulnerability_eye = true;
                let both = new_ability_holder == Some(new_holder);
                spawn_eye_visual(
                    &mut commands,
                    new_holder,
                    EyeType::Invulnerability,
                    &hag_assets,
                    both,
                );
            }
        }
        // If staying on same hag, do nothing
    }

    // Process ability eye
    if let Some(new_holder) = new_ability_holder {
        let needs_flight = current_ability_holder.is_some_and(|cur| cur != new_holder);

        if needs_flight {
            let source = current_ability_holder.expect("checked above");
            // Clear source hag's eye state immediately
            if let Ok((_, _, mut eye_state)) = hags.get_mut(source) {
                eye_state.has_ability_eye = false;
            }
            // Despawn eye visual from source
            for (eye_entity, child_of, eye_visual) in &eye_visuals {
                if child_of.parent() == source && eye_visual.eye_type == EyeType::Ability {
                    commands.entity(eye_entity).try_despawn();
                }
            }
            // Get source position and spawn flying eye
            if let Ok((_, source_transform, _)) = hags.get(source) {
                let start_pos =
                    source_transform.translation + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0);
                commands.spawn((
                    Mesh3d(hag_assets.eye_sprite_mesh.clone()),
                    MeshMaterial3d(hag_assets.ability_eye_material.clone()),
                    Transform::from_translation(start_pos),
                    Billboard,
                    OnGameplayScreen,
                    eye_pulsing_animation(),
                    EyeInFlight {
                        eye_type: EyeType::Ability,
                        target: new_holder,
                        start_pos,
                        progress: 0.0,
                    },
                ));
            }
        } else if current_ability_holder.is_none()
            && let Ok((_, _, mut eye_state)) = hags.get_mut(new_holder)
        {
            eye_state.has_ability_eye = true;
            let both = new_invuln_holder == Some(new_holder);
            spawn_eye_visual(
                &mut commands,
                new_holder,
                EyeType::Ability,
                &hag_assets,
                both,
            );
        }
        // If staying on same hag, do nothing
    }

    // Fix X offsets for eyes that stayed on a hag that now lost or gained a companion eye
    // Despawn and re-spawn visuals for hags whose eye count changed but eyes didn't move
    for (entity, _, eye_state) in &hags {
        let had_invuln = current_invuln_holder == Some(entity);
        let had_ability = current_ability_holder == Some(entity);
        let has_invuln = eye_state.has_invulnerability_eye;
        let has_ability = eye_state.has_ability_eye;

        // If this hag still has an eye but the other eye left/arrived, re-offset
        let old_both = had_invuln && had_ability;
        let new_both = has_invuln && has_ability;
        if old_both != new_both && (has_invuln || has_ability) {
            // Despawn existing eye visuals for this hag
            for (eye_entity, child_of, _) in &eye_visuals {
                if child_of.parent() == entity {
                    commands.entity(eye_entity).try_despawn();
                }
            }
            // Re-spawn with correct offset
            if has_invuln {
                spawn_eye_visual(
                    &mut commands,
                    entity,
                    EyeType::Invulnerability,
                    &hag_assets,
                    new_both,
                );
            }
            if has_ability {
                spawn_eye_visual(
                    &mut commands,
                    entity,
                    EyeType::Ability,
                    &hag_assets,
                    new_both,
                );
            }
        }
    }
}

/// Updates eyes in flight — arcs them toward their target hag and delivers on arrival.
pub fn update_eye_flight(
    time: Res<Time>,
    mut commands: Commands,
    mut eyes: Query<(Entity, &mut EyeInFlight, &mut Transform), Without<Hag>>,
    mut hags: Query<
        (Entity, &Transform, &mut HagEyeState, &Health),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    hag_assets: Res<HagAssets>,
    existing_eye_visuals: Query<(Entity, &ChildOf, &EyeVisual), Without<EyeInFlight>>,
) {
    let delta = time.delta_secs();

    for (eye_entity, mut flight, mut eye_transform) in &mut eyes {
        flight.progress += delta / EYE_TOSS_FLIGHT_DURATION;

        // Get target hag's current position (homing)
        let target_pos = if let Ok((_, target_transform, _, _)) = hags.get(flight.target) {
            target_transform.translation + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0)
        } else {
            // Target died or despawned — just despawn the eye
            commands.entity(eye_entity).try_despawn();
            continue;
        };

        if flight.progress >= 1.0 {
            // Eye arrived — deliver to target hag
            commands.entity(eye_entity).try_despawn();

            if let Ok((_, _, mut eye_state, health)) = hags.get_mut(flight.target) {
                match flight.eye_type {
                    EyeType::Invulnerability => {
                        eye_state.has_invulnerability_eye = true;
                        commands.entity(flight.target).insert(Invulnerable {
                            health_snapshot: health.current,
                        });
                    }
                    EyeType::Ability => eye_state.has_ability_eye = true,
                }

                let has_both = eye_state.has_invulnerability_eye && eye_state.has_ability_eye;

                // If gaining a second eye, re-spawn existing eye with correct offset
                if has_both {
                    for (vis_entity, child_of, _) in &existing_eye_visuals {
                        if child_of.parent() == flight.target {
                            commands.entity(vis_entity).try_despawn();
                        }
                    }
                    // Re-spawn the other eye type with both=true offset
                    let other_type = match flight.eye_type {
                        EyeType::Invulnerability => EyeType::Ability,
                        EyeType::Ability => EyeType::Invulnerability,
                    };
                    spawn_eye_visual(&mut commands, flight.target, other_type, &hag_assets, true);
                }

                spawn_eye_visual(
                    &mut commands,
                    flight.target,
                    flight.eye_type,
                    &hag_assets,
                    has_both,
                );
            }
        } else {
            // Interpolate position with parabolic arc
            let t = flight.progress;
            let x = flight.start_pos.x + (target_pos.x - flight.start_pos.x) * t;
            let z = flight.start_pos.z + (target_pos.z - flight.start_pos.z) * t;
            let base_y = flight.start_pos.y + (target_pos.y - flight.start_pos.y) * t;
            let arc_offset = EYE_TOSS_ARC_HEIGHT * 4.0 * t * (1.0 - t);
            eye_transform.translation = Vec3::new(x, base_y + arc_offset, z);
        }
    }
}

/// Prevents hags with eyes from dying — if their health hits zero, immediately
/// resurrect them. Runs BEFORE corpse conversion so they never become corpses.
pub fn resurrect_eyed_hags(
    mut dying_hags: Query<
        (&HagEyeState, &mut Health),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
) {
    for (eye_state, mut health) in &mut dying_hags {
        if health.is_dead() && (eye_state.has_invulnerability_eye || eye_state.has_ability_eye) {
            health.current = health.max * RESURRECT_HEAL_PERCENT;
        }
    }
}

/// Handles permanent death of blind hags (no eyes) after they become corpses.
pub fn intercept_blind_hag_death(
    mut commands: Commands,
    hag_corpses: Query<Entity, (With<Hag>, With<Corpse>, Without<PermanentlyDead>)>,
    mut living_eye_states: Query<
        (Entity, &mut HagEyeState),
        (
            With<Hag>,
            Without<Corpse>,
            Without<PermanentlyDead>,
            Without<StagingAttacker>,
        ),
    >,
    mut death_tracker: ResMut<HagDeathTracker>,
    eye_visuals: Query<(Entity, &ChildOf, &EyeVisual)>,
    eyes_in_flight: Query<(Entity, &EyeInFlight)>,
) {
    for entity in &hag_corpses {
        commands.entity(entity).insert(PermanentlyDead);
        death_tracker.permanent_deaths += 1;

        match death_tracker.permanent_deaths {
            1 => {
                // First permanent death: invulnerability eye disappears
                for (eye_entity, _, eye_visual) in &eye_visuals {
                    if eye_visual.eye_type == EyeType::Invulnerability {
                        commands.entity(eye_entity).try_despawn();
                    }
                }
                for (flight_entity, flight) in &eyes_in_flight {
                    if flight.eye_type == EyeType::Invulnerability {
                        commands.entity(flight_entity).try_despawn();
                    }
                }
                for (hag_entity, mut eye_state) in &mut living_eye_states {
                    eye_state.has_invulnerability_eye = false;
                    commands.entity(hag_entity).remove::<Invulnerable>();
                }
            }
            2 => {
                // Second permanent death: ability eye also disappears
                for (eye_entity, _, _) in &eye_visuals {
                    commands.entity(eye_entity).try_despawn();
                }
                for (flight_entity, _) in &eyes_in_flight {
                    commands.entity(flight_entity).try_despawn();
                }
                for (_, mut eye_state) in &mut living_eye_states {
                    eye_state.has_ability_eye = false;
                    eye_state.has_invulnerability_eye = false;
                }
            }
            _ => {}
        }
    }
}

/// Applies enrage haste to the last surviving hag when 2 are permanently dead.
pub fn apply_enrage_to_last_hag(
    mut commands: Commands,
    hags: Query<(Entity, &HagEyeState), (With<Hag>, Without<Corpse>, Without<PermanentlyDead>)>,
    death_tracker: Res<HagDeathTracker>,
    existing_haste: Query<&HasteModifier, With<Hag>>,
) {
    if death_tracker.permanent_deaths < 2 {
        return;
    }

    for (entity, _) in &hags {
        // Only add haste if not already enraged
        if existing_haste.get(entity).is_err() {
            commands
                .entity(entity)
                .insert(HasteModifier::new(ENRAGE_SPEED_BONUS, f32::MAX));
        }
    }
}
