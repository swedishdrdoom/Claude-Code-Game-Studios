# Boss Encounter

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Fifteen Minutes of Escalation

## Overview

The Boss Encounter is the climax of every run — a single massive enemy with
Hero armor that spawns at the 15-minute mark. The boss is a pure stat check:
no special abilities, no phases, no gimmicks. It walks toward the tower like
any other enemy, just with enormous HP, high damage, and armor that resists
everything except Normal and Chaos damage. The boss tests whether the player's
15-minute build can output enough DPS to kill it before it kills them. The
boss is the answer to "was this build good enough?"

## Player Fantasy

The boss is the final exam. Everything the player has built over 15 minutes —
every weapon purchased, every upgrade stacked, every reroll gambled — is tested
against one enemy. When the boss melts in seconds, the player feels vindicated.
When it slowly grinds through their HP while they desperately buy last-minute
weapons from the still-active shop, that's peak "Greed Kills" tension. The boss
doesn't need abilities to be dramatic — the 15 minutes of escalation leading up
to it provide all the drama.

## Detailed Design

### Core Rules

1. **Trigger**: The boss spawns when the Run Manager transitions from Playing
   to Boss state (timer reaches 15:00).

2. **Spawn**: The boss spawns from the spawn ring at a random angle, like any
   normal enemy. It walks in a straight line toward the tower at its defined
   move speed.

3. **Stats**: The boss is an enemy entity with Hero armor type. Its stats are
   defined in the Enemy Data JSON:
   - Very high MaxHP (exact value TBD during balancing)
   - High Damage per attack
   - Moderate MoveSpeed (not fast — gives time to DPS)
   - High FrostResistance (e.g., 0.8 — frost stacks apply at 20%)
   - High StunResistance (e.g., 0.8 — stun duration reduced to 20%)
   - GoldBounty: 0 (killing the boss awards Victory, not gold)

4. **Hero Armor**: The boss uses the Hero armor type from the damage matrix:

   | Damage Type | Multiplier |
   |-------------|-----------|
   | Normal | 100% |
   | Piercing | 50% |
   | Siege | 50% |
   | Magic | 50% |
   | Chaos | 100% |

   Only Normal and Chaos deal full damage. All other types are halved.

5. **No Special Abilities**: The boss has no unique mechanics. It walks, it
   attacks, it takes damage. The challenge comes from its stat block and
   armor type, not from special moves.

6. **Existing Enemies**: When the boss spawns, all existing enemies remain
   active. They continue moving and attacking. No new enemies spawn. The
   player must handle both the boss and the remaining horde.

7. **Shop Active**: The shop remains fully active during the boss phase. The
   player can buy weapons, upgrades, and reroll while fighting the boss.
   This is a final "Greed Kills" moment — do you spend gold on last-minute
   power, or focus on the fight?

8. **Victory**: When the boss's HP reaches 0, the Run Manager transitions
   to Victory state. All remaining enemies are irrelevant — the player wins.

9. **Defeat**: If the tower's HP reaches 0 during the boss phase (from boss
   attacks, remaining enemies, or both), the Run Manager transitions to
   Defeat state. The boss killed you.

10. **No Timer**: The boss phase has no time limit. The player has unlimited
    time to kill the boss — but the boss (and remaining enemies) are actively
    dealing damage, so time is limited by HP.

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Inactive** | Run not at 15:00 yet | Timer reaches 15:00 | No boss exists |
| **Active** | Timer reaches 15:00 | Boss HP reaches 0 OR Tower HP reaches 0 | Boss spawned, walking, attacking. Existing enemies still active. |
| **Defeated** | Boss HP reaches 0 | Run Manager transitions to Victory | Boss death animation. Victory triggered. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Run Manager** | Reads | Listens for Playing → Boss transition to spawn the boss |
| **Enemy Data** | Reads | Boss enemy definition (Hero armor, stats) |
| **Enemy System** | Uses | Boss is spawned and managed as a regular enemy entity through the Enemy System |
| **Damage Calculation** | Uses | All damage to/from boss uses standard damage pipeline. Hero armor multipliers apply. |
| **Status Effects** | Interacts | Boss has high FrostResistance and StunResistance. Status effects apply but are significantly reduced. Fire burning/explosion still works at full effect. |
| **Wave Escalation** | Reads | Wave Escalation stops spawning; Boss Encounter spawns the boss instead |
| **Shop System** | Independent | Shop continues operating normally during boss phase |
| **Gold Economy** | Independent | Gold income continues. Boss awards 0 bounty (Victory is the reward). |
| **HUD** | Downstream | May display boss HP bar or indicator |

## Formulas

No unique formulas. The boss uses standard Enemy System movement, standard
Damage Calculation pipeline, and standard Status Effects application. All
math is defined in those systems' GDDs.

### Key Derived Values

```
// How long the boss survives (theoretical, ignoring remaining enemies)
boss_time_to_kill = boss_hp / player_effective_dps_vs_hero_armor

// For Normal/Chaos weapons: effective DPS = full weapon DPS
// For Piercing/Siege/Magic weapons: effective DPS = weapon DPS * 0.5
```

```
// How long until boss reaches tower
boss_travel_time = spawn_ring_radius / boss_move_speed
```

