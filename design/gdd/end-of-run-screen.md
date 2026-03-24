# End-of-Run Screen

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story

## Overview

The End-of-Run Screen displays after every run (Victory or Defeat) and shows
the player's run statistics, final build, and outcome. It provides a moment of
reflection — "what did I build and how did it perform?" — before the player
restarts or returns to the menu. The screen also serves as the "one more run"
hook: seeing your stats makes you want to beat them.

## Player Fantasy

The end-of-run screen is the scoreboard. It tells you whether your build was
great or terrible, in numbers. "1,200 enemies killed with 2.3M total damage"
feels like a victory lap. "Died at 8:23 with only 3 weapons" tells you what
went wrong. The screen should make the player think "I could do better" and
hit restart.

## Detailed Design

### Core Rules

1. **Trigger**: Appears when Run Manager transitions to Victory or Defeat.

2. **Display Duration**: Stays on screen until the player chooses an action
   (restart or return to menu). No auto-dismiss.

3. **Outcome Header**: Large "VICTORY" or "DEFEAT" text with appropriate
   visual treatment (green/gold for victory, red for defeat).

4. **Statistics Display**: Shows key metrics from Run Statistics:
   - Time survived (MM:SS)
   - Enemies killed
   - Total damage dealt
   - Total gold earned
   - Total gold spent
   - Weapons purchased
   - Upgrades purchased
   - Rerolls used

5. **Final Build Display**: Shows the player's complete build at run end:
   - All weapons (grouped by name with count)
   - All upgrades applied
   - Key tower stats at time of death/victory (HP, armor, mana shield, etc.)

6. **Actions**: Two buttons:
   - **Restart**: Immediately starts a new run (GracePeriod)
   - **Main Menu**: Returns to MainMenu state

7. **Gameplay Frozen**: During the end-of-run screen, all gameplay is frozen.
   No enemies move, no projectiles fly, no timers tick. The arena is visible
   behind the screen (dimmed overlay).

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Visible** | Victory or Defeat | Player clicks Restart or Main Menu | Stats displayed, actions available |
| **Hidden** | Any other state | Victory or Defeat | Not rendered |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Run Statistics** | Reads | Gets all tracked metrics for display |
| **Run Manager** | Reads/Writes | Reads outcome (Victory/Defeat). Writes state transition on Restart/Main Menu. |
| **Tower Entity** | Reads | Final tower stats for build display |

## Formulas

No formulas. Display only.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Victory and Defeat on same frame | Victory screen shown (per Run Manager edge case rules). | Consistent with Run Manager decision. |
| Player presses Restart immediately | New run starts. No minimum display time. | Respect the player's time. |
| Very long stat values | Abbreviate (1.2M, 45.3K). | Prevent layout overflow. |
| 0 weapons purchased (died immediately) | Display "0 weapons" with empty build section. | Valid but sad run. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Run Statistics** | Upstream (hard) | Run data to display |
| **Run Manager** | Upstream (hard) | Outcome and state transitions |
| **Tower Entity** | Upstream (soft) | Final build snapshot |

## Tuning Knobs

None — this is a display system.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Victory screen appears | Gold/green overlay, "VICTORY" text, celebratory particles | Victory fanfare / jingle | High |
| Defeat screen appears | Red/dark overlay, "DEFEAT" text | Defeat sting / somber tone | High |
| Stat numbers count up | Numbers animate from 0 to final value | Subtle tick sound during count | Medium |
| Restart button hovered | Button highlight | None | Low |

## UI Requirements

### Layout

```
┌──────────────────────────────┐
│         V I C T O R Y        │
│                              │
│  Time:     14:23             │
│  Enemies:  1,247             │
│  Damage:   2.3M              │
│  Gold:     45,200 earned     │
│            38,500 spent      │
│  Weapons:  18 purchased      │
│  Upgrades: 12 purchased      │
│  Rerolls:  23 used           │
│                              │
│  ─── Final Build ───         │
│  Frost Bow ×5                │
│  Mortar Launcher ×3          │
│  Fire Bow ×2  ...            │
│                              │
│  [RESTART]    [MAIN MENU]    │
└──────────────────────────────┘
```

## Acceptance Criteria

- [ ] Screen appears on Victory with "VICTORY" header
- [ ] Screen appears on Defeat with "DEFEAT" header
- [ ] All Run Statistics metrics are displayed correctly
- [ ] Final build (weapons + upgrades) is displayed
- [ ] Restart button starts a new run immediately
- [ ] Main Menu button returns to MainMenu state
- [ ] Gameplay is frozen behind the screen
- [ ] Screen does not auto-dismiss — waits for player input
- [ ] Large numbers are abbreviated (K, M)

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should there be a "death replay" showing what killed you? | Game Designer | Post-demo | Cool feature but significant scope |
| Should stats compare to previous best? | Game Designer | Post-demo | Requires persistent stats storage |
| Should the build be shareable (screenshot, code)? | Game Designer | Post-demo | Great for community engagement |
