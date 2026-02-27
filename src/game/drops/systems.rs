use bevy::prelude::*;

use rand::Rng;
use rand::seq::SliceRandom;

use crate::config::save_data::{load_unified_save, unlock_ingredient};
use crate::game::cauldron::brews::Ingredient;
use crate::game::components::OnGameplayScreen;
use crate::game::constants::WIZARD_POSITION;
use crate::game::messages::IngredientCollectedMessage;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

use super::components::{FlyingToWizard, IngredientDrop, LockedIngredients};
use super::constants::*;
use super::messages::SpawnIngredientDropMessage;

/// Initializes the locked ingredients cache from save data.
pub(super) fn init_locked_ingredients(mut locked: ResMut<LockedIngredients>) {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.ingredients)
        .unwrap_or_default();

    locked.locked = Ingredient::all()
        .iter()
        .filter(|ingredient| {
            let debug_name = format!("{:?}", ingredient);
            !unlocked.contains(&debug_name)
        })
        .copied()
        .collect();
}

/// Listens for enemy death messages and randomly spawns ingredient drops.
pub(super) fn spawn_ingredient_drops(
    mut commands: Commands,
    spell_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drop_events: MessageReader<SpawnIngredientDropMessage>,
    locked: Res<LockedIngredients>,
) {
    if locked.locked.is_empty() {
        // All ingredients unlocked, consume events and skip
        for _ in drop_events.read() {}
        return;
    }

    let mut rng = rand::thread_rng();

    for event in drop_events.read() {
        // Roll for drop chance
        if rng.r#gen::<f64>() > DROP_CHANCE {
            continue;
        }

        // Pick a random locked ingredient
        let Some(ingredient) = locked.locked.choose(&mut rng).copied() else {
            continue;
        };

        let drop_material = materials.add(StandardMaterial {
            base_color: DROP_COLOR,
            emissive: DROP_EMISSIVE.into(),
            unlit: true,
            ..default()
        });

        commands.spawn((
            Mesh3d(spell_assets.cross_plane_sphere.clone()),
            MeshMaterial3d(drop_material),
            Transform::from_xyz(event.position.x, DROP_BASE_Y, event.position.z)
                .with_scale(Vec3::splat(DROP_CUBE_SIZE / 2.0)),
            IngredientDrop {
                ingredient,
                time_alive: 0.0,
            },
            OnGameplayScreen,
        ));
    }
}

/// Animates drops with bobbing and size pulsing.
pub(super) fn animate_drops(
    mut drops: Query<(&IngredientDrop, &mut Transform), Without<FlyingToWizard>>,
) {
    for (drop, mut transform) in &mut drops {
        // Bobbing
        let bob_offset =
            (drop.time_alive * BOB_FREQUENCY * std::f32::consts::TAU).sin() * BOB_AMPLITUDE;
        transform.translation.y = DROP_BASE_Y + bob_offset;

        // Size pulse
        let pulse = 1.0
            + (drop.time_alive * PULSE_FREQUENCY * std::f32::consts::TAU).sin() * PULSE_AMPLITUDE;
        transform.scale = Vec3::splat(pulse);
    }
}

/// Ticks drop lifetimes for animation purposes.
pub(super) fn tick_drop_lifetimes(time: Res<Time>, mut drops: Query<&mut IngredientDrop>) {
    for mut drop in &mut drops {
        drop.time_alive += time.delta_secs();
    }
}

/// Moves flying drops toward the wizard and collects them on arrival.
pub(super) fn fly_drops_to_wizard(
    mut commands: Commands,
    time: Res<Time>,
    mut flying: Query<(Entity, &FlyingToWizard, &mut Transform)>,
    mut locked: ResMut<LockedIngredients>,
    mut collected_msg: MessageWriter<IngredientCollectedMessage>,
) {
    for (entity, flying_drop, mut transform) in &mut flying {
        let direction = WIZARD_POSITION - transform.translation;
        let distance = direction.length();

        if distance < COLLECTION_DISTANCE {
            // Collect the ingredient
            unlock_ingredient(flying_drop.ingredient);
            locked.locked.retain(|i| *i != flying_drop.ingredient);
            collected_msg.write(IngredientCollectedMessage {
                ingredient: flying_drop.ingredient,
            });
            commands.entity(entity).despawn();
            continue;
        }

        // Move toward wizard
        let move_distance = FLY_SPEED * time.delta_secs();
        let normalized = direction.normalize();
        transform.translation += normalized * move_distance;

        // Add arc: raise Y based on progress
        let progress = 1.0 - (distance / flying_drop.total_distance).clamp(0.0, 1.0);
        let arc = (progress * std::f32::consts::PI).sin() * FLY_ARC_HEIGHT;
        // Blend between ground level and wizard Y, plus arc
        let base_y =
            flying_drop.start_pos.y + (WIZARD_POSITION.y - flying_drop.start_pos.y) * progress;
        transform.translation.y = base_y + arc;
    }
}
