use bevy::prelude::*;

use super::components::*;
use super::constants;
use super::messages::*;
use super::resources::CauldronBuffs;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::input::messages::BlockSpellInput;
use crate::game::units::components::{Corpse, Health, Team};

/// Spawns the cauldron entity as a charcoal circle on the castle wall next to the wizard.
pub fn spawn_cauldron(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let circle = Circle::new(constants::CAULDRON_RADIUS);

    commands.spawn((
        Mesh3d(meshes.add(circle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: constants::CAULDRON_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(constants::CAULDRON_POSITION),
        Cauldron,
        CauldronState::default(),
        Billboard,
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
