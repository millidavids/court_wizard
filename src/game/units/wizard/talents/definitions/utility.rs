use super::TalentDefinition;

pub(super) fn battle_hymn_talents() -> [[TalentDefinition; 3]; 3] {
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

pub(super) fn guardian_circle_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Reinforced Wards",
                description: "Temp HP amount increased by 40%.",
                locked_text: "Thicker shields. The wizard's solution to every problem since wizard school.",
                implemented: true,
            },
            TalentDefinition {
                name: "Enduring Protection",
                description: "Temp HP duration increased by 60%.",
                locked_text: "The shield lasts so long the defenders forget they have one.",
                implemented: true,
            },
            TalentDefinition {
                name: "Expansive Aegis",
                description: "Circle radius increased by 50%, but temp HP amount reduced by 15%.",
                locked_text: "A wider net catches more soldiers. Metaphor works either way.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Retaliating Wards",
                description: "When temp HP is fully broken, a burst of force damage hits nearby enemies.",
                locked_text: "Hit the shield, the shield hits back. Karma, but magical.",
                implemented: true,
            },
            TalentDefinition {
                name: "Fortified Resolve",
                description: "Shielded units deal 20% more damage while they have temp HP.",
                locked_text: "A shield makes you braver. Bravery makes you hit harder.",
                implemented: true,
            },
            TalentDefinition {
                name: "Rapid Deployment",
                description: "Cast time halved.",
                locked_text: "Two circles? At the same time? The wizard is clearly showing off.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Sanctuary",
                description: "Shielded units also take 30% reduced damage while they have temp HP.",
                locked_text: "Step inside the glowing circle. Yes, I know how that sounds. Trust me.",
                implemented: true,
            },
            TalentDefinition {
                name: "Martyrdom",
                description: "When a shielded unit dies, its remaining temp HP explodes as AoE damage to nearby enemies.",
                locked_text: "They gave their life. And their shield. And everyone nearby's eardrums.",
                implemented: true,
            },
            TalentDefinition {
                name: "Chain Ward",
                description: "When a shielded unit dies, its temp HP jumps to the nearest unshielded ally. Up to 3 hops.",
                locked_text: "The shield of the fallen passes to the living. Very poetic. Very practical.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn sleep_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Deep Slumber",
                description: "Sleep duration increased by 40%.",
                locked_text: "They sleep deeper. The wizard could learn from them. He hasn't slept in days.",
                implemented: true,
            },
            TalentDefinition {
                name: "Lullaby",
                description: "Circle radius increased by 40%.",
                locked_text: "A bigger nap zone. Like a very aggressive daycare.",
                implemented: true,
            },
            TalentDefinition {
                name: "Nightmare Fuel",
                description: "Wake-up bonus damage increased by 50%.",
                locked_text: "They wake up and IMMEDIATELY regret it.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Narcoleptic Wave",
                description: "Sleep spreads to nearby awake enemies over 3 seconds (expanding radius).",
                locked_text: "The sleep is contagious. Like yawning, but magical and involuntary.",
                implemented: true,
            },
            TalentDefinition {
                name: "Night Terrors",
                description: "Sleeping enemies take minor damage per second (not enough to wake them).",
                locked_text: "They dream of being hurt. The dreams are accurate.",
                implemented: true,
            },
            TalentDefinition {
                name: "Drowsy",
                description: "Cast time halved. Mana cost reduced by 25%.",
                locked_text: "Quicker casting. The wizard is getting sleepy just thinking about it. Hmm.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Comatose",
                description: "Sleeping enemies can only be woken by taking damage equal to 30% of their max HP. Small damage doesn't wake them.",
                locked_text: "They sleep through everything. EVERYTHING. The wizard's roommate in wizard college was like this.",
                implemented: true,
            },
            TalentDefinition {
                name: "Dreamwalker",
                description: "Sleeping enemies sleepwalk back toward their spawn at half speed for 30 seconds. They don't target or attack while sleepwalking.",
                locked_text: "They walk in their sleep. Away from the castle. The wizard finds this hilarious.",
                implemented: true,
            },
            TalentDefinition {
                name: "Eternal Slumber",
                description: "Enemies that fall asleep below 25% HP never wake up (instant kill).",
                locked_text: "They drift off peacefully. No refunds. No returns. No waking.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn haste_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Alacrity",
                description: "Speed bonus increased by 40%.",
                locked_text: "Faster fast. The fasting of the fast. Fast.",
                implemented: true,
            },
            TalentDefinition {
                name: "Extended Rush",
                description: "Buff duration increased by 50%.",
                locked_text: "They stay fast for longer. The wizard invented caffeine, basically.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Cast",
                description: "Cast time reduced by 50%.",
                locked_text: "Casting a speed spell... faster. The irony is not lost on the wizard.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Adrenaline Surge",
                description: "Hasted units also gain +20% attack speed.",
                locked_text: "They move fast AND hit fast. They're basically hummingbirds with swords.",
                implemented: true,
            },
            TalentDefinition {
                name: "Momentum",
                description: "Hasted units deal 25% more damage for 2 seconds after the buff ends.",
                locked_text: "Newton's first law: a soldier in motion stays in motion. And hits harder.",
                implemented: true,
            },
            TalentDefinition {
                name: "Fleet Feet",
                description: "Hasted units dodge the first attack made against them.",
                locked_text: "So fast the attacks miss. Honestly, this feels like cheating.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Time Warp",
                description: "Hasted units take their turns twice as fast (double attack speed AND movement). Duration halved.",
                locked_text: "Speeding up time itself. The wizard's calendar just had a panic attack.",
                implemented: true,
            },
            TalentDefinition {
                name: "Slow Zone",
                description: "Haste also creates a lingering slow field on the ground for enemies. Speed buff for allies, slow for enemies in the same zone.",
                locked_text: "Fast friends, slow enemies. The wizard's approach to time management.",
                implemented: true,
            },
            TalentDefinition {
                name: "Chain Haste",
                description: "When the buff expires, it jumps to the nearest un-hasted ally. Loses 20% effectiveness per jump. Up to 4 jumps.",
                locked_text: "The speed is contagious. Patient zero is doing laps around the castle.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn teleport_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Wide Aperture",
                description: "Source circle radius increased by 50%.",
                locked_text: "Bigger circle means more soldiers. Or more confused soldiers. Either way.",
                implemented: true,
            },
            TalentDefinition {
                name: "Hasty Translocation",
                description: "Second cast time reduced by 40%.",
                locked_text: "Teleporting faster means less time to reconsider your terrible destination choice.",
                implemented: true,
            },
            TalentDefinition {
                name: "Lingering Gate",
                description: "Destination marker persists for 5 seconds, allowing a second teleport to the same spot.",
                locked_text: "The portal stays open. In case you forgot someone. Or your keys.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Disorienting Arrival",
                description: "Teleported units are stunned for 2 seconds on arrival, then gain +20% attack speed for 3s.",
                locked_text: "Surprise! You're somewhere else now. Take a moment. Take several moments.",
                implemented: true,
            },
            TalentDefinition {
                name: "Swap",
                description: "Instead of place-then-teleport, swap all units between two circles simultaneously.",
                locked_text: "Musical chairs, but with soldiers and existential dread.",
                implemented: true,
            },
            TalentDefinition {
                name: "Emergency Recall",
                description: "Can be instant-cast without a destination to teleport all allies in radius back to the King's position.",
                locked_text: "EVERYONE COME HOME. NOW. No, I don't care what you were doing.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Dimensional Rift",
                description: "Creates a persistent two-way portal between source and destination for 10 seconds. Units walk through freely.",
                locked_text: "A door. The wizard invented a door. A magical, reality-bending door, but still.",
                implemented: true,
            },
            TalentDefinition {
                name: "Up",
                description: "Single cast: teleports all units at the target straight up into the sky. They fall back down and take damage on impact.",
                locked_text: "What goes up must come down. Preferably on their heads.",
                implemented: true,
            },
            TalentDefinition {
                name: "Scatterport",
                description: "Teleports each unit in the source to a random location within a large radius. Lower mana cost.",
                locked_text: "Where did they go? Everywhere. Literally everywhere. Good luck regrouping.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn mind_control_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Iron Will",
                description: "All mind control effects last 40% longer.",
                locked_text: "Longer control. The wizard's grip on their mind is uncomfortably firm.",
                implemented: true,
            },
            TalentDefinition {
                name: "Deep Domination",
                description: "Controlled units deal 25% more damage.",
                locked_text: "They fight harder for you than they ever fought for themselves. That's either inspiring or depressing.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Subjugation",
                description: "Cast time reduced by 40%.",
                locked_text: "Faster mind control. Consent is not a factor. Neither is ethics class.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Puppet Master",
                description: "Max controlled units increased to 5.",
                locked_text: "More puppets. The wizard's hand is getting tired. Metaphorically. Mind-hands don't cramp.",
                implemented: true,
            },
            TalentDefinition {
                name: "Traitor's Mark",
                description: "Controlled enemies cause nearby enemies to take 15% more damage (demoralization).",
                locked_text: "Their friends turned on them. That hurts. Also the swords hurt. Everything hurts.",
                implemented: true,
            },
            TalentDefinition {
                name: "Amnesia",
                description: "When mind control ends, the target is confused for 3 seconds (attacks random targets).",
                locked_text: "They forgot which side they were on. Then they forgot what 'sides' are.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Dominate",
                description: "Controlled unit is permanent until it dies. Only one permanent unit at a time.",
                locked_text: "They work for you now. Forever. The wizard skipped the part about 'temporary' in the spellbook.",
                implemented: true,
            },
            TalentDefinition {
                name: "Mass Hysteria",
                description: "All enemies in a radius attack each other for 4 seconds. Not true mind control. Costs all your mana.",
                locked_text: "Everyone fights everyone. It's like Black Friday but with swords.",
                implemented: true,
            },
            TalentDefinition {
                name: "Sleeper Agent",
                description: "Controlled enemy appears normal when MC ends, but betrays allies after 5 seconds with a 200% damage attack.",
                locked_text: "Trust issues: the spell. They look normal. They seem fine. They are NOT fine.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn fog_cloud_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Dense Fog",
                description: "Evasion chance increased to 55%.",
                locked_text: "Thicker fog. Harder to see. The wizard can't see either, but that's a feature.",
                implemented: true,
            },
            TalentDefinition {
                name: "Expanding Mists",
                description: "Zone radius increased by 40%.",
                locked_text: "More fog. The visibility report says 'no.'",
                implemented: true,
            },
            TalentDefinition {
                name: "Clinging Haze",
                description: "Evasion persists for 2 seconds after leaving the fog.",
                locked_text: "The fog follows you. Clingy fog. The fog has attachment issues.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Blinding Mist",
                description: "Units inside the fog have their attack range halved.",
                locked_text: "They can't see far. They're swinging at shadows. Some of those shadows swing back.",
                implemented: true,
            },
            TalentDefinition {
                name: "Concealing Veil",
                description: "Units inside the fog cannot be targeted by ranged attacks from outside the fog.",
                locked_text: "Out of sight, out of range. The wizard invented stealth technology.",
                implemented: true,
            },
            TalentDefinition {
                name: "Disorienting Vapors",
                description: "Units inside have a 20% chance to attack an ally instead of their target.",
                locked_text: "The fog makes them confused. They start hitting each other. Entertainment value: priceless.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Phantom Fog",
                description: "Fog creates illusory defenders that units waste attacks on (30% of attacks target phantoms).",
                locked_text: "Fake soldiers in the fog. The units can't tell what's real. Neither can the wizard, honestly.",
                implemented: true,
            },
            TalentDefinition {
                name: "Choking Fog",
                description: "Fog also deals minor damage per second to all units inside.",
                locked_text: "The fog fights back. It's gone from cover to weapon. Fog with an attitude.",
                implemented: true,
            },
            TalentDefinition {
                name: "Rolling Fog",
                description: "Fog slowly moves in the direction units are coming from, meeting them earlier.",
                locked_text: "The fog approaches. It has places to be. People to obscure.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn arcane_crystal_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Refined Facets",
                description: "Sub-projectile damage increased by 25%.",
                locked_text: "Sharper crystal, sharper projectiles. The wizard polished it. With magic, not a cloth.",
                implemented: true,
            },
            TalentDefinition {
                name: "Wider Prism",
                description: "Crystal range increased by 40%.",
                locked_text: "A bigger target to aim at. Even the wizard can hit it now.",
                implemented: true,
            },
            TalentDefinition {
                name: "Enduring Crystal",
                description: "Crystal duration increased by 30%.",
                locked_text: "The crystal lasts longer. It's the Energizer Bunny of magical artifacts.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Overcharged Matrix",
                description: "Sub-projectile count increased by 50% (rounded up).",
                locked_text: "More projectiles per spell. The crystal is an overachiever.",
                implemented: true,
            },
            TalentDefinition {
                name: "Resonance Cascade",
                description: "Crystal stores absorbed spell energy. After 3 absorptions, emits a powerful burst in all directions.",
                locked_text: "Charge it up, let it rip. The crystal has been watching too many anime.",
                implemented: true,
            },
            TalentDefinition {
                name: "Spell Echo",
                description: "Crystal has a 30% chance to duplicate an absorbed spell entirely (full damage copy).",
                locked_text: "Sometimes the crystal just... copies your homework. Magical plagiarism.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Crystal Network",
                description: "Place up to 3 crystals. Spells chain between crystals, amplifying at each one.",
                locked_text: "A network of crystals. The wizard built a magical internet. It runs on fireballs.",
                implemented: true,
            },
            TalentDefinition {
                name: "Prismatic Explosion",
                description: "Crystal explodes when it expires, dealing massive damage of every damage type in a large radius.",
                locked_text: "The crystal goes out in a blaze of... every element. It's beautiful AND lethal.",
                implemented: true,
            },
            TalentDefinition {
                name: "Auto-Crystal",
                description: "Crystal becomes a permanent magic missile turret. One per level, persists between levels unless dispelled. No longer absorbs spells.",
                locked_text: "The crystal casts spells by itself. The wizard is being replaced by a ROCK.",
                implemented: true,
            },
        ],
    ]
}
