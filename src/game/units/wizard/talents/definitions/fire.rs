use super::TalentDefinition;

pub(super) fn fireball_talents() -> [[TalentDefinition; 3]; 3] {
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

pub(super) fn meteor_fall_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Dense Barrage",
                description: "Meteor spawn rate increased by 30%.",
                locked_text: "More rocks from the sky. Simple but effective.",
                implemented: true,
            },
            TalentDefinition {
                name: "Scorching Impact",
                description: "Explosion and ground fire damage increased by 30%.",
                locked_text: "Hotter meteors. The ground remembers.",
                implemented: true,
            },
            TalentDefinition {
                name: "Wide Devastation",
                description: "Storm radius increased by 30%.",
                locked_text: "A wider rain of destruction. Cover more ground.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Molten Core",
                description: "Ground fire lasts twice as long and deals 50% more damage.",
                locked_text: "The ground burns longer. Much longer.",
                implemented: true,
            },
            TalentDefinition {
                name: "Tracking Meteors",
                description: "Meteors steer toward nearby enemies as they fall.",
                locked_text: "Smart rocks. What a time to be alive.",
                implemented: true,
            },
            TalentDefinition {
                name: "Aftershock",
                description: "Each meteor impact knocks nearby enemies outward and deals bonus damage.",
                locked_text: "The impact sends everything flying. Physics!",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Extinction Event",
                description: "After 5 seconds of channeling, one massive meteor strikes the entire storm area for 100 damage.",
                locked_text: "The big one. Dinosaurs hate this trick.",
                implemented: true,
            },
            TalentDefinition {
                name: "Volcanic Eruption",
                description: "Meteors landing near existing ground fire trigger eruption bursts with escalating damage.",
                locked_text: "Fire on fire on fire. It keeps getting worse.",
                implemented: true,
            },
            TalentDefinition {
                name: "Meteor Shower",
                description: "Triple meteor spawn rate, but each meteor is smaller and weaker. Half mana cost.",
                locked_text: "Quantity has a quality all its own.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn wall_of_fire_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Numeric modifiers
        [
            TalentDefinition {
                name: "Infernal Intensity",
                description: "Fire damage increased by 100%.",
                locked_text: "Some like it hot. These flames like it hotter.",
                implemented: true,
            },
            TalentDefinition {
                name: "Firebreak",
                description: "Wall width increased by 80% and duration increased by 25%.",
                locked_text: "A wider wall keeps more things on the wrong side of it.",
                implemented: true,
            },
            TalentDefinition {
                name: "Flash Fire",
                description: "Max wall length increased by 50% and damage increased by 50%, but duration reduced by 40%.",
                locked_text: "Burns twice as bright, lasts half as long. Worth it.",
                implemented: true,
            },
        ],
        // Tier 2: Secondary effects
        [
            TalentDefinition {
                name: "Searing Heat",
                description: "Units inside the wall have healing received reduced by 50%.",
                locked_text: "Hard to apply bandages when you're on fire.",
                implemented: true,
            },
            TalentDefinition {
                name: "Scorched Earth",
                description: "After the wall expires, it leaves a burnt zone for 8 seconds that slows units by 30%.",
                locked_text: "The fire is gone. The ground remembers.",
                implemented: true,
            },
            TalentDefinition {
                name: "Spreading Flames",
                description: "Units that leave the wall continue to burn for 3 seconds, taking 50% of the wall's damage per tick.",
                locked_text: "Stop, drop, and roll? Too late.",
                implemented: true,
            },
        ],
        // Tier 3: Transformative upgrades
        [
            TalentDefinition {
                name: "Firestorm",
                description: "When an enemy dies inside the wall, a fire explosion deals 5 damage to all enemies within 60 units.",
                locked_text: "Every death fans the flames. Literally.",
                implemented: true,
            },
            TalentDefinition {
                name: "Twin Walls",
                description: "Places 2 parallel walls offset by the wall width, each at 60% damage.",
                locked_text: "Why build one wall when you can build two for twice the price?",
                implemented: true,
            },
            TalentDefinition {
                name: "Consuming Inferno",
                description: "Wall damage increases by 15% per second it has been active, up to +300%.",
                locked_text: "It starts as a campfire. It ends as a crematorium.",
                implemented: true,
            },
        ],
    ]
}
