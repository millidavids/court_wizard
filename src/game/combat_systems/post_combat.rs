//! Post-combat reactions: invulnerability and corpse conversion.

use bevy::prelude::*;
use rand::Rng;

use super::super::cauldron::components::{
    CauldronDamageBonus, CauldronDamageResistance, CauldronSpeedModifier,
};
use super::super::components::Velocity;
use super::super::units::archer::Archer;
use super::super::units::boss::components::Boss;
use super::super::units::components::{
    AttackTiming, CORPSE_MATERIAL_VARIANTS, Corpse, Flying, Health, Hitbox, Invulnerable,
    MovementSpeed, ResidualFireDamaged, SpellDamaged, Team,
};
use super::super::units::infantry::components::Infantry;
use super::super::units::systems::corpse_material_for_team;

use crate::game::achievements::messages::{
    CloseCallMessage, DefenderKilledBySpellMessage, EnemyKilledMessage, ScorchedEarthMessage,
};
use crate::game::achievements::resources::AchievementResource;

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn enforce_invulnerability(
    mut query: Query<(&mut Invulnerable, &mut Health), Without<Corpse>>,
) {
    for (mut invuln, mut health) in &mut query {
        // Restore health to at least the snapshot (damage negated, heals preserved)
        health.current = health.current.max(invuln.health_snapshot).min(health.max);
        invuln.health_snapshot = health.current;
    }
}

