use bevy::prelude::*;
use bevy_steamworks::Client;

use crate::config::save_data::lifetime_stat_totals;

// Steam stat API names — must match the Steamworks partner-site definitions exactly.
const STAT_TOTAL_VICTORIES: &str = "STAT_TOTAL_VICTORIES";
const STAT_TOTAL_BATTLES: &str = "STAT_TOTAL_BATTLES";
const STAT_ATTACKERS_KILLED: &str = "STAT_ATTACKERS_KILLED";
const STAT_DEFENDERS_LOST: &str = "STAT_DEFENDERS_LOST";
const STAT_UNDEAD_KILLED: &str = "STAT_UNDEAD_KILLED";
const STAT_HIGHEST_ENDLESS_LEVEL: &str = "STAT_HIGHEST_ENDLESS_LEVEL";
const STAT_ROGUELITE_CLEARS: &str = "STAT_ROGUELITE_CLEARS";
const STAT_WIZARDS_UNLOCKED: &str = "STAT_WIZARDS_UNLOCKED";
const STAT_FRIENDLY_FIRE_KILLS: &str = "STAT_FRIENDLY_FIRE_KILLS";
const STAT_BOSSES_DEFEATED: &str = "STAT_BOSSES_DEFEATED";
const STAT_SPELLS_CAST: &str = "STAT_SPELLS_CAST";
const STAT_PLAYTIME_SECONDS: &str = "STAT_PLAYTIME_SECONDS";
const STAT_INSIGHT_BONUSES_MAXED: &str = "STAT_INSIGHT_BONUSES_MAXED";
const STAT_SPELLS_UNLOCKED: &str = "STAT_SPELLS_UNLOCKED";

/// Saturating `u32` → `i32`: Steam stats are `i32`, lifetime totals are `u32`.
fn clamp_i32(value: u32) -> i32 {
    value.min(i32::MAX as u32) as i32
}

/// Saturating `f64` seconds → `i32` (floored, never negative).
fn clamp_f64(value: f64) -> i32 {
    value.max(0.0).min(i32::MAX as f64) as i32
}

/// Mirrors the save's authoritative lifetime totals up to Steam stats.
///
/// Each stat is pushed only when the new value EXCEEDS the value Steam currently
/// holds, so a lifetime stat can never regress — protecting against a progress reset
/// (`clear_progress` resets unlock counts), a Steam-Cloud conflict, or a derived value
/// that briefly dips (e.g. roguelite clears recovered from trimmed run history). The
/// first push still backfills, since an unset/not-yet-loaded stat reads as 0.
///
/// `get_stat_i32`/`set_stat_i32`/`store_stats` require current-user stats to have been
/// delivered first; a very early push (e.g. the first `MainMenu` frame on a cold
/// launch) may no-op, which is fine — later pushes (MetaGame / ScoreScreen) correct it.
/// Errors are intentionally ignored for that reason.
pub(super) fn sync_stats_to_steam(client: Res<Client>) {
    let totals = lifetime_stat_totals();
    let user_stats = client.user_stats();

    let entries: [(&str, i32); 14] = [
        (STAT_TOTAL_VICTORIES, clamp_i32(totals.total_victories)),
        (STAT_TOTAL_BATTLES, clamp_i32(totals.total_battles)),
        (STAT_ATTACKERS_KILLED, clamp_i32(totals.attackers_killed)),
        (STAT_DEFENDERS_LOST, clamp_i32(totals.defenders_lost)),
        (STAT_UNDEAD_KILLED, clamp_i32(totals.undead_killed)),
        (
            STAT_HIGHEST_ENDLESS_LEVEL,
            clamp_i32(totals.highest_endless_level),
        ),
        (STAT_ROGUELITE_CLEARS, clamp_i32(totals.roguelite_clears)),
        (STAT_WIZARDS_UNLOCKED, clamp_i32(totals.wizards_unlocked)),
        (
            STAT_FRIENDLY_FIRE_KILLS,
            clamp_i32(totals.friendly_fire_kills),
        ),
        (STAT_BOSSES_DEFEATED, clamp_i32(totals.bosses_defeated)),
        (STAT_SPELLS_CAST, clamp_i32(totals.spells_cast)),
        (STAT_PLAYTIME_SECONDS, clamp_f64(totals.play_time_secs)),
        (
            STAT_INSIGHT_BONUSES_MAXED,
            clamp_i32(totals.insight_bonuses_maxed),
        ),
        (STAT_SPELLS_UNLOCKED, clamp_i32(totals.spells_unlocked)),
    ];

    let mut changed = false;
    for (name, value) in entries {
        // Read-as-0 when unset/not-loaded so the first push backfills; never regress.
        let current = user_stats.get_stat_i32(name).unwrap_or(0);
        if value > current && user_stats.set_stat_i32(name, value).is_ok() {
            changed = true;
        }
    }

    if changed {
        let _ = user_stats.store_stats();
    }
}
