//! Game-mode lifecycle and toggle systems.

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::constants::endless_effectiveness_bonus;
use crate::game::resources::CurrentLevel;
use crate::game::units::components::{Effectiveness, Health, Team, TemporaryHitPoints};

use super::components::{
    ActiveToggles, FortifiedHordeShield, GameMode, RogueliteModifiers, RogueliteRunState,
    ToggleModifier, is_endless_mode, is_roguelite_mode,
};

pub(super) fn cleanup_game_mode(mut commands: Commands) {
    commands.remove_resource::<GameMode>();
    commands.remove_resource::<RogueliteRunState>();
    commands.remove_resource::<RogueliteModifiers>();
    commands.remove_resource::<ActiveToggles>();
    commands.remove_resource::<super::components::LastCastSpell>();
    commands.remove_resource::<super::components::WizardCycleTimer>();
    commands.remove_resource::<super::components::AttritionState>();
}

/// Initializes toggle-specific resources when entering a game.
pub(super) fn init_toggle_resources(
    mut commands: Commands,
    toggles: Option<Res<ActiveToggles>>,
    config: Res<GameConfig>,
) {
    if toggles
        .as_ref()
        .is_some_and(|t| t.is_active(ToggleModifier::SpellRotation))
    {
        commands.insert_resource(super::components::LastCastSpell::default());
    }

    // Attrition: initialize survivor counts (only if not already set from a prior level)
    if toggles
        .as_ref()
        .is_some_and(|t| t.is_active(ToggleModifier::Attrition))
    {
        // Only insert if not already present (preserves counts between levels)
        commands.init_resource::<super::components::AttritionState>();
    }

    if toggles
        .as_ref()
        .is_some_and(|t| t.is_active(ToggleModifier::WizardCycle))
    {
        // Get unlocked wizard types from save data
        let unlocked_names = crate::config::save_data::load_unified_save()
            .map(|s| s.player.unlocked_content.wizard_types)
            .unwrap_or_default();
        let unlocked_types: Vec<crate::config::WizardType> = crate::config::WizardType::all()
            .iter()
            .filter(|wt| {
                unlocked_names
                    .iter()
                    .any(|name| name == &format!("{:?}", wt))
            })
            .copied()
            .collect();

        // Find current wizard's index in the unlocked list
        let current_index = unlocked_types
            .iter()
            .position(|wt| *wt == config.wizard_type)
            .unwrap_or(0);

        commands.insert_resource(super::components::WizardCycleTimer {
            timer: super::components::WizardCycleTimer::INTERVAL,
            unlocked_types,
            current_index,
        });
    }
}

/// Initializes a new roguelite run when entering MetaGame in Roguelite mode.
/// Only runs if no existing run is in progress (i.e., first entry, not returning from battle).
pub(super) fn init_roguelite_run(
    mut config: ResMut<GameConfig>,
    mut current_level: ResMut<CurrentLevel>,
    mut commands: Commands,
    game_mode: Option<Res<GameMode>>,
    existing_run: Option<Res<RogueliteRunState>>,
) {
    if !is_roguelite_mode(game_mode.as_deref()) {
        return;
    }
    if existing_run.is_some() {
        return; // Run already in progress (returning from a battle victory)
    }

    // Fresh roguelite run: reset level and transient state
    config.current_level = 1;
    current_level.0 = 1;
    config.saved_walls.clear();
    config.saved_crystals.clear();
    config.saved_flora.clear();
    config.saved_trampling = Default::default();
    config.saved_trees.clear();
    config.saved_ponds.clear();
    config.saved_bushes.clear();
    config.saved_boulders.clear();
    config.efficiency_ratios.clear();

    commands.insert_resource(RogueliteRunState {
        started_at: crate::config::save_data::current_timestamp(),
        level_stats: vec![],
    });
}

/// Applies an effectiveness bonus to all attacker units in Endless mode past the final
/// introduction tier. This makes each level progressively harder.
pub(super) fn apply_endless_scaling(
    game_mode: Option<Res<GameMode>>,
    current_level: Res<CurrentLevel>,
    mut attackers: Query<(&mut Effectiveness, &Team)>,
) {
    if !is_endless_mode(game_mode.as_deref()) {
        return;
    }
    let bonus = endless_effectiveness_bonus(current_level.0);
    if bonus <= 0.0 {
        return;
    }
    for (mut eff, team) in &mut attackers {
        if *team != Team::Attackers {
            continue;
        }
        // Boost attacker base effectiveness — the recalculate system will
        // propagate this to `current` via the normal formula.
        eff.base += bonus;
    }
}

