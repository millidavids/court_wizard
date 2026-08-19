//! Wizard tower decorations: arcane rune text + run state UI.

use super::setup::*;
use bevy::prelude::*;

use std::f32::consts::TAU;

use bevy::ui::UiTransform;

use crate::game::units::wizard::components::Spell;

use super::super::components::{ArcaneRuneText, OnWizardTowerScreen};
use super::super::constants::*;
use super::super::materials::ArcaneRuneMaterial;

/// Returns all currently unlocked spells from save data.
pub(super) fn spawn_arcane_rune_text(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
) {
    let rune_font: Handle<Font> = asset_server.load("fonts/Oxford-Runes.ttf");
    let unlocked_spells = get_unlocked_spells();

    if unlocked_spells.is_empty() {
        return;
    }

    // Split unlocked spells by category into four concentric rings.
    // Radii sit between the shader's geometric rings:
    //   Shader rings: 0.44, 0.42 (outer), 0.32 (middle), 0.22 (inner), 0.06 (core)
    // Directions alternate CCW / CW / CCW / CW from outermost to innermost.
    use crate::game::units::wizard::components::SpellCategory;
    use crate::ui::constants::spell_category_color;
    let rings: &[(SpellCategory, f32, f32)] = &[
        (SpellCategory::Control, 0.47, -0.04),
        (SpellCategory::Offense, 0.37, 0.06),
        (SpellCategory::Support, 0.27, -0.08),
        (SpellCategory::Utility, 0.16, 0.10),
    ];

    for &(category, radius, speed) in rings {
        let color = spell_category_color(category);
        let spells: Vec<&Spell> = unlocked_spells
            .iter()
            .filter(|s| s.category() == category)
            .collect();
        if spells.is_empty() {
            continue;
        }
        let count = spells.len();
        for (i, spell) in spells.iter().enumerate() {
            let base_angle = (i as f32 / count as f32) * TAU;
            spawn_rune_text_node(
                parent,
                spell.display_name(),
                base_angle,
                radius,
                speed,
                color,
                &rune_font,
            );
        }
    }
}

/// Spawns a single rune text entity at a position on a circle.
pub(super) fn spawn_rune_text_node(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    angle: f32,
    radius: f32,
    speed: f32,
    color: Color,
    font: &Handle<Font>,
) {
    parent.spawn((
        Text::new(label),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(RUNE_TEXT_SIZE),
            ..default()
        },
        TextColor(color.with_alpha(RUNE_TEXT_ALPHA)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
        UiTransform::default(),
        ArcaneRuneText {
            angle,
            radius,
            speed,
        },
    ));
}

/// Rebuilds the orbiting rune text and updates the shader when spells are unlocked.
pub(crate) fn rebuild_rune_on_spell_unlock(
    mut commands: Commands,
    mut spell_researched: MessageReader<crate::game::messages::SpellResearchedMessage>,
    asset_server: Res<AssetServer>,
    root_query: Query<Entity, With<OnWizardTowerScreen>>,
    text_query: Query<Entity, With<ArcaneRuneText>>,
    mut materials: ResMut<Assets<ArcaneRuneMaterial>>,
) {
    // Drain all queued messages this frame; otherwise a multi-spell commit
    // rebuilds over consecutive frames, snap-jumping the angle each time.
    if spell_researched.read().count() == 0 {
        return;
    }

    let unlocked_count = get_unlocked_spells().len() as f32;
    for (_id, mat) in materials.iter_mut() {
        mat.data.unlocked_count = unlocked_count;
    }

    for entity in &text_query {
        commands.entity(entity).despawn();
    }

    let Ok(root) = root_query.single() else {
        return;
    };
    commands.entity(root).with_children(|root| {
        spawn_arcane_rune_text(root, &asset_server);
    });
}

/// Updates the time uniform on the arcane rune background material each frame.
pub(crate) fn update_arcane_rune_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<ArcaneRuneMaterial>>,
) {
    let elapsed = time.elapsed_secs();
    for (_id, mat) in materials.iter_mut() {
        mat.data.time = elapsed;
    }
}

/// Updates positions and rotations of orbiting spell name text around the rune circles.
/// Uses percentage-based positioning so text circles are inherently concentric with
/// the shader's geometric pattern (both reference the same parent coordinate space).
pub(crate) fn update_arcane_rune_text(
    time: Res<Time>,
    bg_query: Query<&ComputedNode, With<super::super::components::ArcaneRuneBackground>>,
    mut text_query: Query<(&ArcaneRuneText, &mut Node, &mut UiTransform, &ComputedNode)>,
) {
    let Ok(bg_computed) = bg_query.single() else {
        return;
    };
    let size = bg_computed.size();
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }
    let ratio = size.y / size.x;
    let elapsed = time.elapsed_secs();

    for (rune_text, mut node, mut ui_transform, text_computed) in &mut text_query {
        let current_angle = rune_text.angle + elapsed * rune_text.speed;
        let text_size = text_computed.size();
        let half_w_pct = text_size.x / size.x * 50.0;
        let half_h_pct = text_size.y / size.y * 50.0;
        let pct_x = 50.0 + rune_text.radius * current_angle.cos() * ratio * 100.0 - half_w_pct;
        let pct_y = 50.0 + rune_text.radius * current_angle.sin() * 100.0 - half_h_pct;
        node.left = Val::Percent(pct_x);
        node.top = Val::Percent(pct_y);
        ui_transform.rotation = Rot2::radians(current_angle + std::f32::consts::FRAC_PI_2);
    }
}

// ---------------------------------------------------------------------------
// Debug: F3 toggle UI visibility
// ---------------------------------------------------------------------------

/// Toggles visibility of the wizard tower UI panels so the background is visible.
#[cfg(debug_assertions)]
pub(crate) fn toggle_debug_background(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_query: Query<&mut Visibility, With<super::super::components::WizardTowerUiContent>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        for mut vis in &mut ui_query {
            *vis = match *vis {
                Visibility::Hidden => Visibility::Inherited,
                _ => Visibility::Hidden,
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Cleanup
// ---------------------------------------------------------------------------

/// Removes wizard tower tab resources when exiting the meta-game.
pub(crate) fn cleanup_wizard_tower_tab_resources(mut commands: Commands) {
    commands.remove_resource::<WizardTowerTab>();
    commands.remove_resource::<RightPanelView>();
}
