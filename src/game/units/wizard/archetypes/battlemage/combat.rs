//! Battlemage combat: state, movement, and weapon firing.

use super::components::SwordArcMaterial;
use super::components::*;
use super::constants::*;
use super::messages::*;
use super::resources::{BattlemageAssets, BattlemagePhase, BattlemageState};
use crate::config::GameConfig;
use crate::config::input_bindings::InputBindings;
use crate::game::components::{OnGameplayScreen, Velocity};
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::messages::BlockSpellInput;
use crate::game::units::components::{
    Corpse, Health, MovementSpeed, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::components::{Mana, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Initialize or reset the battlemage state resource and cached assets.
pub(super) fn reset_battlemage_state(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(BattlemageState::default());
    commands.insert_resource(SwordArcMaterial(materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.6),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    })));
}

/// Blocks normal spell casting while the battlemage is on the field.
pub(super) fn block_spells_on_field(
    state: Res<BattlemageState>,
    mut block_spell: MessageWriter<BlockSpellInput>,
) {
    if state.phase != BattlemagePhase::Idle {
        block_spell.write(BlockSpellInput);
    }
}

/// Handles the retreat message — despawns avatar and restores wizard.
pub(super) fn handle_retreat(
    mut messages: MessageReader<RetreatMessage>,
    mut state: ResMut<BattlemageState>,
    mut commands: Commands,
    avatar_query: Query<Entity, With<BattlemageAvatar>>,
    mut wizard_query: Query<&mut Visibility, With<Wizard>>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    for _ in messages.read() {
        if state.phase != BattlemagePhase::OnField {
            return;
        }
        // Despawn the avatar
        for entity in &avatar_query {
            commands.entity(entity).try_despawn();
        }
        // Play teleport sound and restore wizard visibility
        audio::play_sfx(
            &mut commands,
            &sfx.teleport_cast,
            WIZARD_POSITION,
            &config,
            &sfx,
        );
        if let Ok(mut visibility) = wizard_query.single_mut() {
            *visibility = Visibility::Inherited;
        }
        state.phase = BattlemagePhase::Idle;
        state.retreated = true;
    }
}

/// Handles WASD / left-stick movement for the battlemage avatar.
/// Also updates the avatar's `BattlemageFacing` so spells aim along the
/// current movement direction (no separate aim input for this archetype).
#[allow(clippy::too_many_arguments)]
pub(super) fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    active_device: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    aim_settings: Res<crate::game::input::gamepad::resources::GamepadAimSettings>,
    mut commands: Commands,
    mut avatar_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &MovementSpeed,
            Option<&mut BattlemageFacing>,
        ),
        With<BattlemageAvatar>,
    >,
    state: Res<BattlemageState>,
) {
    if state.phase != BattlemagePhase::OnField {
        return;
    }

    let Ok((avatar_entity, mut transform, mut velocity, speed, maybe_facing)) =
        avatar_query.single_mut()
    else {
        return;
    };

    let mut input = Vec2::ZERO;
    if let Some(key) = bindings.battlemage.move_forward
        && keyboard.pressed(key)
    {
        input.y -= 1.0; // -Z is "forward" toward the battlefield
    }
    if let Some(key) = bindings.battlemage.move_backward
        && keyboard.pressed(key)
    {
        input.y += 1.0;
    }
    if let Some(key) = bindings.battlemage.move_left
        && keyboard.pressed(key)
    {
        input.x -= 1.0;
    }
    if let Some(key) = bindings.battlemage.move_right
        && keyboard.pressed(key)
    {
        input.x += 1.0;
    }

    // Gamepad: left stick overrides keyboard when active and deflected.
    if let Some(gamepad_entity) = active_device.gamepad_entity()
        && let Ok(gamepad) = gamepads.get(gamepad_entity)
    {
        let lx = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let ly = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        // Negate Y so stick-up is -Z (forward toward the battlefield).
        let shaped = crate::game::input::gamepad::systems::apply_deadzone_and_curve(
            Vec2::new(lx, -ly),
            aim_settings.deadzone,
            aim_settings.response_curve,
        );
        if shaped != Vec2::ZERO {
            input = shaped;
        }
    }

    let input_normalized = if input.length_squared() > 0.0 {
        input.normalize()
    } else {
        Vec2::ZERO
    };

    if input_normalized.length_squared() > 0.0 {
        let facing = BattlemageFacing(input_normalized);
        match maybe_facing {
            Some(mut f) => *f = facing,
            None => {
                commands.entity(avatar_entity).insert(facing);
            }
        }
    }

    let dt = time.delta_secs();
    let max_speed = speed.0;

    // Apply acceleration from input
    velocity.x += input_normalized.x * PLAYER_ACCELERATION * dt;
    velocity.z += input_normalized.y * PLAYER_ACCELERATION * dt;

    // Apply damping
    let damping = PLAYER_DAMPING.powf(dt * 60.0);
    velocity.x *= damping;
    velocity.z *= damping;

    // Clamp to max speed
    let current_speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
    if current_speed > max_speed {
        let scale = max_speed / current_speed;
        velocity.x *= scale;
        velocity.z *= scale;
    }

    // Apply velocity to position
    transform.translation.x += velocity.x * dt;
    transform.translation.z += velocity.z * dt;

    // Clamp to battlefield bounds
    let half_field = crate::game::constants::BATTLEFIELD_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_field, half_field);
    transform.translation.z = transform.translation.z.clamp(-half_field, half_field);
}