/// Applies the roguelite enemy effectiveness modifier to all attacker units.
/// Multiplies base effectiveness by the modifier value (1.0 = no change).
pub(super) fn apply_roguelite_effectiveness(
    modifiers: Option<Res<RogueliteModifiers>>,
    mut attackers: Query<(&mut Effectiveness, &Team)>,
) {
    let Some(modifiers) = modifiers else {
        return;
    };
    let mult = modifiers.enemy_effectiveness;
    if (mult - 1.0).abs() < f32::EPSILON {
        return;
    }
    for (mut eff, team) in &mut attackers {
        if *team != Team::Attackers {
            continue;
        }
        eff.base *= mult;
    }
}

/// Applies Glass Cannon and Veteran Defenders toggle effects to defenders.
pub(super) fn apply_defender_toggles(
    toggles: Option<Res<ActiveToggles>>,
    mut defenders: Query<(&mut Effectiveness, &mut Health, &Team)>,
) {
    let Some(ref t) = toggles else {
        return;
    };
    let glass_cannon = t.is_active(ToggleModifier::GlassCannon);
    let veteran = t.is_active(ToggleModifier::VeteranDefenders);
    if !glass_cannon && !veteran {
        return;
    }
    for (mut eff, mut health, team) in &mut defenders {
        if *team != Team::Defenders {
            continue;
        }
        if glass_cannon {
            eff.base *= 2.0;
            health.max *= 0.5;
            health.current = health.current.min(health.max);
        }
        if veteran {
            eff.base *= 1.5;
            health.max *= 2.5;
            health.current = health.max;
        }
    }
}

/// Fortified Horde toggle: first-wave attackers spawn with temporary damage shields
/// and a pulsing yellow glow.
pub(super) fn apply_fortified_horde(
    toggles: Option<Res<ActiveToggles>>,
    mut commands: Commands,
    attackers: Query<(Entity, &Team), Without<TemporaryHitPoints>>,
) {
    if !toggles
        .as_ref()
        .is_some_and(|t| t.is_active(ToggleModifier::FortifiedHorde))
    {
        return;
    }
    const SHIELD_AMOUNT: f32 = 50.0;
    const SHIELD_DURATION: f32 = 60.0;
    for (entity, team) in &attackers {
        if *team != Team::Attackers {
            continue;
        }
        commands.entity(entity).insert((
            TemporaryHitPoints::new(SHIELD_AMOUNT, SHIELD_DURATION),
            FortifiedHordeShield,
        ));
    }
}

/// Pulses a yellow emissive glow on enemies with the Fortified Horde shield.
pub(super) fn animate_fortified_horde_glow(
    time: Res<Time>,
    shielded: Query<&MeshMaterial3d<StandardMaterial>, With<FortifiedHordeShield>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if shielded.is_empty() {
        return;
    }
    let t = time.elapsed_secs();
    // Pulse between dim and bright yellow (2 Hz)
    let intensity = (t * std::f32::consts::TAU * 2.0).sin() * 0.5 + 0.5;
    let glow = intensity * 4.0;

    for mat_handle in &shielded {
        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.emissive = bevy::color::LinearRgba::new(glow, glow * 0.8, 0.0, 1.0);
        }
    }
}

/// Removes the FortifiedHordeShield marker and resets emissive when shield expires.
pub(super) fn cleanup_fortified_horde_glow(
    mut commands: Commands,
    shielded: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        (With<FortifiedHordeShield>, Without<TemporaryHitPoints>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mat_handle) in &shielded {
        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.emissive = bevy::color::LinearRgba::new(0.0, 0.0, 0.0, 1.0);
        }
        commands.entity(entity).remove::<FortifiedHordeShield>();
    }
}

