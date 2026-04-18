use std::collections::HashMap;

use bevy::prelude::*;
use bevy_steamworks::Leaderboard;
use crossbeam_channel::{Receiver, Sender, unbounded};

use super::constants::LeaderboardId;

/// Cache of resolved Steam leaderboard handles, plus a channel used by the async
/// `find_or_create_leaderboard` callbacks to deliver handles back into the ECS world.
#[derive(Resource)]
pub(super) struct LeaderboardHandles {
    pub(super) map: HashMap<LeaderboardId, Leaderboard>,
    tx: Sender<(LeaderboardId, Leaderboard)>,
    rx: Receiver<(LeaderboardId, Leaderboard)>,
}

impl LeaderboardHandles {
    pub(super) fn sender(&self) -> Sender<(LeaderboardId, Leaderboard)> {
        self.tx.clone()
    }

    pub(super) fn drain(&mut self) {
        while let Ok((id, leaderboard)) = self.rx.try_recv() {
            self.map.insert(id, leaderboard);
        }
    }
}

impl Default for LeaderboardHandles {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self {
            map: HashMap::new(),
            tx,
            rx,
        }
    }
}
