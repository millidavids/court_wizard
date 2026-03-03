use crate::game::units::wizard::components::Spell;

/// A single talent definition with display data.
#[allow(dead_code)]
pub(crate) struct TalentDefinition {
    /// Display name of the talent.
    pub name: &'static str,
    /// Description of the mechanical effect (shown when unlocked).
    pub description: &'static str,
    /// Humorous flavor text (shown when locked).
    pub locked_text: &'static str,
    /// Whether this talent's gameplay effect is actually implemented.
    pub implemented: bool,
}

/// Returns the 3×3 talent definitions for a spell.
/// Outer array = tiers (0..3), inner array = choices (0..3).
pub(crate) fn talent_definitions(spell: Spell) -> [[TalentDefinition; 3]; 3] {
    match spell {
        Spell::MagicMissile => magic_missile_talents(),
        Spell::Fireball => fireball_talents(),
        Spell::BattleHymn => battle_hymn_talents(),
        // All other spells get placeholder talents
        _ => placeholder_talents(spell),
    }
}

fn magic_missile_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Volley",
                description: "Fire 5 missiles at 80% damage instead of 3 at 100%.",
                locked_text: "More missiles means more problems. For them.",
                implemented: true,
            },
            TalentDefinition {
                name: "Heavy Ordnance",
                description: "Fire only 1 missile, but at 4x damage with a larger collision radius.",
                locked_text: "Quality over quantity. One big angry missile.",
                implemented: true,
            },
            TalentDefinition {
                name: "Swift Salvo",
                description: "Cooldown reduced by 25%, but mana cost increased by 50%.",
                locked_text: "Speed costs extra. Wizard union rules.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Seeker Swarm",
                description: "When a missile kills its target, it splits into 2 missiles at 20% damage.",
                locked_text: "Death begets more death. Poetic, really.",
                implemented: true,
            },
            TalentDefinition {
                name: "Arcane Barrage",
                description: "Click to begin automatically casting free volleys every 5s. Requires concentration.",
                locked_text: "Hold the button. Keep holding. Don't stop.",
                implemented: true,
            },
            TalentDefinition {
                name: "Piercing Bolts",
                description: "Missiles pass through the first target, hitting up to 2 enemies.",
                locked_text: "Two birds, one glowing death projectile.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Missile Storm",
                description: "Fire 4x as many mini-missiles with heavy wobble at 25% damage each.",
                locked_text: "When in doubt, fill the sky with explosions.",
                implemented: true,
            },
            TalentDefinition {
                name: "Arcane Detonation",
                description: "Missiles explode on impact for small AoE dealing 20% damage in radius.",
                locked_text: "Every missile is a fireball if you believe hard enough.",
                implemented: true,
            },
            TalentDefinition {
                name: "Guided Devastation",
                description: "All missiles steer toward your cursor at 1.5x damage. No target homing.",
                locked_text: "Cruise missile, wizard edition.",
                implemented: true,
            },
        ],
    ]
}

fn fireball_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Wider Blast",
                description: "Explosion radius increased by 50%. Damage unchanged.",
                locked_text: "Bigger boom. Same firepower. More coverage.",
                implemented: true,
            },
            TalentDefinition {
                name: "Lingering Flames",
                description: "Explosion duration increased by 80%. Same total damage, more area denial.",
                locked_text: "The fire stays. The screaming continues.",
                implemented: true,
            },
            TalentDefinition {
                name: "Focused Blast",
                description: "Explosion radius halved, but damage doubled.",
                locked_text: "Precision pyrotechnics. A niche field.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Cluster Bomb",
                description: "On impact, spawns 3 mini-fireballs in random directions.",
                locked_text: "One fireball is never enough for a real pyromaniac.",
                implemented: true,
            },
            TalentDefinition {
                name: "Napalm",
                description: "Fireball leaves a burning trail as it flies, damaging units in its path.",
                locked_text: "Everything the fireball touches becomes fire too.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Ignition",
                description: "Cast time reduced to 2.0s instead of 3.0s.",
                locked_text: "Less dramatic. More practical.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Meteor",
                description: "Fireball drops from the sky instead of flying. Faster arrival, larger impact zone.",
                locked_text: "Why throw fire sideways when you can drop it from space?",
                implemented: true,
            },
            TalentDefinition {
                name: "Scorched Earth",
                description: "Explosion leaves persistent burning ground for 5 seconds.",
                locked_text: "Area denial through aggressive landscaping.",
                implemented: true,
            },
            TalentDefinition {
                name: "Chain Ignition",
                description: "Hit enemies take 50% more damage from all sources for 3 seconds.",
                locked_text: "Set them on fire AND make them fragile. Efficient.",
                implemented: true,
            },
        ],
    ]
}

