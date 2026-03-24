# Run Manager

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Fifteen Minutes of Escalation

## Overview

The Run Manager owns the lifecycle of a single 15-minute run — from the moment
the player starts to the moment they win or die. It manages the game state
machine (menu, countdown, playing, boss, victory, defeat), tracks elapsed time,
and broadcasts state transitions so that every other system knows what phase the
run is in. Nine systems depend on it. The Run Manager does not control gameplay
directly — it provides the clock and the state that other systems react to.

## Player Fantasy

The player doesn't think about the Run Manager — they feel it as the mounting
pressure of the clock. The 15-minute structure creates a three-act arc in every
run: the scramble (Act 1), the build (Act 2), and the climax (Act 3 / boss).
The timer is a constant reminder that the run is finite, that every second of
indecision is a second wasted. This system is the metronome that makes "Fifteen
Minutes of Escalation" feel like a complete story with a beginning, middle,
and end.

## Detailed Design

### Core Rules

1. **Run Duration**: The standard run lasts 15 minutes (900 seconds) of active
   gameplay, measured from the end of the grace period to the boss spawn.

2. **Grace Period**: Each run begins with a grace period (default: 5 seconds)
   where the shop is active but no enemies spawn. This gives the player time to
   evaluate the first shop offering and buy an initial weapon.

3. **Run Timer**: A visible countdown/countup timer tracks elapsed time. The timer
   starts at 0:00 when the grace period ends and counts up to 15:00.

4. **Boss Trigger**: When the timer reaches 15:00, the Run Manager transitions
   to the Boss phase. The timer freezes. Enemy waves stop spawning (existing
   enemies remain). The boss spawns.

5. **Victory**: The player wins when the boss's HP reaches 0 during the Boss phase.
   Transition to Victory state.

6. **Defeat**: The player loses when the tower's HP reaches 0 at any point during
   the run (grace period, playing, or boss phase). Transition to Defeat state.

7. **Restart**: From Victory or Defeat, the player can immediately restart a new
   run. All runtime state is reset — tower stats, weapons, gold, enemies, timer.
   Content Database is not reloaded (it persists across runs).

8. **Pause**: The game can be paused at any time during the grace period, playing,
   or boss phases. All gameplay freezes. The shop is not interactable while paused.

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **MainMenu** | App launch or return from run | Player presses "Start" | No gameplay. Title screen displayed. |
| **GracePeriod** | Player starts a run | Grace period timer expires | Shop is active, tower is alive, no enemies spawn. Timer shows "Get Ready" or similar. |
| **Playing** | Grace period ends | Timer reaches 15:00 OR tower HP reaches 0 | Full gameplay. Enemies spawn, weapons fire, shop refreshes, gold flows. Timer counts up. |
| **Boss** | Timer reaches 15:00 | Boss HP reaches 0 OR tower HP reaches 0 | Timer frozen. No new enemy waves. Boss is active. Existing enemies remain. Shop still active. |
| **Victory** | Boss HP reaches 0 | Player chooses restart or menu | Gameplay frozen. Victory screen with run statistics. |
| **Defeat** | Tower HP reaches 0 (any phase) | Player chooses restart or menu | Gameplay frozen. Defeat screen with run statistics. |
| **Paused** | Player pauses (any active phase) | Player unpauses | All gameplay frozen. Overlay displayed. Returns to previous state on unpause. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Gold Economy** | Reads run state | Resets gold to starting amount on run start. Pauses income during Paused state. |
| **Enemy System** | Reads run state | Spawns enemies only during Playing state. Stops spawning during GracePeriod, Boss, Victory, Defeat. |
| **Wave Escalation** | Reads elapsed time | Uses `elapsed_time` (0.0–900.0) to determine current difficulty multiplier and wave composition. |
| **Boss Encounter** | Reads state transition | Listens for Playing → Boss transition to spawn the boss entity. |
| **HUD** | Reads elapsed time + state | Displays timer (countup during Playing, frozen during Boss). Shows state-specific UI (grace period text, boss alert). |
| **Run Statistics** | Reads state transitions | Records run start time, end time, and outcome (Victory/Defeat) for the end-of-run summary. |
| **End-of-Run Screen** | Reads state | Activates on Victory or Defeat transition. Displays run summary. |
| **Audio System** | Reads state + time | Triggers state-specific music (menu theme, combat music, boss music, victory/defeat stings). Uses elapsed time for adaptive music escalation. |
| **Main Menu** | Reads/writes state | Triggers MainMenu → GracePeriod transition. Receives Victory/Defeat → MainMenu transition. |
| **Shop System** | Reads run state | Shop is active during GracePeriod, Playing, and Boss. Inactive during Paused, Victory, Defeat. Resets inventory on run start. |

