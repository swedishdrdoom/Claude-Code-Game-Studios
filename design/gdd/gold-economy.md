# Gold Economy

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Greed Kills

## Overview

The Gold Economy is the resource system that drives all player decisions.
Gold is earned from killing enemies (bounty) and from passive income
(gold per second from upgrades). Gold is spent in the shop on weapons
and upgrades. The economy is the core of the "Greed Kills" pillar —
every gold piece spent is a choice between getting stronger and staying
alive. The economy resets completely each run.

## Player Fantasy

Gold is power and gold is temptation. The number ticking up in the corner
is a constant promise: "You could buy something amazing right now." But
spending means committing, and the shop might offer something better in
30 seconds. The economy creates a poker-like tension — do you go all-in
on this Rare weapon, or hold for the next refresh? When you buy a Cursed
Treasure (-100 HP regen, +5000 gold) and immediately spend it on two Epic
upgrades, you're gambling your life for power. That's the fantasy.

## Detailed Design

### Core Rules

1. **Starting Gold**: Each run begins with 5,000 gold.

2. **Gold Sources**:
   - **Kill Bounty**: Base 10 gold per enemy killed. Modified by kill
     bounty % upgrades (Bounty Hunter: +50%, Transmute: +100%, Golden
     Ring: +200%).
   - **Passive Income**: 0 gold/sec base. Gained entirely from upgrades
     (Magic Coin: +5/sec, Gold Mine: +10/sec +10%/sec, Entangled Gold
     Mine: +10/sec +1/sec every 30 seconds).
   - **Instant Gold**: One-time gold injections from risk/reward upgrades
     (Philosopher's Stone: -1000 HP, +2000 gold; Cursed Treasure: -100
     HP regen, +5000 gold).

3. **Gold Sinks**:
   - **Item Purchases**: Rarity-based pricing (Common: 500, Uncommon:
     2,000, Rare: 5,000, Epic: 10,000).
   - **Shop Rerolls**: Reroll the current shop offerings. Cost escalates
     persistently throughout the entire run (see Reroll rules below).

4. **No Gold Cap**: Gold can accumulate without limit.

5. **No Debt**: The player cannot spend gold they don't have. If gold
   < item price or reroll cost, the action is blocked.

6. **Run Reset**: Gold resets to 5,000 starting gold on each new run.
   All income upgrades, bounty bonuses, and reroll cost reset.

### Reroll Rules

1. **Starting Cost**: First reroll of the run costs 100 gold.

2. **Escalation**: Each reroll adds 150 gold to the cost. The cost
   escalates persistently across the entire run — it never resets.
   ```
   reroll_cost = 100 + (total_rerolls_this_run * 150)
   ```

   | Reroll # | Cost | Cumulative Spent |
   |----------|------|-----------------|
   | 1 | 100 | 100 |
   | 2 | 250 | 350 |
   | 3 | 400 | 750 |
   | 4 | 550 | 1,300 |
   | 5 | 700 | 2,000 |
   | 6 | 850 | 2,850 |
   | 7 | 1,000 | 3,850 |
   | 10 | 1,450 | 7,750 |
   | 20 | 2,950 | 30,500 |

3. **Max 5 Rerolls Per Shop Cycle**: Within a single 30-second shop cycle,
   the player can reroll a maximum of 5 times. The counter resets when the
   shop auto-refreshes. The cost does NOT reset — only the per-cycle limit.

4. **Reroll Does Not Reset Timer**: Rerolling the shop does NOT restart the
   30-second refresh timer. The timer keeps counting down to the next
   automatic refresh regardless of rerolls.

### Kill Bounty Calculation

```
bounty = base_bounty * (1 + total_bounty_bonus_percent)
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| base_bounty | int | 10+ | Enemy data (default: 10) | Gold per kill before bonuses |
| total_bounty_bonus_percent | f32 | 0.0–5.0+ | Sum of upgrade bonuses | Additive stack of all bounty % upgrades |

**Example**: With Bounty Hunter (+50%) and Transmute (+100%):
- total_bounty_bonus_percent = 1.5
- bounty = 10 * (1 + 1.5) = 25 gold per kill

**Transmute special**: 10% chance to receive an additional +100% bounty on a kill.
When triggered: bounty = 10 * (1 + 1.5 + 1.0) = 35 gold for that kill.

### Passive Income Calculation

```
gold += total_gold_per_second * delta_time
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| total_gold_per_second | f32 | 0.0–100+ | Sum of upgrade effects | All gold/sec sources combined |

