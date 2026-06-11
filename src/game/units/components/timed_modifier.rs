/// Trait for modifier components with a timed duration that can expire.
///
/// Implementing this trait allows use of the generic `update_timed_modifier::<T>` system
/// which automatically ticks and removes expired modifiers.
pub trait TimedModifier {
    /// Tick the modifier's timer by `delta` seconds. Returns `true` if expired.
    fn tick(&mut self, delta: f32) -> bool;
}

#[macro_export]
macro_rules! impl_timed_modifier {
    ($($ty:ty),* $(,)?) => {
        $(impl $crate::game::units::components::TimedModifier for $ty {
            fn tick(&mut self, delta: f32) -> bool {
                self.update(delta)
            }
        })*
    };
}
pub(crate) use impl_timed_modifier;
