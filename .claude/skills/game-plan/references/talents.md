# New Talent Tree Reference

## Overview

Each spell has a 3x3 talent grid: 3 tiers with 3 talent choices per tier. Players pick one talent per tier, unlocking tiers progressively.

## Critical Design Rule: Cross-Tier Compatibility

**Every talent in every tier MUST work correctly with all talents in the OTHER tiers.** Since players pick one talent per tier, any combination of (1 from Tier 1) + (1 from Tier 2) + (1 from Tier 3) must be a valid, functional build. That means:

- **3 x 3 x 3 = 27 possible builds** per spell. All 27 must work.
- Talents within the **same tier** do NOT need to work together (they are mutually exclusive choices).
- When designing a talent, consider how it interacts with each of the 6 talents in the other two tiers.
- Avoid talents that contradict or break talents in other tiers (e.g., "removes all projectiles" in Tier 1 would break a Tier 3 talent that modifies projectile behavior).
- If a talent fundamentally changes the spell's mechanic (e.g., "beam now sweeps in an arc"), ensure all talents in other tiers still apply meaningfully to the modified version.

### Cross-Tier Compatibility Checklist

When designing talents, fill out this matrix for each new talent:

```
New talent: [Name] (Tier X)

Works with Tier Y talents:
- [Talent Y-1]: [How they interact] ✓
- [Talent Y-2]: [How they interact] ✓
- [Talent Y-3]: [How they interact] ✓

Works with Tier Z talents:
- [Talent Z-1]: [How they interact] ✓
- [Talent Z-2]: [How they interact] ✓
- [Talent Z-3]: [How they interact] ✓
```

## Talent Tier Design Philosophy

- **Tier 1**: Simple numeric modifiers or basic mechanic tweaks (radius, damage, cooldown, count).
- **Tier 2**: Adds new secondary effects or changes spell behavior (splitting, chaining, healing, debuffs).
- **Tier 3**: Transformative or powerful upgrades that significantly alter how the spell plays (new cast mode, massive AoE, chain reactions).

## File Structure

Talent definitions live in `src/game/units/wizard/talents/definitions.rs`. Each spell gets a function returning `[[TalentDefinition; 3]; 3]` (outer = tiers, inner = choices).

Talent gameplay effects are implemented in the spell's own module files (`systems.rs`, `components.rs`, `constants.rs`), reading from `TalentResources` to check which talents are active.

## TalentDefinition Struct

```rust
pub(crate) struct TalentDefinition {
    pub name: &'static str,
    pub description: &'static str,      // Shown when unlocked
    pub locked_text: &'static str,      // Humorous flavor text when locked
    pub implemented: bool,              // Set to true when gameplay effect works
}
```

## Step-by-Step: Adding a Talent Tree

### 1. Design the 9 talents

Plan all 9 talents (3 tiers x 3 choices) before writing any code. Use the cross-tier compatibility checklist above to verify all 27 combinations work.

### 2. Add definitions in `definitions.rs`

Add a new function following the existing pattern:

```rust
fn spell_name_talents() -> [[TalentDefinition; 3]; 3] {
    [
        // Tier 1
        [
            TalentDefinition {
                name: "Talent Name",
                description: "Mechanical effect description.",
                locked_text: "Humorous flavor text.",
                implemented: true, // or false if gameplay not yet coded
            },
            // ... 2 more Tier 1 talents
        ],
        // Tier 2
        [ /* ... 3 talents */ ],
        // Tier 3
        [ /* ... 3 talents */ ],
    ]
}
```

Then add the match arm in `talent_definitions()`:

```rust
Spell::SpellName => spell_name_talents(),
```

### 3. Implement gameplay effects

In the spell's `systems.rs`, read active talents from `TalentResources` and branch behavior accordingly. Common patterns:

- **Numeric modifiers**: Multiply constants by talent-based factors
- **New behaviors**: Add conditional logic gated on talent checks
- **New components**: Add talent-specific components for complex effects

### 4. Verify cross-tier interactions

Test or mentally trace all 27 talent combinations to ensure no conflicts.

## Writing Style

- **Descriptions**: Clear, concise mechanical effects. Include specific numbers (percentages, durations, counts).
- **Locked text**: Short, humorous, in-universe flavor. Should hint at what the talent does without revealing exact mechanics. Keep it punchy (1-2 sentences max).

## Examples of Good Cross-Tier Design

From Magic Missile:
- Tier 1 "Volley" (5 missiles at 80% dmg) works with Tier 2 "Seeker Swarm" (kills split into 2) — more missiles = more split chances
- Tier 1 "Heavy Ordnance" (1 missile at 4x dmg) works with Tier 3 "Arcane Detonation" (AoE on impact) — one big explosion
- All Tier 1 options work with all Tier 2 and Tier 3 options regardless of combination

## Examples of Bad Cross-Tier Design (Avoid)

- Tier 1: "Spell no longer fires projectiles" + Tier 2: "Projectiles pierce targets" — direct contradiction
- Tier 1: "Converts to fire damage" + Tier 3: "Deals bonus ice damage" — thematic conflict that confuses the build identity
- Tier 2: "Requires channeling" + Tier 3: "Instant cast" — mechanical contradiction
