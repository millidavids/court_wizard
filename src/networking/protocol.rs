//! Network message protocol.
//!
//! Defines the serializable message types sent over iroh/QUIC channels.
//! Reliable channel: commands, events, and setup messages.
//! Unreliable channel: raw binary state snapshots (handled separately).

use serde::{Deserialize, Serialize};

use crate::config::WizardType;
use crate::game::units::wizard::components::Spell;

/// Wire-protocol version.
///
/// Bumped on any breaking change to `NetworkMessage`, `SpellVisualSnapshot`,
/// `GameSnapshot`, or any sub-struct on the wire. Peers exchange their
/// version in a dedicated `HandshakeVersion` message (sent BEFORE any other
/// message). If it doesn't match, the lobby errors out with a clear
/// version-mismatch error.
///
/// The exchange is done as a SEPARATE message (rather than embedded in
/// `PlayerInfo`) so peers on a previous binary that don't know about
/// `HandshakeVersion` decode it as an unknown enum variant → bincode error
/// → message dropped with a warning. The new binary then never receives a
/// `HandshakeVersion` from the old peer, detects the mismatch via the
/// "first message must be HandshakeVersion" invariant, and disconnects.
///
/// History:
/// - 1: initial versioned protocol (adds `HandshakeVersion` variant, adds
///   `cast_events` to `SpellVisualSnapshot`).
/// - 2: adds `width` to `BeamSnapshot` and the `DisintegrateChannel`
///   `SpellSoundId` ordinal (multiplayer spell-visual overhaul).
/// - 3: adds `UnitFlags::POLYMORPH` (1 << 11) to the unit snapshot so the guest
///   renders host-cast sheep. No new wire field — repurposes a free flag bit.
/// - 4: `GameOver` now carries a `HostMatchSummary` (side kills/deaths + host
///   wizard spell stats) and a new `WizardStatsReport` variant carries the guest
///   wizard's spell stats, for the multiplayer post-match scoreboard.
/// - 5: full wizard-archetype multiplayer support. Adds `WeatherChanged`
///   (Meteorologist) and the Swordcerer avatar control messages
///   (`SwordcererAvatarSpawn` / `SwordcererAvatarInput` / `SwordcererAvatarDied` /
///   `SwordcererAvatarRetreat`) — all appended AFTER `HandshakeVersion` to keep
///   its wire index frozen. Also adds gun-visual + sword-arc `CastEventKind`
///   variants, the `FlameGroundFire` `SpellEffectKind`, the
///   `SWORDCERER_AVATAR` / `SMELLY` `UnitFlags` bits, and new `SpellSoundId`
///   entries (Excremage fart, gun shots, weather lightning).
/// - 6: adds `CauldronBuffsSync` (guest Alchemist army buffs replicated to the
///   host) — appended after `HandshakeVersion`.
/// - 7: drops the unused `SwordcererAvatarRetreat` variant and trims
///   `CauldronBuffsSync` to the replicated scalars; removes `ExcremageFart`.
/// - 8: adds the `Forfeit` message (pause-menu forfeit, appended after
///   `HandshakeVersion`) and the `UnitFlags::IN_MELEE` bit (so the guest hears
///   melee battle ambience). Both additive; semantics only.
/// - 9: adds `effectiveness_bonus` to `CauldronBuffsSync` so a guest Alchemist's
///   Effectiveness brew reaches their army (now isolated on its own
///   `Effectiveness.cauldron_spell_bonus` field, away from poison).
/// - 10: co-op mode. `StartGame` gains `coop_mode: Option<u8>` (None = versus,
///   Some(SessionMode ordinal) = co-op) and `level: u32` (the host's endless/
///   roguelite level, ignored in versus). Adds the co-op lifecycle messages
///   `CoopLevelOver` / `CoopRunEnded` — appended AFTER `Forfeit` to keep every
///   prior wire index frozen.
/// - 11: co-op lobby restructure. Adds `HostModeSelection` (host→guest live
///   broadcast of the selected game-mode/level so the guest's Multiplayer-tab
///   left panel mirrors what's about to start). Appended after `CoopRunEnded`.
/// - 12: synchronized co-op pause. `StartGame` gains `urgent: bool` (whether the
///   co-op roguelite Urgent toggle is active, so the guest — which has no
///   `ActiveToggles` — knows to disable pause-sync). Adds `CoopPauseSync`
///   (either peer broadcasts the authoritative pause state; the initiator
///   re-sends it as a self-healing heartbeat). Appended after `HostModeSelection`.
/// - 13: per-unit buff visuals. Widens `UnitSnapshot.flags` from u16 to u32
///   (bincode positional — breaking) and adds the `BERSERKER_RAGE` /
///   `BATTLE_HYMN` / `TEMP_HP` / `HASTE` / `HEALING` bits so lingering buff
///   indicators render on guest ghosts regardless of who cast the buff.
/// - 14: adds `flags: u8` to `BeamSnapshot` (bincode positional — breaking) with
///   `BEAM_FLAG_FROM_CRYSTAL`, so the receiving peer can tell an Arcane Crystal's
///   beams from a wizard's and skip the wizard-only glow cone and origin flare.
///   Without it the flare — an opaque sphere scaled to `FLARE_RADIUS` and centred on
///   the beam origin — was drawn on top of the crystal that emitted it.
pub const PROTOCOL_VERSION: u32 = 14;

