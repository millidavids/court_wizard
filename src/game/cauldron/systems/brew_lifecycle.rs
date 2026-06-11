use bevy::prelude::*;

use super::super::components::*;
use super::super::constants;
use super::super::messages::*;
use super::super::resources::{CauldronBuffs, PhilosophersStoneUsed};
use crate::config::save_data::{load_unified_save, new_unified_save, save_unified};
use crate::game::components::OnGameplayScreen;
use crate::game::input::messages::BlockSpellInput;
use crate::game::messages::ComboDiscoveredMessage;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::telekinesis::components::TransmutationStacks;
use crate::game::units::wizard::spells::telekinesis::constants::TRANSMUTATION_POTENCY_PER_STACK;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Handles StartBrewMessage to begin brewing.
pub fn handle_start_brew(
    mut messages: MessageReader<StartBrewMessage>,
    mut cauldron_query: Query<&mut CauldronState, With<Cauldron>>,
    config: Res<crate::config::GameConfig>,
) {
    for message in messages.read() {
        if let Ok(mut state) = cauldron_query.single_mut()
            && state.is_idle()
        {
            let mut duration = message.recipe.brew_time();
            if config.wizard_type == crate::config::WizardType::Alchemist {
                duration *= crate::game::cauldron::brews::constants::ALCHEMIST_BREW_TIME_MULTIPLIER;
            }
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
#[allow(clippy::too_many_arguments)]
pub fn handle_brew_complete(
    mut commands: Commands,
    mut messages: MessageReader<BrewCompleteMessage>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
    spell_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut combo_writer: MessageWriter<ComboDiscoveredMessage>,
    mut transmutation_stacks: Option<ResMut<TransmutationStacks>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut stone_used: ResMut<PhilosophersStoneUsed>,
    // The cauldron's actual transform — the multiplayer guest's cauldron lives at
    // CAULDRON_2_POSITION, so bubbles must follow the entity, not the constant.
    cauldron_query: Query<&Transform, With<Cauldron>>,
) {
    for message in messages.read() {
        // Check for hidden combos — batch unlock to avoid N load+save cycles
        let matching_combos = message.recipe.matching_combos();
        if !matching_combos.is_empty() {
            let mut save_file = load_unified_save().unwrap_or_else(new_unified_save);
            let mut changed = false;
            for combo in &matching_combos {
                let name_str = combo.name.to_string();
                if !save_file.player.unlocked_content.combos.contains(&name_str) {
                    save_file.player.unlocked_content.combos.push(name_str);
                    changed = true;
                    combo_writer.write(ComboDiscoveredMessage {
                        name: combo.name,
                        description: combo.description,
                    });
                }
            }
            if changed {
                save_unified(&save_file);
            }
        }

        // T3: Transmutation — boost brew potency based on stacks
        let potency_mult = transmutation_stacks
            .as_ref()
            .filter(|s| s.count > 0)
            .map(|s| 1.0 + s.count as f32 * TRANSMUTATION_POTENCY_PER_STACK)
            .unwrap_or(1.0);

        let duration_multiplier = if game_config.wizard_type == crate::config::WizardType::Alchemist
        {
            crate::game::cauldron::brews::constants::ALCHEMIST_DURATION_MULTIPLIER
        } else {
            1.0
        };
        cauldron_buffs.apply_recipe_with_potency(
            &message.recipe,
            potency_mult,
            duration_multiplier,
        );

        // Mark Philosopher's Stone as used if it was in the recipe
        if message
            .recipe
            .ingredients
            .iter()
            .any(|i| i.is_philosophers_stone())
        {
            stone_used.0 = true;
        }

        // Reset transmutation stacks after brewing
        if let Some(ref mut stacks) = transmutation_stacks {
            stacks.count = 0;
        }

        // Spawn the expanding bubble visual effect at the cauldron's real
        // position (read from the entity's Transform — single-player/versus host:
        // CAULDRON_POSITION; versus guest: CAULDRON_2_POSITION; co-op: the shared
        // CAULDRON_COOP_POSITION between the wizards).
        let bubble_color = message.recipe.color();
        let spawn_pos = cauldron_query
            .single()
            .map(|t| t.translation)
            .unwrap_or(constants::CAULDRON_POSITION);

        commands.spawn((
            Mesh3d(spell_assets.explosion_sphere.clone()),
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

        // Play cauldron bubbling sound (not affected by Excremage override)
        audio::play_sfx_scaled(
            &mut commands,
            &sfx.cauldron_bubbling,
            spawn_pos,
            &game_config,
            1.0,
        );
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
