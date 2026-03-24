# Wave Escalation

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Fifteen Minutes of Escalation

## Overview

Wave Escalation controls the enemy pressure curve over a 15-minute run. It
determines how many enemies spawn, how fast, and how tough they are at any
given moment. Spawning is continuous — no discrete waves, no breaks. All
non-boss armor types (Unarmored, Light, Medium, Heavy, Fortified) are present
from the start. Difficulty scales through increasing spawn density and enemy
stat multipliers, with a dramatic spike at the 10-minute mark. A special gold
cart event occurs at the 6-minute mark.

## Player Fantasy

The escalation is invisible but felt. The first minute feels manageable — a
handful of enemies walking in from all sides. By minute 5, the player is
buying weapons between glancing at the growing horde. By minute 10, the screen
is dense with enemies and every purchase feels urgent. By minute 14, it's
pure survival — the build either works or it doesn't. The 15-minute arc is
a complete story of escalating tension.

## Detailed Design

### Core Rules

1. **Continuous Spawning**: Enemies spawn continuously throughout the Playing
   state. There are no discrete waves, no pauses, no breaks. The spawn rate
   increases smoothly over time.

2. **All Armor Types From Start**: Unarmored, Light, Medium, Heavy, and
   Fortified enemies all spawn from minute 0. The mix is randomized — each
   spawn picks a random armor type weighted by the composition table. There
   is no scripted introduction of new types.

3. **Armor Type Composition**: The ratio of armor types in the spawn pool
   is fixed throughout the run (not time-dependent):

   | Armor Type | Spawn Weight | Design Role |
   |------------|-------------|-------------|
   | Unarmored | TBD | Fodder — easy kills, gold income |
   | Light | TBD | Swarmers — fast, fragile |
   | Medium | TBD | Baseline — balanced stats |
   | Heavy | TBD | Tanks — slow, beefy |
   | Fortified | TBD | Elites — rare but very tough |

   Weights should ensure a mix where no single damage type dominates.
   Exact weights determined during balancing.

4. **Scaling Phases**: The run has two scaling phases:

   | Phase | Time | Behavior |
   |-------|------|----------|
   | **Ramp** | 0:00–10:00 | Smooth, gradual increase in spawn rate and enemy HP/damage multiplier |
   | **Spike** | 10:00–15:00 | Greatly accelerated scaling. Spawn rate and stats increase much faster. Tests whether the build can handle endgame density. |

5. **Scaling Parameters**: Wave Escalation controls these values as functions
   of elapsed time:
   - **Spawn rate**: Enemies per second (increases over time)
   - **HP multiplier**: Multiplied against base enemy HP (increases over time)
   - **Damage multiplier**: Multiplied against base enemy damage (increases over time)
   - **Move speed multiplier**: Optionally increases slightly (TBD)

6. **Gold Cart Event** (6:00):
   At the 6-minute mark, a special gold cart wave spawns:
   - Gold carts are a unique enemy type with Unarmored armor
   - They do NOT attack the tower
   - They walk to the tower and sit for up to 5 seconds
   - If killed within 5 seconds of arriving, they award 250 gold each
   - If not killed in time, they disappear without awarding gold
   - Multiple gold carts spawn (exact count TBD — enough to be tempting)
   - Regular enemy spawning continues during the gold cart event
   - This is a "Greed Kills" moment: do you have enough DPS to kill carts
     while also handling the ongoing enemy pressure?

7. **Boss Trigger**: At 15:00, Wave Escalation stops spawning enemies.
   The Run Manager transitions to Boss state. Existing enemies remain.

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Inactive** | MainMenu, GracePeriod | Playing state begins | No spawning |
| **Ramp** | Playing begins (0:00) | Timer reaches 10:00 | Gradual scaling of spawn rate and stat multipliers |
| **Spike** | Timer reaches 10:00 | Timer reaches 15:00 | Aggressive scaling increase |
| **Stopped** | Timer reaches 15:00 (Boss) | Run ends | No more spawning. Existing enemies persist. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Run Manager** | Reads | Gets elapsed time and run state. Only spawns during Playing state. |
| **Enemy Data** | Reads | Gets enemy type definitions. Applies HP/damage multipliers on top of base stats. |
| **Enemy System** | Sends commands | Tells Enemy System to spawn enemy of type X at position Y |
| **Arena** | Reads | Gets spawn ring radius for spawn positions |
| **Gold Economy** | Indirect | Gold carts award bounty through the normal kill → bounty pipeline (250 gold bounty) |

## Formulas

### Spawn Rate

```
if elapsed_time <= 600:  // 0-10 minutes (Ramp)
    spawn_rate = BASE_SPAWN_RATE + (RAMP_RATE * elapsed_time)
else:  // 10-15 minutes (Spike)
    ramp_end_rate = BASE_SPAWN_RATE + (RAMP_RATE * 600)
    spike_time = elapsed_time - 600
    spawn_rate = ramp_end_rate + (SPIKE_RATE * spike_time)
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| BASE_SPAWN_RATE | f32 | TBD | Config | Enemies per second at time 0 |
| RAMP_RATE | f32 | TBD | Config | Additional enemies/sec gained per second of elapsed time |
| SPIKE_RATE | f32 | TBD | Config | Additional enemies/sec per second after 10:00 (much higher than RAMP_RATE) |

### HP Multiplier

```
if elapsed_time <= 600:
    hp_mult = 1.0 + (HP_RAMP * elapsed_time)
else:
    ramp_end_mult = 1.0 + (HP_RAMP * 600)
    spike_time = elapsed_time - 600
    hp_mult = ramp_end_mult + (HP_SPIKE * spike_time)