/// Handles RT / left-click magic missile firing from the battlemage avatar.
///
/// Fires in the avatar's current facing direction (most recent movement vector).
/// There is no separate aim input — movement direction IS attack direction.
#[allow(clippy::too_many_arguments)]
pub(super) fn fire_missile(
    mut mouse_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut avatar_query: Query<
        (
            Entity,
            &Transform,
            Option<&BattlemageMissileCooldown>,
            Option<&BattlemageFacing>,
        ),
        With<BattlemageAvatar>,
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<BattlemageAvatar>, Without<Corpse>)>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    state: Res<BattlemageState>,
    mut wizard_query: Query<&mut Mana, With<Wizard>>,
    battlemage_assets: Res<BattlemageAssets>,
) {
    if state.phase != BattlemagePhase::OnField {
        return;
    }
    if mouse_pressed.read().next().is_none() {
        return;
    }

    let Ok((avatar_entity, avatar_transform, cooldown, facing)) = avatar_query.single_mut() else {
        return;
    };

    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

    let Ok(mut mana) = wizard_query.single_mut() else {
        return;
    };
    if !mana.consume(MISSILE_MANA_COST) {
        return;
    }

    let spawn_pos = avatar_transform.translation + Vec3::new(0.0, 30.0, 0.0);

    let facing_vec = facing.copied().unwrap_or_default().0;
    let direction = Vec3::new(facing_vec.x, 0.0, facing_vec.y).normalize_or_zero();

    let initial_velocity = direction * MISSILE_SPEED;

    // Find nearest enemy target
    let target = targets
        .iter()
        .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
        .min_by(|a, b| {
            let dist_a = spawn_pos.distance(a.1.translation);
            let dist_b = spawn_pos.distance(b.1.translation);
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, _, _)| e);

    use crate::game::units::wizard::spells::magic_missile::components::{
        MagicMissile, TargetTeams,
    };

    let mut missile = MagicMissile::new(
        initial_velocity,
        0.0,
        target,
        1.0,
        TargetTeams::AttackersAndUndead,
        3000.0,
        spawn_pos,
    );
    missile.damage = MISSILE_DAMAGE;

    commands.spawn((
        Mesh3d(visual_assets.magic_missile_mesh.clone()),
        MeshMaterial3d(visual_assets.magic_missile.clone()),
        Transform::from_translation(spawn_pos),
        missile,
        OnGameplayScreen,
    ));

    audio::play_sfx(
        &mut commands,
        &sfx.magic_missile_cast,
        spawn_pos,
        &config,
        &sfx,
    );

    // Trigger casting animation
    commands.entity(avatar_entity).insert(
        crate::game::units::components::CombatAnimation::new_casting(
            battlemage_assets.casting_texture.clone(),
            battlemage_assets.sprite_texture.clone(),
        ),
    );

    commands
        .entity(avatar_entity)
        .insert(BattlemageMissileCooldown {
            remaining: MISSILE_COOLDOWN,
        });
}

