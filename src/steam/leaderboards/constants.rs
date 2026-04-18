use bevy_steamworks::{LeaderboardDisplayType, LeaderboardSortMethod};

/// Identifies which Steam leaderboard a submission targets.
///
/// Each variant maps to a dashboard-registered leaderboard via `steam_board_name`.
/// The string names here must match the Steamworks partner site entries exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum LeaderboardId {
    EndlessLevel,
    RogueliteScoreDefault,
    RogueliteScoreModified,
    RogueliteClearTimeDefault,
    RogueliteClearTimeModified,
}

impl LeaderboardId {
    pub(super) const fn all() -> [LeaderboardId; 5] {
        use LeaderboardId::*;
        [
            EndlessLevel,
            RogueliteScoreDefault,
            RogueliteScoreModified,
            RogueliteClearTimeDefault,
            RogueliteClearTimeModified,
        ]
    }
}

pub(super) fn steam_board_name(id: LeaderboardId) -> &'static str {
    match id {
        LeaderboardId::EndlessLevel => "endless_level",
        LeaderboardId::RogueliteScoreDefault => "roguelite_score_default",
        LeaderboardId::RogueliteScoreModified => "roguelite_score_modified",
        LeaderboardId::RogueliteClearTimeDefault => "roguelite_clear_time_default",
        LeaderboardId::RogueliteClearTimeModified => "roguelite_clear_time_modified",
    }
}

pub(super) fn sort_method(id: LeaderboardId) -> LeaderboardSortMethod {
    match id {
        LeaderboardId::RogueliteClearTimeDefault | LeaderboardId::RogueliteClearTimeModified => {
            LeaderboardSortMethod::Ascending
        }
        _ => LeaderboardSortMethod::Descending,
    }
}

pub(super) fn display_type(id: LeaderboardId) -> LeaderboardDisplayType {
    match id {
        LeaderboardId::RogueliteClearTimeDefault | LeaderboardId::RogueliteClearTimeModified => {
            LeaderboardDisplayType::TimeSeconds
        }
        _ => LeaderboardDisplayType::Numeric,
    }
}
