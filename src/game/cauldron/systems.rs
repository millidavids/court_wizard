use bevy::math::Affine2;
use bevy::prelude::*;

use super::components::*;
use super::constants;
use super::messages::*;
use super::resources::{CauldronAssets, CauldronBuffs};
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::BlockSpellInput;
use crate::game::units::components::{Corpse, Health, Team};

/// Loads the cauldron sprite sheet texture.
pub fn load_cauldron_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite_texture = asset_server.load("images/cauldron-64px-9.png");
    commands.insert_resource(CauldronAssets { sprite_texture });
}

/// Spawns the cauldron entity as an animated sprite billboard.
pub fn spawn_cauldron(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cauldron_assets: Res<CauldronAssets>,
) {
    // Create a quad mesh for the billboard
    let quad_mesh = Rectangle::new(
        constants::CAULDRON_SPRITE_SIZE,
        constants::CAULDRON_SPRITE_SIZE,
    );

    // UV transform for first frame: scale to 1/3 to show only top-left frame
    let grid_size = constants::CAULDRON_SPRITE_GRID_SIZE as f32;
    let frame_scale = 1.0 / grid_size;
    let uv_transform = Affine2::from_scale(Vec2::splat(frame_scale));

    // Create material with sprite sheet texture
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(cauldron_assets.sprite_texture.clone()),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        uv_transform,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(quad_mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(constants::CAULDRON_POSITION),
        Cauldron,
        CauldronState::default(),
        CauldronAnimation::new(),
        OrthogonalBillboard,
        OnGameplayScreen,
    ));
}

/// Handles StartBrewMessage to begin brewing.
pub fn handle_start_brew(
    mut messages: MessageReader<StartBrewMessage>,
    mut cauldron_query: Query<&mut CauldronState, With<Cauldron>>,
) {
    for message in messages.read() {
        if let Ok(mut state) = cauldron_query.single_mut()
            && state.is_idle()
        {
            let duration = message.recipe.brew_time();
            state.start_brewing(message.recipe.clone(), duration);
        }
    }
}

/// Ticks the brew timer and sends BrewCompleteMessage when done.
pub fn update_brew_timer(
    time: Res<Time>,
    mut cauldron_query: Query<&mut CauldronState, With<Cauldron>>,
    mut brew_complete: MessageWriter<BrewCompleteMessage>,
) {
    if let Ok(mut state) = cauldron_query.single_mut()
        && let Some(completed_recipe) = state.tick(time.delta_secs())
    {
        brew_complete.write(BrewCompleteMessage {
            recipe: completed_recipe,
        });
    }
}

/// Handles BrewCompleteMessage to apply the recipe's buff and spawn visual bubble.
pub fn handle_brew_complete(
    mut commands: Commands,
    mut messages: MessageReader<BrewCompleteMessage>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for message in messages.read() {
        cauldron_buffs.apply_recipe(&message.recipe);

        // Spawn the expanding bubble visual effect
        let bubble_color = message.recipe.color();
        let spawn_pos = Vec3::new(
            constants::CAULDRON_POSITION.x,
            constants::BREW_BUBBLE_HEIGHT,
            constants::CAULDRON_POSITION.z,
        );

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: bubble_color.with_alpha(constants::BREW_BUBBLE_INITIAL_ALPHA),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                cull_mode: None,
                ..default()
            })),
            Transform::from_translation(spawn_pos).with_scale(Vec3::ZERO),
            BrewBubble {
                time_alive: 0.0,
                duration: constants::BREW_BUBBLE_DURATION,
                color: bubble_color,
            },
            OnGameplayScreen,
        ));
    }
}

/// Ticks active buff timers and removes expired buffs.
pub fn tick_active_buffs(time: Res<Time>, mut cauldron_buffs: ResMut<CauldronBuffs>) {
    if cauldron_buffs.has_active_buffs() {
        cauldron_buffs.tick(time.delta_secs());
    }
}

