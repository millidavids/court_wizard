//! Pins the on-disk TOML formats (`saves_v2.json`, `config.toml`) across `toml`
//! upgrades. Existing saves must keep loading, signatures must not drift
//! silently, and a player who rolls back to a toml 0.8 build must still be able
//! to read what this build wrote.

use super::save_cache::{compute_save_signature, new_unified_save};
use super::save_structs::{UnifiedSaveFile, WizardSave};
use crate::config::GameConfig;
use crate::config::resources::ConfigFile;
use crate::game::seeded_rng::resources::MAX_SEED;

// One entry per map: HashMap iteration order isn't stable across runs.
const WIZARD_TOML: &str = r#"
id = "wizard-1"
wizard_type = "RuneCaster"
current_level = 3
highest_level_achieved = 4
created_at = 1700000000
last_played_at = 1700000500

[efficiency_ratios]
"1" = 0.3
"#;

/// `fixture()` as written by toml 0.8 — the format already on players' disks.
const TOML_0_8_SAVE: &str = r#"[metadata]
version = 2
signature = ""

[player]
total_levels_completed = 0
total_games_played = 0
total_defenders_killed = 0
total_attackers_killed = 0
total_undead_killed = 0
total_defenders_killed_by_spell = 0
total_spells_cast = 0
total_bosses_defeated = 0
total_roguelite_clears = 0
total_play_time_secs = 1234.5
unlocked_achievements = []
arcane_insight = 0
applied_migrations = ["wall_of_dirt_retier_v1"]
completed_tutorials = []
unlocked_toggles = []

[player.unlocked_content]
spells = [
    "MagicMissile",
    "Entangle",
    "GuardianCircle",
    "Telekinesis",
]
ingredients = ["Lavender"]
wizard_types = ["BoringOleMage"]
combos = []
units = [
    "Infantry",
    "Archer",
    "King",
    "KingsGuard",
]

[player.spell_research_progress]

[player.spell_talent_progress]

[player.spell_talent_selections]

[player.insight_bonuses]

[player.insight_bonus_progress]

[[wizards]]
id = "wizard-1"
wizard_type = "RuneCaster"
current_level = 3
highest_level_achieved = 4
created_at = 1700000000
last_played_at = 1700000500
saved_walls = []
saved_crystals = []
saved_flora = []

[wizards.efficiency_ratios]
1 = 0.30000001192092896

[wizards.action_bar_slots]

[wizards.saved_trampling]
grid_size = 0

[wizards.roguelite]
run_history = []

[wizards.endless_best_stats]

[wizards.terrain_per_level]
"#;

fn fixture() -> UnifiedSaveFile {
    let mut save = new_unified_save();
    save.player.total_play_time_secs = 1234.5;
    let wizard: WizardSave = toml::from_str(WIZARD_TOML).expect("fixture wizard parses");
    save.wizards.push(wizard);
    save
}

#[test]
fn save_signature_is_pinned() {
    assert_eq!(
        compute_save_signature(&fixture()),
        "f5a619a2f60f92c72d9fd1685e92344c"
    );
}

#[test]
fn saves_written_by_toml_0_8_still_load() {
    let save: UnifiedSaveFile = toml::from_str(TOML_0_8_SAVE).expect("old save parses");
    assert_eq!(save.player.total_play_time_secs, 1234.5);
    assert_eq!(save.wizards.len(), 1);
    assert_eq!(save.wizards[0].efficiency_ratios["1"], 0.3);
}

// toml 0.8 rejects integers above i64::MAX, so a full-range u64 seed would
// make the save unreadable after a rollback. Seeds are capped at MAX_SEED.
#[test]
fn toml_0_8_builds_can_read_new_saves() {
    let mut save = fixture();
    let run = format!(
        "victory = true\nlevels_completed = 5\nstarted_at = 1\nended_at = 2\n\
         level_stats = []\nseed = {}",
        MAX_SEED - 1
    );
    save.wizards[0]
        .roguelite
        .run_history
        .push(toml::from_str(&run).expect("fixture run parses"));
    let written = toml::to_string_pretty(&save).expect("serializes");
    let read: UnifiedSaveFile = toml_legacy::from_str(&written).expect("toml 0.8 reads it");
    assert_eq!(
        read.wizards[0].roguelite.run_history[0].seed,
        Some(MAX_SEED - 1)
    );
}

#[test]
fn toml_0_8_builds_can_read_new_config() {
    let config = ConfigFile {
        game: GameConfig {
            master_volume: 0.3,
            ..Default::default()
        },
        ..Default::default()
    };
    let written = toml::to_string_pretty(&config).expect("serializes");
    let read: ConfigFile = toml_legacy::from_str(&written).expect("toml 0.8 reads it");
    assert_eq!(read.game.master_volume, 0.3);
}
