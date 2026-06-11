pub(crate) mod buff_application;
pub(crate) mod casting;
pub(crate) mod talent_effects;

pub use casting::handle_berserker_rage_casting;
pub use talent_effects::{
    cleanup_berserker_rage_talents, contagious_rage_spread, final_stand_explosion,
    frenzy_check_system, tick_undying_fury_active, undying_fury_trigger, update_final_stand_vfx,
};
