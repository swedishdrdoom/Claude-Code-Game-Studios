# Shop UI

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Greed Kills (the interface for greed)

## Overview

The Shop UI is the visual interface for the Shop System — the always-visible
panel where the player buys weapons and upgrades during combat. It renders a
2×4 grid of item cards (4 weapons, 4 upgrades) with a reroll button, refresh
timer, and affordability indicators. This is the most critical UI in the game:
it must be scannable in under 2 seconds, buyable in a single click, and never
obscure so much of the arena that the player can't see incoming threats. The
Shop UI is the highest-risk UI system in the project due to Bevy's immature
UI ecosystem.

## Player Fantasy

The shop should feel like a weapon dealer's table at a battlefield market.
Quick, dirty, immediate. The player should be able to glance at the shop,
identify the one item they want, click it, and return attention to the arena —
all in under 3 seconds. The shop should never feel like a menu; it should feel
like part of the battlefield.

## Detailed Design

### Core Rules

1. **Always Visible**: The shop panel is visible at all times during active
   gameplay (GracePeriod, Playing, Boss). It occupies a fixed portion of the
   screen — likely the right side or bottom.

2. **Layout**: 2 rows × 4 columns of item cards.
   - **Top row**: 4 weapon cards
   - **Bottom row**: 4 upgrade cards (including spikes upgrades)
   - **Reroll button**: Below or beside the grid
   - **Refresh timer**: Visible countdown to next auto-refresh

3. **Item Card Contents**: Each card displays:
   - Item icon (from Content Database image field)
   - Item name
   - Rarity indicator (color-coded border or background)
     - Common: Gray/White
     - Uncommon: Green
     - Rare: Blue
     - Epic: Purple
   - Gold cost (derived from rarity)
   - Brief stat summary or key info:
     - Weapons: Damage, DPS, attack pattern, range
     - Upgrades: Description text

4. **Affordability**: Items the player cannot afford are visually dimmed
   (reduced opacity, grayed out, or desaturated). The buy interaction is
   blocked for unaffordable items.

5. **Purchase Interaction**: Single click/tap on an affordable item card
   to buy it. No confirmation dialog — speed is critical. The card
   animates out and the slot becomes empty.

6. **Empty Slots**: After purchasing, the slot shows an empty/sold state
   until the next refresh or reroll.

7. **Reroll Button**: Shows current reroll cost and rerolls remaining
   this cycle (e.g., "Reroll: 250g (3/5)"). Dimmed when unaffordable
   or 0 rerolls remaining.

8. **Refresh Timer**: Visible countdown (seconds) to next auto-refresh.
   When it hits 0, all 8 slots repopulate with new items.

9. **Screen Real Estate**: The shop must not cover more than ~30% of the
   screen width/height. The arena, tower, and enemies must remain visible
   and readable alongside the shop.

10. **Tooltip/Hover**: Optional for MVP — hovering over a card could show
    expanded stats. But the card itself must contain enough info for a
    buy decision without hovering.

### States and Transitions

| State | Behavior |
|-------|----------|
| **Active** | Full interaction. Cards clickable. Reroll clickable. Timer ticking. |
| **Paused** | Visible but frozen. No clicks processed. Timer paused. |
| **Inactive** | Hidden or collapsed (MainMenu, Victory, Defeat). |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Shop System** | Reads | Gets current items, prices, affordability, reroll cost, timer, rerolls remaining |
| **Gold Economy** | Reads | Gets gold balance for affordability display |
| **Content Database** | Reads | Gets item icons, names, descriptions |
| **HUD** | Coordinates | Shares screen space — must not overlap HUD elements |
| **Weapon System** | Indirect | Purchases trigger weapon addition (via Shop System) |
| **Tower Entity** | Indirect | Purchases trigger upgrade application (via Shop System) |

## Formulas

