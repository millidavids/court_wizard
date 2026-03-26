use bevy::math::Affine2;
use bevy::prelude::*;

use super::archetypes::arcanorouter::ArcanoRouterBonuses;
use super::components::*;
use super::constants;
use super::messages::*;
use super::spells::magic_missile_constants;
use crate::config::{GameConfig, WizardType};
use crate::game::cauldron::resources::CauldronBuffs;
use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::MouseButtonState;
use crate::game::units::components::{Health, Hitbox, MovementSpeed, Team};
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Loads the wizard sprite sheet texture.
pub fn load_wizard_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite_texture = asset_server.load("images/wizard_idle-128px-9.png");
    commands.insert_resource(WizardAssets { sprite_texture });
}

/// Sets up the wizard when entering the InGame state.
///
/// Spawns the wizard entity as an animated sprite billboard on the castle platform.
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
    if config.wizard_type == WizardType::BoringOleMage {
        wizard.spell_range = constants::DEFAULT_SPELL_RANGE * 1.05;
        wizard.mana_cost_multiplier = 0.95; // 5% cheaper
        wizard.spell_power_multiplier = 1.05;
        wizard.cast_speed_multiplier = 1.05;
    } else if config.wizard_type == WizardType::Shepherd {
        wizard.spell_power_multiplier =
            super::archetypes::shepherd::constants::SPELL_POWER_MULTIPLIER;
    }

    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(quad_mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(WIZARD_POSITION),
        hitbox,
        Health::new(constants::HEALTH),
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

    // Warglock doesn't use spells — skip priming the default spell
    if config.wizard_type != WizardType::Warglock {
        entity_commands.insert(magic_missile_constants::PRIMED_MAGIC_MISSILE);
    }

    entity_commands.insert(Team::Defenders);

    // Add archetype-specific components
    if config.wizard_type == WizardType::Arcanorouter {
        entity_commands.insert(ArcanoRouterBonuses::default());
    }
}

/// Updates the wizard sprite sheet animation.
pub fn update_wizard_animation(
    time: Res<Time>,
    mut wizard_query: Query<
        (&mut WizardAnimation, &MeshMaterial3d<StandardMaterial>),
        With<Wizard>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const FRAME_SCALE: f32 = 1.0 / constants::WIZARD_SPRITE_GRID_SIZE as f32;
    let frame_scale_vec = Vec2::splat(FRAME_SCALE);

    for (mut animation, material_handle) in &mut wizard_query {
        if animation.tick(time.delta_secs())
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
}

/// Regenerates wizard mana over time, scaled by cauldron buffs.
pub fn regenerate_mana(
    time: Res<Time>,
    cauldron_buffs: Res<CauldronBuffs>,
    infinite_mana: Res<crate::ui::action_bar::InfiniteMana>,
    mut wizards: Query<(&mut Mana, &ManaRegen), With<Wizard>>,
) {
    for (mut mana, regen) in &mut wizards {
        if infinite_mana.0 {
            mana.current = mana.max;
        } else {
            let rate = regen.rate * cauldron_buffs.mana_regen_multiplier();
            mana.regenerate(rate * time.delta_secs());
        }
    }
}

/// Handles PrimeSpellMessage to update the wizard's primed spell.
/// This allows UI systems to request spell changes without directly accessing components.
/// Applies cauldron spell power buff as a base empowerment multiplier.
pub fn handle_prime_spell_messages(
    mut messages: MessageReader<PrimeSpellMessage>,
    cauldron_buffs: Res<CauldronBuffs>,
    mut wizard_query: Query<&mut PrimedSpell, With<LocalWizard>>,
    active_talents: Option<Res<ActiveTalents>>,
) {
    for message in messages.read() {
        if let Ok(mut primed_spell) = wizard_query.single_mut() {
            let mut spell = message.spell;
            if cauldron_buffs.spell_power_multiplier() > 1.0 {
                spell = spell
                    .with_empowerment(spell.empowerment * cauldron_buffs.spell_power_multiplier());
            }
            let cast_speed = cauldron_buffs.cast_speed_multiplier();
            if cast_speed > 1.0 {
                spell.cast_time /= cast_speed;
            }
            let range_mult = cauldron_buffs.spell_range_multiplier();
            if range_mult > 1.0 {
                spell.range_multiplier *= range_mult;
            }
            // Apply talent-based cast time modifiers
            if let Some(ref talents) = active_talents {
                let mult = talent_cast_time_multiplier(spell.spell, talents);
                if mult < 1.0 {
                    spell.cast_time *= mult;
                }
            }
            *primed_spell = spell;
        }
    }
}

/// Returns the talent-based cast time multiplier for a spell (1.0 = no change).
fn talent_cast_time_multiplier(spell: Spell, talents: &ActiveTalents) -> f32 {
    match spell {
        Spell::BlackHole => {
            if talents.get_selection(Spell::BlackHole, 0) == Some(2) {
                super::spells::black_hole_constants::QUICK_COLLAPSE_CAST_TIME_MULT
            } else {
                1.0
            }
        }
        _ => 1.0,
    }
}

/// Manages empowerment consumption and reset based on casting state transitions.
///
/// When a cast starts (transition to Casting), marks empowerment as consumed.
/// When returning to Resting, resets empowerment if it was previously consumed.
/// This ensures empowerment only applies to a single cast (including full channel duration).
pub fn reset_empowerment_after_cast(
    mut wizard_query: Query<
        (&CastingState, &mut PrimedSpell),
        (With<Wizard>, Changed<CastingState>),
    >,
) {
    for (casting_state, mut primed_spell) in &mut wizard_query {
        match casting_state {
            // When starting a cast, mark empowerment as consumed
            CastingState::Casting { .. } => {
                primed_spell.consume_empowerment();
            }
            // When returning to rest, reset empowerment if it was consumed
            CastingState::Resting => {
                if primed_spell.should_reset_empowerment() {
                    primed_spell.reset_empowerment();
                }
            }
            // Channeling state doesn't affect empowerment
            CastingState::Channeling { .. } => {}
        }
    }
}

/// Cancels any active casting when leaving the Running state.
///
/// Prevents spells from continuing to cast when entering menus or paused state.
/// Also resets the mouse button state to prevent lingering input.
pub fn cancel_active_casts(
    mut wizard_query: Query<&mut CastingState, With<Wizard>>,
    mut mouse_state: ResMut<MouseButtonState>,
) {
    for mut casting_state in &mut wizard_query {
        if !matches!(*casting_state, CastingState::Resting) {
            casting_state.cancel();
        }
    }
    // Reset mouse state when exiting running state
    mouse_state.left_consumed = false;
}

/// Applies the wizard's stat multipliers to the currently primed spell.
///
/// Runs for all archetypes whenever the Wizard component changes. Recalculates
/// the effective cast time and empowerment based on the wizard's multipliers.
/// For archetypes with default 1.0 multipliers this is effectively a no-op.
pub fn apply_wizard_stats_to_primed_spell(
    mut wizard_query: Query<(&Wizard, &mut PrimedSpell), Changed<Wizard>>,
) {
    for (wizard, mut primed_spell) in wizard_query.iter_mut() {
        let base_cast_time = primed_spell.spell.primed_config().cast_time;
        primed_spell.cast_time = base_cast_time / wizard.cast_speed_multiplier;
        primed_spell.empowerment = wizard.spell_power_multiplier;
    }
}
