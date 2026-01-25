use bevy::prelude::*;

use super::components::*;
use super::constants;
use super::spells::magic_missile_constants;
use super::styles::*;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::units::components::{Health, Hitbox, MovementSpeed};

/// Sets up the wizard when entering the InGame state.
///
/// Spawns the wizard entity as a triangle on the castle platform in 3D space.
pub fn setup_wizard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Define wizard hitbox (cylinder) - this determines sprite size
    let hitbox = Hitbox::new(constants::HITBOX_RADIUS, constants::HITBOX_HEIGHT);

    // Spawn wizard as a triangle billboard sized to match the hitbox
    let wizard_width = hitbox.sprite_width();
    let wizard_height = hitbox.sprite_height();
    let wizard_triangle = Triangle2d::new(
        Vec2::new(0.0, wizard_height / 2.0), // Top vertex
        Vec2::new(-wizard_width / 2.0, -wizard_height / 2.0), // Bottom-left
        Vec2::new(wizard_width / 2.0, -wizard_height / 2.0), // Bottom-right
    );

    commands.spawn((
        Mesh3d(meshes.add(wizard_triangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WIZARD_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(WIZARD_POSITION),
        hitbox,
        Health::new(constants::HEALTH),
        MovementSpeed::new(0.0), // Wizard doesn't move
        Mana::new(constants::MANA),
        ManaRegen::new(constants::MANA_REGEN),
        CastingState::new(),
        Wizard::new(constants::DEFAULT_SPELL_RANGE),
        magic_missile_constants::PRIMED_MAGIC_MISSILE,
        OnGameplayScreen,
    ));
}

/// Regenerates wizard mana over time.
pub fn regenerate_mana(time: Res<Time>, mut wizards: Query<(&mut Mana, &ManaRegen), With<Wizard>>) {
    for (mut mana, regen) in &mut wizards {
        mana.regenerate(regen.rate * time.delta_secs());
    }
}

/// Handles PrimeSpellMessage to update the wizard's primed spell.
/// This allows UI systems to request spell changes without directly accessing components.
pub fn handle_prime_spell_messages(
    mut messages: MessageReader<PrimeSpellMessage>,
    mut wizard_query: Query<&mut PrimedSpell, With<Wizard>>,
) {
    for message in messages.read() {
        if let Ok(mut primed_spell) = wizard_query.single_mut() {
            *primed_spell = message.spell;
        }
    }
}
