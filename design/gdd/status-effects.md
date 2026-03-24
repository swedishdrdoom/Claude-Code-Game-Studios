# Status Effects

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story

## Overview

Status Effects are persistent debuffs applied to enemies by weapons with element
subtypes (Fire, Frost, Poison) or special abilities (Stun, damage amplification).
They add a secondary layer of depth beyond raw damage — Frost controls enemy
movement, Fire creates chain explosions on death, Poison grinds down HP over
time, and Stun locks enemies in place. Status effects are what make hybrid
weapons (e.g., "Piercing and Frost") mechanically distinct from their pure
damage type counterparts.

## Player Fantasy

Status effects are where builds get their identity. A Frost build doesn't just
kill enemies — it freezes the entire arena into a crawling glacier. A Fire build
turns every kill into a chain reaction of explosions. A Poison build watches
enemies melt away after a single hit. These effects are the "and also" that
makes a weapon feel special: "It's a bow, AND ALSO it freezes everything."

## Detailed Design

### Core Rules

There are 5 status effect categories:

---

#### 1. Frost (Stacking Slow → Freeze)

- **Application**: Weapons with the Frost subtype apply Frost stacks on hit.
  Stack count is defined per weapon (e.g., "Frost (10 stacks)").
- **Slow Effect**: Each Frost stack reduces enemy movement speed and attack
  speed by 1%. Stacks up to 50 maximum.
- **Duration**: Frost stacks last 5 seconds. Timer refreshes on new stack
  application. If no new stacks are applied for 5 seconds, all stacks decay
  to 0 instantly.
- **Freeze**: At 50 stacks, the enemy is frozen for 3 seconds. During freeze:
  - Enemy cannot move or attack
  - Enemy is still targetable and takes damage
  - After freeze ends, Frost stacks reset to 0
- **Hero Armor Resistance**: Enemies with Hero armor have high FrostResistance
  (defined in Enemy Data). Frost stacks applied are multiplied by
  (1 - FrostResistance). E.g., 0.8 resistance means only 20% of stacks apply.
- **Upgrades**:
  - Shatter: "+25% Damage to enemies who are or have been Frozen"
  - Frost Armor: "Attackers receive 5 stacks of Frost" (defensive frost)
  - Dazing Stuns: Extends freeze duration (treated as a stun for this upgrade)

---

#### 2. Fire (Death Explosion)

- **Application**: Weapons with the Fire subtype apply the "Burning" debuff on
  hit. Burning is a binary state — an enemy is either burning or not.
- **Fire Damage Value**: When the burning debuff is applied, it stores a fire
  damage value based on the weapon: the weapon's "Fire (X% of damage)" stat.
  E.g., a 200-damage weapon with "Fire (5% of damage)" stores 10 fire damage.
  If a stronger fire weapon hits the same enemy, the stored value updates to
  the higher amount.
- **Death Explosion**: When a burning enemy dies, it explodes dealing the
  stored fire damage as AoE damage to all enemies within a fixed radius.
  - Explosion radius: TBD (configurable)
  - Explosion damage type: Fire (for purposes of fire damage upgrades)
- **Chain Reactions**: If the explosion kills another burning enemy, that
  enemy also explodes. Chains can cascade indefinitely.
- **Upgrades**:
  - Searing Heat: "+10% Fire damage" — increases stored fire damage value
  - Ignite: "When a burning enemy dies, spreads the burning debuff to an
    enemy within 600 range" — spreads debuff to a non-burning enemy nearby
  - Molten Spikes: "Attackers have a 20% chance to be lit on fire, receiving
    200 Fire stacks" — Note: this applies burning debuff with 200 as the
    stored fire damage value, not 200 "stacks"

---

#### 3. Poison (Damage Over Time)

- **Application**: Weapons with the Poison subtype apply a Poison DoT on hit.
  Duration and damage are defined per weapon (e.g., "Poison (300 damage over
  2 seconds)").
- **Tick Rate**: Poison deals damage every 0.5 seconds (4 ticks for a 2-second
  poison). Damage per tick = total_damage / (duration / tick_rate).
- **Stacking/Refreshing**: If the same Poison source hits again, the duration
  refreshes but damage does not stack. Different Poison sources (different
  weapons) stack independently — multiple Poison DoTs can run simultaneously.
- **Damage Type**: Poison damage uses the Poison damage type for upgrade
  interactions (Potent Poison: "+10% Poison damage"). Poison damage does NOT
  go through the armor type damage matrix — it is true damage.