**Tuning target**: boss_time_to_kill should be 30-90 seconds for a
well-built character. boss_travel_time should be 10-20 seconds (boss
reaches tower and starts attacking before it dies, creating pressure).

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Boss and tower die on same frame | Victory takes priority. Player killed the boss. | Reward the player's effort. Mutual destruction = win. |
| Tower dies to a remaining enemy while boss is still alive | Defeat. The boss didn't kill you, but you still lost. | Death is death regardless of source. |
| All remaining enemies die before boss | Boss is the only enemy. Player focuses fire. | Clean 1v1 scenario. |
| Player has no Normal or Chaos weapons | All damage to boss is at 50% multiplier. Slow but not impossible. | Build diversity matters. Player pays for not having boss-effective damage types. |
| Player has built pure Chaos | Full 100% damage to boss. Boss melts. Chaos build rewarded. | Chaos is the "answer" to Hero armor. |
| Boss is frozen (50 Frost stacks with high resistance) | Possible but extremely difficult. At 0.8 resistance, need 250 stacks worth of frost hits. Freeze lasts 3 * 0.2 = 0.6 seconds. | Not impossible, but barely worth the investment. |
| Boss stunned repeatedly | Each stun is 20% duration. A 2-second stun becomes 0.4 seconds. Rapid stun weapons can interrupt briefly. | Stun build against boss is a minor annoyance, not a strategy. |
| Player buys weapons from shop during boss fight | Fully allowed. New weapons start firing immediately. | "Greed Kills" — spending gold during the boss is a valid last-ditch strategy. |
| Fire explosion on a burning boss | Boss takes fire explosion damage normally. Fire damage is not in the damage matrix (it's a status effect). | Fire builds work against boss through explosions, not through armor bypassing. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Run Manager** | Upstream (hard) | Triggers boss spawn |
| **Enemy Data** | Upstream (hard) | Boss stat definition |
| **Enemy System** | Upstream (hard) | Boss is managed as a regular enemy |
| **Damage Calculation** | Upstream (hard) | Standard damage pipeline |
| **Wave Escalation** | Upstream (hard) | Stops spawning when boss appears |
| **Status Effects** | Upstream (soft) | Reduced CC on boss |
| **Shop System** | Peer (soft) | Shop operates independently during boss |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Boss MaxHP | TBD | — | Longer boss fight; more DPS required; more tension | Shorter fight; build quality less tested |
| Boss Damage | TBD | — | Boss threatens tower faster; tighter time-to-kill window | Boss is less threatening; player has more time |
| Boss MoveSpeed | TBD | — | Less time before boss reaches tower; DPS window shorter | More time to damage boss before it reaches tower |
| Boss FrostResistance | TBD (0.8) | 0.5–0.95 | Frost is near-useless against boss | Frost builds can meaningfully slow boss |
| Boss StunResistance | TBD (0.8) | 0.5–0.95 | Stun barely interrupts boss | Stun-lock boss becomes viable |

**Knob interactions**: Boss HP / player effective DPS = time-to-kill. Boss
damage × boss attack speed = DPS against tower. Boss travel time from spawn
ring = free DPS window before boss starts attacking. All three interact to
determine difficulty.

**Critical balance point**: A player who invested in Normal or Chaos weapons
should find the boss challenging but fair. A player with zero Normal/Chaos
should find it brutal but theoretically possible (at 50% damage, everything
still works, just slower).

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Boss spawn alert (before spawn) | "BOSS INCOMING" text flash, screen edge warning | Boss horn / alarm sound | High |
| Boss entity visible | Significantly larger sprite than normal enemies. Distinct silhouette. | Heavy footstep/rumble | High |
| Boss takes damage | Same as enemy hit flash, but more pronounced | Heavy impact sound | High |
| Boss dies | Large death explosion, Victory trigger | Massive explosion + victory fanfare | High |
| Boss attacks tower | Large attack animation | Heavy slam sound | High |

## UI Requirements

| Information | Display Location | Update Frequency | Condition |
|-------------|-----------------|-----------------|-----------|
| Boss HP bar | Top of screen or near boss sprite | Every frame | During Boss state |
| Boss name | Above boss HP bar | Static | During Boss state |

## Acceptance Criteria

- [ ] Boss spawns from the spawn ring when timer reaches 15:00
- [ ] Boss has Hero armor type (Normal 100%, Piercing 50%, Siege 50%, Magic 50%, Chaos 100%)
- [ ] Boss has high FrostResistance and StunResistance
- [ ] Boss walks toward tower in a straight line like any other enemy
- [ ] Boss attacks tower with its damage value on its attack cooldown
- [ ] Boss HP reaching 0 triggers Victory
- [ ] Tower HP reaching 0 during boss phase triggers Defeat
- [ ] Mutual destruction (same frame) = Victory
- [ ] Existing enemies remain active during boss phase
- [ ] No new enemies spawn during boss phase
- [ ] Shop is fully active during boss phase
- [ ] Boss has no special abilities (pure stat check)
- [ ] Boss awards 0 gold bounty on kill
- [ ] Boss HP bar is visible during boss phase
- [ ] Boss is visually distinct from regular enemies (larger, unique sprite)

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| What are the boss's exact stat values (HP, damage, speed, attack cooldown)? | Systems Designer | During balancing | Requires playtesting with typical 15-minute builds |
| Should there be a warning before the boss spawns (e.g., at 14:00)? | HUD GDD | Before implementation | Alert at 14:00-14:30 would give players time to prep |
| Should the boss have a name and visual theme? | Art Director | Alpha | "Tower of Doom" boss = "Doom" itself? |
| Should there be multiple boss variants for replayability? | Game Designer | Post-demo | Different bosses with different Hero-armor variants could add variety |
| Is one boss enough, or should there be a mini-boss at a mid-point (e.g., 10:00)? | Game Designer | Vertical Slice / playtesting | Could add a mid-run skill check at the scaling spike |
