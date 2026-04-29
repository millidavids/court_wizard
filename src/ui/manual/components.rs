//! Manual screen components.

use bevy::prelude::*;

/// Marker component for entities that belong to the manual screen.
#[derive(Component)]
pub(super) struct OnManualScreen;

/// Marker component for the back button.
#[derive(Component)]
pub(super) struct BackButton;

/// Marker component for the scrollable content area.
#[derive(Component)]
pub(super) struct ScrollableManualContainer;

/// Marker component for the content panel whose children are rebuilt on tab switch.
#[derive(Component)]
pub(super) struct ManualContentPanel;

/// Identifies which tab a button corresponds to.
#[derive(Component)]
pub(super) struct ManualTabButton(pub ManualTab);

#[derive(Component)]
pub(super) struct ChangelogWebsiteButton;

/// Which tab is currently active.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) enum ManualTab {
    #[default]
    Instructions,
    Changelog,
    Credits,
    License,
    Health,
    Privacy,
}

impl ManualTab {
    pub fn all() -> &'static [ManualTab] {
        &[
            ManualTab::Instructions,
            ManualTab::Changelog,
            ManualTab::Credits,
            ManualTab::License,
            ManualTab::Health,
            ManualTab::Privacy,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ManualTab::Instructions => "Instructions",
            ManualTab::Changelog => "Changelog",
            ManualTab::Credits => "Credits",
            ManualTab::License => "License",
            ManualTab::Health => "Health",
            ManualTab::Privacy => "Privacy",
        }
    }
}