- **Additional Effects**: Some Poison weapons also apply debuffs:
  - Poison Glaive: "Attacks increase Poison damage taken by 20% for 2 seconds"
  - Poison Spear: "25% attack speed reduction and 1000 damage over 2 seconds"
- **Upgrades**:
  - Potent Poison: "+10% Poison damage"
  - Poisonous Spikes: "Attackers are poisoned for 2 seconds, taking 100
    Spikes/Poison damage every 0.5 seconds"

---

#### 4. Stun (Timed Disable)

- **Application**: Weapons with a Stun ability apply a stun on hit for the
  defined duration (e.g., "Stun (2 seconds)").
- **Effect**: Stunned enemies cannot move or attack. They remain targetable
  and take full damage.
- **Refreshing**: If a stunned enemy is stunned again, the stun duration
  refreshes to the new stun's duration (does not stack additively).
- **Hero Armor Resistance**: Enemies with Hero armor have high StunResistance
  (defined in Enemy Data). Stun duration is multiplied by (1 - StunResistance).
  E.g., 0.8 resistance means a 2-second stun lasts 0.4 seconds.
- **Upgrades**:
  - Dazing Stuns: "+50% Stun Duration"
  - Bash: "+20% Damage to Stunned enemies"

---

#### 5. Damage Amplification Debuffs

- **Application**: Certain weapons apply temporary damage amplification debuffs
  to enemies on hit:
  - Death Coil: "+20% Chaos damage taken for 1 second"
  - Thorn: "+20% Normal damage taken for 1 second"
  - Glaive Thrower: "+20% Piercing damage taken for 2 seconds"
  - Poison Glaive: "+20% Poison damage taken for 2 seconds"
- **Stacking**: Multiple applications from the same weapon refresh duration.
  Different damage amp sources stack additively.
- **Interaction**: Damage amps are applied in the Damage Calculation pipeline
  as the final multiplier (see Damage Calculation GDD, step 6).

---

### General Rules

1. **Multiple Effects**: An enemy can have multiple status effects active
   simultaneously (e.g., Burning + Poisoned + 30 Frost stacks).

2. **Effect Ownership**: Status effects are tied to the enemy entity, not
   the weapon that applied them. If the weapon is sold/lost (future feature),
   active effects persist until they expire.

3. **Death Clears Effects**: When an enemy dies, all status effects are
   removed. Fire's death explosion triggers before cleanup.

4. **Pause Behavior**: Status effect timers (DoT ticks, stun duration, frost
   decay, damage amp duration) pause during Paused state.

5. **Visual Indicators**: Each status effect type has a visual indicator on
   the affected enemy (color tint, particle, or icon). Must be readable at
   scale (hundreds of enemies).

### States and Transitions (Per Enemy)

Status effects are per-enemy modifiers, not system-level states. Each enemy
tracks its own set of active effects. No global state machine.

| Effect | Applied When | Expires When | While Active |
|--------|-------------|-------------|-------------|
| Frost Stacks | Hit by Frost weapon | 5 seconds without new stacks | Slowed by 1% per stack. Frozen at 50. |
| Burning | Hit by Fire weapon | Never (persists until death) | Explodes on death. |
| Poison | Hit by Poison weapon | Duration expires (e.g., 2 seconds) | Takes damage every 0.5 seconds. |
| Stunned | Hit by Stun weapon | Duration expires | Cannot move or attack. |
| Damage Amp | Hit by amp weapon | Duration expires (1-2 seconds) | Takes increased damage of specified type. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Damage Calculation** | Upstream | Damage Calc invokes status effect application after resolving a hit. Damage amps feed back into Damage Calc for subsequent hits. |
| **Enemy System** | Reads/Writes | Reads enemy state (alive/dead). Writes movement speed modifier (Frost), attack speed modifier (Frost), stun state. Listens for death events (Fire explosion). |
| **Weapon System** | Upstream | Weapon fire events include element subtype and ability data. Status Effects reads these to determine what to apply. |
| **Enemy Data** | Reads | Reads FrostResistance and StunResistance for Hero-armor enemies. |
| **Projectile System** | Upstream | Hit events from projectiles trigger status effect application. |
| **Boss Encounter** | Interacts | Boss has Hero armor with high Frost/Stun resistance. Status effects apply but are significantly reduced. |
| **VFX / Juice** | Downstream | Each effect type triggers visual indicators on affected enemies. Fire explosion triggers AoE VFX. |
| **Audio System** | Downstream | Fire explosion, freeze, poison tick may have audio cues. |

