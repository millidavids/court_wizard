//! Hit detection for chain lightning spells absorbed by crystals.

use super::super::setup::{
    find_random_targets_in_range, increment_resonance, scaled_count, spell_echo_multiplier,
};
use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::chain_lightning::systems as chain_lightning_systems;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;

/// Detects chain lightning hitting crystals and emits lightning arcs.
/// This is called when chain lightning bounces to a crystal (crystal is added
/// as a valid bounce target in the chain lightning systems).
#[allow(clippy::too_many_arguments)]
pub(crate) fn detect_chain_lightning_hits(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    mut crystals: Query<
        (Entity, &mut ArcaneCrystal, Option<&mut ResonanceCascade>),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    bolts: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningBolt,
    >,
    groups: Query<
        &crate::game::units::wizard::spells::chain_lightning::components::ChainLightningGroup,
    >,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    mut progress: ResMut<BattleTalentProgress>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    // Check if any bolt's last_hit_position matches a crystal position
    // (chain lightning system sets crystal as a bounce target, so the bolt
    // will have the crystal's position as last_hit_position after bouncing to it)
    for (crystal_entity, mut crystal, mut resonance) in &mut crystals {
        if crystal.permanent {
            continue;
        }
        for bolt in &bolts {
            // Check if this bolt just bounced to this crystal
            let dist = bolt.last_hit_position.distance(crystal.position);
            if dist > crystal.collision_radius {
                continue;
            }

            // Check if we're in the group's hit list (meaning we were targeted)
            let Ok(group) = groups.get(bolt.group_entity) else {
                continue;
            };
            if !group.hit_entities.contains(&crystal_entity) {
                continue;
            }

            crystal.mark_absorption();
            crystal.remembered_spell = Some(RememberedSpell::ChainLightning);
            crystal.auto_cast_timer = 0.0;

            let rng = &mut game_rng.0;
            let echo_mult = spell_echo_multiplier(rng, crystal.spell_echo);
            let count = scaled_count(LIGHTNING_ARC_COUNT, crystal.count_mult) * echo_mult;

            // Emit arcs to random targets
            let enemies =
                find_random_targets_in_range(rng, crystal.position, crystal.range, count, &targets);

            let damage = bolt.current_damage * DAMAGE_SCALE * crystal.damage_mult;

            for (target_entity, target_pos) in &enemies {
                // Apply damage
                if let Ok((mut health, mut temp_hp, has_spell_shield, team)) =
                    health_query.get_mut(*target_entity)
                {
                    apply_spell_damage_with_team(
                        &mut commands,
                        *target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        DamageType::Electric,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                }

                // Spawn arc visual using shared chain lightning helper
                chain_lightning_systems::spawn_arc(
                    &mut commands,
                    &visual_assets,
                    crystal.position,
                    *target_pos,
                    0,
                    crystal.empowerment,
                );
            }

            // Track progress
            progress.increment(Spell::ArcaneCrystal, count as u32);

            // Resonance cascade
            increment_resonance(&mut resonance);

            break; // Only process once per crystal per frame
        }
    }
}