/// Handles LT / right-click sword swing from the battlemage avatar.
///
/// Swings in the avatar's current facing direction (most recent movement vector).
#[allow(clippy::too_many_arguments)]
pub(super) fn sword_swing(
    mut mouse_right_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    arc_material: Res<SwordArcMaterial>,
    mut avatar_query: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            Option<&BattlemageSwordCooldown>,
            Option<&BattlemageFacing>,
        ),
        With<BattlemageAvatar>,
    >,
    state: Res<BattlemageState>,
    battlemage_assets: Res<BattlemageAssets>,
) {
    if state.phase != BattlemagePhase::OnField {
        return;
    }
    if mouse_right_pressed.read().next().is_none() {
        return;
    }

    let Ok((avatar_entity, avatar_transform, mut velocity, cooldown, facing)) =
        avatar_query.single_mut()
    else {
        return;
    };

    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

    let avatar_pos = avatar_transform.translation;

    let direction = facing.copied().unwrap_or_default().0.normalize_or_zero();

    // Build a semicircle mesh (fan of triangles covering ±90° from swing direction)
    let segments = 16u32;
    let half_angle = std::f32::consts::FRAC_PI_2; // 90 degrees each side
    let base_angle = direction.y.atan2(direction.x); // angle of swing direction in XZ

    let mut positions = vec![[0.0f32, 0.0, 0.0]]; // center vertex
    let mut normals = vec![[0.0f32, 1.0, 0.0]];
    let mut uvs = vec![[0.5f32, 0.5]];
    for i in 0..=segments {
        let frac = i as f32 / segments as f32;
        let angle = base_angle - half_angle + frac * 2.0 * half_angle;
        let x = SWORD_ARC_RADIUS * angle.cos();
        let z = SWORD_ARC_RADIUS * angle.sin();
        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }
    let mut indices = Vec::new();
    for i in 0..segments {
        indices.push(0);
        indices.push(i + 1);
        indices.push(i + 2);
    }

    use bevy::mesh::{Indices, PrimitiveTopology};
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    let arc_mesh = meshes.add(mesh);

    // Position the arc centered on the avatar
    let arc_pos = Vec3::new(avatar_pos.x, 2.0, avatar_pos.z);

    commands.spawn((
        Mesh3d(arc_mesh),
        MeshMaterial3d(arc_material.0.clone()),
        Transform::from_translation(arc_pos),
        SwordArc {
            time_alive: 0.0,
            duration: SWORD_ARC_DURATION,
            direction,
            damage_dealt: false,
        },
        OnGameplayScreen,
    ));

    // Lunge the avatar toward the cursor via velocity impulse
    velocity.x += direction.x * SWORD_LUNGE_SPEED;
    velocity.z += direction.y * SWORD_LUNGE_SPEED;

    // Trigger attack animation
    commands.entity(avatar_entity).insert(
        crate::game::units::components::CombatAnimation::new_attack(
            battlemage_assets.attacking_texture.clone(),
            battlemage_assets.sprite_texture.clone(),
        ),
    );

    commands
        .entity(avatar_entity)
        .insert(BattlemageSwordCooldown {
            remaining: SWORD_COOLDOWN,
        });
}

/// Updates sword arc visuals and checks collisions with enemies.
pub(super) fn update_sword_arcs(
    time: Res<Time>,
    mut commands: Commands,
    mut arc_query: Query<(Entity, &mut SwordArc, &Transform)>,
    mut enemies: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &Team,
        ),
        (Without<SwordArc>, Without<Corpse>),
    >,
) {
    for (arc_entity, mut arc, arc_transform) in &mut arc_query {
        arc.time_alive += time.delta_secs();

        // Deal damage on the first frame
        if !arc.damage_dealt {
            arc.damage_dealt = true;
            let arc_pos = arc_transform.translation;

            for (enemy_entity, enemy_transform, mut health, temp_hp, team) in &mut enemies {
                if *team != Team::Attackers && *team != Team::Undead {
                    continue;
                }

                let diff = enemy_transform.translation - arc_pos;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();

                if dist > SWORD_ARC_RADIUS {
                    continue;
                }

                // Check if enemy is within the arc's angular sweep
                let enemy_angle = Vec2::new(diff.x, diff.z).normalize_or_zero();
                let dot = arc.direction.dot(enemy_angle);
                // cos(60°) = 0.5 → within ±60° of swing direction
                if dot > 0.5 {
                    apply_spell_damage(
                        &mut commands,
                        enemy_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        SWORD_DAMAGE,
                        DamageType::Force,
                        false,
                    );
                }
            }
        }

        // Despawn after duration
        if arc.time_alive >= arc.duration {
            commands.entity(arc_entity).try_despawn();
        }
    }
}
