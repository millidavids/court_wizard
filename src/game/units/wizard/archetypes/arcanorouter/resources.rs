use bevy::prelude::*;

use super::constants::*;

/// Represents which slider is being adjusted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderType {
    Range,
    Mana,
    Power,
    Speed,
}

impl SliderType {
    /// Returns all slider types in order
    pub const fn all() -> [SliderType; 4] {
        [
            SliderType::Range,
            SliderType::Mana,
            SliderType::Power,
            SliderType::Speed,
        ]
    }
}

/// Captures the Arcanorouter's `range_allocation` at multiplayer match start so
/// it can be pinned during the setup stage (the player may still adjust
/// mana/power/speed, but range must not rise). Present only for an Arcanorouter
/// in multiplayer; inserted in `init_mp_game`, removed in `cleanup_mp_game`.
#[derive(Resource, Debug, Clone, Copy)]
pub(in crate::game) struct ArcanoRouterSetupBaseline(pub f32);

/// Tracks the current allocation state for all four resource sliders.
///
/// This resource manages a fixed pool of 400% that is distributed across
/// four stats: range, mana cost, spell power, and cast speed. Each slider
/// can range from 25% to 200%, and adjusting one slider automatically
/// redistributes the remaining percentage among the other three sliders.
#[derive(Resource, Debug, Clone)]
pub struct ArcanoRouterState {
    pub range_allocation: f32,
    pub mana_allocation: f32,
    pub power_allocation: f32,
    pub speed_allocation: f32,
}

impl Default for ArcanoRouterState {
    fn default() -> Self {
        Self {
            range_allocation: DEFAULT_ALLOCATION,
            mana_allocation: DEFAULT_ALLOCATION,
            power_allocation: DEFAULT_ALLOCATION,
            speed_allocation: DEFAULT_ALLOCATION,
        }
    }
}

impl ArcanoRouterState {
    /// Returns the total allocation across all sliders (should always be ~400.0)
    pub fn total_allocation(&self) -> f32 {
        self.range_allocation + self.mana_allocation + self.power_allocation + self.speed_allocation
    }

    /// Gets the current allocation for a specific slider
    pub fn get_allocation(&self, slider: SliderType) -> f32 {
        match slider {
            SliderType::Range => self.range_allocation,
            SliderType::Mana => self.mana_allocation,
            SliderType::Power => self.power_allocation,
            SliderType::Speed => self.speed_allocation,
        }
    }

    /// Sets the allocation for a specific slider (used internally)
    fn set_allocation(&mut self, slider: SliderType, value: f32) {
        match slider {
            SliderType::Range => self.range_allocation = value,
            SliderType::Mana => self.mana_allocation = value,
            SliderType::Power => self.power_allocation = value,
            SliderType::Speed => self.speed_allocation = value,
        }
    }

    /// Adjusts a slider's allocation and redistributes the change across other sliders.
    ///
    /// The delta is distributed evenly among the three non-adjusted sliders,
    /// then all values are clamped to [MIN_ALLOCATION, MAX_ALLOCATION] and
    /// normalized to maintain the total pool of 400%.
    pub fn adjust_allocation(&mut self, slider: SliderType, delta: f32) {
        // Get current value and calculate new value
        let old_value = self.get_allocation(slider);
        let new_value = (old_value + delta).clamp(MIN_ALLOCATION, MAX_ALLOCATION);

        // Calculate actual change (might be less than delta due to clamping)
        let actual_delta = new_value - old_value;

        // If no actual change, return early
        if actual_delta.abs() < 0.01 {
            return;
        }

        // Apply the change to the target slider
        self.set_allocation(slider, new_value);

        // Distribute the inverse delta evenly across the other three sliders
        let adjustment_per_slider = -actual_delta / 3.0;

        for other_slider in SliderType::all() {
            if other_slider != slider {
                let current = self.get_allocation(other_slider);
                let adjusted =
                    (current + adjustment_per_slider).clamp(MIN_ALLOCATION, MAX_ALLOCATION);
                self.set_allocation(other_slider, adjusted);
            }
        }

        // Normalize to ensure total is exactly 400.0
        self.normalize();
    }

    /// Normalizes all allocations to maintain the target total of 400%.
    ///
    /// This is called after adjustments to correct for any rounding errors
    /// or clamping side effects.
    fn normalize(&mut self) {
        let total = self.total_allocation();
        let target = TOTAL_ALLOCATION_POOL;

        // If we're already very close to the target, don't bother normalizing
        if (total - target).abs() < 0.01 {
            return;
        }

        // Scale all allocations proportionally
        let scale = target / total;
        self.range_allocation *= scale;
        self.mana_allocation *= scale;
        self.power_allocation *= scale;
        self.speed_allocation *= scale;
    }

