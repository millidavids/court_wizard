use bevy::prelude::*;

use super::utils::{SpellCircleIndicator, update_spell_indicators};
use super::visual_assets::{AuraSphereMaterial, FireExplosionSphereMaterial};
use super::wall_of_stone::wall_material::WallOfStoneMaterial;
use crate::game::battlefield::load_battlefield_assets;
use crate::game::run_conditions::{any_exist, is_spell_effects_active};
use crate::state::AppState;

use super::arcane_crystal::ArcaneCrystalPlugin;
use super::banishment::BanishmentPlugin;
use super::battle_hymn::BattleHymnPlugin;
use super::berserker_rage::BerserkerRagePlugin;
use super::black_hole::BlackHolePlugin;
use super::chain_lightning::ChainLightningPlugin;
use super::disintegrate::DisintegratePlugin;
use super::dispel::DispelPlugin;
use super::entangle::EntanglePlugin;
use super::finger_of_death::FingerOfDeathPlugin;
use super::fireball::FireballPlugin;
use super::fog_cloud::FogCloudPlugin;
use super::grease::GreasePlugin;
use super::guardian_circle::GuardianCirclePlugin;
use super::haste::HastePlugin;
use super::healing_plume::HealingPlumePlugin;
use super::lightning_rod::LightningRodPlugin;
use super::magic_missile::MagicMissilePlugin;
use super::mark_of_death::MarkOfDeathPlugin;
use super::meteor_fall::MeteorFallPlugin;
use super::mind_control::MindControlPlugin;
use super::plague_wind::PlagueWindPlugin;
use super::polymorph::PolymorphPlugin;
use super::raise_the_dead::RaiseTheDeadPlugin;
use super::sleep::SleepPlugin;
use super::spike_growth::SpikeGrowthPlugin;
use super::squall::SquallPlugin;
use super::systems;
use super::telekinesis::TelekinesisPlugin;
use super::teleport::TeleportPlugin;
use super::vfx::VfxPlugin;
use super::wall_of_fire::WallOfFirePlugin;
use super::wall_of_stone::plugin::WallOfStonePlugin;

/// Plugin that handles wizard spells and projectiles.
pub struct SpellsPlugin;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                MaterialPlugin::<FireExplosionSphereMaterial>::default(),
                MaterialPlugin::<AuraSphereMaterial>::default(),
                MaterialPlugin::<WallOfStoneMaterial>::default(),
            ))
            .add_systems(
                Startup,
                (
                    super::visual_assets::init_spell_visual_assets
                        .after(load_battlefield_assets),
                    super::visual_assets::capture_original_spell_colors
                        .after(super::visual_assets::init_spell_visual_assets),
                    super::audio::load_spell_sfx_assets,
                ),
            )
            .add_systems(
                OnEnter(AppState::Loading),
                super::visual_assets::refresh_spell_visuals_for_wizard,
            )
            .add_plugins((
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
                SleepPlugin,
            ))
            .add_plugins((
                GreasePlugin,
                PlagueWindPlugin,
                MeteorFallPlugin,
                BanishmentPlugin,
                PolymorphPlugin,
                ArcaneCrystalPlugin,
                DispelPlugin,
                MindControlPlugin,
            ))
            .add_plugins(VfxPlugin)
            .add_systems(
                Update,
                update_spell_indicators
                    .run_if(any_exist::<SpellCircleIndicator>())
                    .run_if(is_spell_effects_active),
            )
            .add_systems(
                Update,
                (
                    systems::move_projectiles,
                    systems::check_projectile_collisions,
                    systems::update_spell_effects,
                    systems::despawn_distant_projectiles,
                )
                    .chain()
                    .run_if(is_spell_effects_active),
            );
    }
}