**Broadcast mechanism**: Run state transitions are implemented as Bevy `States`
(using the `States` derive macro). Systems use `in_state()` run conditions to
activate/deactivate based on the current state. Elapsed time is exposed as a
Bevy `Resource` (`Res<RunTimer>`).

## Formulas

### Run Timer

```
elapsed_time += delta_time  (only during Playing state)
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| elapsed_time | f32 | 0.0–900.0 | RunTimer resource | Seconds since grace period ended |
| delta_time | f32 | ~0.016 | Bevy Time resource | Frame delta (at 60fps) |

**Expected output range**: 0.0 to 900.0 seconds (15 minutes).

### Grace Period Timer

```
grace_remaining -= delta_time  (only during GracePeriod state)
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| grace_remaining | f32 | 0.0–5.0 | RunTimer resource | Seconds remaining in grace period |

No complex formulas. The Run Manager is a clock and state machine, not a
calculation engine.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Tower dies during grace period | Transition to Defeat. Grace period does not grant invulnerability. | Keep rules simple and consistent. |
| Tower dies at the exact frame the boss spawns | Defeat takes priority. Boss does not spawn. | The player lost — don't tease them with a boss they can't fight. |
| Boss HP reaches 0 on the same frame tower HP reaches 0 | Victory takes priority. The player killed the boss. | Reward the player's effort. Mutual destruction = player wins. |
| Player pauses during grace period | Grace period timer also pauses. Resumes on unpause. | Pausing must freeze everything. |
| Player alt-tabs / loses focus | Game auto-pauses (desktop standard). | Runs are timed — losing focus without pausing would be unfair. |
| Run timer precision: floating-point drift over 15 minutes | Timer is displayed as integer seconds (floor). Boss triggers at >= 900.0. Drift of <1ms over 15 min is acceptable. | Float32 has sufficient precision for a 900-second timer. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| (none) | Upstream | Run Manager has no upstream dependencies. |
| **Gold Economy** | Downstream (hard) | Needs run state for gold reset and income pausing |
| **Enemy System** | Downstream (hard) | Needs run state to know when to spawn |
| **Wave Escalation** | Downstream (hard) | Needs elapsed time for difficulty curve |
| **Boss Encounter** | Downstream (hard) | Needs Playing → Boss transition |
| **HUD** | Downstream (hard) | Needs elapsed time and state for timer display |
| **Shop System** | Downstream (hard) | Needs run state to know when shop is active |
| **Run Statistics** | Downstream (soft) | Records run timing and outcome |
| **End-of-Run Screen** | Downstream (soft) | Activates on Victory/Defeat |
| **Audio System** | Downstream (soft) | Uses state for music transitions |
| **Main Menu** | Downstream (soft) | Entry/exit point for runs |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Run duration | 900 seconds (15 min) | 300–1200 | Longer runs, more build time, more shop refreshes, higher risk of boredom | Shorter runs, less build depth, more frantic, higher replayability |
| Grace period | 5 seconds | 3–10 | More time to evaluate first shop, less pressure at start | Immediate pressure, no free shopping time |

**Knob interactions**: Run duration directly affects Wave Escalation curves,
Gold Economy totals, and shop refresh count (duration / 30s). Changing run
duration requires re-tuning all time-dependent systems.

## Visual/Audio Requirements

N/A — The Run Manager has no direct visual output. Timer display is owned by
the HUD. State-specific music is owned by the Audio System. The Run Manager
provides the data; other systems render it.

## UI Requirements

N/A — Timer display and state indicators are owned by the HUD system. The Run
Manager exposes `elapsed_time` and current state as readable resources.

## Acceptance Criteria

- [ ] Run timer counts from 0 to 900 seconds during Playing state
- [ ] Timer freezes when transitioning to Boss state
- [ ] Grace period lasts exactly the configured duration before transitioning to Playing
- [ ] No enemies spawn during GracePeriod
- [ ] Shop is active during GracePeriod, Playing, and Boss states
- [ ] Tower death during any active phase transitions to Defeat
- [ ] Boss death during Boss phase transitions to Victory
- [ ] Restart from Victory/Defeat resets all runtime state and begins a new GracePeriod
- [ ] Pause freezes all gameplay including timers
- [ ] Auto-pause triggers on window focus loss
- [ ] State transitions fire correctly for all downstream systems (verified by system integration tests)
- [ ] No frame-time dependency: timer advances by delta_time, not by frame count

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should there be a visual/audio warning before the boss spawns (e.g., at 14:00)? | Boss Encounter GDD | When Boss Encounter is designed | Depends on boss intro design |
| Should the grace period include a visible countdown overlay (3-2-1) or just a text prompt? | HUD GDD | When HUD is designed | UX decision |
