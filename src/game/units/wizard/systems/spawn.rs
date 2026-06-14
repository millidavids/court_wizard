use bevy::math::Affine2;
use bevy::prelude::*;

use super::super::archetypes::arcanorouter::ArcanoRouterBonuses;
use super::super::components::*;
use super::super::constants;
use super::super::spells::magic_missile_constants;
use crate::config::{GameConfig, WizardType};
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::constants::WIZARD_POSITION;
use crate::game::units::components::{Health, Hitbox, Invulnerable, MovementSpeed, Team};

/// Sets up the wizard when entering the InGame state.
///
/// Spawns the wizard entity as an animated sprite billboard on the castle platform.
/// Applies archetype-identity stat bonuses to a freshly built `Wizard`.
///
/// Shared by single-player `setup_wizard` and multiplayer `spawn_mp_wizard` so
/// the two never drift apart. This covers ONLY archetype *identity* multipliers
/// (the flat bonuses that define how an archetype plays). Progression bonuses
/// (`InsightBonuses`) are deliberately NOT applied here: single-player layers
/// them on afterward, while multiplayer leaves them off so a player's unlock
/// progress can't tilt a competitive match.
pub(crate) fn apply_archetype_stat_bonuses(wizard: &mut Wizard, wizard_type: WizardType) {
    match wizard_type {
        WizardType::BoringOleMage => {
            wizard.spell_range = constants::DEFAULT_SPELL_RANGE * 1.05;
            wizard.mana_cost_multiplier = 0.95; // 5% cheaper
            wizard.spell_power_multiplier = 1.05;
            wizard.cast_speed_multiplier = 1.05;
        }
        WizardType::Shepherd => {
            wizard.spell_power_multiplier =
                super::super::archetypes::shepherd::SPELL_POWER_MULTIPLIER;
        }
        _ => {}
    }
}

pub fn setup_wizard(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    config: &GameConfig,
    wizard_assets: &WizardAssets,
) {
    let hitbox = Hitbox::new(constants::HITBOX_RADIUS, constants::HITBOX_HEIGHT);

    // Create a quad mesh matching the sprite aspect ratio
    let quad_mesh = Rectangle::new(
        constants::WIZARD_SPRITE_WIDTH,
        constants::WIZARD_SPRITE_HEIGHT,
    );

    // UV transform for first frame: scale to 1/3 to show only one cell
    let grid_size = constants::WIZARD_SPRITE_GRID_SIZE as f32;
    let frame_scale = 1.0 / grid_size;
    let uv_transform = Affine2::from_scale(Vec2::splat(frame_scale));

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(wizard_assets.sprite_texture.clone()),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        uv_transform,
        ..default()
    });

    // Build wizard with archetype-specific stat bonuses
    let mut wizard = Wizard::new(constants::DEFAULT_SPELL_RANGE);
    apply_archetype_stat_bonuses(&mut wizard, config.wizard_type);

    // Apply permanent insight bonuses on top of archetype stats (single-player
    // only — multiplayer omits these to keep matches balanced; see
    // `apply_archetype_stat_bonuses`).
    let insight = crate::game::insight_bonuses::InsightBonuses::from_save();
    wizard.spell_power_multiplier *= insight.spell_damage_mult;
    wizard.spell_range *= insight.spell_range_mult;
    wizard.cast_speed_multiplier *= insight.cast_speed_mult;
    wizard.mana_cost_multiplier *= insight.mana_cost_mult;

    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(quad_mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(WIZARD_POSITION),
        hitbox,
        Health::new(constants::HEALTH),
        // The wizard is a non-combatant caster: it can never take damage or die.
        // `enforce_invulnerability` snapshots and restores its health every frame,
        // before corpse conversion can ever see it as dead.
        Invulnerable {
            health_snapshot: constants::HEALTH,
        },
        MovementSpeed(0.0), // Wizard doesn't move
        Mana::new(constants::MANA),
        ManaRegen::new(constants::MANA_REGEN),
        CastingState::new(),
        wizard,
        LocalWizard,
        WizardAnimation::new(),
        Billboard,
        OnGameplayScreen,
    ));

    // Skip priming Magic Missile for archetypes that shouldn't start with it:
    // Warglock fires guns, the Randomancer only casts what its roulette rolls,
    // and the Shepherd can't cast offensive spells.
    if !matches!(
        config.wizard_type,
        WizardType::Warglock | WizardType::Randomancer | WizardType::Shepherd
    ) {
        entity_commands.insert(magic_missile_constants::PRIMED_MAGIC_MISSILE);
    }

    entity_commands.insert(Team::Defenders);
    // Keep the wizard out of the multiplayer snapshot stream (it carries
    // `Team::Defenders`, which `assign_network_ids` would otherwise pick up).
    entity_commands.insert(crate::game::multiplayer::components::NoSnapshot);

    // Add archetype-specific components
    if config.wizard_type == WizardType::Arcanorouter {
        entity_commands.insert(ArcanoRouterBonuses::default());
    }
}
