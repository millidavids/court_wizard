//! iroh connection management — endpoint creation, address exchange, and I/O loops.

mod endpoint;
mod guest;
mod helpers;
mod host;
mod io;
mod reliable;
mod runtime_loop;
mod ticket;
mod unreliable;

pub(super) use runtime_loop::run_transport; // pub(crate) in runtime_loop; re-exported as pub(super) to transport