/// Messages sent over the reliable iroh/QUIC channel between peers.
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
    ///
    /// `coop_mode` is `None` for a versus match (existing behavior) and
    /// `Some(SessionMode ordinal)` for co-op (`0` = CoopEndless, `1` =
    /// CoopRoguelite). `level` is the host's endless/roguelite level so the
    /// co-op guest generates matching seeded terrain; it is ignored in versus.
    /// NOTE: adding fields to this existing variant is a wire-layout change,
    /// made safe by the v10 bump — the handshake rejects any v9↔v10 pairing
    /// before a `StartGame` is ever exchanged.
    StartGame {
        seed: u64,
        coop_mode: Option<u8>,
        level: u32,
        /// True when this is a co-op roguelite match with the Urgent toggle
        /// active. The guest has no `ActiveToggles`, so it learns the flag here
        /// and stores it on its `MultiplayerSession.coop_urgent`. Ignored in
        /// versus and co-op endless. (Wire-layout change to an existing variant,
        /// made safe by the v12 bump — the handshake rejects a v11↔v12 pairing
        /// before any `StartGame` is exchanged.)
        urgent: bool,
    },

    /// Player has finished loading.
    GameLoaded,

    /// Host notifies guest that the game is over, carrying the host's
    /// authoritative match summary (side kills/deaths + host wizard spell stats)
    /// so both peers can render the post-match scoreboard.
    GameOver {
        result: GameOverResult,
        summary: HostMatchSummary,
    },

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

    /// Guest → host, sent once after the guest reaches the score screen: the
    /// guest wizard's total spell damage and healing this match, so the host can
    /// fill in the enemy column of its scoreboard.
    WizardStatsReport {
        spell_damage: f32,
        spell_healing: f32,
    },

    /// Wire-protocol version handshake. Sent FIRST by both peers on
    /// connection, BEFORE `PlayerInfo`. The receiver compares against its
    /// own `PROTOCOL_VERSION`; on mismatch, the lobby transitions to
    /// `Failed` with a clear message.
    ///
    /// **Placement matters**: `HandshakeVersion`'s wire index is FROZEN.
    /// bincode 1.x encodes an enum variant by its POSITIONAL INDEX in
    /// declaration order, so this variant's index must never change —
    /// older binaries (which don't know about it) decode it as an unknown
    /// variant index → bincode returns `Err` → message dropped with a warn,
    /// and a peer on a different protocol version decodes the wrong variant
    /// unless every prior variant keeps its index. The new binary then
    /// detects the mismatch via the "first message must be HandshakeVersion"
    /// invariant + version compare. **Append all future variants AFTER this
    /// one; never insert a variant before it.** (The new trailing variants
    /// are only exchanged between two same-version peers after a successful
    /// handshake, so older peers never receive them.)
    HandshakeVersion { version: u32 },

    // --- Variants below are appended after `HandshakeVersion` to keep its
    //     wire index frozen (see the note above). v5+ only. ---
    /// Meteorologist weather change. Sent host→guest to replicate the active
    /// weather (so the opponent renders rain/storm), and guest→host when the
    /// guest is the Meteorologist (the host is authoritative for the weather
    /// simulation). `weather` is `None` when cleared, else a `WeatherType`
    /// ordinal (`WeatherType::from_u8`). `intensity` is the 0..1 ramp.
    WeatherChanged { weather: Option<u8>, intensity: f32 },

    /// Guest → host: the guest (a Swordcerer) deployed its avatar at the given
    /// battlefield position. The host spawns the authoritative avatar on the
    /// guest's army team.
    SwordcererAvatarSpawn { x: f32, z: f32 },

    /// Guest → host: per-tick avatar control input for a guest-controlled
    /// Swordcerer avatar (movement delta + attack intents + facing). The host
    /// applies it to the authoritative avatar entity.
    SwordcererAvatarInput {
        dx: f32,
        dz: f32,
        fire_missile: bool,
        swing_sword: bool,
        facing_x: f32,
        facing_z: f32,
    },

    /// Host → guest: the guest's Swordcerer avatar died on the authoritative
    /// simulation; the guest tears down its local UI / control state. (Retreat
    /// is death-only, so there is no separate voluntary-retreat message.)
    SwordcererAvatarDied,

    /// Guest → host: the guest Alchemist's current army-buff scalars, sent when
    /// they change. The host applies them to the guest's army (Attackers).
    /// Effectiveness rides its own dedicated `Effectiveness.cauldron_spell_bonus`
    /// field (separate from poison's `spell_bonus`), so it replicates safely.
    /// The enemy-slow is still omitted — it targets the host's army, which the
    /// host's own cleanup manages.
    CauldronBuffsSync {
        heal_per_second: f32,
        damage_bonus: f32,
        resistance_percent: f32,
        shield_per_second: f32,
        speed_bonus: f32,
        effectiveness_bonus: f32,
    },

    /// Guest → host: the guest forfeited the match. The host ends the game with
    /// itself as the winner and ships the resulting `GameOver` back.
    Forfeit,

    // --- Co-op lifecycle (v10). Host drives the guest through the multi-level
    //     endless/roguelite flow, since the host runs the authoritative SP
    //     simulation in `AppState::InGame` while the guest spectates+casts in
    //     `AppState::MultiplayerGame`. Appended after `Forfeit`. ---
    /// Host → guest: a co-op level just ended. The guest mirrors the outcome to
    /// its score screen and records its own progression/achievements from the
    /// shared totals (its own spell stats are tracked locally). `level` is the
    /// host's level number for this battle (used by the guest's endless
    /// contiguity rule).
    CoopLevelOver {
        victory: bool,
        level: u32,
        defenders_killed: u32,
        attackers_killed: u32,
        undead_killed: u32,
        /// Wall-clock duration of the level, so the guest records a real
        /// `elapsed_time` in its own endless best-stats instead of 0.
        elapsed_time: f32,
        host_spell_damage: f32,
        host_spell_healing: f32,
    },

    /// Host → guest: the co-op run/session ended (roguelite run finished, or an
    /// endless defeat). The guest finalizes its own save record and returns to
    /// the wizard tower.
    CoopRunEnded { victory: bool },

    /// Host → guest (lobby): a live broadcast of the host's currently-selected
    /// game mode so the guest's Multiplayer-tab left panel mirrors what's about
    /// to start. Sent while the host sits in the WizardSelect lobby on any tab.
    /// `detail_lines` are host-formatted summary lines (roguelite modifier
    /// summary, endless level info) shown verbatim by the guest.
    HostModeSelection {
        mode: HostMode,
        host_wizard: WizardType,
        level: u32,
        is_continue: bool,
        detail_lines: Vec<String>,
    },

    /// Either peer → other (v12): the authoritative co-op pause state. Sent when a
    /// peer pauses or resumes, and re-sent by the initiator as a low-rate heartbeat
    /// (and for a short grace after resuming) so a dropped packet can't leave the
    /// other player stuck paused. `initiator_is_host` identifies who owns the
    /// pause (only that peer can resume); it also breaks a simultaneous-pause tie
    /// deterministically in the host's favor. Co-op + non-Urgent only.
    CoopPauseSync {
        paused: bool,
        initiator_is_host: bool,
    },
}

