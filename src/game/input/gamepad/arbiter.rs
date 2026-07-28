//! Shared input-device arbitration state.
//!
//! Both `ActiveInputDevice` writers — the gilrs detector
//! (`systems::connection::detect_active_input_device`) and the Steam Input poll
//! (`crate::steam::input` — runs right after it in the same `PreUpdate` chain) —
//! consult this resource so they enforce identical rules:
//!
//! - Mouse/keyboard events reclaim focus (debounced against held pad inputs).
//! - A pad button edge is unambiguous intent and claims focus immediately,
//!   except on a frame that itself had mouse/keyboard events.
//! - Stick deflection is weak intent: it is measured against a tracked resting
//!   baseline (never as absolute position, so a drifting stick can't claim
//!   focus) and additionally gated behind [`DEVICE_SWITCH_GRACE_SECS`] of
//!   mouse/keyboard silence.

use bevy::prelude::*;

use super::constants::{
    BASELINE_MAX_STEP_SECS, DEVICE_SWITCH_GRACE_SECS, DEVICE_SWITCH_STICK_MAGNITUDE,
    STICK_ABSOLUTE_INTENT, STICK_BASELINE_TAU_SECS,
};

/// Tracks one input source's four stick axes against a slowly-adapting resting
/// baseline, classifying deflection *away from rest* as intent.
#[derive(Debug, Default, Clone)]
pub(crate) struct StickIntentTracker {
    baseline: Option<[f32; 4]>,
    last_intent: bool,
}

impl StickIntentTracker {
    /// Feed the current axis values; returns true when the stick shows intent.
    ///
    /// Intent = deflection from the tracked baseline past
    /// [`DEVICE_SWITCH_STICK_MAGNITUDE`], or any axis near full deflection
    /// ([`STICK_ABSOLUTE_INTENT`] — no stick *drifts* to full throw, so this
    /// catches deliberate pushes slow enough for the baseline to chase, e.g.
    /// smooth assistive controllers).
    ///
    /// The baseline only adapts while there is NO intent (freezing during
    /// deflection stops a >~0.6s hold from teaching the baseline the held
    /// position, which would turn the release spring-back into phantom
    /// intent), and each step is clamped so a frame hitch can't snap the
    /// baseline onto a momentarily-held stick.
    ///
    /// First sight seeds the baseline at the current position and reports no
    /// intent, so a pad connected mid-deflection can't instantly claim focus.
    pub(crate) fn assess(&mut self, axes: [f32; 4], dt: f32) -> bool {
        let Some(baseline) = self.baseline.as_mut() else {
            self.baseline = Some(axes);
            self.last_intent = false;
            return false;
        };

        let deflection_sq: f32 = axes
            .iter()
            .zip(baseline.iter())
            .map(|(v, b)| (v - b) * (v - b))
            .sum();
        let near_full = axes.iter().any(|v| v.abs() >= STICK_ABSOLUTE_INTENT);
        let intent = near_full
            || deflection_sq > DEVICE_SWITCH_STICK_MAGNITUDE * DEVICE_SWITCH_STICK_MAGNITUDE;

        if !intent {
            let alpha = (dt.min(BASELINE_MAX_STEP_SECS) / STICK_BASELINE_TAU_SECS).min(1.0);
            for (b, v) in baseline.iter_mut().zip(axes) {
                *b += (v - *b) * alpha;
            }
        }
        self.last_intent = intent;
        intent
    }

    /// Result of the most recent [`Self::assess`] call (one frame stale for
    /// readers scheduled before this source's producer).
    pub(crate) fn deflected(&self) -> bool {
        self.last_intent
    }

    /// Forget the baseline (call on connect/disconnect so a returning pad
    /// re-seeds at its current position instead of a stale rest point).
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Cross-producer arbitration state. Written by `detect_active_input_device`
/// (which owns the mouse/keyboard event readers and runs first), read by the
/// Steam Input poll.
#[derive(Resource, Debug, Default)]
pub(crate) struct InputDeviceArbiter {
    /// Time (Time<Real> elapsed secs) of the most recent mouse/keyboard event.
    /// `None` = no event yet this session, which counts as quiet.
    pub(crate) last_mk_activity: Option<f64>,
    /// Whether the current frame had any mouse/keyboard events. Producers must
    /// not claim the device on such a frame.
    pub(crate) mk_event_this_frame: bool,
    /// Stick baseline for the (single) Steam Input pad.
    pub(crate) steam_stick: StickIntentTracker,
}

impl InputDeviceArbiter {
    /// True when mouse/keyboard has been silent long enough for *weak* (stick)
    /// intent to claim the device.
    pub(crate) fn mk_quiet(&self, now: f64) -> bool {
        self.last_mk_activity
            .is_none_or(|t| now - t >= DEVICE_SWITCH_GRACE_SECS)
    }
}
