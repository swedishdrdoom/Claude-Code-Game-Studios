use bevy::prelude::*;

use crate::components::combat::{BossKilledEvent, TowerDestroyedEvent};
use crate::components::run::*;

pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<RunTimer>()
            .init_resource::<RunStats>()
            .add_message::<TowerDestroyedEvent>()
            .add_message::<BossKilledEvent>()
            .add_systems(Update, (
                update_grace_period.run_if(in_state(GameState::GracePeriod)),
                update_run_timer.run_if(in_state(GameState::Playing)),
                check_boss_trigger.run_if(in_state(GameState::Playing)),
                handle_tower_destroyed,
                handle_boss_killed.run_if(in_state(GameState::Boss)),
            ));
    }
}

fn update_grace_period(
    mut timer: ResMut<RunTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.grace_remaining -= time.delta_secs();
    if timer.grace_remaining <= 0.0 {
        timer.grace_remaining = 0.0;
        next_state.set(GameState::Playing);
        info!("Grace period ended — Playing!");
    }
}

fn update_run_timer(mut timer: ResMut<RunTimer>, time: Res<Time>) {
    timer.elapsed += time.delta_secs();
}

fn check_boss_trigger(
    timer: Res<RunTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if timer.elapsed >= RUN_DURATION {
        next_state.set(GameState::Boss);
        info!("Boss time! Timer stopped at {:.1}s", timer.elapsed);
    }
}

fn handle_tower_destroyed(
    mut events: MessageReader<TowerDestroyedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _event in events.read() {
        next_state.set(GameState::Defeat);
        info!("Tower destroyed — Defeat!");
    }
}

fn handle_boss_killed(
    mut events: MessageReader<BossKilledEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _event in events.read() {
        next_state.set(GameState::Victory);
        info!("Boss killed — Victory!");
    }
}
