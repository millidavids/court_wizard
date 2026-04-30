use crate::game::units::wizard::components::Spell;

mod electric;
mod fire;
mod force;
mod frost;
mod nature;
mod necrotic;
mod utility;

/// A single talent definition with display data.
pub(crate) struct TalentDefinition {
    /// Display name of the talent.
    pub name: &'static str,
    /// Description of the mechanical effect (shown when unlocked).
    pub description: &'static str,
    /// Humorous flavor text (shown when locked).
    pub locked_text: &'static str,
    /// Whether this talent's gameplay effect is actually implemented.
    #[allow(dead_code)]
    pub implemented: bool,
}

/// Returns the 3×3 talent definitions for a spell.
/// Outer array = tiers (0..3), inner array = choices (0..3).
pub(crate) fn talent_definitions(spell: Spell) -> [[TalentDefinition; 3]; 3] {
    match spell {
        Spell::MagicMissile => force::magic_missile_talents(),
        Spell::Fireball => fire::fireball_talents(),
        Spell::BattleHymn => utility::battle_hymn_talents(),
        Spell::Disintegrate => frost::disintegrate_talents(),
        Spell::MeteorFall => fire::meteor_fall_talents(),
        Spell::ChainLightning => electric::chain_lightning_talents(),
        Spell::Telekinesis => force::telekinesis_talents(),
        Spell::GuardianCircle => utility::guardian_circle_talents(),
        Spell::FingerOfDeath => force::finger_of_death_talents(),
        Spell::MarkOfDeath => necrotic::mark_of_death_talents(),
        Spell::LightningRod => electric::lightning_rod_talents(),
        Spell::BlackHole => force::black_hole_talents(),
        Spell::PlagueWind => necrotic::plague_wind_talents(),
        Spell::WallOfFire => fire::wall_of_fire_talents(),
        Spell::WallOfStone => frost::wall_of_stone_talents(),
        Spell::Entangle => nature::entangle_talents(),
        Spell::SpikeGrowth => nature::spike_growth_talents(),
        Spell::Squall => electric::squall_talents(),
        Spell::Sleep => utility::sleep_talents(),
        Spell::Grease => nature::grease_talents(),
        Spell::Polymorph => nature::polymorph_talents(),
        Spell::MindControl => utility::mind_control_talents(),
        Spell::Haste => utility::haste_talents(),
        Spell::Teleport => utility::teleport_talents(),
        Spell::RaiseTheDead => necrotic::raise_the_dead_talents(),
        Spell::HealingPlume => nature::healing_plume_talents(),
        Spell::FogCloud => utility::fog_cloud_talents(),
        Spell::BerserkerRage => necrotic::berserker_rage_talents(),
        Spell::Banishment => necrotic::banishment_talents(),
        Spell::Dispel => force::dispel_talents(),
        Spell::ArcaneCrystal => utility::arcane_crystal_talents(),
    }
}
