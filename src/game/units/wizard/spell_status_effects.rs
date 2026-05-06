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

use bevy::prelude::*;

use crate::config::WizardType;

use super::spell_enum::Spell;

/// Color used for the "Status effects" section header in spell descriptions.
pub(crate) const STATUS_HEADER_COLOR: Color = Color::srgb(0.95, 0.85, 0.55);

/// Player-facing status effects surfaced in spell descriptions.
///
/// Scope: damage-over-time, slows / freezes, hard crowd-control, and the
/// Excremage-only Smelly effect. Buffs are intentionally omitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusEffectKind {
    Burning,
    Poisoned,
    Shocked,
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
            StatusEffectKind::Shocked => "Shocked",
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

    /// Themed accent color used for the effect's name in the rendered list.
    pub(crate) fn color(&self) -> Color {
        match self {
            StatusEffectKind::Burning => Color::srgb(1.00, 0.55, 0.10),
            StatusEffectKind::Poisoned => Color::srgb(0.55, 0.85, 0.30),
            StatusEffectKind::Shocked => Color::srgb(1.00, 0.95, 0.35),
            StatusEffectKind::Frost => Color::srgb(0.55, 0.80, 1.00),
            StatusEffectKind::Slowed => Color::srgb(0.65, 0.85, 0.95),
            StatusEffectKind::Frozen => Color::srgb(0.85, 0.95, 1.00),
            StatusEffectKind::Rooted => Color::srgb(0.55, 0.75, 0.40),
            StatusEffectKind::Slept => Color::srgb(0.70, 0.65, 0.95),
            StatusEffectKind::Polymorphed => Color::srgb(0.95, 0.65, 0.85),
            StatusEffectKind::Banished => Color::srgb(0.75, 0.55, 0.95),
            StatusEffectKind::Marked => Color::srgb(1.00, 0.40, 0.40),
            StatusEffectKind::Smelly => Color::srgb(0.65, 0.45, 0.20),
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
            StatusEffectKind::Shocked => {
                "The target crackles with electricity for a few seconds, periodically firing small lightning arcs at nearby enemies. Wet targets take extra arc damage."
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

/// Spawns the colored "Status effects" header + one line per effect into the
/// given parent. Each effect line is a `Text` with the effect's themed color
/// holding a `TextSpan` child for the description in `body_color`.
///
/// No-ops when `effects` is empty. Optional `entry_node` is applied to each
/// effect-name `Text` (e.g. to set a `max_width` on a fixed-width panel).
pub(crate) fn spawn_status_effects_section(
    parent: &mut ChildSpawnerCommands,
    effects: &[StatusEffectKind],
    font_size: f32,
    body_color: Color,
    entry_node: Option<Node>,
) {
    if effects.is_empty() {
        return;
    }
    let mut header = parent.spawn((
        Text::new("Status effects"),
        TextFont::from_font_size(font_size),
        TextColor(STATUS_HEADER_COLOR),
    ));
    if let Some(node) = entry_node.clone() {
        header.insert(node);
    }
    for effect in effects {
        let mut line = parent.spawn((
            Text::new(effect.name()),
            TextFont::from_font_size(font_size),
            TextColor(effect.color()),
        ));
        if let Some(node) = entry_node.clone() {
            line.insert(node);
        }
        line.with_children(|child| {
            child.spawn((
                TextSpan::new(format!(" — {}", effect.description())),
                TextFont::from_font_size(font_size),
                TextColor(body_color),
            ));
        });
    }
}

/// Returns the status effects to surface in the spell description for the
/// given wizard archetype.
///
/// Non-Excremage: the spell's normal effect list (may be empty).
/// Excremage on a spell that *would* normally apply something: a single
/// Smelly entry, since all damage routes to Poop and only Smelly applies.
/// Excremage on a spell with no normal effects: empty (nothing to swap).
pub(crate) fn effective_status_effects(
    spell: Spell,
    wizard_type: WizardType,
) -> &'static [StatusEffectKind] {
    let normal = spell.status_effects();
    if wizard_type == WizardType::Excremage {
        if normal.is_empty() {
            &[]
        } else {
            &[StatusEffectKind::Smelly]
        }
    } else {
        normal
    }
}
