use super::TalentDefinition;

pub(super) fn chain_lightning_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Conducting Bolts",
                description: "Bounce range doubled, but damage reduced by 30%.",
                locked_text: "Longer arms on the lightning. It can reach the back row now.",
                implemented: true,
            },
            TalentDefinition {
                name: "High Voltage",
                description: "Initial strike damage increased by 80%, but damage falls off faster between bounces.",
                locked_text: "Hit harder up front. The rest is just sparks and crying.",
                implemented: true,
            },
            TalentDefinition {
                name: "Static Charge",
                description: "Hit enemies are slowed by 20% for 2 seconds.",
                locked_text: "Your hair stands up. Then you slow down. Then you regret your life choices.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Forked Lightning",
                description: "Each bounce splits to 3 targets instead of 2.",
                locked_text: "Why hit two when you can hit three? Basic math wizardry.",
                implemented: true,
            },
            TalentDefinition {
                name: "Overcharge",
                description: "No damage falloff between bounces, but splits reduced to 1 target and max bounces reduced to 5.",
                locked_text: "One bolt, full power, every time. Quality over quantity.",
                implemented: true,
            },
            TalentDefinition {
                name: "Magnetic Pull",
                description: "Hit enemies are pulled toward the bolt's previous position.",
                locked_text: "Come here. No really. The lightning insists.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Thunderstorm",
                description: "Fires 3 simultaneous chain lightning strikes at different targets. Quadruple mana cost.",
                locked_text: "Three bolts for the price of three. The wizard union approved this math.",
                implemented: true,
            },
            TalentDefinition {
                name: "Chain Reaction",
                description: "Enemies killed by chain lightning explode, dealing AoE damage and starting a sub-chain from the corpse.",
                locked_text: "Death is contagious. Very, very contagious.",
                implemented: true,
            },
            TalentDefinition {
                name: "Living Lightning",
                description: "Max bounces increased to 100, bouncing until no unhit targets remain. Double mana cost.",
                locked_text: "The lightning has a mind of its own. It won't stop until everyone's been introduced.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn lightning_rod_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Numeric modifiers
        [
            TalentDefinition {
                name: "Taller Rod",
                description: "Rod duration increased by 50%.",
                locked_text: "They say size doesn't matter. They're wrong.",
                implemented: true,
            },
            TalentDefinition {
                name: "Rapid Strikes",
                description: "Lightning strikes 35% faster.",
                locked_text: "The sky barely gets a break.",
                implemented: true,
            },
            TalentDefinition {
                name: "Wider Arc",
                description: "Arc radius increased by 50% and hits 3 additional targets per strike.",
                locked_text: "Personal space? Never heard of it.",
                implemented: true,
            },
        ],
        // Tier 2: Secondary effects
        [
            TalentDefinition {
                name: "Chain Reaction",
                description: "Each arc chains to 1 additional nearby enemy for 50% damage.",
                locked_text: "One zap is never enough.",
                implemented: true,
            },
            TalentDefinition {
                name: "Magnetic Field",
                description: "Enemies hit by arcs are slowed by 40% for 2 seconds.",
                locked_text: "Electrifying. Literally paralyzing.",
                implemented: true,
            },
            TalentDefinition {
                name: "Overcharge",
                description: "Every 3rd strike deals 2.5x damage.",
                locked_text: "Third time's the charm. And the burn.",
                implemented: true,
            },
        ],
        // Tier 3: Transformative upgrades
        [
            TalentDefinition {
                name: "Storm Spire",
                description: "Places 2 rods instead of 1. Each rod deals 60% damage and lasts 70% as long.",
                locked_text: "Why have one lightning rod when you can have two?",
                implemented: true,
            },
            TalentDefinition {
                name: "Tesla Coil",
                description: "Arc damage increases by 15% with each strike. Stacks indefinitely.",
                locked_text: "It just keeps getting angrier.",
                implemented: true,
            },
            TalentDefinition {
                name: "Lightning Nexus",
                description: "Killing a unit with arcs triggers a weaker bonus strike. Successive bonus strikes deal diminishing damage.",
                locked_text: "Death begets lightning begets death.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn squall_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Bitter Cold",
                description: "Damage per ice shard increased by 30%.",
                locked_text: "Colder ice. Somehow. The wizard doesn't understand thermodynamics and neither should you.",
                implemented: true,
            },
            TalentDefinition {
                name: "Howling Winds",
                description: "Storm radius increased by 30%.",
                locked_text: "The storm got bigger. The enemies got sadder.",
                implemented: true,
            },
            TalentDefinition {
                name: "Freezing Rain",
                description: "Ice spawns 40% faster but each shard deals 20% less damage.",
                locked_text: "More ice, less damage per ice. It's a quantity play.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Permafrost",
                description: "Enemies hit 3+ times become frozen solid for 2 seconds (can't move or attack).",
                locked_text: "Freeze! No literally. The wizard is very literal.",
                implemented: true,
            },
            TalentDefinition {
                name: "Hailstones",
                description: "Some ice shards are replaced with larger hailstones that deal 3x damage.",
                locked_text: "Big ice. Big damage. Big problem for anyone standing in the circle.",
                implemented: true,
            },
            TalentDefinition {
                name: "Sleet Storm",
                description: "Enemies inside the storm have a 40% chance to miss their attacks.",
                locked_text: "Can't hit what you can't see through a blizzard. Tactical meteorology.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Absolute Zero",
                description: "Storm continuously drains mana and applies a stacking slow and frost damage to enemies inside. The longer they stay, the slower they get.",
                locked_text: "Entropy stops. Time stops. The wizard's heating bill does not stop.",
                implemented: true,
            },
            TalentDefinition {
                name: "Blizzard",
                description: "Storm follows the cursor slowly while channeling instead of staying in place.",
                locked_text: "A movable ice storm. The wizard is basically a weather god now. A very petty weather god.",
                implemented: true,
            },
            TalentDefinition {
                name: "Ice Age",
                description: "Storm leaves frozen ground that slows enemies by 30%. Persists after channeling ends.",
                locked_text: "The ice never melts. Global warming has met its match: one very stubborn wizard.",
                implemented: true,
            },
        ],
    ]
}
