# Shop System

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Greed Kills

## Overview

The Shop System is the player's only way to acquire weapons and upgrades. It
displays 8 items in a 2×4 grid — 4 weapons on the top row, 4 upgrades
(including spikes) on the bottom row. The shop is always visible during
gameplay and auto-refreshes every 30 seconds. Players can buy items with gold
or reroll the offerings (up to 5 times per cycle, with persistently escalating
cost). The shop is active during GracePeriod, Playing, and Boss states. This
system is the mechanical core of "Greed Kills" — every purchase is a live
decision under combat pressure.

## Player Fantasy

The shop is the devil on your shoulder. It's always there, always tempting,
always showing you something you want but can't quite afford. The 30-second
refresh timer creates urgency — "Do I buy this now or wait for something
better?" The reroll button is the slot machine pull — each reroll might
reveal the perfect weapon for your build, but it costs gold you could spend
on something safe. When the shop shows triple Frost Bombs on a reroll and
you have exactly enough gold, that's the moment.

## Detailed Design

### Core Rules

1. **Layout**: 2 rows × 4 columns.
   - **Top row (4 slots)**: Weapons — any weapon from the weapon database.
     Does NOT include spikes-type items.
   - **Bottom row (4 slots)**: Upgrades — any upgrade from the upgrade
     database, INCLUDING spikes damage upgrades.
   - **Reroll button**: One button to reroll all 8 slots simultaneously.

2. **Refresh Cycle**: The shop auto-refreshes every 30 seconds during active
   gameplay. The refresh timer:
   - Starts when the run begins (GracePeriod)
   - Counts down continuously during GracePeriod, Playing, and Boss states
   - Pauses during Paused state
   - Does NOT reset on reroll — rerolling replaces items but doesn't restart
     the 30-second timer

3. **Initial Shop**: The first shop offering appears immediately at run start
   (beginning of GracePeriod). The 30-second timer begins counting from this
   point.

4. **Item Generation**: When the shop refreshes (auto or reroll), each slot
   independently rolls a new item:
   - Roll rarity first (weighted random)
   - Filter available items by rarity and slot type (weapon or upgrade)
   - Pick a random item from the filtered pool
   - An item can appear multiple times in the same shop (duplicates allowed)

5. **Rarity Weights**: Each slot rolls rarity independently:

   | Rarity | Weight | Approximate % |
   |--------|--------|--------------|
   | Common | TBD | ~40% |
   | Uncommon | TBD | ~35% |
   | Rare | TBD | ~20% |
   | Epic | TBD | ~5% |

   Exact weights TBD during balancing. These weights are fixed throughout
   the run — rarity does not scale with time.

6. **Purchasing**:
   - Player clicks an item to buy it
   - Gold cost = rarity price (Common: 500, Uncommon: 2000, Rare: 5000,
     Epic: 10000)
   - If player has sufficient gold, deduct gold and apply the item:
     - Weapons: Add a new weapon instance to the tower's weapon list
     - Upgrades: Apply the upgrade effect to the tower's stats
   - The purchased slot becomes empty for the remainder of the cycle
   - Empty slots are not refilled until the next refresh/reroll

7. **Rerolling**:
   - Reroll replaces ALL 8 slots (including empty ones from purchases)
     with new random items
   - Cost: 100 + (total_rerolls_this_run × 150) — see Gold Economy GDD
   - Maximum 5 rerolls per 30-second shop cycle
   - Reroll counter per cycle resets on auto-refresh
   - Reroll cost counter is persistent across the entire run
   - Rerolling does NOT reset the 30-second refresh timer

8. **Purchased Items**: Once bought, items are permanent for the run.
   Weapons and upgrades cannot be sold, refunded, or swapped.

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Active** | GracePeriod begins | Run ends (Victory/Defeat) | Shop displays items. Purchases and rerolls enabled. Timer ticking. |
| **Paused** | Game paused | Game unpaused | Shop visible but frozen. No purchases, no rerolls, timer paused. |
| **Inactive** | Victory/Defeat/MainMenu | New run starts | Shop not displayed or grayed out. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Content Database** | Reads | Queries weapon and upgrade pools for item generation |
| **Gold Economy** | Bidirectional | Checks gold balance for affordability. Deducts gold on purchase and reroll. Reads reroll cost. |
| **Weapon System** | Writes | Adds weapon instances to tower's weapon list on purchase |
| **Tower Entity** | Writes | Applies upgrade effects to tower stats on purchase |
| **Run Manager** | Reads | Run state determines shop active/paused/inactive. Elapsed time for refresh timer. |
| **Shop UI** | Provides data | Shop System provides current items, prices, affordability, reroll cost, timer, rerolls remaining to Shop UI for rendering |

