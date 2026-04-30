use super::TalentDefinition;

pub(super) fn disintegrate_talents() -> [[TalentDefinition; 3]; 3] {
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

pub(super) fn wall_of_stone_talents() -> [[TalentDefinition; 3]; 3] {
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
