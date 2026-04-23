use super::components::SwordArcMaterial;
use super::components::*;
use super::constants::*;
use super::messages::*;
use super::resources::{BattlemageAssets, BattlemagePhase, BattlemageState};
use crate::config::GameConfig;
use crate::config::input_bindings::InputBindings;
use crate::game::components::{Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::WIZARD_POSITION;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ArchetypeUI;
use crate::game::input::messages::{BlockSpellInput, MouseClicked};
use crate::game::units::components::{
    AttackTiming, Corpse, Effectiveness, FacingDirection, Health, Hitbox, MovementSpeed, Team,
    TemporaryHitPoints, WalkingAnimation, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::wizard::components::{Mana, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
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

/// Ticks battlemage cooldowns.
pub(super) fn tick_cooldowns(
    time: Res<Time>,
    mut commands: Commands,
    mut missile_cooldowns: Query<(Entity, &mut BattlemageMissileCooldown)>,
    mut sword_cooldowns: Query<(Entity, &mut BattlemageSwordCooldown)>,
) {
    for (entity, mut cd) in &mut missile_cooldowns {
        cd.remaining -= time.delta_secs();
        if cd.remaining <= 0.0 {
            commands
                .entity(entity)
                .remove::<BattlemageMissileCooldown>();
        }
    }
    for (entity, mut cd) in &mut sword_cooldowns {
        cd.remaining -= time.delta_secs();
        if cd.remaining <= 0.0 {
            commands.entity(entity).remove::<BattlemageSwordCooldown>();
        }
    }
}

/// Checks if the battlemage avatar has died and triggers retreat.
pub(super) fn check_avatar_death(
    avatar_query: Query<&Health, With<BattlemageAvatar>>,
    state: Res<BattlemageState>,
    mut retreat_messages: MessageWriter<RetreatMessage>,
) {
    if state.phase != BattlemagePhase::OnField {
        return;
    }

    if let Ok(health) = avatar_query.single()
        && health.current <= 0.0
    {
        retreat_messages.write(RetreatMessage);
    }
}

/// Spawns the battlemage health bar UI when the avatar is on the field.
pub(super) fn spawn_health_bar(
    mut commands: Commands,
    state: Res<BattlemageState>,
    existing_bar: Query<Entity, With<BattlemageHealthBar>>,
) {
    // Only spawn when transitioning to OnField and no bar exists
    if !state.is_changed() || state.phase != BattlemagePhase::OnField || !existing_bar.is_empty() {
        return;
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Percent(50.0),
                width: Val::Px(200.0),
                height: Val::Px(16.0),
                margin: UiRect::left(Val::Px(-100.0)), // center
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderColor::all(Color::srgba(0.3, 0.3, 0.8, 1.0)),
            BorderRadius::all(Val::Px(3.0)),
            BattlemageHealthBar,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.4, 1.0)),
                BorderRadius::all(Val::Px(2.0)),
                BattlemageHealthBarFill,
            ));
        });
}

/// Updates the battlemage health bar fill.
pub(super) fn update_health_bar(
    avatar_query: Query<&Health, With<BattlemageAvatar>>,
    mut fill_query: Query<&mut Node, With<BattlemageHealthBarFill>>,
) {
    if let Ok(health) = avatar_query.single()
        && let Ok(mut node) = fill_query.single_mut()
    {
        let pct = (health.current / health.max * 100.0).clamp(0.0, 100.0);
        node.width = Val::Percent(pct);
    }
}

/// Despawns the health bar when the avatar is no longer on the field.
pub(super) fn despawn_health_bar(
    mut commands: Commands,
    state: Res<BattlemageState>,
    bar_query: Query<Entity, With<BattlemageHealthBar>>,
) {
    if state.is_changed() && state.phase != BattlemagePhase::OnField {
        for entity in &bar_query {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns the "Enter the Fray" button UI at the bottom center of the screen.
pub(crate) fn spawn_enter_fray_button(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(FRAY_BUTTON_BOTTOM),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            EnterFrayRoot,
            OnGameplayScreen,
            ArchetypeUI,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.4, 0.5, 1.0, 1.0)),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.4, 0.9)),
                    EnterFrayButton,
                ))
                .with_child((
                    Text::new(FRAY_BUTTON_TEXT),
                    TextFont::from_font_size(FRAY_BUTTON_FONT_SIZE),
                    TextColor(Color::srgba(0.8, 0.85, 1.0, 1.0)),
                    EnterFrayButtonText,
                ));
        });
}

