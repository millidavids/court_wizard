use bevy::math::Affine2;
use bevy::prelude::*;

use super::components::*;
use super::constants;
use super::messages::*;
use super::resources::{CauldronAssets, CauldronBuffs};
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::input::messages::BlockSpellInput;
use crate::game::messages::ComboDiscoveredMessage;
use crate::game::units::components::{Corpse, Effectiveness, Health, Team, TemporaryHitPoints};
use crate::game::units::wizard::components::{LocalWizard, Mana};
use crate::game::units::wizard::spells::telekinesis::components::TransmutationStacks;
use crate::game::units::wizard::spells::telekinesis::constants::TRANSMUTATION_POTENCY_PER_STACK;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

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
    spell_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut combo_writer: MessageWriter<ComboDiscoveredMessage>,
    mut transmutation_stacks: Option<ResMut<TransmutationStacks>>,
) {
    for message in messages.read() {
        // Check for hidden combos and notify
        for combo in message.recipe.matching_combos() {
            combo_writer.write(ComboDiscoveredMessage {
                name: combo.name,
                description: combo.description,
            });
        }

        // T3: Transmutation — boost brew potency based on stacks
        let potency_mult = transmutation_stacks
            .as_ref()
            .filter(|s| s.count > 0)
            .map(|s| 1.0 + s.count as f32 * TRANSMUTATION_POTENCY_PER_STACK)
            .unwrap_or(1.0);

        if potency_mult > 1.0 {
            cauldron_buffs.apply_recipe_with_potency(&message.recipe, potency_mult);
        } else {
            cauldron_buffs.apply_recipe(&message.recipe);
        }

        // Reset transmutation stacks after brewing
        if let Some(ref mut stacks) = transmutation_stacks {
            stacks.count = 0;
        }

        // Spawn the expanding bubble visual effect
        let bubble_color = message.recipe.color();
        let spawn_pos = Vec3::new(
            constants::CAULDRON_POSITION.x,
            constants::BREW_BUBBLE_HEIGHT,
            constants::CAULDRON_POSITION.z,
        );

        commands.spawn((
            Mesh3d(spell_assets.cross_plane_sphere.clone()),
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
            commands.entity(entity).try_despawn();
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

/// Updates the cauldron sprite sheet animation.
pub fn update_cauldron_animation(
    time: Res<Time>,
    mut cauldron_query: Query<
        (&mut CauldronAnimation, &MeshMaterial3d<StandardMaterial>),
        With<Cauldron>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Pre-calculate constant values
    const FRAME_SCALE: f32 = 1.0 / constants::CAULDRON_SPRITE_GRID_SIZE as f32;
    let frame_scale_vec = Vec2::splat(FRAME_SCALE);

    if let Ok((mut animation, material_handle)) = cauldron_query.single_mut()
        && animation.tick(time.delta_secs())
        && let Some(material) = materials.get_mut(material_handle)
    {
        let (offset_x, offset_y) = animation.uv_offset();

        material.uv_transform = Affine2::from_scale_angle_translation(
            frame_scale_vec,
            0.0,
            Vec2::new(offset_x, offset_y),
        );
    }
}

/// Adds brewing visual effects when brewing starts and resets when done.
pub fn start_brewing_effects(
    mut commands: Commands,
    mut cauldron_query: Query<
        (
            Entity,
            &CauldronState,
            Option<&CauldronBrewingEffects>,
            &mut Transform,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (With<Cauldron>, Changed<CauldronState>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, state, existing_effects, mut transform, material_handle) in &mut cauldron_query {
        match state {
            CauldronState::Brewing { recipe, .. } => {
                // Only add if it doesn't already exist
                if existing_effects.is_none() {
                    commands.entity(entity).insert(CauldronBrewingEffects {
                        pulse_timer: 0.0,
                        original_scale: Vec3::ONE,
                        recipe_color: recipe.color(),
                    });
                }
            }
            CauldronState::Idle | CauldronState::Cooldown { .. } => {
                // Reset visual state BEFORE removing component
                if existing_effects.is_some() {
                    // Reset scale to original
                    transform.scale = Vec3::ONE;

                    // Reset color to white
                    if let Some(material) = materials.get_mut(material_handle) {
                        material.base_color = Color::WHITE;
                    }

                    // Now remove the component
                    commands.entity(entity).remove::<CauldronBrewingEffects>();
                }
            }
        }
    }
}

/// Updates brewing visual effects (pulsing and color tinting).
pub fn update_brewing_effects(
    mut cauldron_query: Query<
        (
            &mut Transform,
            &CauldronBrewingEffects,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<Cauldron>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Pre-calculate white color in linear space (constant for all iterations)
    let white_linear = Color::WHITE.to_linear();

    for (mut transform, effects, material_handle) in &mut cauldron_query {
        // Calculate pulse progress based on timer
        let pulse_cycle = (effects.pulse_timer % constants::BREWING_PULSE_DURATION)
            / constants::BREWING_PULSE_DURATION;

        // Sine wave for smooth pulsing (0 to 1 to 0)
        let pulse_progress = (pulse_cycle * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        // Calculate scale multiplier using pre-calculated range
        let scale_multiplier = constants::BREWING_PULSE_SCALE_MIN
            + pulse_progress * constants::BREWING_PULSE_SCALE_RANGE;

        // Apply scale multiplier to ORIGINAL scale (not current scale)
        transform.scale = effects.original_scale * scale_multiplier;

        // Fade color strength in and out using pre-calculated range
        let color_strength = constants::BREWING_COLOR_ALPHA_MIN
            + pulse_progress * constants::BREWING_COLOR_ALPHA_RANGE;

        // Blend between white (base sprite) and recipe color using mix
        if let Some(material) = materials.get_mut(material_handle) {
            // Convert recipe color to linear space
            let recipe_linear = effects.recipe_color.to_linear();

            // Lerp between white and recipe color
            let blended_linear = white_linear.mix(&recipe_linear, color_strength);

            // Convert back to sRGB
            material.base_color = Color::from(blended_linear);
        }
    }
}

/// Updates the brewing effects timer separately to avoid borrow issues.
pub fn update_brewing_timer(
    time: Res<Time>,
    mut cauldron_query: Query<&mut CauldronBrewingEffects, With<Cauldron>>,
) {
    let delta = time.delta_secs();
    for mut effects in &mut cauldron_query {
        effects.pulse_timer += delta;
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

/// Applies or removes CauldronDamageBonus on all defenders based on active buffs.
pub fn buff_defender_damage(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    defenders: Query<(Entity, &Team, Option<&CauldronDamageBonus>), Without<Corpse>>,
) {
    let bonus = cauldron_buffs.defender_damage_bonus();
    for (entity, team, existing) in &defenders {
        if *team == Team::Defenders {
            if bonus > 0.0 {
                if existing.is_none() {
                    commands.entity(entity).insert(CauldronDamageBonus(bonus));
                }
            } else if existing.is_some() {
                commands.entity(entity).remove::<CauldronDamageBonus>();
            }
        }
    }
}

/// Applies or removes CauldronDamageResistance on all defenders based on active buffs.
pub fn buff_defender_resistance(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    defenders: Query<(Entity, &Team, Option<&CauldronDamageResistance>), Without<Corpse>>,
) {
    let resistance = cauldron_buffs.damage_resistance_percent();
    for (entity, team, existing) in &defenders {
        if *team == Team::Defenders {
            if resistance > 0.0 {
                if existing.is_none() {
                    commands
                        .entity(entity)
                        .insert(CauldronDamageResistance(resistance));
                }
            } else if existing.is_some() {
                commands.entity(entity).remove::<CauldronDamageResistance>();
            }
        }
    }
}

/// Applies or removes CauldronSpeedModifier on units based on active buffs.
///
/// Defenders get a speed bonus (Meadowsweet), attackers/undead get a slow (Valerian).
pub fn apply_cauldron_speed_modifiers(
    mut commands: Commands,
    cauldron_buffs: Res<CauldronBuffs>,
    units: Query<(Entity, &Team, Option<&CauldronSpeedModifier>), Without<Corpse>>,
) {
    let defender_bonus = cauldron_buffs.defender_speed_bonus();
    let attacker_slow = cauldron_buffs.attacker_slow_percent();

    for (entity, team, existing) in &units {
        let modifier = match team {
            Team::Defenders => {
                if defender_bonus > 0.0 {
                    Some(defender_bonus)
                } else {
                    None
                }
            }
            Team::Attackers | Team::Undead => {
                if attacker_slow > 0.0 {
                    Some(-attacker_slow)
                } else {
                    None
                }
            }
        };

        if let Some(value) = modifier {
            if existing.is_none() {
                commands.entity(entity).insert(CauldronSpeedModifier(value));
            }
        } else if existing.is_some() {
            commands.entity(entity).remove::<CauldronSpeedModifier>();
        }
    }
}

/// Removes all cauldron buff components from units when no buffs are active.
///
/// This runs when buffs have just expired to clean up lingering components
/// that were inserted by the per-frame buff systems.
pub fn cleanup_cauldron_buff_components(
    mut commands: Commands,
    units: Query<
        (
            Entity,
            Option<&CauldronDamageBonus>,
            Option<&CauldronDamageResistance>,
            Option<&CauldronSpeedModifier>,
        ),
        Without<Corpse>,
    >,
    mut defenders: Query<(&mut Effectiveness, &Team), Without<Corpse>>,
    mut wizard: Query<&mut Mana, With<LocalWizard>>,
) {
    for (entity, damage_bonus, resistance, speed_mod) in &units {
        if damage_bonus.is_some() {
            commands.entity(entity).remove::<CauldronDamageBonus>();
        }
        if resistance.is_some() {
            commands.entity(entity).remove::<CauldronDamageResistance>();
        }
        if speed_mod.is_some() {
            commands.entity(entity).remove::<CauldronSpeedModifier>();
        }
    }
    // Reset defender effectiveness bonus
    for (mut effectiveness, team) in &mut defenders {
        if *team == Team::Defenders && effectiveness.spell_bonus != 0.0 {
            effectiveness.spell_bonus = 0.0;
        }
    }
    // Reset wizard mana max to base
    let base_max = crate::game::units::wizard::constants::MANA;
    for mut mana in &mut wizard {
        if mana.max != base_max {
            mana.max = base_max;
            if mana.current > mana.max {
                mana.current = mana.max;
            }
        }
    }
}

/// Grants temporary hit points to defenders based on active DefenderShieldPerSecond buff.
pub fn shield_defenders(
    time: Res<Time>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut defenders: Query<(Entity, &Team, Option<&mut TemporaryHitPoints>), Without<Corpse>>,
    mut commands: Commands,
) {
    let shield_per_second = cauldron_buffs.defender_shield_per_second();
    if shield_per_second <= 0.0 {
        return;
    }
    let shield_amount = shield_per_second * time.delta_secs();
    const MAX_SHIELD: f32 = 20.0;

    for (entity, team, temp_hp) in &mut defenders {
        if *team != Team::Defenders {
            continue;
        }
        if let Some(mut existing) = temp_hp {
            existing.amount = (existing.amount + shield_amount).min(MAX_SHIELD);
            existing.time_remaining = 5.0; // Keep alive while buff is active
        } else {
            commands
                .entity(entity)
                .insert(TemporaryHitPoints::new(shield_amount, 5.0));
        }
    }
}

/// Applies max mana multiplier to the wizard's mana pool based on active cauldron buffs.
pub fn apply_max_mana_buff(
    cauldron_buffs: Res<CauldronBuffs>,
    mut wizard: Query<&mut Mana, With<LocalWizard>>,
) {
    let multiplier = cauldron_buffs.max_mana_multiplier();
    if multiplier <= 1.0 {
        return;
    }
    let base_max = crate::game::units::wizard::constants::MANA;
    let new_max = base_max * multiplier;
    for mut mana in &mut wizard {
        mana.max = new_max;
        if mana.current > mana.max {
            mana.current = mana.max;
        }
    }
}

/// Applies effectiveness bonus to all defenders based on active cauldron buffs.
pub fn buff_defender_effectiveness(
    cauldron_buffs: Res<CauldronBuffs>,
    mut defenders: Query<(&mut Effectiveness, &Team), Without<Corpse>>,
) {
    let bonus = cauldron_buffs.effectiveness_bonus();
    for (mut effectiveness, team) in &mut defenders {
        if *team == Team::Defenders {
            effectiveness.spell_bonus = bonus;
        }
    }
}