## Formulas

### Rarity Roll

```
roll = random(0, total_weight)
if roll < common_weight: rarity = Common
elif roll < common_weight + uncommon_weight: rarity = Uncommon
elif roll < common_weight + uncommon_weight + rare_weight: rarity = Rare
else: rarity = Epic
```

### Refresh Timer

```
refresh_timer -= delta_time  (during active states only)
if refresh_timer <= 0:
    regenerate_all_slots()
    refresh_timer = 30.0
    rerolls_this_cycle = 0
```

### Reroll Cost

```
reroll_cost = 100 + (total_rerolls_this_run * 150)
```

(Defined in Gold Economy GDD. Shop System reads this value.)

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| All 8 items purchased before refresh | Shop shows 8 empty slots until refresh or reroll. | Buying everything is valid. Reroll refills all slots. |
| Player buys item then rerolls | Empty slot from purchase is refilled along with all other slots. | Reroll replaces everything. |
| Reroll shows the exact same items | Possible (random). Each slot rolls independently. | True random, not shuffle. Duplicates happen. |
| Same weapon appears in multiple weapon slots | Valid. Player can buy both — adds two instances. | No restriction on duplicates in the shop. |
| Player rerolls 5 times, then auto-refresh happens | Per-cycle reroll counter resets to 0. Player can reroll 5 more times. Cost continues escalating. | Per-cycle limit resets, persistent cost does not. |
| Shop refreshes during boss phase | Yes — shop is active during Boss. New items appear. Player can still buy weapons/upgrades during the boss fight. | "Greed Kills" extends to the boss phase. |
| Gold exactly equals item cost | Purchase succeeds. Gold goes to 0. | Exact match is valid. |
| Player spams buy button | Each click processes one purchase. If gold is insufficient after first buy, subsequent clicks are blocked. | No double-purchasing from fast clicks. |
| Black Market upgrade purchased | Opens a special selection UI (Uncommon weapon or spikes upgrade of player's choosing). Persists until a choice is made. | Special upgrade with custom behavior — deferred to Vertical Slice implementation. |
| Duplicator upgrade purchased | Grants +1 copy of the next Rare weapon or spikes upgrade purchased. | Tracks a "next purchase" modifier — deferred to Vertical Slice implementation. |
| Multiplication Gems purchased | Grants 3 extra copies of the next Common (500 gold) upgrade purchased. | Tracks a "next purchase" modifier — deferred to Vertical Slice implementation. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Content Database** | Upstream (hard) | Item pools for generation |
| **Gold Economy** | Upstream (hard) | Gold balance and reroll cost |
| **Weapon System** | Downstream (hard) | Adds weapons on purchase |
| **Tower Entity** | Downstream (hard) | Applies upgrades on purchase |
| **Run Manager** | Upstream (hard) | Run state and timing |
| **Shop UI** | Downstream (hard) | Renders shop data |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Weapon slots | 4 | 3–6 | More weapon options per refresh; higher chance of finding desired weapon | Fewer options; rerolls more important |
| Upgrade slots | 4 | 3–6 | More upgrade options per refresh | Fewer options |
| Refresh interval | 30 seconds | 15–60 | More time to evaluate; fewer decision points per run | More frequent decisions; faster pace |
| Common rarity weight | TBD (~40%) | 20–60% | More cheap options; faster early scaling | Fewer commons; economy slower early |
| Uncommon rarity weight | TBD (~35%) | 20–50% | More mid-tier options | Fewer uncommons |
| Rare rarity weight | TBD (~20%) | 5–30% | Rares appear more often; builds specialize faster | Rares are rarer; rerolls more important for finding them |
| Epic rarity weight | TBD (~5%) | 1–15% | Epics appear regularly; powerful builds form sooner | Epics are very rare; seeing one feels special |
| Max rerolls per cycle | 5 | 3–10 | More chances to find desired items | Must accept what's offered more often |

**Knob interactions**: Rarity weights × item prices × gold income determine
how quickly the player can build power. Refresh interval × max rerolls
determines total item exposure per run (at 30s with 5 rerolls = up to 6
sets of 8 items per cycle = 48 items seen per 30-second window).

**Key ratio**: At 30-second refresh with a 15-minute run, the player sees
30 auto-refreshes. With up to 5 rerolls each, the theoretical maximum
is 180 shop views × 8 items = 1,440 items seen. In practice, players
will reroll selectively — maybe 2-3 times per cycle on average.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Shop refreshes (auto) | Items animate in / slide in | Refresh swoosh sound | High |
| Shop rerolled | Items shuffle / spin and resolve | Dice roll / slot machine sound | High |
| Item purchased | Item slides out of slot, slot empties | Purchase cha-ching sound | High |
| Item unaffordable | Item card dimmed / grayed out | None (visual only) | Medium |
| Reroll button unaffordable | Button dimmed / grayed out | None | Medium |
| Refresh timer near 0 | Timer pulses or flashes | Optional: tick sound in last 5 seconds | Low |

## UI Requirements

| Information | Display Location | Update Frequency | Condition |
|-------------|-----------------|-----------------|-----------|
| 4 weapon items | Shop panel — top row | On refresh/reroll/purchase | Always during active run |
| 4 upgrade items | Shop panel — bottom row | On refresh/reroll/purchase | Always during active run |
| Item name, icon, rarity, price | Per item card | Static per item | Always |
| Affordability indicator | Per item card (dim if unaffordable) | On gold change | Always |
| Reroll button + cost | Shop panel | On reroll | Always |
| Rerolls remaining this cycle | Near reroll button | On reroll/refresh | Always |
| Refresh timer | Shop panel | Every frame | Always |

(Detailed UI layout and interaction design owned by Shop UI GDD.)

## Acceptance Criteria

- [ ] Shop displays 4 weapons (top row) and 4 upgrades (bottom row)
- [ ] Spikes upgrades appear in the upgrade row, not the weapon row
- [ ] Shop auto-refreshes every 30 seconds during active gameplay
- [ ] First shop appears immediately at run start (GracePeriod)
- [ ] Refresh timer pauses during Paused state
- [ ] Purchasing deducts correct gold amount (rarity-based pricing)
- [ ] Purchased weapon is added to tower's weapon list as independent instance
- [ ] Purchased upgrade is applied to tower stats immediately
- [ ] Purchased slot becomes empty until next refresh/reroll
- [ ] Reroll replaces all 8 slots (including empty ones)
- [ ] Reroll cost follows Gold Economy formula (100 + N×150)
- [ ] Maximum 5 rerolls per 30-second cycle
- [ ] Per-cycle reroll counter resets on auto-refresh
- [ ] Reroll does not reset the 30-second refresh timer
- [ ] Items cannot be purchased if gold is insufficient
- [ ] Reroll is blocked if gold is insufficient or 5 rerolls used this cycle
- [ ] Shop is active during GracePeriod, Playing, and Boss states
- [ ] Shop is frozen during Paused state
- [ ] Duplicate items can appear in the same shop offering
- [ ] All item generation uses rarity weights from config (no hardcoded probabilities)

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| What are the exact rarity weights? | Systems Designer | During balancing | Requires playtesting to tune item availability curve |
| Should rarity weights change over time (e.g., more Rares later in the run)? | Game Designer | Vertical Slice | Could add temporal progression to item quality |
| How do special upgrades (Black Market, Duplicator, Multiplication Gems) interact with the shop? | Shop System v2 | Vertical Slice | These have unique buy flows that need custom UI |
| Should bought items leave a "sold" indicator or just an empty slot? | Shop UI GDD | When Shop UI is designed | UX decision |
| Can the player see what's coming on the next refresh? | Game Designer | Vertical Slice | A preview mechanic could add strategy but also complexity |
