//! Network message protocol.
//!
//! Defines the serializable message types sent over the WebRTC data channels.
//! Reliable channel: commands, events, and setup messages.
//! Unreliable channel: raw binary state snapshots (handled separately).

use serde::{Deserialize, Serialize};

use crate::config::WizardType;
use crate::game::units::wizard::components::Spell;

/// Messages sent over the reliable WebRTC data channel between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Ping with a timestamp for RTT measurement.
    Ping { timestamp_ms: f64 },

    /// Pong response echoing back the original timestamp.
    Pong { timestamp_ms: f64 },

    /// Player info exchanged after connection (unlocked content).
    PlayerInfo {
        wizard_types: Vec<WizardType>,
        spells: Vec<Spell>,
    },

    /// Player selected a wizard type.
    WizardSelected(WizardType),

    /// Player is ready to start the match.
    ReadyUp,

    /// Player cancelled their ready state.
    Unready,

    /// Host tells guest to start loading the match, sharing the run seed so
    /// both peers seed their RNG identically.
    StartGame { seed: u64 },

    /// Player has finished loading.
    GameLoaded,

    /// Host notifies guest that the game is over.
    GameOver(GameOverResult),

    /// Player is ready for a rematch (sent from score screen).
    RematchReady,

    /// A wall was placed or removed — update pathfinding grid.
    ///
    /// Sent bidirectionally when either player places a Wall of Dirt so
    /// the other client can update its pathfinding grid.
    WallPlaced {
        /// AABB of the wall obstacle: [min_x, min_z, max_x, max_z].
        bounds: [f32; 4],
        /// true = wall placed (Blocked), false = wall removed (Removed).
        placed: bool,
    },

    /// Guest tells the host to teleport units within a radius.
    ///
    /// Unit positions are host-authoritative, so the guest sends this message
    /// instead of moving units locally.
    TeleportUnits {
        source_x: f32,
        source_z: f32,
        dest_x: f32,
        dest_z: f32,
        radius: f32,
    },

    /// Guest tells the host that one of its spells just impacted a unit.
    ///
    /// Status-effect bookkeeping is host-authoritative: the host receives this
    /// message, inserts the standard `PendingDamageEffect` on the matching
    /// local entity (resolved through `NetworkEntityMap`), and SP's existing
    /// `process_pending_damage_effects` pipeline takes over from there —
    /// FireDoT / FrostAccumulation / Shocked / Poisoned all flow through that
    /// shared code path on the host. The host's normal snapshot then ships
    /// the resulting status flag (e.g. `UnitFlags::FIRE_EFFECT`) back to the
    /// guest, which renders the visual via `RemoteFireEffect` etc.
    SpellHitUnit {
        target_network_id: u32,
        damage: f32,
        /// Serialized `DamageType` ordinal — see `DamageType::from_u8`.
        damage_type: u8,
    },

    /// Guest tells the host to apply a non-damage status effect to a unit
    /// (sleep, root, polymorph, mind control, banish, mark, haste, etc.).
    ///
    /// Used for ALL spell side-effects that aren't HP damage. The host inserts
    /// the corresponding SP component on the authoritative unit so its combat
    /// / movement / animation systems respect the state. The guest keeps the
    /// component on the local ghost too — when the host's snapshot arrives,
    /// the host-driven simulation overrides the ghost's position anyway, but
    /// the local component lets guest-side visual systems (sleep Z animation,
    /// polymorph sprite swap, etc.) react immediately without a round-trip.
    ApplyStatusEffect {
        target_network_id: u32,
        /// `StatusEffectKind` ordinal — see `StatusEffectKind::from_u8`.
        kind: u8,
        /// Effect duration in seconds. Some effects ignore this (e.g.
        /// permanent dominate-style talents) and use the flags payload.
        duration: f32,
        /// Effect-specific magnitude: damage multiplier (Mark), speed bonus
        /// (Haste), wake threshold (Comatose), heal amount (TempHP), etc.
        /// Interpretation depends on `kind`.
        magnitude: f32,
        /// Status-specific talent flags packed into 32 bits. The receiver's
        /// match arm for each `kind` decodes whichever bits matter for it.
        flags: u32,
    },

    /// Guest tells the host to raise (or convert) a specific corpse into an
    /// Undead unit. Carries the corpse's network ID, the talent flags that
    /// drive Plague Bearer / Perpetual Unrest / Revenant Lord / Empowered
    /// Undead / Undead Detonation variants, and the cast's empowerment
    /// multiplier (so HP / damage bonuses derive correctly host-side).
    RaiseCorpse {
        target_network_id: u32,
        flags: u32,
        empowerment: f32,
    },

    /// Guest tells the host to despawn the spell-effect entity identified by
    /// its network ID (used by Dispel; the host owns the authoritative entity).
    DispelSpellEffect { target_network_id: u32 },

    /// Guest tells the host to strip `SpellShield` from a unit (Dispel impact
    /// hitting a shielded king on the host).
    DispelShield { target_network_id: u32 },
}

