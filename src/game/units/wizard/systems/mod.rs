mod asset_loading;
mod runtime;
mod spawn;

pub use asset_loading::load_wizard_assets;
pub use runtime::{
    apply_wizard_stats_to_primed_spell, cancel_active_casts, handle_prime_spell_messages,
    mana_on_kill, regenerate_mana, reset_empowerment_after_cast, sync_mana_cost_multiplier,
    track_spell_casts_for_rotation, update_wizard_animation,
};
pub(crate) use spawn::apply_archetype_stat_bonuses;
pub use spawn::setup_wizard;
