//! Benchmarking diagnostics module.
//!
//! Entire module is only compiled when the `benchmarking` cargo feature is
//! enabled (via the `bench-release` profile). The regular release build
//! contains zero bytes of this code.

mod plugin;
mod resources;
mod systems;

pub(crate) use plugin::BenchmarkingPlugin;