/// Which game mode the host has selected in the lobby, broadcast to the guest in
/// `NetworkMessage::HostModeSelection`. A shared enum (rather than a raw `u8`) so
/// the broadcaster and the guest's renderer can't drift out of sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HostMode {
    /// Host is on a non-startable tab (Multiplayer/Study) — no mode chosen yet.
    #[default]
    Browsing,
    Endless,
    Roguelite,
    Versus,
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
///
/// Many constants in this module are deliberately ahead of their senders —
/// the bit layouts document the wire protocol for talent variants we plan
/// to forward but haven't yet wired the cast-side packing for. They are
/// allowed dead code so the protocol stays self-documenting in one place.
#[allow(dead_code)] // bit layouts deliberately ahead of their senders; kept for wire-protocol self-documentation
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
    pub const POLYMORPH_PIG: u32 = 1 << 5;
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

/// Host's authoritative end-of-match summary, sent inside `GameOver`. Field
/// names are side-neutral; each receiver maps them to its own perspective by
/// role (host commands Defenders, guest commands Attackers).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HostMatchSummary {
    /// Defenders that died = host deaths = guest kills.
    pub defenders_killed: u32,
    /// Attackers + undead that died = host kills = guest deaths.
    pub attackers_and_undead_killed: u32,
    /// Host wizard's total spell damage this match.
    pub host_spell_damage: f32,
    /// Host wizard's total spell healing this match.
    pub host_spell_healing: f32,
}