Passive income starts at 0 and is built entirely from upgrades. Income
pauses during Paused state.

**Scaling income upgrades** (tick every 30 seconds):
- Entangled Gold Mine: +1 gold/sec every 30 seconds (permanently)
- These scaling effects stack with each copy purchased

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Active** | Run starts (GracePeriod) | Run ends (Victory/Defeat) | Gold earned, spent, tracked. Income ticks. |
| **Paused** | Game paused | Game unpaused | Income paused. No spending (shop inactive). |
| **Inactive** | Run ends | New run starts | No gold activity. Gold value displayed on end-of-run screen. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Run Manager** | Reads state | Resets gold on run start. Pauses income during Paused. |
| **Shop System** | Bidirectional | Shop reads gold balance for affordability checks. Shop deducts gold on purchase and reroll. |
| **Enemy System** | Receives kill events | Awards bounty gold when an enemy dies. |
| **Tower Entity** | Reads upgrade effects | Instant gold upgrades (Philosopher's Stone, Cursed Treasure) add gold directly. |
| **HUD** | Exposes gold balance | HUD displays current gold. |
| **Shop UI** | Exposes gold + reroll cost | Shop UI shows current gold, reroll cost, and affordability per item. |
| **Run Statistics** | Exposes totals | Tracks total gold earned, total gold spent, total rerolls for end-of-run summary. |

## Formulas

### Kill Bounty

```
bounty = base_bounty * (1 + total_bounty_bonus_percent)
```

See Kill Bounty Calculation above for full details and examples.

### Passive Income Per Frame

```
gold += total_gold_per_second * delta_time
```

### Reroll Cost

```
reroll_cost = 100 + (total_rerolls_this_run * 150)
```

### Gold Per Second (with % bonus)

```
total_gold_per_second = base_gold_per_second * (1 + gold_per_second_bonus_percent)
```

Gold Mine grants both flat (+10/sec) and percentage (+10%/sec) income.
The percentage applies to the total flat income from all sources.

**Example**: 2× Magic Coin (+5/sec each) + 1× Gold Mine (+10/sec, +10%/sec):
- base_gold_per_second = 5 + 5 + 10 = 20
- total_gold_per_second = 20 * (1 + 0.10) = 22 gold/sec

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Gold goes to 0 exactly | Valid state. Player must wait for income or kills. Cannot buy or reroll. | No debt allowed. |
| Player tries to buy item costing more than current gold | Purchase blocked. UI shows item as unaffordable. | No debt. |
| Player tries to reroll with insufficient gold | Reroll blocked. Button grayed out. | No debt. |
| Player uses all 5 rerolls in a cycle, then shop auto-refreshes | Reroll-per-cycle counter resets to 0. Player can reroll 5 more times (at continued escalating cost). | Per-cycle limit resets, persistent cost does not. |
| Kill bounty with many stacked % bonuses | All bounty % bonuses stack additively, then multiply base bounty once. No multiplicative stacking between upgrades. | Additive stacking is predictable and prevents runaway scaling. |
| Passive income during grace period | Income ticks during grace period (if any income upgrades exist from starting gold purchases). | Grace period is part of the run; income should flow. |
| Philosopher's Stone purchased when tower has < 1000 HP | Tower loses 1000 max HP (may die), gold is awarded. The gold gain and HP loss happen simultaneously. | Risk/reward is the design. Buying this at low HP can kill you. |
| Transmute 10% proc: does it proc per hit or per kill? | Per kill. Each enemy death has a 10% chance for the bonus bounty. | Per-kill is clearer and doesn't scale with attack speed. |
| Gold display: floating point rounding | Display gold as integer (floor). Internal tracking uses float for income ticks. | Players see whole numbers. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Run Manager** | Upstream (hard) | Needs run state for reset, pause, and active tracking |
| **Shop System** | Downstream (hard) | Reads gold for purchases and rerolls |
| **Enemy System** | Downstream (hard) | Awards bounty on kill |
| **HUD** | Downstream (soft) | Displays gold balance |
| **Shop UI** | Downstream (soft) | Shows affordability and reroll cost |
| **Run Statistics** | Downstream (soft) | Records total gold earned/spent |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Starting gold | 5,000 | 1,000–10,000 | More early purchases; less early pressure; more build options at start | Tighter opening; first kill bounties matter more; fewer starting options |
| Base kill bounty | 10 | 5–50 | Faster gold income from kills; economy scales faster with wave density | Slower kill-based income; passive income upgrades become more important |
| Reroll starting cost | 100 | 50–500 | First reroll is cheap; encourages rerolling early | First reroll is expensive; discourages casual rerolling |
| Reroll cost increment | 150 | 50–300 | Rerolls become expensive quickly; less fishing for specific items | Rerolls stay cheap longer; more build-around strategies viable |
| Max rerolls per cycle | 5 | 3–10 | More chances to find desired items per cycle | Fewer chances; must be more accepting of what the shop offers |

**Knob interactions**: Starting gold divided by item prices determines how
many items a player can buy before needing income. At 5,000 starting gold:
1 Rare, or 2 Uncommons + 2 Commons, or 10 Commons. Base bounty × enemy
density determines kill-based income rate — this interacts directly with
Wave Escalation's spawn rate.

**Economy target**: By minute 7-8, a player with moderate income investment
should have enough gold/sec that reroll costs feel manageable but not free.
A player who invested nothing in income should feel gold-starved.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Gold earned (kill bounty) | Gold number popup at enemy death location | Coin clink sound | High |
| Gold earned (passive income) | Subtle gold counter increment animation | None (too frequent) | Low |
| Gold spent (purchase) | Gold counter decreases with animation | Purchase/cash register sound | High |
| Gold spent (reroll) | Gold counter decreases | Dice roll / shuffle sound | Medium |
| Insufficient gold | Item/reroll button flashes red briefly | Error buzz | Medium |

## UI Requirements

| Information | Display Location | Update Frequency | Condition |
|-------------|-----------------|-----------------|-----------|
| Current gold balance | HUD — prominent, always visible | Every frame | Always during run |
| Reroll cost | Shop UI — on reroll button | On reroll or shop refresh | When shop is visible |
| Rerolls remaining this cycle | Shop UI — near reroll button | On reroll or shop refresh | When shop is visible |
| Item affordability | Shop UI — per item card | On gold change | When shop is visible |
| Gold per second | HUD — small indicator near gold balance | On change | When gold/sec > 0 |

## Acceptance Criteria

- [ ] Run starts with 5,000 gold
- [ ] Killing an enemy awards 10 base bounty gold
- [ ] Kill bounty scales correctly with bounty % upgrades (additive stacking)
- [ ] Passive income of 0/sec at start; increases with income upgrades
- [ ] Gold/sec pauses during Paused state
- [ ] Reroll costs 100 for the first reroll of the run
- [ ] Each subsequent reroll costs 150 more than the previous
- [ ] Reroll cost persists across shop cycles (never resets within a run)
- [ ] Maximum 5 rerolls per 30-second shop cycle
- [ ] Reroll does not reset the 30-second shop refresh timer
- [ ] Cannot purchase items or reroll when gold is insufficient
- [ ] Gold resets to 5,000 on run restart
- [ ] Philosopher's Stone awards +2000 gold and costs -1000 max HP simultaneously
- [ ] Gold displayed as integer (floor of internal float value)
- [ ] Transmute 10% bonus proc is per kill, not per hit

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Do different enemy types have different base bounties, or all 10? | Enemy Data GDD | When specific enemies are designed | Enemy Data schema supports per-enemy GoldBounty field |
| Should gold/sec be shown as a number in the HUD, or is it too much info? | HUD GDD | When HUD is designed | UX decision — may clutter the screen |
| Does the 30-second shop cycle timer reset on run start, or does the first shop appear immediately? | Shop System GDD | When Shop System is designed | First shop should be available during grace period |