fn battle_hymn_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Inspiring Words",
                description: "Buff duration increased by 50%.",
                locked_text: "Longer song, longer buff. Music theory is simple.",
                implemented: true,
            },
            TalentDefinition {
                name: "War Drums",
                description: "Damage bonus increased by 50%.",
                locked_text: "LOUDER equals STRONGER. Science.",
                implemented: true,
            },
            TalentDefinition {
                name: "Wide Anthem",
                description: "Buff radius increased by 40%.",
                locked_text: "The acoustics in this battlefield are terrible.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Fortifying Hymn",
                description: "Buffed units gain 20 temporary hit points.",
                locked_text: "Music so good it generates a force field.",
                implemented: true,
            },
            TalentDefinition {
                name: "Echoing Song",
                description: "When the buff expires, it re-applies once at 50% duration.",
                locked_text: "The song gets stuck in their heads.",
                implemented: true,
            },
            TalentDefinition {
                name: "Swift March",
                description: "Buffed units also gain 25% movement speed.",
                locked_text: "Double-time! Left, left, left-right-left!",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Hymn of Legends",
                description: "Damage and attack speed bonuses doubled.",
                locked_text: "This song goes to eleven.",
                implemented: true,
            },
            TalentDefinition {
                name: "Anthem of Resilience",
                description: "Buffed units take 30% reduced damage.",
                locked_text: "Defense through the power of music. Bards were right all along.",
                implemented: true,
            },
            TalentDefinition {
                name: "Chorus of Valor",
                description: "Buff affects ALL defenders on the field, ignoring radius. Double mana cost.",
                locked_text: "The whole army hears the song. Surround sound.",
                implemented: true,
            },
        ],
    ]
}

fn placeholder_talents(spell: Spell) -> [[TalentDefinition; 3]; 3] {
    // Generate thematic placeholder names based on spell type
    let (t1, t2, t3) = placeholder_names(spell);

    [
        [
            TalentDefinition {
                name: t1[0],
                description: "Effect not yet implemented.",
                locked_text: "Patience, young wizard. This talent is still being researched.",
                implemented: false,
            },
            TalentDefinition {
                name: t1[1],
                description: "Effect not yet implemented.",
                locked_text: "The ancient texts are smudged. Check back later.",
                implemented: false,
            },
            TalentDefinition {
                name: t1[2],
                description: "Effect not yet implemented.",
                locked_text: "The wizard council hasn't approved this one yet.",
                implemented: false,
            },
        ],
        [
            TalentDefinition {
                name: t2[0],
                description: "Effect not yet implemented.",
                locked_text: "This scroll is written in a language that hasn't been invented yet.",
                implemented: false,
            },
            TalentDefinition {
                name: t2[1],
                description: "Effect not yet implemented.",
                locked_text: "Coming soon to a wizard tower near you.",
                implemented: false,
            },
            TalentDefinition {
                name: t2[2],
                description: "Effect not yet implemented.",
                locked_text: "The enchantment is still cooling in the forge.",
                implemented: false,
            },
        ],
        [
            TalentDefinition {
                name: t3[0],
                description: "Effect not yet implemented.",
                locked_text: "This power exists only in prophecy. For now.",
                implemented: false,
            },
            TalentDefinition {
                name: t3[1],
                description: "Effect not yet implemented.",
                locked_text: "Even archmages need time to figure this one out.",
                implemented: false,
            },
            TalentDefinition {
                name: t3[2],
                description: "Effect not yet implemented.",
                locked_text: "The universe isn't ready for this talent. Soon.",
                implemented: false,
            },
        ],
    ]
}

