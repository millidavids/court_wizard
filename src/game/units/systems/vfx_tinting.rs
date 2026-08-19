use bevy::prelude::*;

use super::super::components::{
    BerserkerRageModifier, FearModifier, FireDoT, FrostAccumulation, MindControlled,
    OriginalMaterial, Petrified, PolymorphedModifier, RemoteElectricEffect, RemoteFireEffect,
    RemoteFrostEffect, RemotePoisonEffect, RemoteRageEffect, Shocked, SickenedModifier,
    SmellyModifier,
};
use super::super::constants::{
    BERSERKER_RAGE_EFFECT_COLOR, BERSERKER_RAGE_EFFECT_INTENSITY, ELECTRIC_EFFECT_COLOR,
    ELECTRIC_EFFECT_FLICKER_SPEED, ELECTRIC_EFFECT_MAX_INTENSITY, ELECTRIC_EFFECT_MIN_INTENSITY,
    ELITE_EFFECT_COLOR, ELITE_EFFECT_MAX_INTENSITY, ELITE_EFFECT_MIN_INTENSITY,
    ELITE_EFFECT_PULSE_SPEED, FROST_EFFECT_COLOR, FROST_EFFECT_MAX_INTENSITY,
    MIND_CONTROL_EFFECT_COLOR, MIND_CONTROL_EFFECT_INTENSITY, POISON_EFFECT_COLOR,
    POISON_EFFECT_INTENSITY, SHIELD_EFFECT_COLOR, SHIELD_EFFECT_MAX_INTENSITY,
    SHIELD_EFFECT_MIN_INTENSITY, SHIELD_EFFECT_PULSE_SPEED, SICKENED_EFFECT_COLOR,
    SICKENED_EFFECT_INTENSITY, SMELLY_EFFECT_COLOR, SMELLY_EFFECT_INTENSITY,
    UNIT_TYPE_GLOW_MAX_INTENSITY, UNIT_TYPE_GLOW_MIN_INTENSITY, UNIT_TYPE_GLOW_PULSE_SPEED,
    WET_EFFECT_COLOR, WET_EFFECT_INTENSITY,
};
use crate::game::units::wizard::archetypes::meteorologist::components::WetModifier;
use crate::game::units::wizard::spells::mind_control::components::MassHysteriaTarget;

/// Blends a pulsing color effect onto a linear color value.
///
/// Uses a sine wave to oscillate intensity between min and max bounds.
fn blend_pulsing_effect(
    result: &mut LinearRgba,
    effect_color: &LinearRgba,
    elapsed: f32,
    pulse_speed: f32,
    min_intensity: f32,
    max_intensity: f32,
) {
    let pulse = (elapsed * pulse_speed).sin() * 0.5 + 0.5;
    let intensity = min_intensity + pulse * (max_intensity - min_intensity);
    *result = result.mix(effect_color, intensity);
}

