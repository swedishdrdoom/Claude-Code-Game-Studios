# Run Statistics

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story (the build's story told in numbers)

## Overview

Run Statistics tracks per-run metrics — total gold earned, enemies killed,
weapons purchased, damage dealt, time survived, and run outcome. It collects
data passively throughout the run and provides it to the End-of-Run Screen for
display. The system is a passive listener with no gameplay impact — it observes
events from other systems and records them.

## Player Fantasy

The end-of-run stats are the post-game recap. They tell the story of the run
in numbers: "I earned 45,000 gold, killed 1,200 enemies, bought 22 weapons,
and dealt 2.3 million damage." Stats validate the player's build and highlight
what worked (and what didn't). They're also the foundation for future features
like leaderboards and build sharing.

## Detailed Design

### Core Rules

1. **Tracking Period**: Statistics are collected from GracePeriod through
   Victory/Defeat. Reset on new run start.

2. **Tracked Metrics**:

   | Metric | Source | Description |
   |--------|--------|-------------|
   | Run Duration | Run Manager | Time survived (seconds) |
   | Run Outcome | Run Manager | Victory or Defeat |
   | Total Gold Earned | Gold Economy | Sum of all gold from bounties + income + instant |
   | Total Gold Spent | Gold Economy | Sum of all purchases + rerolls |
   | Total Rerolls | Gold Economy | Number of shop rerolls used |
   | Enemies Killed | Enemy System | Total enemy kill count |
   | Damage Dealt | Damage Calculation | Total damage dealt to enemies |
   | Damage Taken | Tower Entity | Total raw damage received by tower |
   | Weapons Purchased | Shop System | Count of weapons bought |
   | Upgrades Purchased | Shop System | Count of upgrades bought |
   | Final Build | Tower Entity | List of all weapons and upgrades at run end |
   | Boss Killed | Boss Encounter | Whether boss was defeated (Victory only) |

3. **Collection Method**: Run Statistics listens to events from other systems.
   It does not poll — it receives notifications.

4. **No Gameplay Impact**: Statistics do not affect gameplay. They are
   purely informational.

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Collecting** | Run starts | Run ends | Listening to events, accumulating totals |
| **Complete** | Run ends | New run starts | Data frozen, available for End-of-Run Screen |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Gold Economy** | Listens | Gold earned/spent events |
| **Enemy System** | Listens | Kill events |
| **Damage Calculation** | Listens | Damage dealt events |
| **Tower Entity** | Listens | Damage taken events |
| **Shop System** | Listens | Purchase events |
| **Run Manager** | Reads | Run duration, outcome |
| **Boss Encounter** | Listens | Boss kill event |
| **End-of-Run Screen** | Downstream | Provides all stats for display |

## Formulas

No formulas. Pure accumulation (sum, count, max).

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Run lasts 0 seconds (die during grace period) | All stats are 0 or minimal. Valid run. | Edge case but possible. |
| Extremely high values (millions of damage) | Display with abbreviations (1.2M, 45.3K). | Prevent text overflow. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Gold Economy** | Upstream (soft) | Gold events |
| **Enemy System** | Upstream (soft) | Kill events |
| **Weapon System** | Upstream (soft) | Build data |
| **Run Manager** | Upstream (hard) | Run timing and outcome |
| **End-of-Run Screen** | Downstream (hard) | Provides stats for display |

## Tuning Knobs

None — this system is pure data collection.

## Visual/Audio Requirements

N/A — Run Statistics has no direct visual output. Display is owned by End-of-Run Screen.

## UI Requirements

N/A — See End-of-Run Screen GDD.

## Acceptance Criteria

- [ ] All tracked metrics accumulate correctly during a run
- [ ] Stats reset on new run start
- [ ] Stats are available to End-of-Run Screen after run ends
- [ ] Large values display with readable formatting
- [ ] Final build snapshot captures all weapons and upgrades

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should per-weapon DPS be tracked for the stats screen? | Game Designer | Alpha | Useful but adds tracking complexity |
| Should historical run stats be persisted across sessions? | Game Designer | Post-demo | Would enable lifetime statistics and leaderboards |
