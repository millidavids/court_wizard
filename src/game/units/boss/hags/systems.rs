use std::collections::HashSet;

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::{EyeTransferTimer, HagAssets, HagDeathTracker};
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::components::Knockback;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FlockingModifier, FlockingVelocity,
    HasteModifier, Health, Hitbox, InMelee, Invulnerable, KingsGuard, MindControlled,
    MovementSpeed, PolymorphedModifier, RetaliationTarget, RootedModifier, RoughTerrainModifier,
    SlowMovementModifier, SleepModifier, TargetingVelocity, Team, Teleportable,
    TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::king::components::King;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Spawns all 3 hags at their designated grid positions.
pub fn spawn_hags(mut commands: Commands, hag_assets: Res<HagAssets>) {
    let hags = [
        (HagIdentity::Justina, JUSTINA_COL, &hag_assets.justina_material),
        (HagIdentity::Martina, MARTINA_COL, &hag_assets.martina_material),
        (HagIdentity::Josephina, JOSEPHINA_COL, &hag_assets.josephina_material),
    ];

    let mut spawned_entities = Vec::new();

    for (identity, col, material) in hags {
        let (spawn_x, spawn_z) = calculate_grid_cell_position(0, col);
        let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

        let hitbox = Hitbox::new(HAG_RADIUS, HAG_HITBOX_HEIGHT);
        let spawn_y = hitbox.height / 2.0 + (HAG_ELLIPSE_DEPTH / 2.0) + 1.0;

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
                Mesh3d(hag_assets.mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(final_x, spawn_y, final_z),
                // Physics
                Velocity {
                    x: initial_velocity.x,
                    z: initial_velocity.z,
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
                identity,
            ))
            .insert((
                HagEyeState::new(),
                HagAttackCooldown::new(),
                BlindWanderState::new(),
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
            .id();

        // Add identity-specific ability components
        match identity {
            HagIdentity::Justina => {
                commands.entity(entity).insert((
                    ChainLightningCooldown::new(),
                    FireballCooldown::new(),
                ));
            }
            HagIdentity::Martina => {
                commands.entity(entity).insert(
                    TeleportPullCooldown::new(),
                );
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
                commands.entity(entity).insert((
                    LeapState::new(),
                    MaulingState::new(),
                ));
            }
        }

        spawned_entities.push(entity);
    }

    // Initialize eye transfer timer
    let mut rng = rand::thread_rng();
    let initial_interval =
        EYE_TRANSFER_BASE_INTERVAL + rng.gen_range(-EYE_TRANSFER_VARIANCE..EYE_TRANSFER_VARIANCE);
    commands.insert_resource(EyeTransferTimer {
        time_remaining: initial_interval,
    });
    commands.insert_resource(HagDeathTracker::new());

    // Assign each eye to a random hag at spawn (different hags)
    if spawned_entities.len() >= 2 {
        let invuln_idx = rng.gen_range(0..spawned_entities.len());
        let mut ability_idx = rng.gen_range(0..spawned_entities.len());
        while ability_idx == invuln_idx {
            ability_idx = rng.gen_range(0..spawned_entities.len());
        }

        let invuln_entity = spawned_entities[invuln_idx];
        let ability_entity = spawned_entities[ability_idx];

        commands.entity(invuln_entity).insert((
            HagEyeState {
                has_invulnerability_eye: true,
                has_ability_eye: false,
            },
            Invulnerable { health_snapshot: HAG_HEALTH },
        ));
        spawn_eye_visual(&mut commands, invuln_entity, EyeType::Invulnerability, &hag_assets, false);

        commands.entity(ability_entity).insert(HagEyeState {
            has_invulnerability_eye: false,
            has_ability_eye: true,
        });
        spawn_eye_visual(&mut commands, ability_entity, EyeType::Ability, &hag_assets, false);
    }
}

/// Updates hag targeting velocity toward nearest enemy.
/// Blind hags skip normal targeting (handled by blind_hag_wandering).
pub fn update_hag_targeting(
    mut commands: Commands,
    mut hags: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity, &HagEyeState),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    all_units: Query<(Entity, &Transform, &Team), (Without<Hag>, Without<Corpse>, Without<Wizard>)>,
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
    mut hags: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &HagEyeState,
            &mut HagAttackCooldown,
            Option<&MaulingState>,
            Option<&CorpseConsumeState>,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
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
        (Without<Hag>, Without<Corpse>, Without<MindControlled>, Without<Wizard>),
    >,
) {
    let delta = time.delta_secs();

    for (hag_entity, hag_transform, hag_hitbox, hag_team, eye_state, mut cooldown, mauling, consuming) in &mut hags {
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
            let attack_range =
                (hag_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
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
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &HagEyeState,
            &mut BlindWanderState,
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
                Option<&SleepModifier>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
            ),
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
) {
    let delta = time.delta_secs();

    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        eye_state,
        mut wander_state,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, banished, polymorphed),
    ) in &mut hags
    {
        // CC'd units cannot move
        if rooted.is_some() || sleeping.is_some() || banished.is_some() {
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
            effectiveness,
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

        // Blind hags deflect their movement direction with random noise (lazy drift)
        // This rotates the velocity without increasing speed
        if eye_state.is_blind() {
            wander_state.timer -= delta;
            if wander_state.timer <= 0.0 {
                let mut rng = rand::thread_rng();
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                wander_state.direction_x = angle.cos();
                wander_state.direction_z = angle.sin();
                wander_state.timer = BLIND_WANDER_DIRECTION_INTERVAL;
            }
            // Blend flow field direction with random noise, keeping original speed
            let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
            if speed > 0.01 {
                let flow_dir_x = velocity.x / speed;
                let flow_dir_z = velocity.z / speed;
                // Mix: 60% flow field, 40% random noise
                let blended_x = flow_dir_x * 0.6 + wander_state.direction_x * 0.4;
                let blended_z = flow_dir_z * 0.6 + wander_state.direction_z * 0.4;
                let blend_len = (blended_x * blended_x + blended_z * blended_z).sqrt();
                if blend_len > 0.01 {
                    velocity.x = (blended_x / blend_len) * speed;
                    velocity.z = (blended_z / blend_len) * speed;
                }
            }
        }
    }
}

/// Applies a strong separation force between hags to keep them spread apart.
pub fn hag_separation(
    mut hags: Query<
        (Entity, &Transform, &mut Velocity),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
) {
    let positions: Vec<(Entity, Vec3)> = hags
        .iter()
        .map(|(e, t, _)| (e, t.translation))
        .collect();

    for (entity, _transform, mut velocity) in &mut hags {
        let my_pos = positions.iter().find(|(e, _)| *e == entity).map(|(_, p)| *p);
        let Some(my_pos) = my_pos else { continue };

        for (other_entity, other_pos) in &positions {
            if *other_entity == entity {
                continue;
            }
            let dx = my_pos.x - other_pos.x;
            let dz = my_pos.z - other_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist < HAG_SEPARATION_DISTANCE && dist > 0.1 {
                // Stronger push the closer they are
                let factor = 1.0 - (dist / HAG_SEPARATION_DISTANCE);
                let push_x = (dx / dist) * HAG_SEPARATION_STRENGTH * factor;
                let push_z = (dz / dist) * HAG_SEPARATION_STRENGTH * factor;
                velocity.x += push_x;
                velocity.z += push_z;
            }
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
    let (mesh, material) = match eye_type {
        EyeType::Invulnerability => (
            hag_assets.eye_mesh.clone(),
            hag_assets.invulnerability_eye_material.clone(),
        ),
        EyeType::Ability => (
            hag_assets.eye_mesh.clone(),
            hag_assets.ability_eye_material.clone(),
        ),
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
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(x_offset, EYE_VISUAL_OFFSET_Y, 0.0),
            EyeVisual { eye_type },
        ))
        .id();

    commands.entity(parent).add_child(eye_entity);
}

/// Ticks the eye transfer timer and launches eyes in flight to new hag holders.
/// Invulnerability is removed immediately when the eye leaves the source hag.
pub fn tick_eye_transfer(
    time: Res<Time>,
    mut commands: Commands,
    mut timer: ResMut<EyeTransferTimer>,
    mut hags: Query<
        (Entity, &Transform, &mut HagEyeState),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    eye_visuals: Query<(Entity, &ChildOf, &EyeVisual)>,
    hag_assets: Res<HagAssets>,
    death_tracker: Res<HagDeathTracker>,
) {
    timer.time_remaining -= time.delta_secs();
    if timer.time_remaining > 0.0 {
        return;
    }

    // Reset timer
    let mut rng = rand::thread_rng();
    timer.time_remaining =
        EYE_TRANSFER_BASE_INTERVAL + rng.gen_range(-EYE_TRANSFER_VARIANCE..EYE_TRANSFER_VARIANCE);

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
            Some(candidates[rng.gen_range(0..candidates.len())])
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
            Some(candidates[rng.gen_range(0..candidates.len())])
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
                if child_of.parent() == source
                    && eye_visual.eye_type == EyeType::Invulnerability
                {
                    commands.entity(eye_entity).try_despawn();
                }
            }
            // Get source position and spawn flying eye
            if let Ok((_, source_transform, _)) = hags.get(source) {
                let start_pos = source_transform.translation
                    + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0);
                commands.spawn((
                    Mesh3d(hag_assets.eye_mesh.clone()),
                    MeshMaterial3d(hag_assets.invulnerability_eye_material.clone()),
                    Transform::from_translation(start_pos),
                    Billboard,
                    OnGameplayScreen,
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
                spawn_eye_visual(&mut commands, new_holder, EyeType::Invulnerability, &hag_assets, both);
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
                let start_pos = source_transform.translation
                    + Vec3::new(0.0, EYE_VISUAL_OFFSET_Y, 0.0);
                commands.spawn((
                    Mesh3d(hag_assets.eye_mesh.clone()),
                    MeshMaterial3d(hag_assets.ability_eye_material.clone()),
                    Transform::from_translation(start_pos),
                    Billboard,
                    OnGameplayScreen,
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
            spawn_eye_visual(&mut commands, new_holder, EyeType::Ability, &hag_assets, both);
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
                spawn_eye_visual(&mut commands, entity, EyeType::Invulnerability, &hag_assets, new_both);
            }
            if has_ability {
                spawn_eye_visual(&mut commands, entity, EyeType::Ability, &hag_assets, new_both);
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
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
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

                let has_both =
                    eye_state.has_invulnerability_eye && eye_state.has_ability_eye;

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
                    spawn_eye_visual(
                        &mut commands,
                        flight.target,
                        other_type,
                        &hag_assets,
                        true,
                    );
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
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
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
    mut living_eye_states: Query<(Entity, &mut HagEyeState), (With<Hag>, Without<Corpse>, Without<PermanentlyDead>)>,
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
            commands.entity(entity).insert(HasteModifier {
                modifier: ENRAGE_SPEED_BONUS,
                time_remaining: f32::MAX, // Permanent
            });
        }
    }
}

// ===== Phase 4: Justina Abilities =====

/// Justina's fire arc — cone AOE damage to enemies in front.
/// Justina's chain lightning — spawns a chain lightning bolt from her position
/// that bounces between nearby enemies using the existing chain lightning system.
pub fn justina_chain_lightning(
    time: Res<Time>,
    mut commands: Commands,
    death_tracker: Res<HagDeathTracker>,
    mut justina_query: Query<
        (
            &Transform,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut ChainLightningCooldown,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    targets: Query<
        (Entity, &Transform, &Team),
        (Without<Hag>, Without<Corpse>, Without<MindControlled>, Without<Wizard>),
    >,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
    visual_assets: Res<SpellVisualAssets>,
) {
    use crate::game::units::wizard::spells::chain_lightning::{
        components::{ChainLightningBolt, ChainLightningGroup},
        constants as cl_constants,
        systems as cl_systems,
    };

    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (transform, team, identity, eye_state, mut cooldown) in &mut justina_query {
        if *identity != HagIdentity::Justina || (!eye_state.has_ability_eye && !enraged) {
            continue;
        }

        cooldown.time_remaining -= delta;
        if cooldown.time_remaining > 0.0 {
            continue;
        }
        cooldown.time_remaining = CHAIN_LIGHTNING_COOLDOWN;

        let hag_pos = transform.translation;

        // Find nearest enemy within range
        let mut nearest: Option<(Entity, Vec3, f32)> = None;
        for (entity, target_transform, target_team) in &targets {
            if !team.is_enemy(target_team) {
                continue;
            }
            let dx = target_transform.translation.x - hag_pos.x;
            let dz = target_transform.translation.z - hag_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist <= CHAIN_LIGHTNING_RANGE
                && (nearest.is_none() || dist < nearest.unwrap().2)
            {
                nearest = Some((entity, target_transform.translation, dist));
            }
        }

        if let Some((first_target, target_pos, _)) = nearest {
            // Apply initial damage to first target
            if let Ok((mut health, mut temp_hp)) = health_query.get_mut(first_target) {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), CHAIN_LIGHTNING_DAMAGE);
            }

            // Spawn chain lightning group for hit tracking
            let group_entity = commands
                .spawn((
                    ChainLightningGroup {
                        hit_entities: HashSet::from([first_target]),
                    },
                    OnGameplayScreen,
                ))
                .id();

            // Spawn the bounce bolt (starts from first target, bounces to others)
            commands.spawn((
                ChainLightningBolt {
                    group_entity,
                    current_damage: CHAIN_LIGHTNING_DAMAGE * cl_constants::DAMAGE_FALLOFF,
                    damage_type: crate::game::units::DamageType::Electric,
                    bounces_remaining: cl_constants::MAX_BOUNCES,
                    last_hit_position: target_pos,
                    bounce_delay_timer: cl_constants::BOUNCE_DELAY,
                    empowerment: 1.0,
                    split_depth: 1,
                    split_count: cl_constants::SPLIT_COUNT,
                    damage_falloff: cl_constants::DAMAGE_FALLOFF,
                    static_charge: false,
                    magnetic_pull: false,
                    chain_reaction: false,
                    bounce_range_mult: 1.0,
                },
                OnGameplayScreen,
            ));

            // Spawn the visual arc from Justina to the first target
            cl_systems::spawn_arc(
                &mut commands,
                &visual_assets,
                hag_pos,
                target_pos,
                0,
                1.0,
            );
        }
    }
}

/// Justina's fireball — shoots fireballs at random defenders.
pub fn justina_fireball(
    time: Res<Time>,
    mut commands: Commands,
    death_tracker: Res<HagDeathTracker>,
    mut justina_query: Query<
        (
            &HagIdentity,
            &HagEyeState,
            &Transform,
            &mut FireballCooldown,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    defender_teams: Query<(&Transform, &Team), (Without<Hag>, Without<Corpse>, Without<MindControlled>, Without<Wizard>)>,
    visual_assets: Res<SpellVisualAssets>,
) {
    let delta = time.delta_secs();

    let enraged = death_tracker.permanent_deaths >= 2;

    for (identity, eye_state, transform, mut cooldown) in &mut justina_query {
        if *identity != HagIdentity::Justina || (!eye_state.has_ability_eye && !enraged) {
            continue;
        }

        cooldown.time_remaining -= delta;
        if cooldown.time_remaining > 0.0 {
            continue;
        }
        cooldown.time_remaining = FIREBALL_COOLDOWN;

        // Collect defender positions
        let defender_positions: Vec<Vec3> = defender_teams
            .iter()
            .filter(|(_, team)| **team == Team::Defenders)
            .map(|(t, _)| t.translation)
            .collect();

        if defender_positions.is_empty() {
            continue;
        }

        let hag_pos = transform.translation;
        let mut rng = rand::thread_rng();

        for _ in 0..FIREBALL_COUNT {
            let idx = rng.gen_range(0..defender_positions.len());
            let target_pos = defender_positions[idx];

            let direction = (target_pos - hag_pos).normalize_or_zero();
            let velocity = direction * FIREBALL_SPEED;
            // Offset spawn past Justina's hitbox so she doesn't hit herself
            let spawn_pos = hag_pos + direction * (HAG_RADIUS + FIREBALL_COLLISION_RADIUS + 5.0);

            crate::game::units::wizard::spells::fireball::systems::spawn_fireball_entity(
                &mut commands,
                &visual_assets,
                spawn_pos,
                velocity,
                FIREBALL_DAMAGE,
                crate::game::units::DamageType::Fire,
                FIREBALL_EXPLOSION_RADIUS,
                FIREBALL_COLLISION_RADIUS,
                0.0,
                FIREBALL_VISUAL_RADIUS,
            );
        }
    }
}

// ===== Phase 4: Josephina Abilities =====

/// Josephina's leap — parabolic arc jump to a random defender, knockback on landing.
#[allow(clippy::type_complexity)]
pub fn josephina_leap(
    time: Res<Time>,
    death_tracker: Res<HagDeathTracker>,
    mut josephina_query: Query<
        (
            Entity,
            &mut Transform,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut LeapState,
            &mut Velocity,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    defenders: Query<(&Transform, &Team), (Without<Hag>, Without<Corpse>, Without<MindControlled>, Without<Wizard>)>,
) {
    let delta = time.delta_secs();

    for (_entity, mut transform, team, identity, eye_state, mut leap, mut velocity) in
        &mut josephina_query
    {
        let enraged = death_tracker.permanent_deaths >= 2;
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            // If eye transferred away mid-leap, cancel and land
            if let LeapState::InAir { target, .. } = &*leap {
                transform.translation.x = target.x;
                transform.translation.z = target.z;
                *leap = LeapState::Idle { cooldown: LEAP_COOLDOWN };
            }
            continue;
        }

        match &mut *leap {
            LeapState::Idle { cooldown } => {
                *cooldown -= delta;
                if *cooldown > 0.0 {
                    continue;
                }

                // Pick a random defender target within leap range
                let josephina_pos = transform.translation;
                let defender_positions: Vec<Vec3> = defenders
                    .iter()
                    .filter(|(_, t)| team.is_enemy(t))
                    .map(|(t, _)| t.translation)
                    .filter(|pos| {
                        let dx = pos.x - josephina_pos.x;
                        let dz = pos.z - josephina_pos.z;
                        (dx * dx + dz * dz).sqrt() <= LEAP_MAX_RANGE
                    })
                    .collect();

                if defender_positions.is_empty() {
                    continue;
                }

                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..defender_positions.len());
                let target = defender_positions[idx];

                *leap = LeapState::InAir {
                    target,
                    start_pos: transform.translation,
                    progress: 0.0,
                };

                // Zero velocity during leap
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            LeapState::InAir {
                target,
                start_pos,
                progress,
            } => {
                *progress += delta / LEAP_FLIGHT_DURATION;

                if *progress >= 1.0 {
                    // Land at target, restore Y to pre-leap height
                    transform.translation.x = target.x;
                    transform.translation.y = start_pos.y;
                    transform.translation.z = target.z;
                    // Knockback is applied by josephina_leap_knockback system
                    *leap = LeapState::Landing { timer: 0.3, knockback_applied: false };
                } else {
                    // Parabolic arc interpolation
                    let t = *progress;
                    let x = start_pos.x + (target.x - start_pos.x) * t;
                    let z = start_pos.z + (target.z - start_pos.z) * t;
                    // Parabolic height: peaks at t=0.5
                    let height_offset = LEAP_MAX_HEIGHT * 4.0 * t * (1.0 - t);
                    transform.translation.x = x;
                    transform.translation.y = start_pos.y + height_offset;
                    transform.translation.z = z;
                }
            }
            LeapState::Landing { timer, .. } => {
                *timer -= delta;
                if *timer <= 0.0 {
                    // After landing, transition to mauling (handled by mauling system)
                    *leap = LeapState::Idle { cooldown: LEAP_COOLDOWN };
                }
            }
        }
    }
}

/// Apply knockback on Josephina's leap landing.
pub fn josephina_leap_knockback(
    mut commands: Commands,
    mut josephina_query: Query<
        (&Transform, &Team, &HagIdentity, &mut LeapState),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<Hag>, Without<Corpse>, Without<Wizard>)>,
) {
    for (transform, team, identity, mut leap) in &mut josephina_query {
        if *identity != HagIdentity::Josephina {
            continue;
        }

        // Only apply knockback once at landing moment
        if let LeapState::Landing { knockback_applied, .. } = leap.as_mut() {
            if *knockback_applied {
                continue;
            }
            *knockback_applied = true;

            let land_pos = transform.translation;
            for (entity, target_transform, target_team) in &targets {
                if !team.is_enemy(target_team) {
                    continue;
                }

                let dx = target_transform.translation.x - land_pos.x;
                let dz = target_transform.translation.z - land_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= LEAP_KNOCKBACK_RADIUS {
                    let direction = if dist > 0.1 {
                        Vec3::new(dx, 0.0, dz)
                    } else {
                        Vec3::X
                    };
                    commands.entity(entity).insert(Knockback::new(
                        direction,
                        LEAP_KNOCKBACK_SPEED,
                        LEAP_KNOCKBACK_DURATION,
                    ));
                }
            }
        }
    }
}

/// Josephina's frenzy — 5x attack speed after leap landing for MAULING_DURATION.
pub fn josephina_vicious_mauling(
    time: Res<Time>,
    death_tracker: Res<HagDeathTracker>,
    mut josephina_query: Query<
        (
            &HagIdentity,
            &HagEyeState,
            &LeapState,
            &mut MaulingState,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
) {
    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (identity, eye_state, leap, mut mauling) in &mut josephina_query {
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            mauling.frenzy_timer = 0.0;
            continue;
        }

        // Activate frenzy on leap landing
        if matches!(leap, LeapState::Landing { .. }) && !mauling.is_frenzied() {
            mauling.frenzy_timer = MAULING_DURATION;
        }

        // Tick down frenzy timer
        if mauling.is_frenzied() {
            mauling.frenzy_timer = (mauling.frenzy_timer - delta).max(0.0);
        }
    }
}

/// Josephina's corpse consume — stationary for 3s near a corpse, heals, despawns corpse.
pub fn josephina_corpse_consume(
    time: Res<Time>,
    mut commands: Commands,
    death_tracker: Res<HagDeathTracker>,
    mut josephina_query: Query<
        (
            Entity,
            &Transform,
            &HagIdentity,
            &HagEyeState,
            &mut Health,
            Option<&mut CorpseConsumeState>,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    corpses: Query<(Entity, &Transform), With<Corpse>>,
) {
    let delta = time.delta_secs();
    let enraged = death_tracker.permanent_deaths >= 2;

    for (entity, transform, identity, eye_state, mut health, consume_state) in
        &mut josephina_query
    {
        if *identity != HagIdentity::Josephina || (!eye_state.has_ability_eye && !enraged) {
            // Cancel consume if eye lost
            if consume_state.is_some() {
                commands.entity(entity).remove::<CorpseConsumeState>();
            }
            continue;
        }

        if let Some(mut state) = consume_state {
            // Currently consuming a corpse
            state.timer -= delta;
            if state.timer <= 0.0 {
                // Heal and despawn corpse
                let heal = health.max * CORPSE_CONSUME_HEAL_PERCENT;
                health.current = (health.current + heal).min(health.max);
                commands.entity(state.corpse_entity).try_despawn();
                commands.entity(entity).remove::<CorpseConsumeState>();
            }
        } else {
            // Check if there's a nearby corpse to consume (only if health not full)
            if health.current >= health.max * CORPSE_CONSUME_HEALTH_THRESHOLD {
                continue;
            }

            let hag_pos = transform.translation;
            let mut nearest_corpse: Option<(Entity, f32)> = None;

            for (corpse_entity, corpse_transform) in &corpses {
                let dx = corpse_transform.translation.x - hag_pos.x;
                let dz = corpse_transform.translation.z - hag_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist < CORPSE_CONSUME_RANGE {
                    if let Some((_, best_dist)) = nearest_corpse {
                        if dist < best_dist {
                            nearest_corpse = Some((corpse_entity, dist));
                        }
                    } else {
                        nearest_corpse = Some((corpse_entity, dist));
                    }
                }
            }

            if let Some((corpse_entity, _)) = nearest_corpse {
                commands.entity(entity).insert(CorpseConsumeState {
                    timer: CORPSE_CONSUME_DURATION,
                    corpse_entity,
                });
            }
        }
    }
}

// ===== Phase 5: Martina Abilities =====

/// Martina's teleport pull — teleports random defenders to her position.
/// King and guards move as a group.
#[allow(clippy::type_complexity)]
pub fn martina_teleport_pull(
    time: Res<Time>,
    death_tracker: Res<HagDeathTracker>,
    mut martina_query: Query<
        (
            &Transform,
            &Team,
            &HagIdentity,
            &HagEyeState,
            &mut TeleportPullCooldown,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    mut defenders: Query<
        (Entity, &mut Transform, &Team, Option<&King>, Option<&KingsGuard>),
        (Without<Hag>, Without<Corpse>, Without<MindControlled>, Without<Wizard>),
    >,
) {
    let delta = time.delta_secs();

    let enraged = death_tracker.permanent_deaths >= 2;

    for (transform, _team, identity, eye_state, mut cooldown) in &mut martina_query {
        if *identity != HagIdentity::Martina || (!eye_state.has_ability_eye && !enraged) {
            continue;
        }

        cooldown.time_remaining -= delta;
        if cooldown.time_remaining > 0.0 {
            continue;
        }
        cooldown.time_remaining = TELEPORT_PULL_COOLDOWN;

        let pull_pos = transform.translation;

        // Collect defender entities (excluding king and guards for special handling)
        let mut regular_defenders: Vec<Entity> = Vec::new();
        let mut king_entity: Option<Entity> = None;
        let mut guard_entities: Vec<Entity> = Vec::new();

        for (entity, _, team, king, guard) in &defenders {
            if *team != Team::Defenders {
                continue;
            }
            if king.is_some() {
                king_entity = Some(entity);
            } else if guard.is_some() {
                guard_entities.push(entity);
            } else {
                regular_defenders.push(entity);
            }
        }

        let mut rng = rand::thread_rng();
        let mut pulled = 0u32;

        // Pull random regular defenders first
        while pulled < TELEPORT_PULL_COUNT && !regular_defenders.is_empty() {
            let idx = rng.gen_range(0..regular_defenders.len());
            let entity = regular_defenders.swap_remove(idx);

            if let Ok((_, mut def_transform, _, _, _)) = defenders.get_mut(entity) {
                def_transform.translation.x = pull_pos.x + rng.gen_range(-20.0..20.0);
                def_transform.translation.z = pull_pos.z + rng.gen_range(-20.0..20.0);
            }
            pulled += 1;
        }

        // If we haven't pulled enough and king is available, pull king + all guards as a group
        if pulled < TELEPORT_PULL_COUNT
            && let Some(king_e) = king_entity
            && let Ok((_, mut king_transform, _, _, _)) = defenders.get_mut(king_e)
        {
            king_transform.translation.x = pull_pos.x + rng.gen_range(-20.0..20.0);
            king_transform.translation.z = pull_pos.z + rng.gen_range(-20.0..20.0);
            // Guards will snap to king via their existing system
        }
    }
}

/// Martina's mind control aura — instantly mind controls any defender inside the radius.
pub fn martina_mind_control(
    mut commands: Commands,
    martina_query: Query<
        (
            &Transform,
            &HagIdentity,
        ),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    defenders: Query<
        (Entity, &Transform, &Team, &FlowFieldInfluence),
        (Without<Hag>, Without<Corpse>, Without<MindControlled>, Without<Wizard>),
    >,
    existing_controlled: Query<&MindControlled, Without<Corpse>>,
) {
    let controlled_count = existing_controlled.iter().count() as u32;
    if controlled_count >= MIND_CONTROL_MAX_CONTROLLED {
        return;
    }

    for (transform, identity) in &martina_query {
        if *identity != HagIdentity::Martina {
            continue;
        }

        let hag_pos = transform.translation;

        for (entity, def_transform, def_team, flow_influence) in &defenders {
            if *def_team != Team::Defenders {
                continue;
            }

            let dx = def_transform.translation.x - hag_pos.x;
            let dz = def_transform.translation.z - hag_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist <= MIND_CONTROL_AURA_RADIUS {
                let original_spawn_pos = match flow_influence {
                    FlowFieldInfluence::Defender { spawn_pos } => Some(*spawn_pos),
                    _ => None,
                };

                commands.entity(entity).insert((
                    MindControlled {
                        time_elapsed: 0.0,
                        wear_off_duration: 300.0, // 5 minutes
                        original_spawn_pos,
                    },
                    FlowFieldInfluence::Attacker,
                ));
            }
        }
    }
}

/// Updates mind-controlled units — they target non-MC same-team allies.
pub fn update_mind_controlled_targeting(
    mut controlled: Query<(Entity, &Transform, &Team, &mut TargetingVelocity, &MindControlled)>,
    all_units: Query<(Entity, &Transform, &Team), (Without<Corpse>, Without<MindControlled>)>,
) {
    for (entity, transform, team, mut targeting, _mc) in &mut controlled {
        // Find nearest ALLY to attack (reversed targeting)
        let pos = transform.translation;
        let mut nearest: Option<(f32, Vec3)> = None;

        for (other_entity, other_transform, other_team) in &all_units {
            if other_entity == entity {
                continue;
            }
            // Target same team (allies become enemies)
            if other_team != team {
                continue;
            }

            let dx = other_transform.translation.x - pos.x;
            let dz = other_transform.translation.z - pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if let Some((best_dist, _)) = nearest {
                if dist < best_dist {
                    nearest = Some((dist, other_transform.translation));
                }
            } else {
                nearest = Some((dist, other_transform.translation));
            }
        }

        if let Some((_, target_pos)) = nearest {
            let dir = Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z).normalize_or_zero();
            targeting.velocity = dir;
        } else {
            targeting.velocity = Vec3::ZERO;
        }
    }
}

/// Updates mind control wear-off timer — removes when duration expires.
/// Also cleans up RetaliationTarget components that point at freed entities.
pub fn update_mind_control_wear_off(
    time: Res<Time>,
    mut commands: Commands,
    mut controlled: Query<(Entity, &mut MindControlled)>,
    retaliators: Query<(Entity, &RetaliationTarget)>,
) {
    let delta = time.delta_secs();

    for (entity, mut mc) in &mut controlled {
        mc.time_elapsed += delta;

        if mc.time_elapsed >= mc.wear_off_duration {
            // Restore original flow field influence before removing mind control
            if let Some(spawn_pos) = mc.original_spawn_pos {
                commands
                    .entity(entity)
                    .insert(FlowFieldInfluence::Defender { spawn_pos });
            }

            commands.entity(entity).remove::<MindControlled>();

            // Remove RetaliationTarget from any units retaliating against this entity
            for (retaliator_entity, retaliation) in &retaliators {
                if retaliation.0 == entity {
                    commands
                        .entity(retaliator_entity)
                        .remove::<RetaliationTarget>();
                }
            }
        }
    }
}

/// Mind-controlled units attack their own team (gated by global attack cycle).
/// They skip other mind-controlled units and only attack non-MC allies.
pub fn mind_controlled_combat(
    attack_cycle: Res<crate::game::plugin::GlobalAttackCycle>,
    mut commands: Commands,
    mut controlled: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut AttackTiming, &MindControlled),
        Without<Corpse>,
    >,
    mut potential_targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Corpse>, Without<MindControlled>),
    >,
) {
    let current_time = attack_cycle.current_time;
    let last_time = current_time - 0.016; // approximate last frame

    for (mc_entity, mc_transform, mc_hitbox, mc_team, mut timing, _mc) in &mut controlled {
        if !timing.can_attack(current_time, last_time) {
            continue;
        }

        let mc_pos = mc_transform.translation;

        // Find nearest same-team unit to attack (mind controlled = attacks allies)
        for (entity, target_transform, target_hitbox, target_team, mut health, mut temp_hp) in
            &mut potential_targets
        {
            if entity == mc_entity {
                continue;
            }
            // Attack same team (reversed)
            if target_team != mc_team {
                continue;
            }

            let dx = target_transform.translation.x - mc_pos.x;
            let dz = target_transform.translation.z - mc_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            let attack_range =
                (mc_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

            if dist <= attack_range {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), MIND_CONTROL_COMBAT_DAMAGE);
                timing.last_attack_time = Some(current_time);

                // Victim retaliates — consider the MC attacker a valid target
                commands
                    .entity(entity)
                    .insert(RetaliationTarget(mc_entity));

                break; // One attack per cycle
            }
        }
    }
}

/// Cleans up RetaliationTarget when the target entity is dead or no longer mind-controlled.
pub fn cleanup_retaliation_targets(
    mut commands: Commands,
    retaliators: Query<(Entity, &RetaliationTarget)>,
    mc_units: Query<Entity, With<MindControlled>>,
) {
    for (entity, retaliation) in &retaliators {
        // Remove if the retaliation target is no longer mind-controlled (or despawned)
        if mc_units.get(retaliation.0).is_err() {
            commands.entity(entity).remove::<RetaliationTarget>();
        }
    }
}
