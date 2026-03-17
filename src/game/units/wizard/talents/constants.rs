use crate::game::units::wizard::components::Spell;

/// Returns the tier thresholds for a spell's talent progression.
/// Each spell has 3 tiers, unlocked at these cumulative progress values.
pub(crate) fn tier_thresholds(spell: Spell) -> [u32; 3] {
    match spell {
        Spell::MagicMissile => [50, 400, 1000],
        Spell::Fireball => [30, 200, 700],
        Spell::BattleHymn => [20, 75, 200],
        // Offense spells
        Spell::Disintegrate => [40, 120, 350],
        Spell::ChainLightning => [30, 100, 300],
        Spell::FingerOfDeath => [5, 10, 20],
        Spell::LightningRod => [10, 100, 500],
        Spell::MeteorFall => [20, 70, 200],
        Spell::MarkOfDeath => [15, 50, 150],
        Spell::PlagueWind => [50, 200, 700],
        // Control spells
        Spell::BlackHole => [50, 200, 500],
        Spell::WallOfStone => [15, 50, 150],
        Spell::WallOfFire => [20, 100, 500],
        Spell::Entangle => [20, 70, 200],
        Spell::SpikeGrowth => [20, 100, 500],
        Spell::Squall => [20, 100, 500],
        Spell::Sleep => [15, 50, 150],
        Spell::Grease => [20, 70, 200],
        Spell::Polymorph => [10, 50, 100],
        Spell::MindControl => [10, 50, 100],
        // Support spells
        Spell::GuardianCircle => [20, 70, 200],
        Spell::Haste => [30, 200, 500],
        Spell::Teleport => [20, 100, 500],
        Spell::RaiseTheDead => [10, 50, 100],
        Spell::HealingPlume => [1, 2, 3],
        Spell::FogCloud => [20, 70, 200],
        Spell::BerserkerRage => [15, 50, 150],
        // Utility spells
        Spell::Telekinesis => [1, 5, 10],
        Spell::Banishment => [10, 35, 100],
        Spell::ArcaneCrystal => [15, 50, 150],
        Spell::Dispel => [15, 50, 150],
    }
}

/// Returns a human-readable label for what increments this spell's talent progress.
pub(crate) fn progress_metric_label(spell: Spell) -> &'static str {
    match spell {
        Spell::MagicMissile => "Attackers hit with missiles",
        Spell::Fireball => "Enemies damaged by explosions",
        Spell::BattleHymn => "Defenders inspired",
        Spell::Disintegrate => "Enemies damaged by beam",
        Spell::ChainLightning => "Enemies struck by lightning",
        Spell::FingerOfDeath => "Enemies killed by beam",
        Spell::LightningRod => "Enemies struck by arcs",
        Spell::MeteorFall => "Enemies damaged by meteors",
        Spell::MarkOfDeath => "Enemies marked",
        Spell::PlagueWind => "Enemies poisoned",
        Spell::BlackHole => "Enemies pulled",
        Spell::WallOfStone => "Walls placed",
        Spell::WallOfFire => "Enemies burned by wall",
        Spell::Entangle => "Enemies rooted",
        Spell::SpikeGrowth => "Enemies damaged in zone",
        Spell::Squall => "Enemies slowed by storm",
        Spell::Sleep => "Enemies put to sleep",
        Spell::Grease => "Enemies slowed or burned",
        Spell::Polymorph => "Enemies polymorphed",
        Spell::MindControl => "Enemies controlled",
        Spell::GuardianCircle => "Units shielded",
        Spell::Haste => "Units hasted",
        Spell::Teleport => "Units teleported",
        Spell::RaiseTheDead => "Corpses raised",
        Spell::HealingPlume => "Health restored",
        Spell::FogCloud => "Units hidden in fog",
        Spell::BerserkerRage => "Units enraged",
        Spell::Telekinesis => "Ingredients collected",
        Spell::Banishment => "Enemies banished",
        Spell::ArcaneCrystal => "Projectiles scattered",
        Spell::Dispel => "Effects dispelled",
    }
}