    /// Re-balances mana/power/speed so they sum to
    /// `TOTAL_ALLOCATION_POOL - range_allocation`, leaving `range_allocation`
    /// exactly untouched. Used during the multiplayer setup stage to pin range
    /// while the other three sliders remain adjustable.
    ///
    /// Intentionally does NOT call [`normalize`](Self::normalize), which would
    /// rescale all four sliders and drift the pinned range. Because clamping each
    /// slider to `[MIN, MAX]` can leave the sum off-target, a residual pass
    /// redistributes the remainder onto sliders that still have headroom, so the
    /// pool stays balanced (`range` + the three == `TOTAL_ALLOCATION_POOL`).
    pub(super) fn normalize_excluding_range(&mut self) {
        let target_rest = TOTAL_ALLOCATION_POOL - self.range_allocation;
        let mut vals = [
            self.mana_allocation,
            self.power_allocation,
            self.speed_allocation,
        ];
        let current_rest: f32 = vals.iter().sum();

        if current_rest < 0.01 {
            // Degenerate: split the remaining pool evenly across the three.
            let each = (target_rest / 3.0).clamp(MIN_ALLOCATION, MAX_ALLOCATION);
            vals = [each, each, each];
        } else {
            let scale = target_rest / current_rest;
            for v in &mut vals {
                *v = (*v * scale).clamp(MIN_ALLOCATION, MAX_ALLOCATION);
            }
        }

        // Clamping can leave the sum off target; push the residual onto whichever
        // sliders still have room in the needed direction. `target_rest` is in
        // [200, 375] (range is in [25, 200]) and three sliders span [75, 600], so
        // a feasible distribution always exists; a few passes converge.
        for _ in 0..4 {
            let residual = target_rest - vals.iter().sum::<f32>();
            if residual.abs() < 0.01 {
                break;
            }
            let room: f32 = vals
                .iter()
                .map(|v| {
                    if residual > 0.0 {
                        MAX_ALLOCATION - v
                    } else {
                        v - MIN_ALLOCATION
                    }
                })
                .sum();
            if room < 0.01 {
                break;
            }
            for v in &mut vals {
                let slot_room = if residual > 0.0 {
                    MAX_ALLOCATION - *v
                } else {
                    *v - MIN_ALLOCATION
                };
                *v = (*v + residual * (slot_room / room)).clamp(MIN_ALLOCATION, MAX_ALLOCATION);
            }
        }

        self.mana_allocation = vals[0];
        self.power_allocation = vals[1];
        self.speed_allocation = vals[2];
    }

    /// Converts an allocation percentage to a multiplier (100% = 1.0x)
    #[allow(dead_code)] // used by tests
    pub fn get_multiplier(&self, slider: SliderType) -> f32 {
        self.get_allocation(slider) / 100.0
    }

    /// Resets all allocations to their default values (100% each)
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = ArcanoRouterState::default();
        assert_eq!(state.total_allocation(), 400.0);
        assert_eq!(state.get_multiplier(SliderType::Range), 1.0);
        assert_eq!(state.get_multiplier(SliderType::Mana), 1.0);
        assert_eq!(state.get_multiplier(SliderType::Power), 1.0);
        assert_eq!(state.get_multiplier(SliderType::Speed), 1.0);
    }

    #[test]
    fn test_adjust_allocation() {
        let mut state = ArcanoRouterState::default();

        // Increase power by 30%
        state.adjust_allocation(SliderType::Power, 30.0);

        // Power should be 130%, others should be reduced by 10% each
        assert!((state.power_allocation - 130.0).abs() < 1.0);
        assert!((state.range_allocation - 90.0).abs() < 1.0);
        assert!((state.mana_allocation - 90.0).abs() < 1.0);
        assert!((state.speed_allocation - 90.0).abs() < 1.0);

        // Total should still be 400%
        assert!((state.total_allocation() - 400.0).abs() < 0.1);
    }

    #[test]
    fn test_clamping() {
        let mut state = ArcanoRouterState::default();

        // Try to increase power to beyond max
        state.adjust_allocation(SliderType::Power, 150.0);

        // Power should be clamped to MAX_ALLOCATION
        assert_eq!(state.power_allocation, MAX_ALLOCATION);

        // Total should still be approximately 400%
        assert!((state.total_allocation() - 400.0).abs() < 1.0);
    }

    #[test]
    fn test_minimum_clamping() {
        let mut state = ArcanoRouterState::default();

        // Try to decrease power to beyond min
        state.adjust_allocation(SliderType::Power, -100.0);

        // Power should be clamped to MIN_ALLOCATION
        assert_eq!(state.power_allocation, MIN_ALLOCATION);

        // Total should still be approximately 400%
        assert!((state.total_allocation() - 400.0).abs() < 1.0);
    }

    #[test]
    fn test_normalize_excluding_range_pins_range_and_pool() {
        // A state where one non-range slider is at MAX, so a naive scale+clamp
        // would undershoot the pool (the bug the residual pass fixes).
        let mut state = ArcanoRouterState {
            range_allocation: 25.0,
            mana_allocation: 200.0,
            power_allocation: 25.0,
            speed_allocation: 150.0,
        };

        state.normalize_excluding_range();

        // Range is left exactly untouched.
        assert_eq!(state.range_allocation, 25.0);
        // The other three are re-balanced to fill the rest of the pool, so the
        // total returns to exactly 400 (no silent undershoot from clamping).
        assert!(
            (state.total_allocation() - TOTAL_ALLOCATION_POOL).abs() < 0.1,
            "pool drifted: {}",
            state.total_allocation()
        );
        // Every slider stays within bounds.
        for slider in SliderType::all() {
            let v = state.get_allocation(slider);
            assert!(
                (MIN_ALLOCATION..=MAX_ALLOCATION).contains(&v),
                "out of bounds: {v}"
            );
        }
    }
}
