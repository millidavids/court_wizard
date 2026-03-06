use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{SpikeGrowthIndicator, SpikeGrowthZone};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Health, SlowMovementModifier, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{clamp_cursor_to_spell_range, get_cursor_world_position, spawn_circle_indicator};
use crate::config::GameConfig;

/// Local wizard spike growth casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_spike_growth_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (
            Entity,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpikeGrowthIndicator>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::SpikeGrowth {
        return;
    }

    // Clamp cursor to spell range
    let clamped_cursor = clamp_cursor_to_spell_range(input.cursor_pos, wizard.spell_range, constants::CIRCLE_RADIUS * primed_spell.empowerment);

    // Handle release — clean up indicator
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).try_despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        return;
    }

    let Some(clamped_cursor) = clamped_cursor else {
        return;
    };

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &visual_assets,
                    visual_assets.spike_growth_indicator.clone(),
                    clamped_cursor,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment,
                    constants::CIRCLE_Y_POSITION,
                )
                .insert(SpikeGrowthIndicator::new(clamped_cursor, primed_spell.empowerment))
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            // Update indicator position
            if let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = clamped_cursor;
            }

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                            audio::play_sfx(&mut commands, &sfx.spike_growth_cast, indicator.position, &game_config, &sfx);
                            spawn_spike_growth_zone(
                                &mut commands,
                                &visual_assets,
                                &mut materials,
                                indicator.position,
                                radius,
                                indicator.empowerment,
                                &mut obstacle_events,
                            );
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    mouse_state.left_consumed = true;
                } else {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
        }
    }
}

pub fn update_spike_growth_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut SpikeGrowthIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();
        let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(radius * pulse);
        transform.translation.x = indicator.position.x;
        transform.translation.y = constants::CIRCLE_Y_POSITION;
        transform.translation.z = indicator.position.z;
    }
}

/// Applies periodic damage and slow to ALL units within the spike growth zone.
pub fn apply_spike_growth_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut SpikeGrowthZone>,
    mut targets: Query<(
        Entity,
        &Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&mut SlowMovementModifier>,
        Has<SpellShield>,
    )>,
) {
    let delta = time.delta_secs();

    for mut zone in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;

        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;

            for (entity, transform, mut health, mut temp_hp, existing_slow, has_spell_shield) in
                &mut targets
            {
                let distance = Vec3::new(
                    zone.origin.x - transform.translation.x,
                    0.0,
                    zone.origin.z - transform.translation.z,
                )
                .length();

                if distance <= zone.radius {
                    // Apply damage with Poison type (triggers PoisonedModifier stacking)
                    apply_spell_damage(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        zone.damage_per_tick,
                        DamageType::Poison,
                        has_spell_shield,
                    );
                    if has_spell_shield {
                        continue;
                    }

                    // Apply or refresh spike growth slow
                    if let Some(mut slow) = existing_slow {
                        slow.apply(zone.slow_modifier, zone.slow_duration);
                    } else {
                        commands.entity(entity).insert(SlowMovementModifier::new(
                            zone.slow_modifier,
                            zone.slow_duration,
                        ));
                    }
                }
            }
        }
    }
}

/// Fades spike growth zone visual over the last few seconds.
pub fn fade_spike_growth_zone(
    zones: Query<(&SpikeGrowthZone, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (zone, material_handle) in &zones {
        let Some(material) = materials.get_mut(material_handle) else {
            continue;
        };

        let remaining = zone.duration - zone.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        material.base_color = Color::srgba(0.15, 0.4, 0.05, 0.4 * fade);
    }
}

/// Despawns expired spike growth zones.
pub fn cleanup_spike_growth_zone(
    mut commands: Commands,
    zones: Query<(Entity, &SpikeGrowthZone)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
            let buffered_radius = zone.radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
            });
            commands.entity(entity).try_despawn();
        }
    }
}

pub(crate) fn spawn_spike_growth_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    let duration = constants::ZONE_DURATION * empowerment;
    let damage = constants::DAMAGE_PER_TICK * empowerment;
    let slow_mod = constants::SLOW_MODIFIER * empowerment;
    let slow_dur = constants::SLOW_DURATION * empowerment;

    // Notify pathfinding about hazard zone (buffered so units reroute before reaching it)
    let origin_2d = Vec2::new(position.x, position.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type: ObstacleType::Hazard(15.0),
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
    });

    // Clone material for per-instance fading
    let base_mat = materials
        .get(&assets.spike_growth_zone)
        .cloned()
        .unwrap_or_default();
    let instance_material = materials.add(base_mat);

    commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(instance_material),
        Transform::from_translation(Vec3::new(
            position.x,
            constants::CIRCLE_Y_POSITION,
            position.z,
        ))
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::splat(radius)),
        SpikeGrowthZone::new(
            Vec3::new(position.x, 0.0, position.z),
            radius,
            damage,
            constants::TICK_INTERVAL,
            slow_mod,
            slow_dur,
            duration,
        ),
        NetworkedSpellEffect {
            kind: SpellEffectKind::SpikeGrowthZone,
        },
        OnGameplayScreen,
    ));
}