/// Updates visual tinting on units affected by persistent damage effects.
///
/// Considers both local effects (FireDoT, FrostEffectMarker, Shocked) and
/// remote effect markers (RemoteFireEffect, etc.) from the other multiplayer peer.
///
/// Three-phase logic per entity:
/// 1. Unit has effects but no OriginalMaterial: clone the material, store original
/// 2. Unit has effects and OriginalMaterial: blend effect colors onto cloned material
/// 3. Unit has OriginalMaterial but no effects: restore original, remove OriginalMaterial
#[allow(clippy::type_complexity)]
pub fn update_persistent_effect_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&FireDoT>,
            Option<&FrostAccumulation>,
            Option<&Shocked>,
            Has<RemoteFireEffect>,
            Has<RemoteFrostEffect>,
            Has<RemoteElectricEffect>,
            Has<MindControlled>,
            Has<MassHysteriaTarget>,
            Option<&OriginalMaterial>,
            (
                Has<super::super::components::PoisonedModifier>,
                Has<RemotePoisonEffect>,
                Has<SickenedModifier>,
                Has<SmellyModifier>,
                Has<BerserkerRageModifier>,
                Has<RemoteRageEffect>,
                Has<crate::game::multiplayer::components::GhostEntity>,
                Has<super::super::shielder::components::ShielderDamageReduction>,
                Has<super::super::staging_shield::StagingShieldGlow>,
                Has<super::super::elite::EliteHealthBonus>,
                Option<&super::super::components::UnitTypeGlow>,
                Has<WetModifier>,
                Has<Petrified>,
                Has<FearModifier>,
            ),
        ),
        (
            Or<(
                With<FireDoT>,
                With<FrostAccumulation>,
                With<Shocked>,
                With<RemoteFireEffect>,
                With<RemoteFrostEffect>,
                With<RemoteElectricEffect>,
                With<MindControlled>,
                With<MassHysteriaTarget>,
                With<OriginalMaterial>,
                With<super::super::components::PoisonedModifier>,
                With<RemotePoisonEffect>,
                With<SickenedModifier>,
                With<SmellyModifier>,
                With<BerserkerRageModifier>,
                // Nested Or: the outer filter tuple is at Bevy's 15-element
                // arity cap, so overflow filters live here.
                Or<(
                    With<super::super::shielder::components::ShielderDamageReduction>,
                    With<super::super::staging_shield::StagingShieldGlow>,
                    With<super::super::elite::EliteHealthBonus>,
                    With<super::super::components::UnitTypeGlow>,
                    With<WetModifier>,
                    With<Petrified>,
                    With<FearModifier>,
                    With<RemoteRageEffect>,
                )>,
            )>,
            Without<PolymorphedModifier>,
        ),
    >,
) {
    let elapsed = time.elapsed_secs();

    // Pre-compute linear versions of constant effect colors (avoid per-entity conversion)
    let frost_linear = FROST_EFFECT_COLOR.to_linear();
    let wet_linear = WET_EFFECT_COLOR.to_linear();
    let electric_linear = ELECTRIC_EFFECT_COLOR.to_linear();
    let mc_linear = MIND_CONTROL_EFFECT_COLOR.to_linear();
    let poison_linear = POISON_EFFECT_COLOR.to_linear();
    let sickened_linear = SICKENED_EFFECT_COLOR.to_linear();
    let smelly_linear = SMELLY_EFFECT_COLOR.to_linear();
    let rage_linear = BERSERKER_RAGE_EFFECT_COLOR.to_linear();
    let shield_linear = SHIELD_EFFECT_COLOR.to_linear();
    let elite_linear = ELITE_EFFECT_COLOR.to_linear();

    for (
        entity,
        material_handle,
        fire,
        frost,
        electric,
        remote_fire,
        remote_frost,
        remote_electric,
        has_mind_control,
        has_mass_hysteria,
        original_mat,
        (
            has_poisoned,
            has_remote_poison,
            has_sickened,
            has_smelly,
            has_rage,
            has_remote_rage,
            is_ghost,
            has_shielder_shield,
            has_staging_glow,
            has_elite,
            unit_type_glow,
            has_wet,
            has_petrified,
            has_fear,
        ),
    ) in &query
    {
        // Ghost units carry `RemotePoisonEffect` (mirrored from the host's
        // `PoisonedModifier`) instead of the DoT component itself.
        let has_poisoned = has_poisoned || has_remote_poison;
        // Host-cast Berserker Rage reaches ghosts as the mirrored
        // `RemoteRageEffect` marker. A ghost may ALSO carry a real
        // `BerserkerRageModifier` from the guest's own cast (kept there on
        // purpose so `forward_status_effects_to_host` can relay it), but
        // `update_timed_modifier::<BerserkerRageModifier>` is gated behind
        // host-only `is_gameplay_running`, so that copy never expires on the
        // guest. Trusting it would leave the red tint stuck for the whole
        // match, so on ghosts only the host-authoritative marker counts.
        let has_rage = (has_rage && !is_ghost) || has_remote_rage;
        // The staging glow is a visual-only twin of the shielder's blessing —
        // both render the same golden pulse.
        let has_shield_glow = has_shielder_shield || has_staging_glow;
        let has_fire = fire.is_some() || remote_fire;
        let has_frost = frost.is_some() || remote_frost;
        let has_electric = electric.is_some() || remote_electric;
        let has_mc_visual = has_mind_control || has_mass_hysteria;
        let has_any_effect = has_fire
            || has_frost
            || has_petrified
            || has_fear
            || has_electric
            || has_mc_visual
            || has_poisoned
            || has_sickened
            || has_smelly
            || has_rage
            || has_shield_glow
            || has_elite
            || has_wet
            || unit_type_glow.is_some();

        if has_any_effect && original_mat.is_none() {
            // Phase 1: First effect applied — clone the material and store original
            let current_handle = material_handle.0.clone();
            let Some(current_material) = materials.get(&current_handle) else {
                continue;
            };
            let cloned = current_material.clone();
            let cloned_handle = materials.add(cloned);
            commands
                .entity(entity)
                .queue_silenced(move |mut e: EntityWorldMut| {
                    e.insert((
                        OriginalMaterial(current_handle),
                        MeshMaterial3d(cloned_handle),
                    ));
                });
        } else if has_any_effect {
            // Phase 2: Blend effect colors onto the cloned material
            let Some(original) = original_mat else {
                continue;
            };
            let Some(original_material) = materials.get(&original.0) else {
                continue;
            };
            let base_linear = original_material.base_color.to_linear();

            let mut result_linear = base_linear;

            // Fire tint removed — burning is shown via particle VFX only.

            if has_frost {
                let frost_intensity = frost
                    .map(|f| f.level * FROST_EFFECT_MAX_INTENSITY)
                    .unwrap_or(FROST_EFFECT_MAX_INTENSITY * 0.5);
                result_linear = result_linear.mix(&frost_linear, frost_intensity);
            }

            if has_wet {
                result_linear = result_linear.mix(&wet_linear, WET_EFFECT_INTENSITY);
            }

            if has_electric {
                blend_pulsing_effect(
                    &mut result_linear,
                    &electric_linear,
                    elapsed,
                    ELECTRIC_EFFECT_FLICKER_SPEED,
                    ELECTRIC_EFFECT_MIN_INTENSITY,
                    ELECTRIC_EFFECT_MAX_INTENSITY,
                );
            }

            if has_mc_visual {
                result_linear = result_linear.mix(&mc_linear, MIND_CONTROL_EFFECT_INTENSITY);
            }

            if has_poisoned {
                result_linear = result_linear.mix(&poison_linear, POISON_EFFECT_INTENSITY);
            }

            if has_sickened {
                result_linear = result_linear.mix(&sickened_linear, SICKENED_EFFECT_INTENSITY);
            }

            if has_smelly {
                result_linear = result_linear.mix(&smelly_linear, SMELLY_EFFECT_INTENSITY);
            }

            if has_rage {
                result_linear = result_linear.mix(&rage_linear, BERSERKER_RAGE_EFFECT_INTENSITY);
            }

            if has_shield_glow {
                blend_pulsing_effect(
                    &mut result_linear,
                    &shield_linear,
                    elapsed,
                    SHIELD_EFFECT_PULSE_SPEED,
                    SHIELD_EFFECT_MIN_INTENSITY,
                    SHIELD_EFFECT_MAX_INTENSITY,
                );
            }

            if has_elite {
                blend_pulsing_effect(
                    &mut result_linear,
                    &elite_linear,
                    elapsed,
                    ELITE_EFFECT_PULSE_SPEED,
                    ELITE_EFFECT_MIN_INTENSITY,
                    ELITE_EFFECT_MAX_INTENSITY,
                );
            }

            if let Some(glow) = unit_type_glow {
                let glow_linear = glow.color.to_linear();
                blend_pulsing_effect(
                    &mut result_linear,
                    &glow_linear,
                    elapsed,
                    UNIT_TYPE_GLOW_PULSE_SPEED,
                    UNIT_TYPE_GLOW_MIN_INTENSITY,
                    UNIT_TYPE_GLOW_MAX_INTENSITY,
                );
            }

            if has_fear {
                let purple = LinearRgba::new(0.5, 0.1, 0.6, 1.0);
                result_linear = result_linear.mix(&purple, 0.3);
            }

            if has_petrified {
                let gray = LinearRgba::new(0.4, 0.4, 0.4, 1.0);
                result_linear = result_linear.mix(&gray, 0.85);
            }

            if let Some(mut cloned_material) = materials.get_mut(material_handle) {
                cloned_material.base_color = Color::from(result_linear);
            }
        } else if let Some(original) = original_mat {
            // Phase 3: All effects expired — restore original material
            let restored = original.0.clone();
            commands
                .entity(entity)
                .queue_silenced(move |mut e: EntityWorldMut| {
                    e.insert(MeshMaterial3d(restored));
                    e.remove::<OriginalMaterial>();
                });
        }
    }
}
