use bevy::math::Affine2;
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants;
use crate::game::units::wizard::spells::spell_materials::explosion_fade_opacity;
use crate::game::units::wizard::spells::visual_assets::AuraSphereMaterial;

/// Updates brew bubble: expands continuously, fades opacity to 0, then despawns.
pub fn update_brew_bubble(
    mut commands: Commands,
    time: Res<Time>,
    mut bubbles: Query<(
        Entity,
        &mut BrewBubble,
        &mut Transform,
        &MeshMaterial3d<AuraSphereMaterial>,
    )>,
    mut materials: ResMut<Assets<AuraSphereMaterial>>,
) {
    for (entity, mut bubble, mut transform, material_handle) in &mut bubbles {
        bubble.time_alive += time.delta_secs();

        if bubble.time_alive >= constants::BREW_BUBBLE_DURATION {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = bubble.time_alive / constants::BREW_BUBBLE_DURATION;

        // Scale: expand at constant speed forever
        let radius = constants::BREW_BUBBLE_EXPAND_SPEED * bubble.time_alive;
        transform.scale = Vec3::splat(radius);

        // Hold full opacity, then fade out over the tail of the lifetime —
        // same curve as the other spell bursts. (The aura shader caps visible
        // alpha well below 1.0, so a linear fade from spawn would be pinned
        // at the cap for most of the animation and then cut off abruptly.)
        if let Some(mut material) = materials.get_mut(material_handle) {
            material.opacity = explosion_fade_opacity(progress);
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
        && let Some(mut material) = materials.get_mut(material_handle)
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
                    if let Some(mut material) = materials.get_mut(material_handle) {
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
        if let Some(mut material) = materials.get_mut(material_handle) {
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
