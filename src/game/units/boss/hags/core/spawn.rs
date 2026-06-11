use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::{EyeTransferTimer, HagAssets, HagDeathTracker};
use super::animation::{eye_pulsing_animation, hag_walking_animation};
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    AttackTiming, CommanderAuraSpeedModifier, DamageMultiplier, Effectiveness, FacingDirection,
    FacingDwell, FacingHysteresisBoost, FlockingModifier, FlockingVelocity, Health, Hitbox,
    Invulnerable, MovementSpeed, RoughTerrainModifier, SmoothedFacingVelocity, TargetingVelocity,
    Team, Teleportable,
};
use crate::game::units::random_position_in_cell;

/// Spawns a floating eye visual as a child of the given hag entity.
pub(crate) fn spawn_eye_visual(
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
