use serde::{Deserialize, Serialize};

/// Wizard class types available for selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum WizardType {
    /// Boring Ole Mage - no special mechanics, flat 5% bonus to all stats.
    #[default]
    BoringOleMage,
    /// Rune-based caster - the classic wizard type.
    RuneCaster,
    /// Randomancer - spins a roulette wheel for powerful random spells.
    Randomancer,
    /// Arcanorouter - allocates resources between range, mana, power, and speed.
    Arcanorouter,
    /// Excremage - converts all spells to Poop damage with brown visuals.
    Excremage,
    /// Alchemist - brewing specialist with faster brews, longer buffs, and Philosopher's Stone.
    Alchemist,
    /// Warglock - replaces spells with 5 guns.
    Warglock,
    /// Swordcerer - enters the battlefield as a melee/ranged fighter.
    #[serde(alias = "Battlemage")]
    Swordcerer,
    /// Meteorologist - manipulates weather to apply global status effects.
    Meteorologist,
    /// Shepherd - support-only wizard with no damage-dealing spells, bonus to support effects.
    Shepherd,
    /// Psychopath - wants maximum carnage; must kill 80% of defenders to win.
    Psychopath,
}

impl WizardType {
    /// Returns the display name for this wizard type.
    pub const fn display_name(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => "Boring Ole Mage",
            WizardType::RuneCaster => "Rune Caster",
            WizardType::Randomancer => "Randomancer",
            WizardType::Arcanorouter => "Arcanorouter",
            WizardType::Excremage => "Excremage",
            WizardType::Alchemist => "The Alchemist",
            WizardType::Warglock => "Warglock",
            WizardType::Swordcerer => "Swordcerer",
            WizardType::Meteorologist => "Meteorologist",
            WizardType::Shepherd => "Shepherd",
            WizardType::Psychopath => "Psychopath",
        }
    }

    /// Returns true if this wizard type uses its own exclusive casting mechanic
    /// (rune sequences or roulette spins) and should not access the spell book
    /// or action bar spell priming.
    pub const fn uses_exclusive_casting(&self) -> bool {
        matches!(self, WizardType::RuneCaster | WizardType::Randomancer)
    }

    /// Returns a short description of this wizard type's playstyle.
    pub const fn description(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => {
                "A straightforward wizard with a small bonus to everything."
            }
            WizardType::RuneCaster => "Master rune sequences to empower your spells.",
            WizardType::Randomancer => "Spin the wheel of fate for powerful random spells.",
            WizardType::Arcanorouter => "Route arcane power between range, mana, power, and speed.",
            WizardType::Excremage => "Turn all spells into poop.",
            WizardType::Alchemist => "Master the cauldron with faster brews and stronger potions.",
            WizardType::Warglock => "Who needs spells when you have guns?",
            WizardType::Swordcerer => "Leave the tower. Enter the fray.",
            WizardType::Meteorologist => "Control the weather. Control the battlefield.",
            WizardType::Shepherd => "Heal. Shield. Inspire. Never harm.",
            WizardType::Psychopath => "Burn it all. Both sides.",
        }
    }

    /// Returns a longer description explaining the archetype's mechanics in detail.
    pub const fn long_description(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => {
                "No special mechanics to learn. Your spells are slightly stronger, cheaper, faster, and longer-ranged than other wizards. A solid choice for beginners or anyone who just wants to cast spells without thinking about it."
            }
            WizardType::RuneCaster => {
                "Press Q, W, E, and R to build rune sequences. Single runes cast basic spells, while two-rune combos unlock powerful advanced spells. Successful sequences empower your spells with a 1.25x bonus. Sequences time out if you wait too long between keys."
            }
            WizardType::Randomancer => {
                "Press SPACE to spin a magical roulette wheel. Whatever spell it lands on, you cast — no choosing. In exchange for giving up control, your spells are empowered with a massive 1.75x bonus. Adapting to whatever the wheel gives you is the whole game."
            }
            WizardType::Arcanorouter => {
                "Dynamically allocate a shared pool of arcane energy between four stats: Range, Mana Efficiency, Power, and Speed. Use Q/A, W/S, E/D, and R/F to increase or decrease each slider. Pump everything into Power for devastating spells, or balance your build for versatility. Adjust mid-battle to adapt to the situation."
            }
            WizardType::Excremage => {
                "All your spells deal Poop damage and turn units into smelly messes. Your spells may lack elemental finesse, but nothing clears a battlefield like the smell of fear... and other things."
            }
            WizardType::Alchemist => {
                "Your brews take 20% less time and your buffs last 25% longer. Once per battle, you can add the Philosopher's Stone to a brew — it removes all dilution, so every ingredient brews at full strength. The Stone doesn't count toward the 3-ingredient limit."
            }
            WizardType::Warglock => {
                "Spells? Where you're going, you don't need spells. Your 5 action bar slots become 5 different guns, each with its own ammo pool. Machine gun sprays bullets, magnum hits hard, rocket launcher explodes, shotgun blasts in a cone, and flamethrower burns everything. Reload with R or let it auto-reload when empty."
            }
            WizardType::Swordcerer => {
                "Click 'Enter the Fray' to leave your tower and join the battle as a warrior. Move with WASD, shoot magic missiles with left-click, and swing your sword with right-click. You're fast but vulnerable — if your health hits zero, you teleport back to the tower. While on the field, spells, spell book, and cauldron are disabled."
            }
            WizardType::Meteorologist => {
                "Press Q, W, or E to change the weather. Storm makes units Wet and Charged — shocked units spread electricity to nearby wet targets, electric arcs hit more targets, and random lightning strikes the field. Blizzard makes units Cold — frost spells slow even harder, and can freeze units solid. Drought makes units Dry — fire spells create burning patches on the ground. Weather effects grow stronger the longer they persist."
            }
            WizardType::Shepherd => {
                "You cannot cast any spell that deals damage. No fireballs, no black holes, no spike growth — nothing that hurts. In exchange, all your support spells are 30% more powerful: bigger heals, stronger shields, longer buffs, and tougher walls. Guide your flock to victory through faith alone."
            }
            WizardType::Psychopath => {
                "Victory isn't enough — you need carnage. Your spells deal 30% extra damage to your own defenders. To win a level, at least 70% of your defenders must be dead by the time the last attacker falls. If too many survive, you lose. Efficiency is for cowards."
            }
        }
    }

    /// Returns flavor text shown on the progress screen when the wizard type is locked.
    pub const fn locked_description(&self) -> &'static str {
        match self {
            WizardType::BoringOleMage => "Sometimes boring is best.",
            WizardType::RuneCaster => "Mysterious symbols. Mysterious results.",
            WizardType::Randomancer => "You don't choose the spell. The spell chooses you.",
            WizardType::Arcanorouter => "Geordi would be proud of your power routing.",
            WizardType::Excremage => "Something smells off...",
            WizardType::Alchemist => "The cauldron whispers to those who listen.",
            WizardType::Warglock => "War and sorcery, forged into one.",
            WizardType::Swordcerer => "Some wizards prefer a more... hands-on approach.",
            WizardType::Meteorologist => "The sky darkens. Something is brewing up there.",
            WizardType::Shepherd => "Violence is never the answer... right?",
            WizardType::Psychopath => "Some people just want to watch the world burn.",
        }
    }

    /// Returns all available wizard types.
    /// Stable on-disk key for save data. Pinned to the variant name explicitly so
    /// that renaming a variant (or changing the `Debug` derive) can never silently
    /// orphan existing player saves. A unit test asserts this stays equal to the
    /// `Debug` representation, which is the format already on disk.
    pub fn save_key(&self) -> &'static str {
        match self {
            Self::BoringOleMage => "BoringOleMage",
            Self::RuneCaster => "RuneCaster",
            Self::Randomancer => "Randomancer",
            Self::Arcanorouter => "Arcanorouter",
            Self::Excremage => "Excremage",
            Self::Alchemist => "Alchemist",
            Self::Warglock => "Warglock",
            Self::Swordcerer => "Swordcerer",
            Self::Meteorologist => "Meteorologist",
            Self::Shepherd => "Shepherd",
            Self::Psychopath => "Psychopath",
        }
    }

    pub const fn all() -> &'static [WizardType] {
        &[
            WizardType::BoringOleMage,
            WizardType::RuneCaster,
            WizardType::Randomancer,
            WizardType::Arcanorouter,
            WizardType::Excremage,
            WizardType::Alchemist,
            WizardType::Warglock,
            WizardType::Swordcerer,
            WizardType::Meteorologist,
            WizardType::Shepherd,
            WizardType::Psychopath,
        ]
    }
}
