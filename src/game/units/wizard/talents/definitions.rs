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
        Spell::Disintegrate => disintegrate_talents(),
        Spell::MeteorFall => meteor_fall_talents(),
        Spell::ChainLightning => chain_lightning_talents(),
        Spell::Telekinesis => telekinesis_talents(),
        Spell::GuardianCircle => guardian_circle_talents(),
        Spell::FingerOfDeath => finger_of_death_talents(),
        Spell::MarkOfDeath => mark_of_death_talents(),
        Spell::LightningRod => lightning_rod_talents(),
        Spell::BlackHole => black_hole_talents(),
        Spell::PlagueWind => plague_wind_talents(),
        Spell::WallOfFire => wall_of_fire_talents(),
        Spell::WallOfStone => wall_of_stone_talents(),
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

fn disintegrate_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Focused Lens",
                description: "Beam width reduced by 40%, but damage increased by 30%.",
                locked_text: "A tighter beam cuts deeper. Laser surgery, wizard style.",
                implemented: true,
            },
            TalentDefinition {
                name: "Unfocused Beam",
                description: "Beam is twice as wide but deals only 30% damage.",
                locked_text: "Wider coverage, less intensity. Sometimes more is less.",
                implemented: true,
            },
            TalentDefinition {
                name: "Efficient Channeling",
                description: "Mana cost per second reduced by 30%.",
                locked_text: "Same death ray, less mana drain. Efficiency matters.",
                implemented: true,
            },
        ],
        // Tier 2
        [
            TalentDefinition {
                name: "Forked Beam",
                description: "Fire 3 beams in a fan pattern, each dealing 50% damage.",
                locked_text: "One beam is good. Three beams is three times as good.",
                implemented: true,
            },
            TalentDefinition {
                name: "Escalating Intensity",
                description: "Damage ramps from 50% to 200% over 4s of channeling.",
                locked_text: "Patience is a virtue. A very destructive virtue.",
                implemented: true,
            },
            TalentDefinition {
                name: "Sweeping Destruction",
                description: "Beam auto-sweeps in an arc with +100% damage. Cursor sets the center direction.",
                locked_text: "Why aim when the beam can do it for you?",
                implemented: true,
            },
        ],
        // Tier 3
        [
            TalentDefinition {
                name: "Annihilation Beam",
                description: "Beam rains from the sky at the target location. Locks position on cast. Triple width, double damage, double mana cost.",
                locked_text: "Go big or go home. Preferably go big.",
                implemented: true,
            },
            TalentDefinition {
                name: "Searing Finale",
                description: "When channeling ends, the beam detonates along its entire length.",
                locked_text: "The grand finale. Literally explosive.",
                implemented: true,
            },
            TalentDefinition {
                name: "Beam Fireballs",
                description: "Periodically fires small fireballs along the beam.",
                locked_text: "The beam hums with barely contained energy.",
                implemented: true,
            },
        ],
    ]
}

fn meteor_fall_talents() -> [[TalentDefinition; 3]; 3] {
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

fn chain_lightning_talents() -> [[TalentDefinition; 3]; 3] {
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

fn telekinesis_talents() -> [[TalentDefinition; 3]; 3] {
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

fn guardian_circle_talents() -> [[TalentDefinition; 3]; 3] {
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

fn finger_of_death_talents() -> [[TalentDefinition; 3]; 3] {
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
                description: "Mana threshold reduced to 30%, but damage reduced to 700. Cooldown reduced by 50%.",
                locked_text: "Cheaper, weaker, faster. The fast food of death magic.",
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
                description: "Beam sweeps in an arc over 1 second, hitting everything in its path. Damage reduced to 60%.",
                locked_text: "The finger of death goes through a phase. A 'sweep everything' phase.",
                implemented: true,
            },
            TalentDefinition {
                name: "Necrotic Explosion",
                description: "Killed targets explode for 200 damage in a medium radius.",
                locked_text: "They're already dead, what's a little explosion going to do? Oh, to the people AROUND them.",
                implemented: true,
            },
            TalentDefinition {
                name: "Deathmark",
                description: "Beam instead applies a 5-second debuff. If the target dies during the debuff, a second Finger of Death fires at the nearest enemy automatically (at 50% damage).",
                locked_text: "Kill chain. The only chain letter anyone actually follows up on.",
                implemented: true,
            },
        ],
    ]
}

fn mark_of_death_talents() -> [[TalentDefinition; 3]; 3] {
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

fn lightning_rod_talents() -> [[TalentDefinition; 3]; 3] {
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

fn black_hole_talents() -> [[TalentDefinition; 3]; 3] {
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

fn plague_wind_talents() -> [[TalentDefinition; 3]; 3] {
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

fn wall_of_fire_talents() -> [[TalentDefinition; 3]; 3] {
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

fn wall_of_stone_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1: Numeric modifiers
        [
            TalentDefinition {
                name: "Quarry Master",
                description: "Walls cost 30% less mana and can be 25% longer.",
                locked_text: "Bulk stone discount. The quarry union would be furious.",
                implemented: true,
            },
            TalentDefinition {
                name: "Reinforced Stone",
                description: "Walls have 2x health and are 30% wider.",
                locked_text: "Thicker walls for thicker skulls trying to break through.",
                implemented: true,
            },
            TalentDefinition {
                name: "Quick Foundations",
                description: "Place 2 walls per cast at 60% mana cost each. Second wall starts where the first ends.",
                locked_text: "Two walls, one drag. Efficiency!",
                implemented: true,
            },
        ],
        // Tier 2: Secondary effects
        [
            TalentDefinition {
                name: "Jagged Stone",
                description: "Units attacking this wall take 5 damage per hit back.",
                locked_text: "Hit the wall, the wall hits back.",
                implemented: true,
            },
            TalentDefinition {
                name: "Permafrost Aura",
                description: "Enemies within 80 units of the wall move 30% slower.",
                locked_text: "The stone radiates an unnatural chill.",
                implemented: true,
            },
            TalentDefinition {
                name: "Living Stone",
                description: "Walls regenerate 5% of max HP per second when not being attacked.",
                locked_text: "Given enough time, the wall heals itself.",
                implemented: true,
            },
        ],
        // Tier 3: Transformative upgrades
        [
            TalentDefinition {
                name: "Collapsing Wall",
                description: "When a wall is destroyed, it explodes into rubble dealing 30 damage to nearby enemies.",
                locked_text: "Every wall is a loaded weapon if you wait long enough.",
                implemented: true,
            },
            TalentDefinition {
                name: "Terraformer",
                description: "Walls become permanent and persist between levels.",
                locked_text: "Why build temporary when you can reshape the earth itself?",
                implemented: true,
            },
            TalentDefinition {
                name: "Maze Architect",
                description: "Walls cost 50% less mana. When you have 3+ walls, all walls gain +50% health.",
                locked_text: "A single wall is a suggestion. Three walls is a labyrinth.",
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
        // Disintegrate has real talent definitions — skip placeholder names
        Spell::Disintegrate => (["", "", ""], ["", "", ""], ["", "", ""]),
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
