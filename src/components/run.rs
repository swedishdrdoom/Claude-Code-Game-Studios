use bevy::prelude::*;

/// Game state machine — drives the entire run lifecycle.
/// All systems use `in_state()` run conditions to activate/deactivate.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    Loading,
    #[default]
    MainMenu,
    GracePeriod,
    Playing,
    Boss,
    Victory,
    Defeat,
}

/// Tracks elapsed time during the Playing state.
/// Timer starts at 0.0 when GracePeriod ends, counts up to RUN_DURATION.
#[derive(Resource, Debug)]
pub struct RunTimer {
    pub elapsed: f32,
    pub grace_remaining: f32,
}

impl Default for RunTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            grace_remaining: GRACE_PERIOD_DURATION,
        }
    }
}

/// Run statistics — accumulated during a run, displayed on end-of-run screen.
#[derive(Resource, Debug, Default)]
pub struct RunStats {
    pub enemies_killed: u32,
    pub total_damage_dealt: f64,
    pub total_damage_taken: f64,
    pub total_gold_earned: u32,
    pub total_gold_spent: u32,
    pub weapons_purchased: u32,
    pub upgrades_purchased: u32,
    pub total_rerolls: u32,
}

// Constants
pub const RUN_DURATION: f32 = 900.0; // 15 minutes
pub const GRACE_PERIOD_DURATION: f32 = 30.0;
