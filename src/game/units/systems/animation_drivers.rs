use bevy::math::Affine2;
use bevy::prelude::*;

use super::super::components::{
    ANIMATION_MOVE_THRESHOLD_SQ, AnimationOverride, CombatAnimation, Corpse,
    DeathAnimationFinished, DyingAnimation, FacingDirection, Knockback, PolymorphedModifier,
    PulsingAnimation, RisingAnimation, RoughTerrain, Team, WalkingAnimation,
};
use crate::game::components::Velocity;

/// Advances walking animation frames and updates UV transforms.
/// Skips entities with active combat or dying animations.
#[allow(clippy::type_complexity)]
pub fn update_walking_animation(
    time: Res<Time>,
    mut anim_query: Query<
        (
            &mut WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
            &Velocity,
            &FacingDirection,
        ),
        (
            Without<Corpse>,
            Without<CombatAnimation>,
            Without<DyingAnimation>,
            Without<RisingAnimation>,
            Without<AnimationOverride>,
            Without<PolymorphedModifier>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (mut anim, material_handle, velocity, facing) in &mut anim_query {
        let speed_sq = Vec3::new(velocity.x, 0.0, velocity.z).length_squared();

        // Stationary: reset to frame 0
        if speed_sq < ANIMATION_MOVE_THRESHOLD_SQ {
            if anim.current_frame != 0 {
                anim.current_frame = 0;
                anim.elapsed = 0.0;
                if let Some(mut mat) = materials.get_mut(material_handle) {
                    mat.uv_transform = anim.uv_transform(*facing);
                }
            }
            continue;
        }

        // Advance animation
        if anim.tick(delta)
            && let Some(mut mat) = materials.get_mut(material_handle)
        {
            mat.uv_transform = anim.uv_transform(*facing);
        }
    }
}

/// Advances looping in-place pulsing animations and updates UV transforms.
/// Used for non-moving entities (eyes, idle props) that don't need facing direction.
pub fn update_pulsing_animation(
    time: Res<Time>,
    mut anim_query: Query<(&mut PulsingAnimation, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (mut anim, material_handle) in &mut anim_query {
        if anim.tick(delta)
            && let Some(mut mat) = materials.get_mut(material_handle)
        {
            mat.uv_transform = anim.uv_transform();
        }
    }
}

/// Advances one-shot combat animations (melee attack, ranged shooting).
/// Swaps texture to the combat sheet on start, restores walking sheet when finished.
pub fn update_combat_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut anim_query: Query<
        (
            Entity,
            &mut CombatAnimation,
            &MeshMaterial3d<StandardMaterial>,
            &FacingDirection,
            Option<&WalkingAnimation>,
        ),
        (
            Without<Corpse>,
            Without<RisingAnimation>,
            Without<PolymorphedModifier>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, mut anim, material_handle, facing, walking_anim) in &mut anim_query {
        let Some(mut mat) = materials.get_mut(material_handle) else {
            continue;
        };

        // First frame: swap texture to combat sheet
        if !anim.started {
            anim.started = true;
            mat.base_color_texture = Some(anim.combat_texture.clone());
            mat.uv_transform = anim.uv_transform(*facing);
        }

        // Advance animation
        if anim.tick(delta) {
            if anim.finished() {
                // Restore walking texture and remove component
                mat.base_color_texture = Some(anim.walking_texture.clone());
                // Use the entity's own walking animation UV if available,
                // falling back to the default for standard infantry-sized sheets.
                mat.uv_transform = if let Some(walk) = walking_anim {
                    walk.uv_transform(*facing)
                } else {
                    WalkingAnimation::idle_uv_transform(*facing)
                };
                commands.entity(entity).remove::<CombatAnimation>();
            } else {
                mat.uv_transform = anim.uv_transform(*facing);
            }
        }
    }
}

/// Advances death animations and updates UV transforms.
/// When the animation finishes, inserts `DeathAnimationFinished` marker.
pub fn update_dying_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut anim_query: Query<
        (
            Entity,
            &mut DyingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        Without<DeathAnimationFinished>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, mut anim, material_handle) in &mut anim_query {
        let Some(mut mat) = materials.get_mut(material_handle) else {
            continue;
        };

        // First frame: swap texture to death sheet
        if !anim.started {
            anim.started = true;
            mat.base_color_texture = Some(anim.death_texture.clone());
            mat.uv_transform = anim.uv_transform();
        }

        // Advance animation
        if anim.tick(delta) {
            mat.uv_transform = anim.uv_transform();
            if anim.finished() {
                commands.entity(entity).insert(DeathAnimationFinished);
            }
        }
    }
}

