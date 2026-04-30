use super::TalentDefinition;

pub(super) fn mark_of_death_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Deep Mark",
                description: "Damage amplification increased to 75%.",
                locked_text: "More vulnerable. MORE. The wizard has no concept of 'enough.'",
                implemented: true,
            },
            TalentDefinition {
                name: "Lingering Curse",
                description: "Mark duration increased to 12 seconds.",
                locked_text: "The mark sticks around. Like a bad reputation. Or glitter.",
                implemented: true,
            },
            TalentDefinition {
                name: "Swift Hex",
                description: "Marking refunds 50% of its mana cost if the target dies while marked.",
                locked_text: "Mark, kill, refund. The wizard invented magical couponing.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Spreading Blight",
                description: "If a marked target dies, the mark jumps to the nearest enemy (50% remaining duration).",
                locked_text: "The mark is contagious. Don't worry, it's only fatal.",
                implemented: true,
            },
            TalentDefinition {
                name: "Executioner's Brand",
                description: "Marked targets take an additional burst of damage when they fall below 30% HP.",
                locked_text: "Kicking them while they're down. The wizard learned from cats.",
                implemented: true,
            },
            TalentDefinition {
                name: "Focal Point",
                description: "Defender units prioritize attacking marked targets.",
                locked_text: "HIT THAT ONE. THAT ONE. The wizard has become a backseat driver for combat.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Mass Marking",
                description: "Mark affects all enemies in a radius instead of a single target. Damage amp reduced to 35%.",
                locked_text: "Everyone's marked. It's like a very aggressive roll call.",
                implemented: true,
            },
            TalentDefinition {
                name: "Death's Ledger",
                description: "Marked enemies that die cause a necrotic explosion. More HP the target had = bigger explosion.",
                locked_text: "In death, they give back to the community. Violently.",
                implemented: true,
            },
            TalentDefinition {
                name: "Doom",
                description: "Mark cannot be removed and damage amplification increases by 10% per second.",
                locked_text: "The mark grows. The mark hungers. The wizard is slightly concerned about the mark.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn plague_wind_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Numeric modifiers
        [
            TalentDefinition {
                name: "Virulent Strain",
                description: "Poison damage increased by 60%.",
                locked_text: "This strain of plague was banned by the Geneva Suggestion.",
                implemented: true,
            },
            TalentDefinition {
                name: "Miasma",
                description: "Cloud radius increased by 50%, but duration reduced by 25%.",
                locked_text: "Wider coverage, shorter stay. Like a bad house guest.",
                implemented: true,
            },
            TalentDefinition {
                name: "Lingering Fog",
                description: "Duration increased by 50% and cloud speed reduced by 50%.",
                locked_text: "It's not going anywhere. Neither are they.",
                implemented: true,
            },
        ],
        // Tier 2: Secondary effects
        [
            TalentDefinition {
                name: "Plague Carrier",
                description: "Units that leave the cloud continue to take 50% poison damage for 3 seconds.",
                locked_text: "They can run, but the plague runs faster.",
                implemented: true,
            },
            TalentDefinition {
                name: "Toxic Weakness",
                description: "Units inside the cloud take 25% more damage from all sources.",
                locked_text: "The poison doesn't just hurt. It makes everything else hurt more.",
                implemented: true,
            },
            TalentDefinition {
                name: "Choking Gas",
                description: "Units inside the cloud are slowed by 40%.",
                locked_text: "Hard to swing a sword when you can't breathe.",
                implemented: true,
            },
        ],
        // Tier 3: Transformative upgrades
        [
            TalentDefinition {
                name: "Pandemic",
                description: "When an enemy dies inside the cloud, a half-size cloud spawns at their position for 4 seconds.",
                locked_text: "Patient zero was just the beginning.",
                implemented: true,
            },
            TalentDefinition {
                name: "Twin Plumes",
                description: "Spawns 2 clouds that drift at 45° angles apart instead of 1, each at 65% damage.",
                locked_text: "Two clouds are better than one. The enemy's lungs disagree.",
                implemented: true,
            },
            TalentDefinition {
                name: "Necrotic Rot",
                description: "Poison damage also permanently reduces the target's max health by the same amount.",
                locked_text: "Some wounds don't heal. The wizard calls this a 'feature.'",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn raise_the_dead_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Mass Graves",
                description: "Resurrection radius increased by 60%.",
                locked_text: "Cast a wider net. For corpses. This job description is something else.",
                implemented: true,
            },
            TalentDefinition {
                name: "Hasty Ritual",
                description: "Channeling speed starts at max speed instead of ramping up.",
                locked_text: "Skipping the warm-up chant. The dead don't care about proper procedure.",
                implemented: true,
            },
            TalentDefinition {
                name: "Efficient Necromancy",
                description: "Mana cost per corpse reduced by 30%.",
                locked_text: "Raising the dead on a budget. Student loans don't pay themselves. Wait, or do they?",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Empowered Undead",
                description: "Raised undead have 50% more HP and deal 25% more damage.",
                locked_text: "Better zombies through magic. If you're going to reanimate, reanimate with style.",
                implemented: true,
            },
            TalentDefinition {
                name: "Plague Bearer",
                description: "Raised undead emit a poison aura, dealing damage to nearby living enemies.",
                locked_text: "The undead don't just fight. They bring ambiance. Toxic, lethal ambiance.",
                implemented: true,
            },
            TalentDefinition {
                name: "Corpse Magnet",
                description: "Corpses within a large radius are pulled toward the cursor before resurrection.",
                locked_text: "The corpses come to you! It's like a drive-through but for necromancy.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Revenant Lord",
                description: "Raised undead become powerful Revenants with 5x HP and heavy damage that passively resurrect nearby corpses.",
                locked_text: "One champion of the dead is worth fifty shambling corpses. Economical AND terrifying.",
                implemented: true,
            },
            TalentDefinition {
                name: "Undead Detonation",
                description: "Raised undead explode when they die (again), dealing heavy damage in a radius.",
                locked_text: "They were already dead, so technically this is recycling.",
                implemented: true,
            },
            TalentDefinition {
                name: "Perpetual Unrest",
                description: "When a raised undead kills an enemy, that enemy is automatically raised. No mana cost.",
                locked_text: "It's a pyramid scheme, but with zombies. Honestly, most pyramid schemes already are.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn berserker_rage_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Blood Fury",
                description: "Damage bonus increased to 120%. Vulnerability increased to 65%.",
                locked_text: "More damage in both directions. The berserker philosophy: hit harder, consequences later.",
                implemented: true,
            },
            TalentDefinition {
                name: "Controlled Rage",
                description: "Vulnerability reduced to 30%. Damage bonus reduced to 60%.",
                locked_text: "Slightly calmer rage. The berserker took one anger management class.",
                implemented: true,
            },
            TalentDefinition {
                name: "Primal Roar",
                description: "Buff radius increased by 50%.",
                locked_text: "EVERYONE GETS ANGRY. EVERYONE. Even the archers in the back.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Bloodlust",
                description: "Enraged units heal for 15% of damage dealt.",
                locked_text: "They heal by hitting things. The wizard discovered the world's most violent therapy.",
                implemented: true,
            },
            TalentDefinition {
                name: "Undying Fury",
                description: "Enraged units that would die instead survive at 1 HP for 2 seconds.",
                locked_text: "Too angry to die. Briefly. Very briefly.",
                implemented: true,
            },
            TalentDefinition {
                name: "Frenzy",
                description: "Enraged units gain +30% attack speed as their HP drops below 50%.",
                locked_text: "Lower health, faster attacks. They're panicking productively.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Wrath Incarnate",
                description: "Enraged units deal 200% bonus damage but vulnerability is 100% (double damage taken).",
                locked_text: "Glass cannon mode. Emphasis on GLASS. And CANNON.",
                implemented: true,
            },
            TalentDefinition {
                name: "Contagious Rage",
                description: "When an enraged unit kills an enemy, the nearest calm ally becomes enraged.",
                locked_text: "The rage spreads. It's like office drama but with axes.",
                implemented: true,
            },
            TalentDefinition {
                name: "Final Stand",
                description: "If an enraged unit dies, they explode for damage equal to 50% of their max HP.",
                locked_text: "Even in death, they rage. That's commitment to the bit.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn banishment_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Extended Exile",
                description: "Banish duration increased to 12 seconds.",
                locked_text: "Gone for longer. Where do they go? The wizard's pocket dimension. It's mostly storage.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Dismissal",
                description: "Cast time reduced by 50%.",
                locked_text: "Faster banishment. 'You're gone. NEXT.'",
                implemented: true,
            },
            TalentDefinition {
                name: "Cheap Ticket",
                description: "Mana cost reduced by 30%.",
                locked_text: "Economy class banishment. Same destination, less magical overhead.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Painful Return",
                description: "Banished units take heavy damage when they return.",
                locked_text: "Welcome back! Here's your complementary damage.",
                implemented: true,
            },
            TalentDefinition {
                name: "Displacement",
                description: "Banished enemy reappears at a random location far from where it was banished.",
                locked_text: "They come back but have NO idea where they are. Cosmic disorientation.",
                implemented: true,
            },
            TalentDefinition {
                name: "Dual Banishment",
                description: "Can banish 2 targets simultaneously. Second target costs 50% mana.",
                locked_text: "Two disappearances for the price of one and a half.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Dimensional Shunt",
                description: "Banished units return at half HP regardless of their HP when banished.",
                locked_text: "The pocket dimension is not a pleasant vacation. Zero stars. Would not recommend.",
                implemented: true,
            },
            TalentDefinition {
                name: "Mass Banishment",
                description: "Banishes all units in a radius. Very high mana cost. Short duration (4s).",
                locked_text: "Everyone disappears. The battlefield is briefly a very peaceful meadow.",
                implemented: true,
            },
            TalentDefinition {
                name: "One-Way Trip",
                description: "If the banished enemy's HP is below 20% when banished, they don't come back.",
                locked_text: "Some trips are one-way. The wizard doesn't make the rules. Actually, the wizard does make the rules.",
                implemented: true,
            },
        ],
    ]
}
