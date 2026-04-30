use super::TalentDefinition;

pub(super) fn entangle_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Numeric modifiers
        [
            TalentDefinition {
                name: "Deep Roots",
                description: "Root duration increased by 50%.",
                locked_text: "The vines seem oddly clingy today.",
                implemented: true,
            },
            TalentDefinition {
                name: "Sprawling Thicket",
                description: "Entangle radius increased by 40%, but mana cost increased by 25%.",
                locked_text: "Turns out magic weeds spread just like regular ones.",
                implemented: true,
            },
            TalentDefinition {
                name: "Efficient Growth",
                description: "Mana cost reduced by 40% and cast time reduced by 30%.",
                locked_text: "Gardening on a budget.",
                implemented: true,
            },
        ],
        // Tier 2: Secondary effects
        [
            TalentDefinition {
                name: "Thorny Vines",
                description: "Rooted units take 3 damage per second for the root duration.",
                locked_text: "Nature's barbed wire.",
                implemented: true,
            },
            TalentDefinition {
                name: "Clinging Roots",
                description: "When the root expires, affected enemies are slowed by 40% for 3 seconds.",
                locked_text: "Leaving is harder than arriving.",
                implemented: true,
            },
            TalentDefinition {
                name: "Nourishing Roots",
                description: "The wizard regenerates 3 mana per second for each enemy rooted by Entangle.",
                locked_text: "The vines are hungry, and your strength is on the menu.",
                implemented: true,
            },
        ],
        // Tier 3: Transformative upgrades
        [
            TalentDefinition {
                name: "Overgrowth",
                description: "Entangle area grows 50% larger over its duration. Enemies entering the growing area are also rooted for the remaining duration.",
                locked_text: "The jungle has a mind of its own.",
                implemented: true,
            },
            TalentDefinition {
                name: "Nature's Sanctuary",
                description: "Rooted defenders are not rooted. Instead, they gain 15 temporary HP.",
                locked_text: "The vines know who their friends are... mostly.",
                implemented: true,
            },
            TalentDefinition {
                name: "Stranglehold",
                description: "Enemies rooted for more than 3 seconds take 25 burst damage when the root expires. Enemies killed by this burst don't leave corpses.",
                locked_text: "What the vines take, the earth reclaims.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn spike_growth_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Numeric modifiers
        [
            TalentDefinition {
                name: "Wider Zone",
                description: "Zone radius increased by 40%.",
                locked_text: "The spikes are spreading — someone call a landscaper.",
                implemented: true,
            },
            TalentDefinition {
                name: "Sharper Spikes",
                description: "Damage per tick increased by 60%.",
                locked_text: "These spikes were forged in the school of ouch.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Bloom",
                description: "Cast time reduced by 40% and mana cost reduced by 30%.",
                locked_text: "Gardening on a budget and a deadline.",
                implemented: true,
            },
        ],
        // Tier 2: Secondary effects
        [
            TalentDefinition {
                name: "Thorn Maze",
                description: "Slow effect doubled to 60%. Enemies avoid the zone much more aggressively.",
                locked_text: "Nobody wants to walk through the wizard's hedge maze twice.",
                implemented: true,
            },
            TalentDefinition {
                name: "Poisoned Spikes",
                description: "Units that leave the zone continue taking 2 poison damage per second for 4 seconds.",
                locked_text: "The gift that keeps on giving — and stinging.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quicksand",
                description: "Units inside the zone for more than 2 seconds are rooted in place for 1.5 seconds. Can only trigger once per unit per zone.",
                locked_text: "Step in, stay a while. No really, you have to.",
                implemented: true,
            },
        ],
        // Tier 3: Transformative upgrades
        [
            TalentDefinition {
                name: "Death Garden",
                description: "Zone grows 30% larger over its duration. Units dying inside extend the zone's duration by 3 seconds (max +9s extra).",
                locked_text: "Feed the garden, and the garden feeds on you.",
                implemented: true,
            },
            TalentDefinition {
                name: "Nature's Minefield",
                description: "Casts 3 smaller zones (55% radius) in a triangle pattern instead of 1 large zone. Each zone is independent.",
                locked_text: "Why have one deadly garden when you can have three?",
                implemented: true,
            },
            TalentDefinition {
                name: "Spike Storm",
                description: "Every 2 seconds, launches spike projectiles at up to 3 nearby enemies, dealing 5 damage each.",
                locked_text: "The spikes have learned to fly. Nobody asked for this.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn healing_plume_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Rejuvenating Mists",
                description: "Healing per tick increased by 40%.",
                locked_text: "Stronger healing. The plume went to medical school. Very briefly.",
                implemented: true,
            },
            TalentDefinition {
                name: "Verdant Bloom",
                description: "Zone radius increased by 40%.",
                locked_text: "Bigger healing cloud. Stand in the green circle. Why is this so hard for people?",
                implemented: true,
            },
            TalentDefinition {
                name: "Lasting Remedy",
                description: "Zone duration increased by 50%.",
                locked_text: "The healing lingers. Like the smell of the wizard's potions. Less pleasant, equally effective.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Cleansing Plume",
                description: "Removes debuffs (slow, root, vulnerability) from all units inside the zone.",
                locked_text: "It heals AND cleans. The wizard invented magical soap.",
                implemented: true,
            },
            TalentDefinition {
                name: "Overflow",
                description: "Healing that exceeds max HP becomes temporary HP (up to 20 temp HP).",
                locked_text: "Can't waste the extra healing. Redirect it. The wizard is an efficiency expert.",
                implemented: true,
            },
            TalentDefinition {
                name: "Triage Pulse",
                description: "Allies below 30% HP are healed for double the amount.",
                locked_text: "The sicker you are, the harder it works. The plume is a workaholic.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Font of Life",
                description: "Units that die inside the healing zone are resurrected at 25% HP after 3 seconds. Once per unit.",
                locked_text: "Death is temporary. Healing is permanent. The plume has strong opinions about mortality.",
                implemented: true,
            },
            TalentDefinition {
                name: "Healing Rain",
                description: "The healing zone follows your cursor. Healing per tick reduced by 25%.",
                locked_text: "Portable healing. The wizard is basically an ambulance now.",
                implemented: true,
            },
            TalentDefinition {
                name: "Field Medic",
                description: "The nearest defender in the zone is temporarily converted into a healer for the zone's duration. They fire heal bolts at hurt allies.",
                locked_text: "One soldier puts down their sword and picks up a first aid kit. Involuntarily.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn polymorph_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Extended Transformation",
                description: "Polymorph duration increased to 14 seconds.",
                locked_text: "More time as a sheep. The enemy's wool production is off the charts.",
                implemented: true,
            },
            TalentDefinition {
                name: "Fragile Form",
                description: "Sheep HP reduced to 5 (from 20). Much easier to kill.",
                locked_text: "A very fragile sheep. One good sneeze and it's over.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Shapeshift",
                description: "Cast time reduced by 40%.",
                locked_text: "Baa faster. Wait, that's not-- cast faster.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Explosive Sheep",
                description: "If the sheep is killed, it explodes for medium AoE damage.",
                locked_text: "The sheep explodes. Nobody expected the sheep to explode. NOBODY.",
                implemented: true,
            },
            TalentDefinition {
                name: "Contagious Baas",
                description: "When the polymorph expires, it jumps to the nearest unit. Keeps jumping forever.",
                locked_text: "Sheep making more sheep. The wizard has created a woolen pyramid scheme.",
                implemented: true,
            },
            TalentDefinition {
                name: "Pig Form",
                description: "Target becomes a pig instead. Pig runs away from combat at high speed.",
                locked_text: "Not a sheep? A pig? The wizard is branching out into animal husbandry.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Permanent Livestock",
                description: "If the sheep survives its full duration, the transformation is permanent.",
                locked_text: "The sheep IS the enemy now. It lives on the farm. It seems happy, actually.",
                implemented: true,
            },
            TalentDefinition {
                name: "Mass Polymorph",
                description: "Transforms all enemies in a small radius into sheep. Very high mana cost.",
                locked_text: "An entire flock. The wizard has become a shepherd. A very aggressive shepherd.",
                implemented: true,
            },
            TalentDefinition {
                name: "Dire Sheep",
                description: "The sheep is friendly, has 200 HP, and headbutts enemies for moderate damage.",
                locked_text: "It's a BATTLE sheep. Big horns. Bad attitude. The enemies are deeply confused.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn grease_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Extra Slippery",
                description: "Slow effect increased by 30%.",
                locked_text: "MORE slippery. The enemies can barely stand. Comedy gold.",
                implemented: true,
            },
            TalentDefinition {
                name: "Wider Slick",
                description: "Zone radius increased by 40%.",
                locked_text: "Bigger puddle. The wizard should have been a plumber.",
                implemented: true,
            },
            TalentDefinition {
                name: "Volatile Mixture",
                description: "When ignited, fire damage increased by 50%.",
                locked_text: "The grease burns better now. Better for us, not for them.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Slip and Fall",
                description: "Enemies entering the grease have a 30% chance to fall prone (stunned 1.5s).",
                locked_text: "They fall down. In the grease. This is the wizard's finest hour.",
                implemented: true,
            },
            TalentDefinition {
                name: "Oil Slick",
                description: "Enemies in the grease take 20% more spell damage.",
                locked_text: "The oil gets in the joints. The armor joints. This is not a massage.",
                implemented: true,
            },
            TalentDefinition {
                name: "Lingering Flames",
                description: "Ignited grease resets its duration, burning for the full zone lifetime.",
                locked_text: "Some fires just don't know when to quit. This one never learned.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Chain Combustion",
                description: "When one grease zone ignites, fire spreads to any other grease zones within double range.",
                locked_text: "The fires connect. The wizard's arson network is now fully operational.",
                implemented: true,
            },
            TalentDefinition {
                name: "Grease Geyser",
                description: "On ignition, grease erupts upward, launching enemies into the air. They take fall damage on landing.",
                locked_text: "Oil + fire = geyser. The wizard is basically an oil derrick now.",
                implemented: true,
            },
            TalentDefinition {
                name: "Endless Oil",
                description: "After burning out, grease zone regenerates over 10 seconds, becoming slippery again. Can be re-ignited.",
                locked_text: "It comes back. It always comes back. The self-replenishing grease. Disgusting AND effective.",
                implemented: true,
            },
        ],
    ]
}
