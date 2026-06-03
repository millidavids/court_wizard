//! Swordcerer combat: state, movement, and weapon firing.

use super::components::*;
use super::constants::*;
use super::messages::*;
use super::resources::{SwordcererAssets, SwordcererPhase, SwordcererState};
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

/// Initialize or reset the swordcerer state resource.
pub(super) fn reset_swordcerer_state(mut commands: Commands) {
    commands.insert_resource(SwordcererState::default());
}

/// Blocks normal spell casting while the swordcerer is on the field.
pub(super) fn block_spells_on_field(
    state: Res<SwordcererState>,
    mut block_spell: MessageWriter<BlockSpellInput>,
) {
    if state.phase != SwordcererPhase::Idle {
        block_spell.write(BlockSpellInput);
    }
}

/// Handles the retreat message — despawns avatar and restores wizard.
pub(super) fn handle_retreat(
    mut messages: MessageReader<RetreatMessage>,
    mut state: ResMut<SwordcererState>,
    mut commands: Commands,
    avatar_query: Query<Entity, With<SwordcererAvatar>>,
    mut wizard_query: Query<&mut Visibility, With<Wizard>>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    for _ in messages.read() {
        if state.phase != SwordcererPhase::OnField {
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
        state.phase = SwordcererPhase::Idle;
        state.retreated = true;
    }
}

/// Handles WASD / left-stick movement for the swordcerer avatar.
/// Also updates the avatar's `SwordcererFacing` so spells aim along the
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
            Option<&mut SwordcererFacing>,
        ),
        (With<SwordcererAvatar>, Without<GuestControlledAvatar>),
    >,
    state: Res<SwordcererState>,
) {
    if state.phase != SwordcererPhase::OnField {
        return;
    }

    let Ok((avatar_entity, mut transform, mut velocity, speed, maybe_facing)) =
        avatar_query.single_mut()
    else {
        return;
    };

    let mut input = Vec2::ZERO;
    if let Some(key) = bindings.swordcerer.move_forward
        && keyboard.pressed(key)
    {
        input.y -= 1.0; // -Z is "forward" toward the battlefield
    }
    if let Some(key) = bindings.swordcerer.move_backward
        && keyboard.pressed(key)
    {
        input.y += 1.0;
    }
    if let Some(key) = bindings.swordcerer.move_left
        && keyboard.pressed(key)
    {
        input.x -= 1.0;
    }
    if let Some(key) = bindings.swordcerer.move_right
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
        let facing = SwordcererFacing(input_normalized);
        match maybe_facing {
            Some(mut f) => *f = facing,
            None => {
                commands.entity(avatar_entity).insert(facing);
            }
        }
    }

    apply_avatar_physics(
        &mut transform,
        &mut velocity,
        input_normalized,
        speed.0,
        time.delta_secs(),
    );
}

/// Handles LT / right-click magic missile firing from the swordcerer avatar.
///
/// Fires in the avatar's current facing direction (most recent movement vector).
/// There is no separate aim input — movement direction IS attack direction.
#[allow(clippy::too_many_arguments)]
pub(super) fn fire_missile(
    mut mouse_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut avatar_query: Query<
        (
            Entity,
            &Transform,
            Option<&SwordcererMissileCooldown>,
            Option<&SwordcererFacing>,
        ),
        (With<SwordcererAvatar>, Without<GuestControlledAvatar>),
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<SwordcererAvatar>, Without<Corpse>)>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    state: Res<SwordcererState>,
    mut wizard_query: Query<&mut Mana, With<Wizard>>,
    swordcerer_assets: Res<SwordcererAssets>,
) {
    if state.phase != SwordcererPhase::OnField {
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

    let facing_vec = facing.copied().unwrap_or_default().0;
    let direction = Vec3::new(facing_vec.x, 0.0, facing_vec.y).normalize_or_zero();

    // The host's own avatar is always on the Defenders team.
    spawn_avatar_missile(
        &mut commands,
        &visual_assets,
        &sfx,
        &config,
        &swordcerer_assets,
        &targets,
        avatar_entity,
        avatar_transform.translation,
        direction,
        Team::Defenders,
    );
}

/// Handles RT / left-click sword swing from the swordcerer avatar.
///
/// Swings in the avatar's current facing direction (most recent movement vector).
#[allow(clippy::too_many_arguments)]
pub(super) fn sword_swing(
    mut mouse_right_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut avatar_query: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            Option<&SwordcererSwordCooldown>,
            Option<&SwordcererFacing>,
        ),
        (With<SwordcererAvatar>, Without<GuestControlledAvatar>),
    >,
    state: Res<SwordcererState>,
    swordcerer_assets: Res<SwordcererAssets>,
    mut pending: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    if state.phase != SwordcererPhase::OnField {
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

    let arc_pos = Vec3::new(avatar_pos.x, 2.0, avatar_pos.z);

    spawn_sword_arc(
        &mut commands,
        &mut meshes,
        &mut materials,
        arc_pos,
        direction,
        false,
    );

    // Replicate the swing to the opponent (visual only — damage crosses via CRDT).
    crate::game::multiplayer::spell_sync::emit_cast_event(
        &mut pending,
        crate::networking::snapshot::CastEventKind::SwordArc,
        0,
        arc_pos,
        [direction.x, direction.y, 0.0, 0.0],
    );

    // Lunge the avatar toward the cursor via velocity impulse
    velocity.x += direction.x * SWORD_LUNGE_SPEED;
    velocity.z += direction.y * SWORD_LUNGE_SPEED;

    // Trigger attack animation
    commands.entity(avatar_entity).insert(
        crate::game::units::components::CombatAnimation::new_attack(
            swordcerer_assets.attacking_texture.clone(),
            swordcerer_assets.sprite_texture.clone(),
        ),
    );

    commands
        .entity(avatar_entity)
        .insert(SwordcererSwordCooldown {
            remaining: SWORD_COOLDOWN,
        });
}

/// Shared avatar movement physics — acceleration, damping, max-speed clamp, and
/// battlefield-bounds clamp. Used by both the host's own avatar
/// (`player_movement`) and the guest-controlled avatar
/// (`networking::apply_guest_avatar_input`) so they move identically.
pub(super) fn apply_avatar_physics(
    transform: &mut Transform,
    velocity: &mut Velocity,
    input_dir: Vec2,
    max_speed: f32,
    dt: f32,
) {
    velocity.x += input_dir.x * PLAYER_ACCELERATION * dt;
    velocity.z += input_dir.y * PLAYER_ACCELERATION * dt;

    let damping = PLAYER_DAMPING.powf(dt * 60.0);
    velocity.x *= damping;
    velocity.z *= damping;

    let current_speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
    if current_speed > max_speed {
        let scale = max_speed / current_speed;
        velocity.x *= scale;
        velocity.z *= scale;
    }

    transform.translation.x += velocity.x * dt;
    transform.translation.z += velocity.z * dt;

    let half_field = crate::game::constants::BATTLEFIELD_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_field, half_field);
    transform.translation.z = transform.translation.z.clamp(-half_field, half_field);
}