## Formulas

### Frost Slow

```
movement_speed_modifier = 1.0 - (frost_stacks * 0.01)
attack_speed_modifier = 1.0 - (frost_stacks * 0.01)
// Clamped: at 50 stacks, both are 0.5 (50% speed)
```

### Frost Stack Application (with resistance)

```
applied_stacks = base_stacks * (1 - frost_resistance)
```

### Freeze Duration

```
freeze_duration = BASE_FREEZE_DURATION * (1 + stun_duration_bonus) * (1 - stun_resistance)
```

BASE_FREEZE_DURATION = 3 seconds. Dazing Stuns (+50%) extends this.

### Poison Tick Damage

```
damage_per_tick = (total_poison_damage / duration) * tick_interval
// E.g., 300 damage over 2 seconds, 0.5s ticks = 75 damage per tick
```

### Stun Duration (with resistance)

```
effective_stun = base_stun_duration * (1 + stun_duration_bonus) * (1 - stun_resistance)
```

### Fire Explosion Damage

```
explosion_damage = stored_fire_damage * (1 + fire_damage_bonus_percent)
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| stored_fire_damage | f32 | 5–300 | Weapon fire % × weapon damage | Fire damage stored when burning debuff applied |
| fire_damage_bonus_percent | f32 | 0.0+ | Searing Heat and similar upgrades | % bonus to fire damage |

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Frost stacks exceed 50 | Clamp at 50. Freeze triggers immediately. | 50 is the freeze threshold and cap. |
| Frost stacks at 49, hit adds 10 | Stacks go to 50 (clamped), freeze triggers. Excess stacks wasted. | Clean threshold behavior. |
| Frozen enemy hit by another Frost weapon | New frost stacks are queued. After freeze ends, stacks reset to 0, then queued stacks apply. OR: just ignore frost hits during freeze. | Simpler: ignore frost during freeze. Freeze is already the max effect. |
| Fire explosion kills a burning enemy | Chain explosion triggers. No limit on chain length. | Chain reactions are the fun of fire builds. |
| Fire chain creates infinite loop (enemy A kills B, B kills A) | Not possible — dead enemies can't explode twice. Death is permanent. | Death clears effects, including burning. |
| Poison ticks on enemy with 1 HP | Poison kills the enemy. Awards bounty normally. | Poison kills are valid kills. |
| Multiple Poison sources on same enemy | Each ticks independently. Total Poison DPS = sum of all active poisons. | Different weapons apply different poisons. |
| Stun applied to already-stunned enemy | Duration refreshes to new stun duration (whichever is longer). Does not stack additively. | Prevents perma-stun from rapid stun weapons... partially. High attack speed + stun can still lock enemies. |
| Stun + Freeze on same enemy | Both apply. Enemy is double-disabled. When one expires, the other continues. | Both are disables but from different sources. |
| Boss (Hero armor) with 0.8 Frost resistance | 10-stack Frost hit applies 2 stacks. Need 250 stacks worth of hits to reach 50. Extremely hard to freeze the boss. | Boss should resist CC heavily but not be fully immune. |
| Boss with 0.8 Stun resistance | 2-second stun becomes 0.4 seconds. Brief interrupt but not a lockdown. | Boss is stun-resistant, not stun-immune. |
| Fire explosion: does it apply burning to enemies it hits? | No — explosion deals fire damage but does not apply the burning debuff. Only direct weapon hits apply burning. Ignite upgrade is the exception (explicitly spreads burning on death). | Without this rule, every fire explosion would burning-debuff everything it hits, creating guaranteed chain reactions. |
| Damage amp debuffs: do they stack from multiple weapons of the same type? | Same weapon refreshes duration. Different weapons with the same amp type stack additively. | E.g., two Death Coils each add +20% = +40% Chaos damage taken. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Damage Calculation** | Upstream (hard) | Invokes effect application on hit; reads damage amps |
| **Enemy System** | Bidirectional (hard) | Reads enemy state, writes movement/attack modifiers |
| **Weapon System** | Upstream (hard) | Receives element subtype data from weapon hits |
| **Enemy Data** | Upstream (hard) | Reads Frost/Stun resistance values |
| **Projectile System** | Upstream (hard) | Hit events trigger effect application |
| **Boss Encounter** | Downstream (soft) | Boss has high CC resistance |
| **VFX / Juice** | Downstream (soft) | Visual indicators per effect |
| **Audio System** | Downstream (soft) | Audio cues for major effects |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Frost max stacks | 50 | 20–100 | Harder to freeze; more frost investment needed | Easier freeze; frost builds very strong |
| Frost slow per stack | 1% | 0.5–2% | Stronger slow at fewer stacks | Weaker slow; need more stacks for impact |
| Frost decay time | 5 seconds | 3–10 | More forgiveness between frost hits | Must hit faster to maintain stacks |
| Freeze duration | 3 seconds | 1–5 | Longer disable; very powerful CC | Brief disable; less impactful |
| Poison tick rate | 0.5 seconds | 0.25–1.0 | More frequent ticks; smoother damage | Chunkier ticks; more visible per tick |
| Fire explosion radius | TBD | 100–400 | Bigger explosions; more chain potential | Smaller explosions; more targeted |
| Boss FrostResistance | TBD (0.8?) | 0.5–0.95 | Boss nearly immune to slow/freeze | Boss can be meaningfully slowed |
| Boss StunResistance | TBD (0.8?) | 0.5–0.95 | Boss nearly immune to stun | Boss can be stun-locked with enough investment |

**Knob interactions**: Frost max stacks × frost per weapon × attack speed determines
how fast you can freeze an enemy. Fire explosion radius × enemy density determines
chain reaction potential. Boss resistances × player investment determines whether
CC builds are viable against the boss.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Frost stacks building | Blue tint intensifying on enemy | None (too frequent) | Medium |
| Enemy frozen | Ice crystal encasement effect, enemy stops | Ice crack/freeze sound | High |
| Freeze ends | Ice shatters off enemy | Shatter sound | Medium |
| Burning applied | Small flame icon/particle on enemy | Sizzle sound | Medium |
| Fire death explosion | Orange/red AoE burst at death location | Explosion sound | High |
| Fire chain reaction | Rapid successive explosions | Rapid successive explosion sounds | High |
| Poison active | Green tint or dripping particle on enemy | Subtle bubble/hiss (low priority) | Low |
| Stunned | Stars/swirl above enemy, enemy stops | Impact/daze sound | High |
| Damage amp active | Optional: colored outline matching damage type | None | Low |

## UI Requirements

N/A — Status effects are communicated through visual effects on enemies, not
through UI elements. No status effect bars or icons in the HUD.

## Acceptance Criteria

- [ ] Frost stacks apply and slow enemy movement/attack speed by 1% per stack
- [ ] Frost stacks cap at 50 and trigger 3-second freeze
- [ ] Frost stacks decay to 0 after 5 seconds without application
- [ ] After freeze ends, frost stacks reset to 0
- [ ] Hero armor enemies receive reduced frost stacks based on FrostResistance
- [ ] Burning debuff is binary (on/off), not stacking
- [ ] Burning enemies explode on death dealing stored fire damage in AoE
- [ ] Fire explosions can chain-kill other burning enemies
- [ ] Fire explosions do NOT apply burning debuff (only direct weapon hits do)
- [ ] Ignite upgrade spreads burning debuff to nearby enemy on death
- [ ] Poison deals damage over time at 0.5-second tick intervals
- [ ] Multiple Poison sources stack independently on same enemy
- [ ] Same Poison source refreshes duration, does not stack damage
- [ ] Poison damage bypasses armor type damage matrix (true damage)
- [ ] Stun disables enemy movement and attacks for duration
- [ ] Stun duration is reduced by StunResistance on Hero armor enemies
- [ ] Stun refreshes (does not stack additively) on re-application
- [ ] Damage amp debuffs apply correct multiplier in Damage Calculation
- [ ] All status effect timers pause during Paused state
- [ ] Multiple status effects can be active on same enemy simultaneously

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| What is the fire explosion radius? | Systems Designer | During balancing | Needs testing with enemy density |
| What are the exact Boss Frost/Stun resistance values? | Boss Encounter GDD | When Boss is designed | Should make CC very difficult but not impossible |
| Can enemies be frozen multiple times, or is there a freeze immunity window? | Game Designer | During testing | Multiple freezes may feel too strong; consider brief immunity after freeze |
| Should Poison damage be affected by Fire/Frost/Poison damage upgrades? | Game Designer | During balancing | Currently Poison is true damage; upgrades like Potent Poison add % to poison specifically |
| Does the stored fire damage on a burning enemy update if hit by a stronger fire weapon? | Game Designer | During implementation | Current design: yes, updates to higher value |