No formulas — Shop UI is a display and interaction layer. All logic lives in
Shop System and Gold Economy.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Player clicks item during refresh animation | Click is blocked until refresh completes. | Prevent buying an item that's being replaced. |
| Player clicks empty slot | Nothing happens. | Empty slots are not interactive. |
| Item name is very long | Truncate with ellipsis. Full name visible on hover (if tooltip exists). | Cards must be fixed-size for grid layout. |
| Player rapidly clicks two different items | Both purchases process if gold is sufficient. | No artificial cooldown between purchases. Speed is valued. |
| Reroll button clicked with 0 rerolls remaining | Button is disabled/dimmed. Click does nothing. | Clear visual state prevents confusion. |
| Screen resolution very low | Cards may be small but must remain readable. Minimum card size enforced. | Support but don't optimize for extreme cases. |
| 60+ FPS with shop visible | Shop UI must not cause frame drops. Minimize layout recalculations. | UI performance is critical — shop is always on screen. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Shop System** | Upstream (hard) | All shop data and logic |
| **Gold Economy** | Upstream (hard) | Gold balance for affordability |
| **Content Database** | Upstream (soft) | Item icons and descriptions |
| **HUD** | Peer (soft) | Screen layout coordination |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Shop panel width (% of screen) | TBD (~25-30%) | 20–40% | More card detail visible; less arena visible | Less card info; more arena space |
| Card size | TBD | — | More info per card; fewer cards visible | Less info; more compact |
| Purchase animation duration | TBD | 0.1–0.5 seconds | Satisfying feel but slight delay | Snappier but less feedback |
| Rarity color scheme | Gray/Green/Blue/Purple | — | Standard genre convention; instantly recognizable | — |

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Item card hovered | Subtle highlight or scale-up | None | Medium |
| Item purchased | Card slides out / dissolves, gold number decreases | Cha-ching / purchase sound | High |
| Item unaffordable | Card dimmed, price in red | None | High |
| Reroll activated | All 8 cards flip/shuffle and resolve to new items | Dice roll / shuffle sound | High |
| Auto-refresh | Cards fade out and new cards fade in | Refresh swoosh | High |
| Timer low (last 5 seconds) | Timer pulses or changes color | Optional: tick sound | Low |
| Reroll button disabled | Button grayed out | Error buzz on click attempt | Medium |

## UI Requirements

This IS a UI system — see Core Rules for the full specification.

### Layout Specification

```
┌─────────────────────────────────────────────────┐
│                                                 │
│                                    ┌──────────┐ │
│                                    │ Weapon 1 │ │
│          ARENA                     │ Weapon 2 │ │
│       (game world)                 │ Weapon 3 │ │
│                                    │ Weapon 4 │ │
│                                    ├──────────┤ │
│                                    │ Upgrade1 │ │
│                                    │ Upgrade2 │ │
│                                    │ Upgrade3 │ │
│                                    │ Upgrade4 │ │
│                                    ├──────────┤ │
│                                    │ Reroll   │ │
│                                    │ Timer:23s│ │
│                                    └──────────┘ │
│  [HP BAR]  [GOLD: 3500]  [TIMER: 7:23]        │
└─────────────────────────────────────────────────┘
```

This is a rough layout suggestion. The exact layout will be determined during
UI prototyping. Key constraint: shop on the right, HUD on the bottom/top,
arena fills the remaining space.

## Acceptance Criteria

- [ ] Shop panel is always visible during active gameplay
- [ ] 4 weapon cards in top row, 4 upgrade cards in bottom row
- [ ] Each card shows icon, name, rarity color, and gold cost
- [ ] Unaffordable items are visually dimmed
- [ ] Single click purchases an affordable item
- [ ] Purchased slot shows empty/sold state
- [ ] Reroll button shows current cost and rerolls remaining
- [ ] Reroll button is disabled when unaffordable or 0 rerolls left
- [ ] Refresh timer counts down and is visible
- [ ] Shop does not obscure more than ~30% of the screen
- [ ] Shop is frozen but visible during Paused state
- [ ] No frame drops from shop UI rendering at 60fps
- [ ] Rarity colors are visually distinct and follow genre convention
- [ ] Purchase interaction takes < 0.5 seconds (click to confirmed)

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should the shop be on the right side, bottom, or configurable? | UX Designer | Before UI implementation | Right side is most natural for mouse-heavy interaction |
| Should cards show full weapon stats or just key info (name, damage, DPS)? | UX Designer | Before UI implementation | Balance between information density and readability |
| What Bevy UI approach: native Bevy UI, bevy_egui, or custom? | Architecture Decision | Before implementation | Bevy native UI is limited; bevy_egui is functional but not pretty; custom is most work |
| Should there be a weapon tooltip on hover with full stats? | UX Designer | Alpha | Adds info depth but requires hover state management |
| How should the shop adapt to different aspect ratios? | UX Designer | Before implementation | 16:9 is primary; ultrawide and 4:3 need consideration |
