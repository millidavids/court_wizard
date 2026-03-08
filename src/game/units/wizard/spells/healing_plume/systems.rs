use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{HealingPlumeIndicator, HealingPlumeZone};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{Corpse, Health};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    clamp_cursor_to_spell_range, get_cursor_world_position, spawn_circle_indicator,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;

/// Local wizard healing plume casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_healing_plume_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut HealingPlumeIndicator>,
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
    if primed_spell.spell != Spell::HealingPlume {
        return;
    }

    let clamped_cursor = clamp_cursor_to_spell_range(
        input.cursor_pos,
        wizard.spell_range,
        constants::CIRCLE_RADIUS * primed_spell.empowerment,
    );

    // Handle release -- clean up indicator and SpellCaster
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

    // Manage indicator based on casting state
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(constants::MANA_COST)
                && let Some(pos) = clamped_cursor
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &visual_assets,
                    visual_assets.healing_plume_indicator.clone(),
                    pos,
                    constants::CIRCLE_RADIUS * primed_spell.empowerment,
                    constants::CIRCLE_Y_POSITION,
                )
                .insert(HealingPlumeIndicator::new(pos, primed_spell.empowerment))
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = clamped_cursor
                && let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = pos;
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
        }
    }

    let completed =
        healing_plume_casting_logic(&input, &time, &mut casting_state, &mut mana, primed_spell);

    if completed {
        // Spawn healing zone using indicator position
        if let Ok(caster) = caster_query.get(wizard_entity)
            && let Some(indicator_entity) = caster.indicator_entity
        {
            if let Ok(indicator) = indicator_query.get(indicator_entity) {
                let radius = constants::CIRCLE_RADIUS * indicator.empowerment;
                spawn_healing_plume_zone(
                    &mut commands,
                    &visual_assets,
                    &mut materials,
                    indicator.position,
                    radius,
                    indicator.empowerment,
                );
                audio::play_sfx(
                    &mut commands,
                    &sfx.healing_plume_cast,
                    indicator.position,
                    &game_config,
                    &sfx,
                );
            }
            commands.entity(indicator_entity).try_despawn();
        }
        commands.entity(wizard_entity).remove::<SpellCaster>();
        mouse_state.left_consumed = true;
    }
}

/// Core healing plume casting logic.
///
/// Handles CastingState transitions and mana consumption.
/// Does NOT manage SpellCaster, indicators, or mouse_state -- those are the wrapper's job.
fn healing_plume_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
) -> bool {
    if input.just_released {
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                casting_state.start_cast();
            }
        }
    }

    completed
}

pub fn update_healing_plume_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut HealingPlumeIndicator, &mut Transform)>,
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

/// Applies periodic healing to all non-corpse units within the healing plume zone.
/// Drought synergy: healing is reduced on dry units.
pub fn apply_healing_plume_heal(
    time: Res<Time>,
    mut zones: Query<&mut HealingPlumeZone>,
    mut targets: Query<
        (
            &Transform,
            &mut Health,
            Has<crate::game::units::wizard::archetypes::meteorologist::components::DryModifier>,
        ),
        Without<Corpse>,
    >,
) {
    use crate::game::units::wizard::archetypes::meteorologist::systems::apply_dry_healing_reduction;

    let delta = time.delta_secs();

    for mut zone in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;

        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;

            for (transform, mut health, is_dry) in &mut targets {
                let distance = Vec3::new(
                    zone.origin.x - transform.translation.x,
                    0.0,
                    zone.origin.z - transform.translation.z,
                )
                .length();

                if distance <= zone.radius {
                    health.heal(apply_dry_healing_reduction(zone.heal_per_tick, is_dry));
                }
            }
        }
    }
}

/// Fades healing plume zone visual over the last few seconds.
pub fn fade_healing_plume_zone(
    zones: Query<(&HealingPlumeZone, &MeshMaterial3d<StandardMaterial>)>,
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

        material.base_color = Color::srgba(0.1, 0.7, 0.2, 0.4 * fade);
    }
}

/// Despawns expired healing plume zones.
pub fn cleanup_healing_plume_zone(
    mut commands: Commands,
    zones: Query<(Entity, &HealingPlumeZone)>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            commands.entity(entity).try_despawn();
        }
    }
}

pub(crate) fn spawn_healing_plume_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
) {
    let duration = constants::ZONE_DURATION * empowerment;
    let heal = constants::HEAL_PER_TICK * empowerment;

    let base_mat = materials
        .get(&assets.healing_plume_zone)
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
        HealingPlumeZone::new(
            Vec3::new(position.x, 0.0, position.z),
            radius,
            heal,
            constants::TICK_INTERVAL,
            duration,
        ),
        NetworkedSpellEffect {
            kind: SpellEffectKind::HealingPlumeZone,
        },
        OnGameplayScreen,
    ));
}