/// Attrition toggle: at end of level, count surviving defenders and update AttritionState.
pub(super) fn update_attrition_survivors(
    mut attrition: Option<ResMut<super::components::AttritionState>>,
    infantry: Query<
        (),
        (
            With<crate::game::units::infantry::Infantry>,
            With<Team>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
    archers: Query<
        (),
        (
            With<crate::game::units::archer::components::Archer>,
            With<Team>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
    guards: Query<
        (),
        (
            With<crate::game::units::components::KingsGuard>,
            Without<crate::game::units::components::Corpse>,
        ),
    >,
) {
    let Some(ref mut state) = attrition else {
        return;
    };
    // Count living defenders (infantry includes king's guard, so subtract guards)
    let guard_count = guards.iter().count() as u32;
    let total_infantry = infantry.iter().count() as u32;
    // Infantry query includes KingsGuard (they also have Infantry component),
    // so subtract guards to get pure infantry count
    state.infantry = total_infantry.saturating_sub(guard_count);
    state.archers = archers.iter().count() as u32;
    state.guards = guard_count;
}

/// Wizard Cycle toggle: counts down and swaps wizard archetype when timer expires.
/// Handles announcement, archetype UI despawn/spawn, and Excremage color refresh.
#[allow(clippy::too_many_arguments)]
pub(super) fn tick_wizard_cycle(
    time: Res<Time>,
    mut commands: Commands,
    mut cycle_timer: Option<ResMut<super::components::WizardCycleTimer>>,
    mut config: ResMut<GameConfig>,
    archetype_ui: Query<Entity, With<super::components::ArchetypeUI>>,
    existing_flash: Query<Entity, With<super::components::WizardCycleFlash>>,
    asset_server: Res<AssetServer>,
    arcanorouter_state: Option<
        Res<crate::game::units::wizard::archetypes::arcanorouter::resources::ArcanoRouterState>,
    >,
    wizard_entity_query: Query<Entity, With<crate::game::units::wizard::components::LocalWizard>>,
) {
    let Some(ref mut timer) = cycle_timer else {
        return;
    };
    if timer.unlocked_types.len() <= 1 {
        return;
    }

    timer.timer -= time.delta_secs();
    if timer.timer > 0.0 {
        return;
    }

    // Timer expired — cycle to next archetype
    timer.timer = super::components::WizardCycleTimer::INTERVAL;
    timer.current_index = (timer.current_index + 1) % timer.unlocked_types.len();
    let new_type = timer.unlocked_types[timer.current_index];
    config.wizard_type = new_type;

    // Despawn all old archetype UI
    for entity in &archetype_ui {
        commands.entity(entity).try_despawn();
    }

    // Remove archetype-specific components from wizard
    if let Ok(wizard_entity) = wizard_entity_query.single() {
        commands
            .entity(wizard_entity)
            .remove::<crate::game::units::wizard::archetypes::arcanorouter::ArcanoRouterBonuses>();
    }

    // Spawn new archetype UI
    match new_type {
        crate::config::WizardType::RuneCaster => {
            crate::ui::rune_display::systems::spawn_rune_display(commands.reborrow());
        }
        crate::config::WizardType::Randomancer => {
            crate::ui::roulette_display::systems::spawn_roulette_display(
                commands.reborrow(),
                Res::clone(&asset_server),
            );
        }
        crate::config::WizardType::Arcanorouter => {
            if let Some(ref state) = arcanorouter_state {
                crate::ui::arcanorouter_display::systems::spawn_arcanorouter_display(
                    commands.reborrow(),
                    Res::clone(state),
                );
            }
            // Ensure the wizard entity has ArcanoRouterBonuses
            if let Ok(wizard_entity) = wizard_entity_query.single() {
                commands.entity(wizard_entity).insert(
                    crate::game::units::wizard::archetypes::arcanorouter::ArcanoRouterBonuses::default(),
                );
            }
        }
        crate::config::WizardType::Swordcerer => {
            crate::game::units::wizard::archetypes::swordcerer::systems::spawn_enter_fray_button(
                commands.reborrow(),
            );
        }
        crate::config::WizardType::Warglock => {
            // Initialize GunState if not already present
            commands.init_resource::<crate::game::units::wizard::archetypes::gunslinger::resources::GunState>();
            commands.init_resource::<crate::game::units::wizard::archetypes::gunslinger::resources::FlamethrowerSfx>();
        }
        _ => {} // BoringOleMage, Excremage, etc. have no special UI
    }

    // Flash announcement
    for entity in &existing_flash {
        commands.entity(entity).try_despawn();
    }
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(15.0),
            left: Val::Percent(5.0),
            width: Val::Percent(90.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(format!("Wizard: {}", new_type.display_name())),
        TextFont::from_font_size(28.0),
        TextColor(Color::srgba(0.7, 0.5, 1.0, 1.0)),
        super::components::WizardCycleFlash { timer: 3.0 },
        Pickable::IGNORE,
        GlobalZIndex(998),
        crate::game::components::OnGameplayScreen,
    ));
}

/// Fades and despawns the wizard cycle flash banner.
pub(super) fn update_wizard_cycle_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(
        Entity,
        &mut super::components::WizardCycleFlash,
        &mut TextColor,
    )>,
) {
    for (entity, mut flash, mut text_color) in &mut flash_query {
        flash.timer -= time.delta_secs();
        if flash.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        } else if flash.timer < 1.0 {
            // Fade out over last second
            text_color.0 = text_color.0.with_alpha(flash.timer);
        }
    }
}