/// Advances rising animations (death sheet played in reverse) and swaps the
/// material back to the walking sprite when the animation finishes.
pub fn update_rising_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut anim_query: Query<(
        Entity,
        &mut RisingAnimation,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, mut anim, material_handle) in &mut anim_query {
        let Some(mut mat) = materials.get_mut(material_handle) else {
            continue;
        };

        if !anim.started {
            anim.started = true;
            mat.base_color_texture = Some(anim.death_texture.clone());
            mat.uv_transform = anim.uv_transform();
        }

        if anim.tick(delta) {
            if anim.finished() {
                // Swap back to the walking sprite and reset UV so the walking
                // animation system picks up cleanly from frame 0.
                mat.base_color_texture = Some(anim.walking_texture.clone());
                mat.uv_transform =
                    Affine2::from_scale_angle_translation(anim.frame_uv, 0.0, Vec2::ZERO);
                commands.entity(entity).remove::<RisingAnimation>();
            } else {
                mat.uv_transform = anim.uv_transform();
            }
        }
    }
}

/// Finalizes dying entities whose death animation has completed.
/// Lays the corpse flat, applies corpse tint, and adds rough terrain.
pub fn finalize_dying_to_corpse(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &DyingAnimation,
            &Transform,
            &Team,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<DeathAnimationFinished>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, anim, transform, team, material_handle) in &query {
        // Apply corpse tint and switch to Blend for semi-transparent rendering
        if let Some(mut mat) = materials.get_mut(material_handle) {
            use crate::game::constants::{
                ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR,
            };
            mat.base_color = match *team {
                Team::Defenders => DEFENDER_CORPSE_COLOR,
                Team::Attackers => ATTACKER_CORPSE_COLOR,
                Team::Undead => UNDEAD_CORPSE_COLOR,
            };
            mat.uv_transform = anim.last_frame_uv_transform();
            mat.alpha_mode = AlphaMode::Blend;
        }

        let mut entity_commands = commands.entity(entity);
        lay_corpse_flat(&mut entity_commands, transform.translation);
        entity_commands
            .remove::<DyingAnimation>()
            .remove::<DeathAnimationFinished>();
    }
}

/// Lays a corpse entity flat on the ground. Shared between instant corpse swap
/// and death animation finalization.
pub fn lay_corpse_flat(entity_commands: &mut EntityCommands, position: Vec3) {
    let corpse_transform = Transform::from_xyz(position.x, 1.0, position.z)
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

    entity_commands
        .insert(corpse_transform)
        .insert(RoughTerrain {
            slowdown_factor: 0.4,
        })
        .remove::<crate::game::components::Billboard>();
}

/// Ticks all knockback effects, applying decaying position offsets each frame.
/// Units tumble outward and gradually slow down. Removes the component when expired.
pub fn apply_knockback_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut units: Query<(Entity, &mut Transform, &mut Knockback)>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, mut knockback) in &mut units {
        knockback.remaining -= delta;

        if knockback.remaining <= 0.0 {
            commands.entity(entity).remove::<Knockback>();
            continue;
        }

        // Linear decay: full speed at start, zero at end
        let decay = knockback.remaining / knockback.duration;
        let speed = knockback.speed * decay;

        transform.translation.x += knockback.direction_x * speed * delta;
        transform.translation.z += knockback.direction_z * speed * delta;
    }
}
