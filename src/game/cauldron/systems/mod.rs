mod army_buffs;
mod brew_lifecycle;
mod multiplayer;
mod spawn;
mod visuals;

pub use army_buffs::{
    apply_cauldron_speed_modifiers, apply_max_mana_buff, buff_defender_damage,
    buff_defender_effectiveness, buff_defender_resistance, cleanup_cauldron_buff_components,
    heal_defenders, shield_defenders,
};
pub use brew_lifecycle::{
    block_spells_during_brewing, handle_brew_complete, handle_cancel_brew, handle_start_brew,
    tick_active_buffs, update_brew_timer,
};
pub use multiplayer::{
    apply_guest_army_buffs, receive_cauldron_buffs, reset_remote_cauldron_buffs,
    send_cauldron_buffs_to_host,
};
pub use spawn::{load_cauldron_assets, spawn_cauldron};
pub use visuals::{
    start_brewing_effects, update_brew_bubble, update_brewing_effects, update_brewing_timer,
    update_cauldron_animation,
};