/// When a unit's health reaches zero, this system replaces the unit's material with
/// a pre-loaded corpse material based on team and converts the unit into a corpse
/// that slows living units walking over it.
/// Also records the kill in the kill statistics resource.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn convert_dead_to_corpses(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut kill_stats: ResMut<super::super::resources::KillStats>,
    mut spell_kill_events: MessageWriter<DefenderKilledBySpellMessage>,
    mut enemy_kill_events: MessageWriter<EnemyKilledMessage>,
    mut scorched_earth_events: MessageWriter<ScorchedEarthMessage>,
    mut marked_kill_events: MessageWriter<
        super::super::achievements::messages::MarkedForDeathKillMessage,
    >,
    mut drop_events: MessageWriter<super::super::drops::messages::SpawnIngredientDropMessage>,
    mut close_call_events: MessageWriter<CloseCallMessage>,
    close_call_achievement: Res<super::super::achievements::resources::CloseCallAchievement>,
    wizard_query: Query<&Transform, With<super::super::units::wizard::components::Wizard>>,
    query: Query<
        (
            Entity,
            &Health,
            &Team,
            &Transform,
            Option<&Infantry>,
            Option<&Archer>,
            Option<&super::super::units::assassin::Assassin>,
            Option<&super::super::units::dispeller::components::Dispeller>,
            Option<&super::super::units::shielder::components::Shielder>,
            Option<&super::super::units::healer::components::Healer>,
            Option<&super::super::units::king::components::King>,
            Option<&Boss>,
            Option<&SpellDamaged>,
            (
                Option<&ResidualFireDamaged>,
                Option<&super::super::units::components::MarkedForDeathModifier>,
                Option<&super::super::units::aerialist::Aerialist>,
                Option<&super::super::units::brute::components::Brute>,
                Has<Flying>,
                Option<&super::super::units::teleporter::components::Teleporter>,
            ),
        ),
        (
            Without<Corpse>,
            Without<super::super::units::wizard::spells::fog_cloud::components::PhantomUnit>,
        ),
    >,
    death_assets: (
        Res<super::super::units::infantry::resources::InfantryAssets>,
        Res<super::super::units::archer::resources::ArcherAssets>,
        Res<super::super::units::assassin::resources::AssassinAssets>,
        Res<super::super::units::dispeller::resources::DispellerAssets>,
        Res<super::super::units::undead::resources::UndeadAssets>,
        Res<super::super::units::king::resources::KingAssets>,
    ),
    death_assets_2: (
        Res<super::super::units::shielder::resources::ShielderAssets>,
        Res<super::super::units::healer::resources::HealerAssets>,
        Res<super::super::units::aerialist::resources::AerialistAssets>,
        Res<super::super::units::teleporter::resources::TeleporterAssets>,
    ),
    mut velocity_query: Query<&mut Velocity>,
) {
    let (
        infantry_assets,
        archer_assets,
        assassin_assets,
        dispeller_assets,
        undead_assets,
        king_assets,
    ) = &death_assets;
    let (shielder_assets, healer_assets, aerialist_assets, teleporter_assets) = &death_assets_2;
    for (
        entity,
        health,
        team,
        transform,
        is_infantry,
        is_archer,
        is_assassin,
        is_dispeller,
        is_shielder,
        is_healer,
        is_king,
        _is_boss,
        spell_damaged,
        (residual_fire_damaged, marked_for_death, is_aerialist, is_brute, is_flying, is_teleporter),
    ) in &query
    {
        if health.is_dead() {
            // Record the kill
            kill_stats.record_kill(*team);

            // Send enemy killed message for multi-kill achievements
            if *team == Team::Attackers || *team == Team::Undead {
                enemy_kill_events.write(EnemyKilledMessage);
                // Notify drops system of potential ingredient drop
                drop_events.write(super::super::drops::messages::SpawnIngredientDropMessage {
                    position: transform.translation,
                });
                // Close Call: enemy died within CLOSE_CALL_DISTANCE of wizard
                if close_call_achievement.is_locked()
                    && let Ok(wiz_transform) = wizard_query.single()
                {
                    use super::super::units::wizard::archetypes::battlemage::CLOSE_CALL_DISTANCE;
                    let diff = transform.translation - wiz_transform.translation;
                    let xz_dist = (diff.x * diff.x + diff.z * diff.z).sqrt();
                    if xz_dist <= CLOSE_CALL_DISTANCE {
                        close_call_events.write(CloseCallMessage);
                    }
                }
            }

            // Track spell kills on defenders and king
            if spell_damaged.is_some() {
                if *team == Team::Defenders {
                    kill_stats.record_spell_kill_defender();
                    spell_kill_events.write(DefenderKilledBySpellMessage);
                }

                if is_king.is_some() {
                    kill_stats.record_king_killed_by_spell();
                }
            }

            // Scorched Earth: unit died from residual fire damage
            if residual_fire_damaged.is_some() {
                scorched_earth_events.write(ScorchedEarthMessage);
            }

            // Marked for Death kill: enemy died while marked by Finger of Death
            if marked_for_death.is_some() && (*team == Team::Attackers || *team == Team::Undead) {
                marked_kill_events
                    .write(super::super::achievements::messages::MarkedForDeathKillMessage);
            }

            // Determine if this unit type has a death animation sprite sheet.
            // Undead infantry use undead-specific death texture.
            let death_texture = if is_dispeller.is_some() {
                Some(dispeller_assets.death_texture.clone())
            } else if is_shielder.is_some() {
                Some(shielder_assets.death_texture.clone())
            } else if is_healer.is_some() {
                Some(healer_assets.death_texture.clone())
            } else if is_infantry.is_some() {
                if *team == Team::Undead {
                    Some(undead_assets.death_texture.clone())
                } else {
                    Some(infantry_assets.death_texture.clone())
                }
            } else if is_archer.is_some() {
                Some(archer_assets.death_texture.clone())
            } else if is_assassin.is_some() {
                Some(assassin_assets.death_texture.clone())
            } else if is_aerialist.is_some() {
                Some(aerialist_assets.death_texture.clone())
            } else if is_teleporter.is_some() {
                Some(teleporter_assets.death_texture.clone())
            } else if is_brute.is_some() {
                Some(infantry_assets.death_texture.clone())
            } else {
                None
            };

            let mut entity_commands = commands.entity(entity);
            entity_commands.insert(Corpse);

            // Flying units drop to ground level on death
            if is_flying {
                entity_commands.remove::<Flying>();
            }

            if let Some(death_tex) = death_texture {
                // Unit has death animation: play it before laying flat
                entity_commands
                    .insert(super::super::units::components::DyingAnimation::new(
                        death_tex,
                    ))
                    .remove::<super::super::units::components::CombatAnimation>();
                // Billboard stays so sprite faces camera during death animation
            } else {
                // No death animation: instant corpse swap (king, boss, fallback)
                let idx = game_rng.0.random_range(0..CORPSE_MATERIAL_VARIANTS);

                let (mat, mesh) = if is_king.is_some() {
                    (
                        king_assets.corpse_materials[idx].clone(),
                        king_assets.sprite_mesh.clone(),
                    )
                } else {
                    // Boss and fallback use infantry corpse materials + circle mesh
                    let mat = corpse_material_for_team(
                        &infantry_assets.defender_corpse_materials,
                        &infantry_assets.attacker_corpse_materials,
                        &infantry_assets.undead_corpse_materials,
                        *team,
                        idx,
                    );
                    (mat, infantry_assets.mesh.clone())
                };

                entity_commands
                    .insert(MeshMaterial3d(mat))
                    .insert(Mesh3d(mesh));

                super::super::units::systems::lay_corpse_flat(
                    &mut entity_commands,
                    transform.translation,
                );
            }

            // Mark undead corpses as permanent (cannot be resurrected)
            if *team == Team::Undead {
                entity_commands.insert(super::super::units::components::PermanentCorpse);
            }

            // Reset velocity so corpses don't continue moving
            if let Ok(mut velocity) = velocity_query.get_mut(entity) {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }

            entity_commands
                .remove::<MovementSpeed>()
                .remove::<AttackTiming>()
                .remove::<Hitbox>()
                .remove::<super::super::units::components::CommanderAuraSpeedModifier>()
                .remove::<super::super::units::components::SlowMovementModifier>()
                .remove::<super::super::units::components::FrostAccumulation>()
                .remove::<super::super::units::components::RootedModifier>()
                .remove::<super::super::units::components::HasteModifier>()
                .remove::<super::super::units::components::FireDoT>()
                .remove::<super::super::units::components::ElectricCharge>()
                .remove::<super::super::units::components::PendingDamageEffect>()
                .remove::<super::super::units::components::OriginalMaterial>()
                .remove::<super::super::units::components::RoughTerrainModifier>()
                .remove::<super::super::units::components::MarkedForDeathModifier>()
                .remove::<super::super::units::components::SleepModifier>()
                .remove::<super::super::units::components::NightTerrors>()
                .remove::<super::super::units::components::Comatose>()
                .remove::<super::super::units::components::NarcolepticWave>()
                .remove::<super::super::units::components::Sleepwalking>()
                .remove::<super::super::units::components::BattleHymnModifier>()
                .remove::<super::super::units::components::EchoingSong>()
                .remove::<super::super::units::components::AnthemResilience>()
                .remove::<super::super::units::components::BerserkerRageModifier>()
                // Berserker rage talent components (FinalStand intentionally NOT removed — it fires on death)
                .remove::<super::super::units::wizard::spells::berserker_rage::components::Bloodlust>()
                .remove::<super::super::units::wizard::spells::berserker_rage::components::Frenzy>()
                .remove::<super::super::units::wizard::spells::berserker_rage::components::FrenzyActive>()
                .remove::<super::super::units::wizard::spells::berserker_rage::components::UndyingFury>()
                .remove::<super::super::units::wizard::spells::berserker_rage::components::UndyingFuryActive>()
                .remove::<super::super::units::wizard::spells::berserker_rage::components::ContagiousRage>()
                .remove::<super::super::units::components::FogEvasionModifier>()
                .remove::<super::super::units::wizard::spells::fog_cloud::components::BlindingMistDebuff>()
                .remove::<super::super::units::components::FrozenSolidModifier>()
                .remove::<super::super::units::components::Stunned>()
                .remove::<super::super::units::wizard::spells::teleport::components::DisorientingHaste>()
                .remove::<super::super::units::components::BanishedModifier>()
                .remove::<super::super::units::components::PolymorphedModifier>()
                .remove::<super::super::units::components::PoisonedModifier>()
                .remove::<super::super::units::components::SickenedModifier>()
                .remove::<super::super::units::components::SmellyModifier>()
                .remove::<super::super::units::wizard::spells::mind_control::components::TraitorsMarkAura>()
                .remove::<super::super::units::wizard::spells::mind_control::components::Demoralized>()
                .remove::<super::super::units::wizard::spells::mind_control::components::AmnesiaOnExpiry>()
                .remove::<super::super::units::wizard::spells::mind_control::components::AmnesiaEffect>()
                .remove::<super::super::units::wizard::spells::mind_control::components::DominatedUnit>()
                .remove::<super::super::units::wizard::spells::mind_control::components::MassHysteriaTarget>()
                .remove::<super::super::units::wizard::spells::mind_control::components::SleeperAgentPending>()
                .remove::<super::super::units::wizard::spells::mind_control::components::SleeperAgentActive>()
                .remove::<super::super::units::wizard::spells::haste::components::MomentumBuff>()
                .remove::<super::super::units::wizard::spells::haste::components::MomentumPending>()
                .remove::<super::super::units::wizard::spells::haste::components::FleetFeet>()
                .remove::<super::super::units::wizard::spells::haste::components::ChainHasteSource>()
                .remove::<super::super::units::wizard::spells::healing_plume::components::FieldMedicConverted>()
                .remove::<CauldronDamageBonus>()
                .remove::<CauldronDamageResistance>()
                .remove::<CauldronSpeedModifier>()
                .remove::<super::super::units::components::WalkingAnimation>()
                .remove::<super::super::units::components::FacingDirection>()
                .remove::<super::super::units::king::components::SpellShield>()
                .remove::<super::super::units::shielder::components::ShielderDamageReduction>();
        }
    }
}
