use bevy::prelude::*;

use crate::game::run_conditions::is_gameplay_running;

use super::arcane_crystal::ArcaneCrystalPlugin;
use super::banishment::BanishmentPlugin;
use super::battle_hymn::BattleHymnPlugin;
use super::berserker_rage::BerserkerRagePlugin;
use super::black_hole::BlackHolePlugin;
use super::chain_lightning::ChainLightningPlugin;
use super::disintegrate::DisintegratePlugin;
use super::entangle::EntanglePlugin;
use super::finger_of_death::FingerOfDeathPlugin;
use super::fireball::FireballPlugin;
use super::fog_cloud::FogCloudPlugin;
use super::grease::GreasePlugin;
use super::guardian_circle::GuardianCirclePlugin;
use super::haste::HastePlugin;
use super::healing_plume::HealingPlumePlugin;
use super::hypnotic_pattern::HypnoticPatternPlugin;
use super::lightning_rod::LightningRodPlugin;
use super::magic_missile::MagicMissilePlugin;
use super::mark_of_death::MarkOfDeathPlugin;
use super::meteor_fall::MeteorFallPlugin;
use super::phantasmal_force::PhantasmalForcePlugin;
use super::plague_wind::PlagueWindPlugin;
use super::polymorph::PolymorphPlugin;
use super::raise_the_dead::RaiseTheDeadPlugin;
use super::sleep::SleepPlugin;
use super::spike_growth::SpikeGrowthPlugin;
use super::squall::SquallPlugin;
use super::systems;
use super::telekinesis::TelekinesisPlugin;
use super::teleport::TeleportPlugin;
use super::wall_of_fire::WallOfFirePlugin;
use super::wall_of_stone::plugin::WallOfStonePlugin;

/// Plugin that handles wizard spells and projectiles.
pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MagicMissilePlugin,
            DisintegratePlugin,
            FireballPlugin,
            GuardianCirclePlugin,
            ChainLightningPlugin,
            FingerOfDeathPlugin,
            RaiseTheDeadPlugin,
            TeleportPlugin,
        ))
        .add_plugins((
            WallOfStonePlugin,
            BlackHolePlugin,
            SquallPlugin,
            WallOfFirePlugin,
            EntanglePlugin,
            HastePlugin,
            SpikeGrowthPlugin,
            LightningRodPlugin,
        ))
        .add_plugins((
            TelekinesisPlugin,
            HealingPlumePlugin,
            BattleHymnPlugin,
            BerserkerRagePlugin,
            FogCloudPlugin,
            MarkOfDeathPlugin,
            HypnoticPatternPlugin,
            SleepPlugin,
        ))
        .add_plugins((
            GreasePlugin,
            PlagueWindPlugin,
            PhantasmalForcePlugin,
            MeteorFallPlugin,
            BanishmentPlugin,
            PolymorphPlugin,
            ArcaneCrystalPlugin,
        ))
        .add_systems(
            Update,
            (
                systems::move_projectiles,
                systems::check_projectile_collisions,
                systems::update_spell_effects,
                systems::despawn_distant_projectiles,
            )
                .chain()
                .run_if(is_gameplay_running),
        );
    }
}