/// Discriminator for `NetworkMessage::ApplyStatusEffect`. Add new variants at
/// the end so the wire ordinals stay stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StatusEffectKind {
    Sleep = 0,
    Root = 1,
    Polymorph = 2,
    MindControl = 3,
    Banish = 4,
    Mark = 5,
    Haste = 6,
    BattleHymn = 7,
    BerserkerRage = 8,
    GuardianTempHp = 9,
    Slow = 10,
    Knockback = 11,
    Stun = 12,
    FogEvasion = 13,
}

impl StatusEffectKind {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Sleep),
            1 => Some(Self::Root),
            2 => Some(Self::Polymorph),
            3 => Some(Self::MindControl),
            4 => Some(Self::Banish),
            5 => Some(Self::Mark),
            6 => Some(Self::Haste),
            7 => Some(Self::BattleHymn),
            8 => Some(Self::BerserkerRage),
            9 => Some(Self::GuardianTempHp),
            10 => Some(Self::Slow),
            11 => Some(Self::Knockback),
            12 => Some(Self::Stun),
            13 => Some(Self::FogEvasion),
            _ => None,
        }
    }
}

/// Bit flags packed into `ApplyStatusEffect.flags` for status-specific
/// talents. Each status decodes the bits it cares about; unused bits are
/// ignored. Definitions are kept here so all senders and the host receiver
/// agree on the layout.
pub mod status_flags {
    // Sleep talents
    pub const SLEEP_NIGHT_TERRORS: u32 = 1 << 0;
    pub const SLEEP_COMATOSE: u32 = 1 << 1;
    pub const SLEEP_NARCOLEPTIC_WAVE: u32 = 1 << 2;
    pub const SLEEP_DREAMWALKER: u32 = 1 << 3;
    pub const SLEEP_ETERNAL_SLUMBER: u32 = 1 << 4;
    // Root / Entangle talents
    pub const ROOT_THORNY_VINES: u32 = 1 << 0;
    pub const ROOT_CLINGING_ROOTS: u32 = 1 << 1;
    pub const ROOT_STRANGLEHOLD: u32 = 1 << 2;
    // Polymorph talents
    pub const POLYMORPH_FRAGILE: u32 = 1 << 0;
    pub const POLYMORPH_EXPLOSIVE: u32 = 1 << 1;
    pub const POLYMORPH_CONTAGIOUS: u32 = 1 << 2;
    pub const POLYMORPH_PERMANENT: u32 = 1 << 3;
    pub const POLYMORPH_DIRE: u32 = 1 << 4;
    // MindControl talents
    pub const MC_TRAITORS_MARK: u32 = 1 << 0;
    pub const MC_DEMORALIZE: u32 = 1 << 1;
    pub const MC_AMNESIA: u32 = 1 << 2;
    pub const MC_DOMINATE: u32 = 1 << 3;
    pub const MC_SLEEPER_AGENT: u32 = 1 << 4;
    pub const MC_MASS_HYSTERIA: u32 = 1 << 5;
    // Banishment talents
    pub const BANISH_PAINFUL_RETURN: u32 = 1 << 0;
    pub const BANISH_DISPLACEMENT: u32 = 1 << 1;
    pub const BANISH_DIMENSIONAL_SHUNT: u32 = 1 << 2;
    pub const BANISH_ONE_WAY: u32 = 1 << 3;
    // Mark of Death talents
    pub const MARK_FOCAL_POINT: u32 = 1 << 0;
    pub const MARK_EXECUTIONER_BRAND: u32 = 1 << 1;
    pub const MARK_DEATHS_LEDGER: u32 = 1 << 2;
    // RaiseTheDead talents (carried via RaiseCorpse.flags)
    pub const RAISE_PLAGUE_BEARER: u32 = 1 << 0;
    pub const RAISE_PERPETUAL_UNREST: u32 = 1 << 1;
    pub const RAISE_REVENANT_LORD: u32 = 1 << 2;
    pub const RAISE_UNDEAD_DETONATION: u32 = 1 << 3;
    pub const RAISE_EMPOWERED_UNDEAD: u32 = 1 << 4;
}

/// Result of a multiplayer match.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameOverResult {
    /// The host won the match.
    HostWins,

    /// The guest won the match.
    GuestWins,
}
