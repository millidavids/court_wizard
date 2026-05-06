//! Status-effect descriptions for the spell book and compendium.
//!
//! Spells declare the in-scope status effects they apply via
//! [`Spell::status_effects`]. The renderer composes a "Status effects" section
//! at the bottom of the description so players can see what each spell does
//! beyond raw damage.
//!
//! When the active wizard is the Excremage, all damage is routed to
//! `DamageType::Poop` and applies [`StatusEffectKind::Smelly`] — so the renderer
//! swaps the spell's normal effect list for a single Smelly entry.

use crate::config::WizardType;

use super::spell_enum::Spell;

/// Player-facing status effects surfaced in spell descriptions.
///
/// Scope: damage-over-time, slows / freezes, hard crowd-control, and the
/// Excremage-only Smelly effect. Buffs are intentionally omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusEffectKind {
    Burning,
    Poisoned,
    Frost,
    Slowed,
    Frozen,
    Rooted,
    Slept,
    Polymorphed,
    Banished,
    Marked,
    Smelly,
}

impl StatusEffectKind {
    /// Short label shown at the start of each bullet.
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            StatusEffectKind::Burning => "Burning",
            StatusEffectKind::Poisoned => "Poisoned",
            StatusEffectKind::Frost => "Frost",
            StatusEffectKind::Slowed => "Slowed",
            StatusEffectKind::Frozen => "Frozen",
            StatusEffectKind::Rooted => "Rooted",
            StatusEffectKind::Slept => "Slept",
            StatusEffectKind::Polymorphed => "Polymorphed",
            StatusEffectKind::Banished => "Banished",
            StatusEffectKind::Marked => "Marked",
            StatusEffectKind::Smelly => "Smelly",
        }
    }

    /// One-sentence mechanical explanation. Keep it player-readable —
    /// no code references, no jargon.
    pub(crate) const fn description(&self) -> &'static str {
        match self {
            StatusEffectKind::Burning => {
                "Sets the target on fire. Deals extra damage every half-second for several seconds after the hit."
            }
            StatusEffectKind::Poisoned => {
                "Damages the target over time. Stacks of poison make later spells more deadly."
            }
            StatusEffectKind::Frost => {
                "Builds up frost on the target. Each hit slows them more, and a fully frosted target freezes solid."
            }
            StatusEffectKind::Slowed => {
                "Reduces the target's movement speed for a short time. Stronger slows replace weaker ones."
            }
            StatusEffectKind::Frozen => {
                "The target is locked in place and can't attack until the ice cracks."
            }
            StatusEffectKind::Rooted => {
                "The target is anchored to the ground. They can still attack but can't move."
            }
            StatusEffectKind::Slept => {
                "The target falls asleep — no movement, no attacks. Damage wakes them up."
            }
            StatusEffectKind::Polymorphed => {
                "The target is transformed into a harmless critter for a few seconds."
            }
            StatusEffectKind::Banished => {
                "The target is whisked off the battlefield briefly, returning to where they vanished."
            }
            StatusEffectKind::Marked => {
                "The target takes increased damage from every source until the mark expires."
            }
            StatusEffectKind::Smelly => {
                "The target reeks of filth — their own allies recoil, breaking enemy formations for several seconds."
            }
        }
    }
}

/// Builds the description string shown in the compendium and spell book,
/// including the trailing "Status effects" section.
///
/// When the active wizard is the Excremage, the spell's normal status effect
/// list is replaced with a single Smelly entry on every spell that would
/// have applied something. Spells that apply no in-scope effects normally
/// (Magic Missile, Telekinesis, etc.) keep their plain description.
pub(crate) fn compose_spell_description(spell: Spell, wizard_type: WizardType) -> String {
    let base = spell.description();

    let normal_effects = spell.status_effects();
    let effects: &[StatusEffectKind] = if wizard_type == WizardType::Excremage {
        if normal_effects.is_empty() {
            &[]
        } else {
            &[StatusEffectKind::Smelly]
        }
    } else {
        normal_effects
    };

    if effects.is_empty() {
        return base.to_string();
    }

    let mut out = String::with_capacity(base.len() + 64 * effects.len());
    out.push_str(base);
    out.push_str("\n\nStatus effects");
    for effect in effects {
        out.push_str("\n• ");
        out.push_str(effect.name());
        out.push_str(" — ");
        out.push_str(effect.description());
    }
    out
}
