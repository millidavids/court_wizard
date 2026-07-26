use super::TalentDefinition;

pub(super) fn magic_missile_talents() -> [[TalentDefinition; 3]; 3] {
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
                name: crate::game::units::wizard::spells::magic_missile::constants::ARCANE_BARRAGE_NAME,
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

pub(super) fn telekinesis_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Auto-Target",
                description: "No need to aim near a drop. Casting anywhere auto-targets the nearest drop on the field.",
                locked_text: "Point anywhere. The magic knows what you want.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Grab",
                description: "Cast time is near-instant.",
                locked_text: "Blink and you'll miss it. Literally.",
                implemented: true,
            },
            TalentDefinition {
                name: "Mana Efficiency",
                description: "Mana cost reduced by 50%.",
                locked_text: "Same spell, half the magical effort.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Magnetic Pull",
                description: "Ingredients passively drift toward the wizard.",
                locked_text: "The ingredients come to you. As it should be.",
                implemented: true,
            },
            TalentDefinition {
                name: "Harvest",
                description: "Picking up an ingredient deals minor damage to nearby enemies.",
                locked_text: "Every pickup is a tiny explosion. Convenient.",
                implemented: true,
            },
            TalentDefinition {
                name: "Keen Senses",
                description: "Ingredient drop chance increased by 50% while Telekinesis is equipped.",
                locked_text: "You see things others miss. Shiny things.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Telekinetic Storm",
                description: "Single cast pulls ALL ingredients on the battlefield. Costs 3x mana per ingredient.",
                locked_text: "Why pick up one when you can grab them all?",
                implemented: true,
            },
            TalentDefinition {
                name: "Transmutation",
                description: "Each ingredient collected grants +10% brew potency (stacking). Resets on brew.",
                locked_text: "The ingredients whisper their secrets to you.",
                implemented: true,
            },
            TalentDefinition {
                name: "Psychic Shockwave",
                description: "Each pickup creates a knockback shockwave pushing enemies away.",
                locked_text: "BOOM. Ingredient collected. Enemies scattered.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn black_hole_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Numeric modifiers
        [
            TalentDefinition {
                name: "Denser Core",
                description: "Gravity strength increased by 50%.",
                locked_text: "The heavier the core, the harder they fall. Literally.",
                implemented: true,
            },
            TalentDefinition {
                name: "Expansive Void",
                description: "Max radius increased by 40%, but damage reduced by 20%.",
                locked_text: "Wider is better. Unless you're the one getting pulled in.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Collapse",
                description: "Cast time reduced by 40%.",
                locked_text: "Faster deployment. The void waits for no one.",
                implemented: true,
            },
        ],
        // Tier 2: Secondary effects
        [
            TalentDefinition {
                name: "Event Horizon",
                description: "Units within 25% of center take double damage.",
                locked_text: "Past a certain point, there is no return. Only pain.",
                implemented: true,
            },
            TalentDefinition {
                name: "Crushing Pressure",
                description: "Units inside the black hole are slowed by 40%.",
                locked_text: "Gravity doesn't just pull. It crushes.",
                implemented: true,
            },
            TalentDefinition {
                name: "Void Siphon",
                description: "30% of damage dealt heals the nearest injured defender.",
                locked_text: "The void takes from them and gives to yours. Fair trade.",
                implemented: true,
            },
        ],
        // Tier 3: Transformative upgrades
        [
            TalentDefinition {
                name: "Singularity",
                description: "When the black hole expires, it collapses dealing 150 damage to all units inside.",
                locked_text: "Everything that goes in must come out. Explosively.",
                implemented: true,
            },
            TalentDefinition {
                name: "Twin Stars",
                description: "Spawns 2 black holes at 60% size and gravity instead of 1.",
                locked_text: "Binary star systems are nature's way of saying 'why not both?'",
                implemented: true,
            },
            TalentDefinition {
                name: "Dimensional Rift",
                description: "Every 2 seconds, all enemies inside are teleported to the center and take 30 burst damage.",
                locked_text: "The rift has opinions about where you should stand. The center. Always the center.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn finger_of_death_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Beam Modifiers
        [
            TalentDefinition {
                name: "Death's Reach",
                description: "Beam width increased by 50%, can hit multiple enemies in a line.",
                locked_text: "The finger of death got fatter. We don't talk about it.",
                implemented: true,
            },
            TalentDefinition {
                name: "Soul Harvest",
                description: "Killing a target refunds 15% of max mana.",
                locked_text: "Death pays dividends. The wizard's retirement plan is horrifying.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Draw",
                description: "Cast time reduced by 40%.",
                locked_text: "Point and shoot. The finger of death gets a speed upgrade. Enemies hate this one trick.",
                implemented: true,
            },
        ],
        // Tier 2: Resource/Utility
        [
            TalentDefinition {
                name: "Finger of Undeath",
                description: "Killed targets are instantly raised as undead allies.",
                locked_text: "Killed them AND recruited them. Two birds, one very spooky stone.",
                implemented: true,
            },
            TalentDefinition {
                name: "Death Sentence",
                description: "Mana threshold reduced to 10%. Damage increased to 140% of the target's health. Cooldown reduced by 50%.",
                locked_text: "Cheaper, deadlier, faster. The fast food of death magic.",
                implemented: true,
            },
            TalentDefinition {
                name: "Siphon Life",
                description: "Beam heals the nearest defender for 50% of damage dealt.",
                locked_text: "Someone had to die so that someone else could live. The wizard calls this 'triage.'",
                implemented: true,
            },
        ],
        // Tier 3: Transformative
        [
            TalentDefinition {
                name: "Reaper's Scythe",
                description: "Beam sweeps in an arc over 1 second, hitting everything in its path. Damage reduced to 30% of each target's health.",
                locked_text: "The finger of death goes through a phase. A 'sweep everything' phase.",
                implemented: true,
            },
            TalentDefinition {
                name: "Necrotic Explosion",
                description: "Killed targets explode, dealing 20% of nearby victims' health in a medium radius.",
                locked_text: "They're already dead, what's a little explosion going to do? Oh, to the people AROUND them.",
                implemented: true,
            },
            TalentDefinition {
                name: "Deathmark",
                description: "Beam instead applies a 5-second debuff. If the target dies during the debuff, a second Finger of Death fires at the nearest enemy automatically (at 10% damage).",
                locked_text: "Kill chain. The only chain letter anyone actually follows up on.",
                implemented: true,
            },
        ],
    ]
}

pub(super) fn dispel_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Broad Spectrum",
                description: "Dispel projectile also removes buffs from units hit (not just spell zones).",
                locked_text: "Dispels everything. Buffs, debuffs, self-confidence. Gone.",
                implemented: true,
            },
            TalentDefinition {
                name: "Swift Cancellation",
                description: "Cooldown between casts reduced by 40%.",
                locked_text: "Faster dispelling. For when one magical mistake isn't enough.",
                implemented: true,
            },
            TalentDefinition {
                name: "Efficient Nullification",
                description: "Mana cost reduced to near-zero.",
                locked_text: "Free dispels. The wizard's most cost-effective spell. Accountants love this.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Mana Drain",
                description: "Dispelling an effect restores mana to the wizard equal to 50% of the dispelled spell's cost.",
                locked_text: "Destroying magic AND getting paid for it. The wizard is basically a magical recycler.",
                implemented: true,
            },
            TalentDefinition {
                name: "Explosive Nullification",
                description: "Dispelled spell effects detonate on removal, dealing damage in a small radius.",
                locked_text: "The spell doesn't just go away. It goes away VIOLENTLY.",
                implemented: true,
            },
            TalentDefinition {
                name: "Counterspell",
                description: "Projectile travels 50% faster and has a 25% larger collision radius.",
                locked_text: "Faster, bigger, harder to dodge. The dispel projectile has been working on itself.",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Antimagic Pulse",
                description: "Replaces projectile with an instant radial pulse at the target location that dispels ALL spell effects in a large radius.",
                locked_text: "EVERYTHING STOPS. All magic, gone. The wizard is now the fun police.",
                implemented: true,
            },
            TalentDefinition {
                name: "Spell Reflection",
                description: "Dispelled offensive spells are redirected back at the nearest enemy.",
                locked_text: "Return to sender. The postal service of magical warfare.",
                implemented: true,
            },
            TalentDefinition {
                name: "Null Zone",
                description: "Dispel creates a persistent anti-magic zone at the impact point for 5s. No spells function inside.",
                locked_text: "A zone where magic doesn't work. The wizard has created a very awkward dead zone at parties.",
                implemented: true,
            },
        ],
    ]
}