/// Spawns a homing magic missile from a Swordcerer avatar toward the nearest
/// enemy unit, plays the cast SFX, and starts the avatar's casting animation +
/// missile cooldown. Shared by the host's own avatar (`fire_missile`) and the
/// guest-controlled avatar (`networking::apply_guest_avatar_input`). The enemy
/// teams are derived from the avatar's own team, so a host (Defenders) avatar
/// targets Attackers/Undead and a guest (Attackers) avatar targets
/// Defenders/Undead. Any mana cost is handled by the caller.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_avatar_missile(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    sfx: &SpellSfxAssets,
    config: &GameConfig,
    swordcerer_assets: &SwordcererAssets,
    targets: &Query<(Entity, &Transform, &Team), (Without<SwordcererAvatar>, Without<Corpse>)>,
    avatar_entity: Entity,
    avatar_pos: Vec3,
    direction: Vec3,
    avatar_team: Team,
) {
    use crate::game::units::wizard::spells::magic_missile::components::{MagicMissile, TargetTeams};

    // Enemies are the opposing army plus the always-hostile Undead.
    let (target_teams, enemy_army) = match avatar_team {
        Team::Defenders => (TargetTeams::AttackersAndUndead, Team::Attackers),
        _ => (TargetTeams::DefendersAndUndead, Team::Defenders),
    };

    let spawn_pos = avatar_pos + Vec3::new(0.0, 30.0, 0.0);
    let target = targets
        .iter()
        .filter(|(_, _, team)| **team == enemy_army || **team == Team::Undead)
        .min_by(|a, b| {
            spawn_pos
                .distance(a.1.translation)
                .partial_cmp(&spawn_pos.distance(b.1.translation))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, _, _)| e);

    let mut missile = MagicMissile::new(
        direction * MISSILE_SPEED,
        0.0,
        target,
        1.0,
        target_teams,
        3000.0,
        spawn_pos,
    );
    missile.damage = MISSILE_DAMAGE;
    missile.base_homing_strength = MISSILE_HOMING_STRENGTH;

    commands.spawn((
        Mesh3d(visual_assets.magic_missile_mesh.clone()),
        MeshMaterial3d(visual_assets.magic_missile.clone()),
        Transform::from_translation(spawn_pos),
        missile,
        OnGameplayScreen,
    ));

    audio::play_sfx(commands, &sfx.magic_missile_cast, spawn_pos, config, sfx);

    commands.entity(avatar_entity).insert(
        crate::game::units::components::CombatAnimation::new_casting(
            swordcerer_assets.casting_texture.clone(),
            swordcerer_assets.sprite_texture.clone(),
        ),
    );

    commands
        .entity(avatar_entity)
        .insert(SwordcererMissileCooldown {
            remaining: MISSILE_COOLDOWN,
        });
}