/// Returns themed placeholder talent names for unimplemented spells.
fn placeholder_names(spell: Spell) -> ([&'static str; 3], [&'static str; 3], [&'static str; 3]) {
    match spell {
        Spell::Disintegrate => (
            ["Wider Beam", "Sustained Focus", "Quick Draw"],
            ["Ricochet Ray", "Penetrating Gaze", "Mana Siphon"],
            ["Annihilation Beam", "Prismatic Destruction", "Death Stare"],
        ),
        Spell::ChainLightning => (
            ["Extra Arc", "Charged Strike", "Quick Bolt"],
            ["Storm Surge", "Forked Lightning", "Magnetic Pull"],
            ["Thunderstorm", "Overload", "Zeus's Wrath"],
        ),
        Spell::FingerOfDeath => (
            ["Extended Reach", "Corrupting Touch", "Swift Death"],
            ["Soul Drain", "Spreading Decay", "Dark Power"],
            ["Hand of Doom", "Death's Embrace", "Reaper's Call"],
        ),
        Spell::LightningRod => (
            ["Taller Rod", "Rapid Strikes", "Wider Arc"],
            ["Chain Reaction", "Magnetic Field", "Overcharge"],
            ["Storm Spire", "Tesla Tower", "Lightning Nexus"],
        ),
        Spell::MeteorFall => (
            ["Wider Impact", "Rapid Fall", "Burning Debris"],
            ["Meteor Shower", "Seismic Impact", "Molten Rain"],
            ["Extinction Event", "Star Fall", "Heaven's Fury"],
        ),
        Spell::MarkOfDeath => (
            ["Deeper Mark", "Spreading Mark", "Quick Brand"],
            ["Death Sentence", "Sympathetic Wounds", "Hunter's Mark"],
            ["Doom Brand", "Marked for Extinction", "Final Judgment"],
        ),
        Spell::PlagueWind => (
            ["Wider Cloud", "Lingering Toxin", "Swift Plague"],
            ["Pandemic", "Toxic Eruption", "Poisoned Ground"],
            ["Death Wind", "Blight Storm", "Corruption Wave"],
        ),
        Spell::BlackHole => (
            ["Wider Pull", "Stronger Gravity", "Quick Collapse"],
            ["Event Horizon", "Tidal Force", "Crushing Void"],
            ["Singularity", "Dimensional Rift", "Oblivion"],
        ),
        Spell::WallOfStone => (
            ["Longer Wall", "Reinforced Stone", "Quick Build"],
            ["Fortress", "Maze Walls", "Crumbling Crush"],
            ["Citadel", "Living Stone", "Mountain's Might"],
        ),
        Spell::WallOfFire => (
            ["Longer Wall", "Hotter Flames", "Quick Ignition"],
            ["Inferno Line", "Spreading Fire", "Magma Wall"],
            ["Hell's Gate", "Eternal Flame", "Burning Fortress"],
        ),
        Spell::Entangle => (
            ["Wider Roots", "Thorny Vines", "Quick Growth"],
            ["Crushing Vines", "Spreading Roots", "Poison Ivy"],
            ["Forest's Wrath", "Living Jungle", "Nature's Prison"],
        ),
        Spell::SpikeGrowth => (
            ["Wider Zone", "Sharper Spikes", "Quick Bloom"],
            ["Thorn Maze", "Poisoned Spikes", "Quicksand"],
            ["Death Garden", "Nature's Minefield", "Spike Storm"],
        ),
        Spell::Squall => (
            ["Wider Storm", "Colder Winds", "Quick Gust"],
            ["Blizzard", "Hailstorm", "Frozen Ground"],
            ["Ice Age", "Polar Vortex", "Winter's Wrath"],
        ),
        Spell::Sleep => (
            ["Deeper Sleep", "Wider Lullaby", "Quick Nap"],
            ["Nightmare", "Sleepwalking", "Coma"],
            ["Eternal Slumber", "Dream Prison", "Sandman's Curse"],
        ),
        Spell::Grease => (
            ["Wider Puddle", "Stickier Grease", "Quick Slick"],
            ["Oil Slick", "Banana Peel", "Tar Pit"],
            ["Black Ice", "Grease Fire", "Slip 'n Slide"],
        ),
        Spell::Polymorph => (
            ["Longer Duration", "Smaller Sheep", "Quick Morph"],
            ["Mass Polymorph", "Angry Chicken", "Snail Form"],
            ["Permanent Change", "Dragon Form", "Chaos Morph"],
        ),
        Spell::MindControl => (
            ["Longer Control", "Stronger Will", "Quick Domination"],
            ["Mass Suggestion", "Berserker Puppet", "Double Agent"],
            ["Absolute Control", "Hivemind", "Puppet Master"],
        ),
        Spell::GuardianCircle => (
            ["Wider Circle", "Stronger Shield", "Quick Ward"],
            ["Regenerating Ward", "Reflective Shield", "Fortified Zone"],
            ["Impenetrable Dome", "Guardian Angel", "Sanctuary"],
        ),
        Spell::Haste => (
            ["Longer Duration", "Faster Speed", "Wider Zone"],
            ["Time Warp", "Adrenaline Rush", "Lightning Reflexes"],
            ["Time Stop", "Infinite Speed", "Temporal Mastery"],
        ),
        Spell::Teleport => (
            ["Wider Area", "Longer Range", "Quick Shift"],
            ["Momentum Teleport", "Swap Places", "Phase Shift"],
            ["Mass Teleport", "Dimensional Gate", "Everywhere At Once"],
        ),
        Spell::RaiseTheDead => (
            ["More Corpses", "Stronger Undead", "Quick Raise"],
            ["Skeletal Army", "Zombie Plague", "Death Knight"],
            ["Necropolis", "Lich's Command", "Army of Darkness"],
        ),
        Spell::HealingPlume => (
            ["Wider Zone", "Stronger Healing", "Quick Bloom"],
            ["Regeneration", "Cleansing Mist", "Life Burst"],
            ["Miracle", "Fountain of Life", "Divine Intervention"],
        ),
        Spell::FogCloud => (
            ["Wider Fog", "Thicker Mist", "Quick Cover"],
            ["Blinding Fog", "Toxic Fumes", "Phantom Fog"],
            ["Impenetrable Mist", "Shadow Realm", "Void Cloud"],
        ),
        Spell::BerserkerRage => (
            ["Longer Rage", "More Damage", "Wider Effect"],
            ["Blood Frenzy", "Reckless Abandon", "Unstoppable Force"],
            ["Wrath of Gods", "Berserker's Fury", "Primal Rage"],
        ),
        Spell::Telekinesis => (
            ["Longer Range", "Multi-Grab", "Quick Pull"],
            ["Magnetic Field", "Auto-Collect", "Treasure Sense"],
            ["Gravity Well", "Telekinetic Storm", "Mind Over Matter"],
        ),
        Spell::Banishment => (
            ["Longer Duration", "Multi-Target", "Quick Cast"],
            ["Pocket Dimension", "Weakening Exile", "Delayed Return"],
            ["Permanent Exile", "Void Prison", "Dimensional Lock"],
        ),
        Spell::ArcaneCrystal => (
            ["Larger Crystal", "More Scatters", "Quick Charge"],
            ["Prism Array", "Chain Refraction", "Crystal Shield"],
            ["Crystal Cascade", "Arcane Nexus", "Infinity Crystal"],
        ),
        Spell::Dispel => (
            ["Wider Wave", "Stronger Nullify", "Quick Shot"],
            ["Chain Dispel", "Mana Drain", "Feedback Loop"],
            ["Anti-Magic Zone", "Null Field", "Arcane Silence"],
        ),
        // Fallback for any spell not explicitly listed (shouldn't happen)
        _ => (
            ["Enhancement I", "Enhancement II", "Enhancement III"],
            ["Augment I", "Augment II", "Augment III"],
            ["Mastery I", "Mastery II", "Mastery III"],
        ),
    }
}