/// Handles CancelBrewMessage to cancel the current brew.
pub fn handle_cancel_brew(
    mut messages: MessageReader<CancelBrewMessage>,
    mut cauldron_query: Query<&mut CauldronState, With<Cauldron>>,
) {
    for _ in messages.read() {
        if let Ok(mut state) = cauldron_query.single_mut()
            && state.is_brewing()
        {
            state.cancel();
        }
    }
}

/// Blocks spell input while the cauldron is brewing.
///
/// Sends a BlockSpellInput message each frame during brewing, which piggybacks
/// on the existing spell_input_not_blocked run condition used by all spells.
pub fn block_spells_during_brewing(
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut block_spell: MessageWriter<BlockSpellInput>,
) {
    if let Ok(state) = cauldron_query.single()
        && state.is_brewing()
    {
        block_spell.write(BlockSpellInput);
    }
}

/// Updates brew bubble: expands continuously, fades alpha to 0, then despawns.
pub fn update_brew_bubble(
    mut commands: Commands,
    time: Res<Time>,
    mut bubbles: Query<(
        Entity,
        &mut BrewBubble,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut bubble, mut transform, material_handle) in &mut bubbles {
        bubble.time_alive += time.delta_secs();

        if bubble.time_alive >= bubble.duration {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = bubble.time_alive / bubble.duration;

        // Scale: expand at constant speed forever
        let radius = constants::BREW_BUBBLE_EXPAND_SPEED * bubble.time_alive;
        transform.scale = Vec3::splat(radius);

        // Alpha: fade linearly from initial to 0 over the full duration
        let alpha = constants::BREW_BUBBLE_INITIAL_ALPHA * (1.0 - progress);

        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = bubble.color.with_alpha(alpha);
        }
    }
}

/// Updates orthogonal billboard entities to face the camera directly.
///
/// Unlike the standard billboard which only rotates on Y-axis, this makes
/// the entity fully orthogonal to the camera view (faces camera directly).
pub fn update_orthogonal_billboard(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut billboard_query: Query<&mut Transform, (With<OrthogonalBillboard>, Without<Camera3d>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Make the billboard face the camera while maintaining upright orientation
    for mut transform in &mut billboard_query {
        let camera_pos = camera_transform.translation;

        // World up direction
        let up = Vec3::Y;

        // Use look_at to face the camera while staying upright
        // This creates a rotation where -Z points toward camera and Y points up
        transform.look_at(camera_pos, up);
    }
}

/// Updates the cauldron sprite sheet animation.
pub fn update_cauldron_animation(
    time: Res<Time>,
    mut cauldron_query: Query<
        (&mut CauldronAnimation, &MeshMaterial3d<StandardMaterial>),
        With<Cauldron>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok((mut animation, material_handle)) = cauldron_query.single_mut() {
        if animation.tick(time.delta_secs()) {
            if let Some(material) = materials.get_mut(material_handle) {
                let (offset_x, offset_y) = animation.uv_offset();
                let grid_size = constants::CAULDRON_SPRITE_GRID_SIZE as f32;
                let frame_scale = 1.0 / grid_size;

                material.uv_transform = Affine2::from_scale_angle_translation(
                    Vec2::splat(frame_scale),
                    0.0,
                    Vec2::new(offset_x, offset_y),
                );
            }
        }
    }
}

/// Heals all living defender units based on the active DefenderHealPerSecond buff.
pub fn heal_defenders(
    time: Res<Time>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut defenders: Query<(&mut Health, &Team), Without<Corpse>>,
) {
    let heal_per_second = cauldron_buffs.defender_heal_per_second();
    if heal_per_second <= 0.0 {
        return;
    }
    let heal_amount = heal_per_second * time.delta_secs();
    for (mut health, team) in &mut defenders {
        if *team == Team::Defenders {
            health.heal(heal_amount);
        }
    }
}
