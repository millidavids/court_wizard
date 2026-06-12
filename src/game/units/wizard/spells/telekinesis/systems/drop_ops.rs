use super::super::constants;
use super::talents::TelekinesisConfig;
use super::vfx_systems::apply_harvest_damage;
use crate::config::GameConfig;
use crate::game::drops::components::{FlyingToWizard, IngredientDrop};
use crate::game::units::components::{Health, Team, TemporaryHitPoints};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Converts an ingredient drop entity to the flying-to-wizard state.
pub(super) fn convert_drop_to_flying(
    commands: &mut Commands,
    drop_entity: Entity,
    ingredient: crate::game::cauldron::brews::Ingredient,
    position: Vec3,
) {
    let total_distance = position.distance(crate::game::constants::WIZARD_POSITION);
    commands
        .entity(drop_entity)
        .remove::<IngredientDrop>()
        .insert(FlyingToWizard {
            ingredient,
            start_pos: position,
            total_distance,
        });
}

/// Finds the nearest ingredient drop within the given radius of the cursor position.
pub(super) fn find_nearest_drop<'a>(
    cursor_pos: &Vec3,
    drops: &'a Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    pickup_radius: f32,
) -> Option<(Entity, &'a Transform, &'a IngredientDrop)> {
    let mut nearest: Option<(Entity, &Transform, &IngredientDrop, f32)> = None;

    for (entity, transform, drop) in drops.iter() {
        let dx = transform.translation.x - cursor_pos.x;
        let dz = transform.translation.z - cursor_pos.z;
        let distance = (dx * dx + dz * dz).sqrt();

        if distance <= pickup_radius
            && (nearest.is_none()
                || distance
                    < nearest
                        .as_ref()
                        .expect("nearest is Some because is_none() was false")
                        .3)
        {
            nearest = Some((entity, transform, drop, distance));
        }
    }

    nearest.map(|(e, t, d, _)| (e, t, d))
}

/// Telekinetic Storm: pick up all drops on the battlefield.
/// Picks up as many as mana allows, nearest first.
#[allow(clippy::too_many_arguments)]
pub(super) fn execute_storm_pickup(
    mana: &mut crate::game::units::wizard::components::Mana,
    config: &TelekinesisConfig,
    drops_query: &Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    commands: &mut Commands,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    enemies_query: &mut Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<IngredientDrop>,
    >,
    visual_assets: &SpellVisualAssets,
) -> bool {
    let total_cost = config.mana_cost * constants::STORM_MANA_COST_MULT;

    // Collect all drops (order doesn't matter — all are picked up)
    let all_drops: Vec<(Entity, Vec3, crate::game::cauldron::brews::Ingredient)> = drops_query
        .iter()
        .map(|(e, t, d)| (e, t.translation, d.ingredient))
        .collect();

    if all_drops.is_empty() || !mana.consume(total_cost) {
        return false;
    }

    let mut picked_any = false;
    let mut played_sfx = false;

    for (drop_entity, drop_pos, ingredient) in &all_drops {
        let start_pos = *drop_pos;
        convert_drop_to_flying(commands, *drop_entity, *ingredient, start_pos);

        // Play SFX once
        if !played_sfx {
            audio::play_sfx(commands, &sfx.telekinesis_cast, start_pos, game_config, sfx);
            played_sfx = true;
        }

        // T2: Harvest — damage nearby enemies per pickup
        if config.has_harvest {
            apply_harvest_damage(commands, start_pos, visual_assets, enemies_query);
        }

        picked_any = true;
    }

    picked_any
}