/// Updates sword arc visuals (rapid grow + fade) and checks collisions with
/// enemies on the first frame.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_sword_arcs(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut arc_query: Query<(
        Entity,
        &mut SwordArc,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        Has<GhostSwordArc>,
    )>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<SwordArc>,
            Without<Corpse>,
            Without<SwordcererAvatar>,
            Without<Wizard>,
        ),
    >,
) {
    let dt = time.delta_secs();
    let cos_half_angle = SWORD_ARC_HALF_ANGLE.cos();
    for (arc_entity, mut arc, mut arc_transform, mat_handle, is_ghost) in &mut arc_query {
        // Ghost arcs (the opponent's replicated swing) are visual-only.
        let just_spawned = arc.is_added() && !is_ghost;
        arc.time_alive += dt;

        let grow_t = (arc.time_alive / SWORD_ARC_GROW_DURATION).clamp(0.0, 1.0);
        let grow_eased = 1.0 - (1.0 - grow_t).powi(3);
        arc_transform.scale = Vec3::splat(grow_eased.max(SWORD_ARC_MIN_SCALE));

        let fade_t = (arc.time_alive / arc.duration).clamp(0.0, 1.0);
        let alpha = (1.0 - fade_t).powi(2);
        let new_color = Color::srgba(1.0, 1.0, 1.0, alpha);
        if let Some(mat) = materials.get_mut(&mat_handle.0)
            && mat.base_color != new_color
        {
            mat.base_color = new_color;
        }

        // Friendly fire by design — `Without<SwordcererAvatar>` and
        // `Without<Wizard>` filters in the query keep the avatar and the
        // hidden wizard out of the damage set.
        if just_spawned {
            let arc_pos = arc_transform.translation;
            for (target_entity, target_transform, mut health, temp_hp) in &mut targets {
                let diff = target_transform.translation - arc_pos;
                let dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                if dist > SWORD_ARC_RADIUS {
                    continue;
                }
                let target_angle = Vec2::new(diff.x, diff.z).normalize_or_zero();
                if arc.direction.dot(target_angle) > cos_half_angle {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        SWORD_DAMAGE,
                        DamageType::Force,
                        false,
                    );
                    commands.entity(target_entity).insert(
                        crate::game::units::hit_flash::HitFlash {
                            timer: SWORD_HIT_FLASH_DURATION,
                        },
                    );
                }
            }
        }

        if arc.time_alive >= arc.duration {
            commands.entity(arc_entity).try_despawn();
        }
    }
}

/// Builds the curved strip mesh for one sword swing. Per-vertex colors
/// encode the right-to-left alpha gradient (trailing edge transparent,
/// leading edge opaque) — `StandardMaterial` picks up `ATTRIBUTE_COLOR`
/// automatically when present.
fn build_arc_strip_mesh(
    direction: Vec2,
    radius: f32,
    half_angle: f32,
    thickness: f32,
    segments: u32,
) -> Mesh {
    use bevy::mesh::{Indices, PrimitiveTopology};

    let base_angle = direction.y.atan2(direction.x);
    let inner_r = (radius - thickness * 0.5).max(0.0);
    let outer_r = radius + thickness * 0.5;

    let n = (segments + 1) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n * 2);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n * 2);

    for i in 0..=segments {
        let frac = i as f32 / segments as f32;
        let angle = base_angle - half_angle + frac * 2.0 * half_angle;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        positions.push([inner_r * cos_a, 0.0, inner_r * sin_a]);
        positions.push([outer_r * cos_a, 0.0, outer_r * sin_a]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([frac, 0.0]);
        uvs.push([frac, 1.0]);

        // `frac.powf(2.5)` gives a fast trailing-edge fall-off so the back
        // of the slash thins out into a fine line rather than a hard cut.
        let alpha = frac.powf(2.5);
        colors.push([1.0, 1.0, 1.0, alpha]);
        colors.push([1.0, 1.0, 1.0, alpha]);
    }

    let mut indices: Vec<u32> = Vec::with_capacity((segments * 6) as usize);
    for i in 0..segments {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        let i2 = (i + 1) * 2;
        let i3 = (i + 1) * 2 + 1;
        indices.push(i0);
        indices.push(i2);
        indices.push(i1);
        indices.push(i1);
        indices.push(i2);
        indices.push(i3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Spawns a sword-swing arc (grow + fade visual). Shared by the local
/// `sword_swing` (`ghost = false`) and the opponent's replicated swing
/// (`ghost = true`, tagged `GhostSwordArc` so it deals no damage).
pub(crate) fn spawn_sword_arc(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    pos: Vec3,
    direction: Vec2,
    ghost: bool,
) {
    let arc_mesh = meshes.add(build_arc_strip_mesh(
        direction,
        SWORD_ARC_RADIUS,
        SWORD_ARC_HALF_ANGLE,
        SWORD_ARC_THICKNESS,
        SWORD_ARC_SEGMENTS,
    ));
    // Per-instance material — alpha fades over the arc's lifetime.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let mut entity = commands.spawn((
        Mesh3d(arc_mesh),
        MeshMaterial3d(material),
        Transform::from_translation(pos).with_scale(Vec3::splat(SWORD_ARC_MIN_SCALE)),
        SwordArc {
            time_alive: 0.0,
            duration: SWORD_ARC_DURATION,
            direction,
        },
        OnGameplayScreen,
    ));
    if ghost {
        entity.insert(GhostSwordArc);
    }
}
