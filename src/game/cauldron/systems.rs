use bevy::prelude::*;

use super::components::*;
use super::constants;
use super::messages::*;
use super::resources::CauldronBuffs;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::input::messages::BlockSpellInput;

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
            state.start_brewing(message.brew, message.brew.brew_time());
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
        && let Some(completed_brew) = state.tick(time.delta_secs())
    {
        brew_complete.write(BrewCompleteMessage {
            brew: completed_brew,
        });
    }
}

/// Handles BrewCompleteMessage to apply the brew's buff.
pub fn handle_brew_complete(
    mut messages: MessageReader<BrewCompleteMessage>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
) {
    for message in messages.read() {
        cauldron_buffs.apply_brew(message.brew);
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
