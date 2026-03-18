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
        Spell::Entangle => entangle_talents(),
        Spell::SpikeGrowth => spike_growth_talents(),
        Spell::Squall => squall_talents(),
        Spell::Sleep => sleep_talents(),
        Spell::Grease => grease_talents(),
        Spell::Polymorph => polymorph_talents(),
        Spell::MindControl => mind_control_talents(),
        Spell::Haste => haste_talents(),
        Spell::Teleport => teleport_talents(),
        Spell::RaiseTheDead => raise_the_dead_talents(),
        Spell::HealingPlume => healing_plume_talents(),
        Spell::FogCloud => fog_cloud_talents(),
        Spell::BerserkerRage => berserker_rage_talents(),
        Spell::Banishment => banishment_talents(),
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

fn entangle_talents() -> [[TalentDefinition; 3]; 3] {
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

fn spike_growth_talents() -> [[TalentDefinition; 3]; 3] {
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

fn squall_talents() -> [[TalentDefinition; 3]; 3] {
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

fn sleep_talents() -> [[TalentDefinition; 3]; 3] {
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

fn grease_talents() -> [[TalentDefinition; 3]; 3] {
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

fn polymorph_talents() -> [[TalentDefinition; 3]; 3] {
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

fn mind_control_talents() -> [[TalentDefinition; 3]; 3] {
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

fn haste_talents() -> [[TalentDefinition; 3]; 3] {
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

fn teleport_talents() -> [[TalentDefinition; 3]; 3] {
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

fn raise_the_dead_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Mass Graves",
                description: "Resurrection radius increased by 60%.",
                locked_text:
                    "Cast a wider net. For corpses. This job description is something else.",
                implemented: true,
            },
            TalentDefinition {
                name: "Hasty Ritual",
                description:
                    "Channeling speed starts at max speed instead of ramping up.",
                locked_text:
                    "Skipping the warm-up chant. The dead don't care about proper procedure.",
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
                description:
                    "Raised undead have 50% more HP and deal 25% more damage.",
                locked_text: "Better zombies through magic. If you're going to reanimate, reanimate with style.",
                implemented: true,
            },
            TalentDefinition {
                name: "Plague Bearer",
                description:
                    "Raised undead emit a poison aura, dealing damage to nearby living enemies.",
                locked_text: "The undead don't just fight. They bring ambiance. Toxic, lethal ambiance.",
                implemented: true,
            },
            TalentDefinition {
                name: "Corpse Magnet",
                description:
                    "Corpses within a large radius are pulled toward the cursor before resurrection.",
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
                description:
                    "Raised undead explode when they die (again), dealing heavy damage in a radius.",
                locked_text:
                    "They were already dead, so technically this is recycling.",
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

fn healing_plume_talents() -> [[TalentDefinition; 3]; 3] {
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

fn fog_cloud_talents() -> [[TalentDefinition; 3]; 3] {
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

fn berserker_rage_talents() -> [[TalentDefinition; 3]; 3] {
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

fn banishment_talents() -> [[TalentDefinition; 3]; 3] {
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
            ["Deep Roots", "Sprawling Thicket", "Efficient Growth"],
            ["Thorny Vines", "Clinging Roots", "Nourishing Roots"],
            ["Overgrowth", "Nature's Sanctuary", "Stranglehold"],
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
            ["Alacrity", "Extended Rush", "Quick Cast"],
            ["Adrenaline Surge", "Momentum", "Fleet Feet"],
            ["Time Warp", "Slow Zone", "Chain Haste"],
        ),
        Spell::Teleport => (
            ["Wide Aperture", "Hasty Translocation", "Lingering Gate"],
            ["Disorienting Arrival", "Swap", "Emergency Recall"],
            ["Dimensional Rift", "Up", "Scatterport"],
        ),
        Spell::RaiseTheDead => (
            ["More Corpses", "Stronger Undead", "Quick Raise"],
            ["Skeletal Army", "Zombie Plague", "Death Knight"],
            ["Necropolis", "Lich's Command", "Army of Darkness"],
        ),
        Spell::HealingPlume => (
            ["Rejuvenating Mists", "Verdant Bloom", "Lasting Remedy"],
            ["Cleansing Plume", "Overflow", "Triage Pulse"],
            ["Font of Life", "Healing Rain", "Field Medic"],
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