/// Handles clicks on the "Enter the Fray" button — transitions to location choosing.
pub(super) fn handle_enter_fray_click(
    mut button_clicked: MessageReader<MouseClicked>,
    fray_button_query: Query<Entity, With<EnterFrayButton>>,
    mut state: ResMut<BattlemageState>,
    mut text_query: Query<&mut Text, With<EnterFrayButtonText>>,
) {
    for event in button_clicked.read() {
        if fray_button_query.get(event.button).is_ok() {
            if state.phase == BattlemagePhase::Idle && !state.retreated {
                state.phase = BattlemagePhase::ChoosingLocation;
                if let Ok(mut text) = text_query.single_mut() {
                    **text = FRAY_BUTTON_TEXT_CHOOSING.to_string();
                }
            } else if state.phase == BattlemagePhase::ChoosingLocation {
                // Clicking button again cancels
                state.phase = BattlemagePhase::Idle;
                if let Ok(mut text) = text_query.single_mut() {
                    **text = FRAY_BUTTON_TEXT.to_string();
                }
            }
        }
    }
}

/// Handles left-click / RT on the battlefield to place the avatar during ChoosingLocation phase.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_location_click(
    mut mouse_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut state: ResMut<BattlemageState>,
    mut commands: Commands,
    mut wizard_query: Query<&mut Visibility, With<Wizard>>,
    battlemage_assets: Res<BattlemageAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut text_query: Query<&mut Text, With<EnterFrayButtonText>>,
) {
    if state.phase != BattlemagePhase::ChoosingLocation {
        return;
    }
    if mouse_pressed.read().next().is_none() {
        return;
    }

    let Some(world_pos) = get_cursor_world_position(&camera_query_3d, &corrected_cursor) else {
        return;
    };

    let Ok(mut visibility) = wizard_query.single_mut() else {
        return;
    };

    // Play teleport sound and hide wizard
    audio::play_sfx(
        &mut commands,
        &sfx.teleport_cast,
        WIZARD_POSITION,
        &config,
        &sfx,
    );
    *visibility = Visibility::Hidden;

    // Spawn the avatar at the clicked position
    let hitbox = Hitbox::new(AVATAR_HITBOX_RADIUS, AVATAR_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let material = create_default_sprite_material(
        &mut materials,
        battlemage_assets.sprite_texture.clone(),
        AVATAR_SPRITE_TINT,
    );

    commands.spawn((
        Mesh3d(battlemage_assets.sprite_mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(world_pos.x, spawn_y, world_pos.z),
        Velocity::default(),
        hitbox,
        Health::new(AVATAR_HEALTH),
        MovementSpeed(AVATAR_MOVEMENT_SPEED),
        AttackTiming::new(),
        Effectiveness::new(),
        Team::Defenders,
        (
            BattlemageAvatar,
            BattlemageFacing::default(),
            WalkingAnimation::default(),
            FacingDirection::default(),
            Billboard,
            OnGameplayScreen,
        ),
    ));

    state.phase = BattlemagePhase::OnField;

    // Reset button text
    if let Ok(mut text) = text_query.single_mut() {
        **text = FRAY_BUTTON_TEXT.to_string();
    }
}

/// Shows/hides the "Enter the Fray" button based on battlemage state.
pub(super) fn update_enter_fray_visibility(
    state: Res<BattlemageState>,
    mut root_query: Query<&mut Visibility, With<EnterFrayRoot>>,
) {
    if !state.is_changed() {
        return;
    }
    let visible = matches!(
        state.phase,
        BattlemagePhase::Idle | BattlemagePhase::ChoosingLocation
    ) && !state.retreated;
    for mut vis in &mut root_query {
        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
