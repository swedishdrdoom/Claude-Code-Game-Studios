# HUD

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Greed Kills (information enables faster decisions)

## Overview

The HUD displays all critical gameplay information — tower HP, gold balance,
run timer, and equipped weapons — overlaid on the game world in screen space.
It must be readable at a glance during combat without obscuring the arena. The
HUD shares screen real estate with the Shop UI (which is always visible). The
HUD is information-only; all interaction happens through the Shop UI.

## Player Fantasy

The HUD is the player's dashboard. A quick glance tells them everything they
need to make a shop decision: "How much HP do I have? How much gold? How long
until the boss?" The HUD should feel like a cockpit — clean, immediate, and
never in the way of the action.

## Detailed Design

### Core Rules

1. **Screen Space**: HUD renders in screen space via a separate UI camera,
   on top of the game world. Not affected by game camera zoom.

2. **Always Visible**: HUD elements are displayed during GracePeriod, Playing,
   and Boss states. Hidden during MainMenu. Dimmed but visible during Paused.

3. **Information Elements**:

   | Element | Data Source | Position | Format |
   |---------|-----------|----------|--------|
   | **HP Bar** | Tower Entity | Top-left or top-center | Bar with current/max HP text |
   | **Mana Shield Bar** | Tower Entity | Overlaid on or adjacent to HP bar | Blue/purple bar, shown only when > 0 |
   | **Armor Value** | Tower Entity | Near HP bar | Number + shield icon, shown only when > 0 |
   | **Gold Balance** | Gold Economy | Prominent, always visible | Gold coin icon + number |
   | **Gold Per Second** | Gold Economy | Near gold balance, small | "+X/sec" text, shown only when > 0 |
   | **Run Timer** | Run Manager | Top-center or top-right | MM:SS format, counting up |
   | **Equipped Weapons** | Weapon System | TBD — side panel or bottom | Weapon icons grouped by name with count |

4. **Minimal Design**: The HUD must not overwhelm the screen. The arena,
   enemies, projectiles, AND the shop UI all need space. The HUD should use
   the smallest effective footprint.

5. **Dynamic Visibility**: Some elements only appear when relevant:
   - Mana Shield bar: only when mana shield > 0
   - Armor value: only when armor > 0
   - Gold/sec: only when passive income > 0
   - HP regen indicator: optional, low priority

6. **Grace Period Overlay**: During GracePeriod, display "Get Ready" or a
   countdown overlay. Timer shows 0:00 during grace period.

7. **Boss Alert**: When transitioning to Boss state, display a brief "BOSS"
   alert. Timer freezes and may change color/style to indicate boss phase.

### States and Transitions

| State | Behavior |
|-------|----------|
| **GracePeriod** | Full HUD visible. Timer at 0:00. Grace period overlay text. |
| **Playing** | Full HUD visible. Timer counting up. |
| **Boss** | Full HUD visible. Timer frozen, boss indicator. |
| **Paused** | HUD dimmed but visible behind pause overlay. |
| **Victory/Defeat** | HUD hidden or minimal. End-of-run screen takes over. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Tower Entity** | Reads | HP, max HP, armor, mana shield, regen |
| **Gold Economy** | Reads | Current gold, gold per second |
| **Run Manager** | Reads | Elapsed time, current state |
| **Weapon System** | Reads | Equipped weapons list (names + counts) |
| **Shop UI** | Shares screen | Must not overlap with shop panel. Coordinate layout. |

## Formulas

No formulas — the HUD is purely a display system.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| HP bar reaches 0 | Bar shows empty. Defeat transition happens (Run Manager handles this). | HUD just displays; doesn't trigger game logic. |
| Gold exceeds display space (e.g., 999,999) | Truncate or use compact format (e.g., "999.9K"). | Large gold values from income stacking are possible. |
| Timer at 15:00 during boss phase | Timer freezes at 15:00. Display indicator that timer is frozen. | Player should know time has stopped. |
| Window very small | HUD scales or uses minimum readable size. Elements may overlap. | Support but don't optimize for tiny windows. |
| 20+ unique weapons equipped | Weapon display scrolls or uses compact view. | Theoretical max is high; display must handle it. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Tower Entity** | Upstream (hard) | HP, armor, mana shield data |
| **Gold Economy** | Upstream (hard) | Gold balance and income rate |
| **Run Manager** | Upstream (hard) | Timer and state |
| **Weapon System** | Upstream (soft) | Equipped weapons for display |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| HP bar size | TBD | — | Easier to read HP at a glance | More screen space for gameplay |
| HUD opacity | 100% | 50–100% | More visible; more obstructive | Less obstructive; harder to read |
| Weapon display grouping | By name + count | — | Compact display for duplicates | — |

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| HP drops | HP bar decreases, brief red flash | None (tower hit sound covers it) | High |
| HP regenerates | HP bar increases, subtle green | None | Low |
| Mana shield absorbs | Mana shield bar decreases, blue flash | None (shield sound covers it) | Medium |
| Gold changes | Gold number animates (count up/down) | None (purchase/kill sounds cover it) | Medium |
| Boss alert | Large "BOSS" text flashes on screen | Boss horn/alert sound | High |

## UI Requirements

This IS a UI system — see Core Rules section 3 for the full element table.

## Acceptance Criteria

- [ ] HP bar accurately reflects current/max HP in real-time
- [ ] Mana shield bar appears when mana shield > 0, hidden otherwise
- [ ] Armor value appears when armor > 0, hidden otherwise
- [ ] Gold balance updates immediately on earn/spend
- [ ] Gold/sec displayed when passive income > 0
- [ ] Timer counts up during Playing, freezes during Boss
- [ ] Timer displays in MM:SS format
- [ ] Equipped weapons displayed with duplicate grouping (name × count)
- [ ] HUD does not overlap Shop UI
- [ ] HUD renders in screen space (not affected by game camera)
- [ ] HUD visible during GracePeriod, Playing, Boss states
- [ ] HUD hidden or minimal during Victory/Defeat
- [ ] Boss alert displays on Playing → Boss transition

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Exact HUD layout (where does each element go on screen)? | UX Designer | Before UI implementation | Depends on Shop UI placement — must coordinate |
| Should equipped weapons show icons or just names + counts? | Art Director | Alpha | Icons are better but require all 88 weapon icons |
| Should HP regen rate be displayed? | UX Designer | Alpha | Adds info but may be too granular |
| Should there be a DPS indicator? | Game Designer | Vertical Slice | Useful for optimizer players but adds complexity |
