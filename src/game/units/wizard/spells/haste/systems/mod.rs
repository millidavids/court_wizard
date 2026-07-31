mod casting;
mod effects;
mod speed_lines;

pub use casting::handle_haste_casting;
pub use effects::handle_haste_expiry;
pub use effects::tick_haste_slow_zone;
pub use speed_lines::{HasteSpeedLine, emit_haste_speed_line_vfx, update_haste_speed_lines};