spawned_enemy_hp = base_hp * hp_mult
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| HP_RAMP | f32 | TBD | Config | HP multiplier increase per second during ramp |
| HP_SPIKE | f32 | TBD | Config | HP multiplier increase per second during spike |

### Damage Multiplier

Same structure as HP multiplier with its own ramp/spike rates.

### Spawn Position

```
angle = random(0, 2π)
position = (cos(angle) * SPAWN_RING_RADIUS, sin(angle) * SPAWN_RING_RADIUS)
```

### Enemy Type Selection

```
armor_type = weighted_random(spawn_weight_table)
```

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Gold carts spawn during heavy enemy pressure at 6:00 | Both gold carts and regular enemies are on screen simultaneously. Player must prioritize. | This IS the design — "Greed Kills." |
| Gold cart reaches tower and 5-second timer expires | Cart despawns. No gold awarded. No damage to tower. | Missed opportunity, not punishment. |
| Gold cart killed by splash/area damage incidentally | Awards 250 gold normally. | Incidental kills still count. |
| All gold carts killed instantly by a strong build | Player gets a gold windfall. Rewards building DPS early. | Good builds should feel rewarding. |
| Spawn rate exceeds entity budget at extreme times | Cap spawn rate at a maximum that keeps the game at 60fps. Allow enemies to get stronger (HP/damage) instead of spawning more. | Performance must not degrade. |
| Player survives to 15:00 with minimal build | Enemies at 15:00 are extremely tough. This is expected — the spike phase should punish underinvestment. | The scaling curve IS the difficulty. |
| Game paused during gold cart 5-second timer | Timer pauses too. Cart waits. | Consistent with all pause behavior. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Run Manager** | Upstream (hard) | Elapsed time and run state |
| **Enemy Data** | Upstream (hard) | Base enemy stats to apply multipliers to |
| **Enemy System** | Downstream (hard) | Receives spawn commands |
| **Arena** | Upstream (hard) | Spawn ring radius |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| BASE_SPAWN_RATE | TBD | 0.5–3.0 enemies/sec | More pressure from the start | Gentler opening |
| RAMP_RATE | TBD | 0.001–0.01 per second | Faster ramp; mid-game harder | Slower ramp; mid-game easier |
| SPIKE_RATE | TBD | 2x–5x RAMP_RATE | Brutal final 5 minutes | Gentler endgame |
| HP_RAMP | TBD | — | Spongier enemies over time | Enemies stay fragile longer |
| HP_SPIKE | TBD | — | Endgame enemies are walls | Endgame is DPS-checkable |
| Damage multiplier rates | TBD | — | Enemies hit harder over time | Tower HP pool lasts longer |
| Gold cart count (6:00) | TBD | 3–10 | More gold opportunity; more DPS required | Smaller windfall |
| Gold cart bounty | 250 gold | 100–500 | Bigger reward for killing carts | Less impactful event |
| Gold cart linger time | 5 seconds | 3–10 | Easier to kill in time | Must have DPS ready immediately |
| Armor type spawn weights | TBD | — | Changing weights shifts which damage types are most valuable | — |

**Knob interactions**: Spawn rate × enemy HP determines total "tankiness per second"
the player's build must overcome. This must scale in harmony with the player's
expected gold income and weapon DPS growth. Too fast = unwinnable. Too slow =
no tension.

**Critical tuning target**: At 10:00, a moderately-built tower should feel
pressured but alive. At 13:00, it should feel desperate. At 15:00, it should
feel like the boss is a mercy compared to another minute of escalation.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Gold cart spawns | Distinct gold-colored cart sprite, visually different from combat enemies | Special spawn jingle/chime | High |
| Gold cart sitting at tower | Pulsing gold glow, timer indicator above cart | Ticking/countdown sound | High |
| Gold cart killed | Coins burst from cart | Coin shower sound | High |
| Gold cart despawns (missed) | Cart fades away / drives off | Sad trombone or missed-opportunity sound | Medium |
| Scaling spike at 10:00 | Optional: screen tint shift, intensity change | Optional: music shifts to more intense track | Low |

## UI Requirements

| Information | Display Location | Update Frequency | Condition |
|-------------|-----------------|-----------------|-----------|
| Gold cart timer | Above gold cart sprite | Every frame | When gold cart is sitting at tower |

## Acceptance Criteria

- [ ] Enemies spawn continuously during Playing state with no breaks
- [ ] All 5 non-boss armor types spawn from minute 0
- [ ] Spawn rate increases gradually from 0:00 to 10:00 (Ramp phase)
- [ ] Spawn rate increases dramatically from 10:00 to 15:00 (Spike phase)
- [ ] Enemy HP and damage scale with multipliers over time
- [ ] Gold carts spawn at the 6:00 mark
- [ ] Gold carts do not attack the tower
- [ ] Gold carts award 250 gold when killed within 5 seconds of arriving
- [ ] Gold carts despawn without gold if not killed in time
- [ ] Spawning stops at 15:00 when boss spawns
- [ ] Existing enemies remain active after spawning stops
- [ ] Spawn rate is capped at a performance-safe maximum
- [ ] All scaling values are configurable (no hardcoded curves)

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| What are the exact scaling curve values (BASE_SPAWN_RATE, RAMP_RATE, SPIKE_RATE, HP/damage rates)? | Systems Designer | During balancing prototype | Requires playtesting to tune feel |
| What are the spawn weight ratios for armor types? | Game Designer | During balancing | Should create a mix where no single damage type dominates |
| How many gold carts spawn at 6:00? | Game Designer | Before prototype | Enough to be tempting (5-8?) |
| Should there be additional special events beyond the 6:00 gold carts? | Game Designer | Vertical Slice | Could add more events to break up the continuous ramp |
| Should move speed also scale, or just HP/damage/density? | Game Designer | During balancing | Faster enemies compress reaction time but may feel unfair |
