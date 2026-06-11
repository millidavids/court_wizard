use bevy::prelude::*;

/// Global scale factor for all unit sizes (sprite dimensions, hitbox radii, and size-dependent distances).
pub const UNIT_SCALE: f32 = 4.0;

// ===== Team Base Colors =====

/// Base color for defender units (light gray).
pub const DEFENDER_BASE: Color = Color::srgb(0.65, 0.65, 0.65);

/// Base color for attacker units (dark gray).
pub const ATTACKER_BASE: Color = Color::srgb(0.3, 0.3, 0.3);

/// Blends a tint color into a base color at the given strength (0.0 = pure base, 1.0 = pure tint).
pub const fn tint(base: Color, tint_color: Color, strength: f32) -> Color {
    let Color::Srgba(b) = base else {
        return base;
    };
    let Color::Srgba(t) = tint_color else {
        return base;
    };
    Color::Srgba(bevy::color::Srgba {
        red: b.red + (t.red - b.red) * strength,
        green: b.green + (t.green - b.green) * strength,
        blue: b.blue + (t.blue - b.blue) * strength,
        alpha: b.alpha,
    })
}

/// Dims a color toward gray and applies an alpha value.
/// Used for corpse materials — tints toward gray and makes semi-transparent.
pub const fn dim(base: Color, gray_strength: f32, alpha: f32) -> Color {
    let Color::Srgba(b) = base else {
        return base;
    };
    let gray = 0.5;
    Color::Srgba(bevy::color::Srgba {
        red: b.red + (gray - b.red) * gray_strength,
        green: b.green + (gray - b.green) * gray_strength,
        blue: b.blue + (gray - b.blue) * gray_strength,
        alpha,
    })
}

// ===== Common Tint Colors =====

pub const TINT_RED: Color = Color::srgb(1.0, 0.2, 0.2);

// ===== Undead Color =====

/// Base color for undead units (purple).
pub const UNDEAD_BASE: Color = Color::srgb(0.5, 0.2, 0.7);

// ===== Corpse Colors (tinted red then dimmed) =====

/// Alpha for corpse materials — semi-transparent so alive units show through.
pub const CORPSE_ALPHA: f32 = 0.4;

/// Corpse color for defender units.
pub const DEFENDER_CORPSE_COLOR: Color = dim(tint(DEFENDER_BASE, TINT_RED, 0.4), 0.3, CORPSE_ALPHA);
/// Corpse color for attacker units.
pub const ATTACKER_CORPSE_COLOR: Color = dim(tint(ATTACKER_BASE, TINT_RED, 0.4), 0.3, CORPSE_ALPHA);
/// Corpse color for undead units.
pub const UNDEAD_CORPSE_COLOR: Color = dim(tint(UNDEAD_BASE, TINT_RED, 0.4), 0.3, CORPSE_ALPHA);
/// Corpse color for the king.
pub const KING_CORPSE_COLOR: Color = dim(
    tint(Color::srgb(0.65, 0.65, 0.9), TINT_RED, 0.4),
    0.3,
    CORPSE_ALPHA,
);

// ===== Earth/Stone Colors =====

/// Dark reddish-brown stone color used for stone underlays, wall sides, trampling,
/// and dirt borders. The canonical "earth" color throughout the battlefield.
pub const STONE_COLOR_DARK: Color = Color::srgb(0.11, 0.04, 0.03);

/// Light reddish-brown stone color, paired with STONE_COLOR_DARK for noise blending
/// on stone surfaces, wall of stone sides, and dirt border accents.
pub const STONE_COLOR_LIGHT: Color = Color::srgb(0.20, 0.10, 0.07);
