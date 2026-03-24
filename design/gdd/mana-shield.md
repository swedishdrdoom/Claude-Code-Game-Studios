# Mana Shield

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Greed Kills (defensive investment vs offensive)

## Overview

Mana Shield is a secondary damage absorption layer on the tower that sits in
front of armor. It absorbs raw, unmitigated enemy damage — meaning it burns
faster than HP (which benefits from armor reduction) but provides complete HP
protection while active. Mana Shield starts at 0 and is built entirely from
upgrades. It does not regenerate naturally — it is restored through specific
upgrades (on-kill effects, flat amounts). Mana Shield creates a defensive
build path that competes with offensive investment for gold.

## Player Fantasy

Mana Shield is the magical barrier. When it's up, you feel invulnerable — hits
flash blue instead of red, and your HP doesn't move. When it breaks, you feel
exposed. The tension is that Mana Shield absorbs raw damage (no armor benefit),
so it depletes faster than you'd expect. It's a buffer, not a solution — you
still need to kill enemies or the shield will evaporate.

## Detailed Design

### Core Rules

1. **Starting Value**: 0 Mana Shield at run start. Built entirely from upgrades.

2. **Damage Absorption**: Mana Shield absorbs incoming enemy damage BEFORE
   armor is applied. This is raw, unmitigated damage.
   - If mana_shield >= incoming_damage: shield absorbs all, HP untouched.
   - If mana_shield < incoming_damage: shield absorbs what it can, remainder
     passes through armor → flat reduction → HP.

3. **No Natural Regen**: Mana Shield does not regenerate over time by default.
   Restoration comes only from upgrades:
   - Maw of Death: "+2000 Mana Shield. Restore 15 mana when an enemy dies."
   - Soulstealer weapon: "Attacks grant +3 Mana Shield per enemy hit."
   - Mana Shield upgrade: "+1000 Mana Shield" (increases max, grants that amount)

4. **Max Mana Shield**: The sum of all Mana Shield upgrade values is the maximum.
   Current mana shield cannot exceed max. On-kill/on-hit restoration is capped
   at max.

5. **Shield Break**: When mana shield drops to 0, the shield is "broken." There
   is no cooldown or penalty — restoration effects immediately begin refilling
   it. The visual "shield active" indicator turns off.

6. **Upgrade Interactions**:
   - Arcane Mark (Epic): "+10000 Mana Shield. +1% Damage per 2000 Mana Shield
     while Mana Shield is active." — Mana Shield becomes a damage amplifier.
   - Energy Shield (Rare): "+4000 Mana Shield. +25% Damage Reduction while
     Mana Shield is active." — While shield is > 0, tower takes 25% less
     damage to both shield and HP.

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Inactive** | Mana Shield max is 0 (no upgrades purchased) | First Mana Shield upgrade purchased | No shield mechanics. Damage goes straight to armor → HP. |
| **Active** | Current mana shield > 0 | Current mana shield reaches 0 | Shield absorbs raw damage. Visual indicator on. Conditional upgrades active (Arcane Mark, Energy Shield). |
| **Depleted** | Current mana shield reaches 0 | Restoration effect refills above 0 | Shield not absorbing. Damage goes to armor → HP. Conditional upgrades inactive. Visual indicator off. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Tower Entity** | Internal | Mana Shield is a tower stat. Tower damage pipeline checks mana shield first. |
| **Damage Calculation** | Upstream | Tower damage pipeline invokes mana shield absorption before armor. |
| **Enemy System** | Reads kill events | On-kill restoration effects (Maw of Death) trigger on enemy death. |
| **Weapon System** | Reads hit events | On-hit restoration effects (Soulstealer) trigger on weapon hit. |
| **Shop System** | Writes | Purchasing Mana Shield upgrades increases max and current. |
| **HUD** | Downstream | Displays mana shield bar. |

## Formulas

### Damage Absorption

```
if mana_shield > 0:
    // Energy Shield check
    if energy_shield_active:
        effective_damage = raw_damage * 0.75  // 25% reduction
    else:
        effective_damage = raw_damage

    absorbed = min(mana_shield, effective_damage)
    mana_shield -= absorbed
    remaining = effective_damage - absorbed
    // remaining goes through armor → flat reduction → HP
else:
    // No shield — full damage to armor → HP pipeline
    remaining = raw_damage
```

### On-Kill Restoration

```
mana_shield = min(mana_shield + restore_per_kill, max_mana_shield)
```

### Arcane Mark Damage Bonus

```
if mana_shield > 0:
    bonus_damage_percent = (current_mana_shield / 2000) * 0.01
    // At 10,000 shield: +5% damage
    // At 20,000 shield: +10% damage
```

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Mana shield at 1, incoming damage is 500 | Shield absorbs 1 damage. Remaining 499 goes through armor → HP. | Shield absorbs what it can, no more. |
| Multiple Mana Shield upgrades purchased | Max mana shield = sum of all. Each purchase also grants its value as current shield. | +1000 Mana Shield means +1000 max AND +1000 current. |
| Mana shield max reduced (future mechanic) | Clamp current to new max. | Consistent with HP max reduction behavior. |
| On-kill restore when shield is at max | No effect. Shield stays at max. | Can't overfill. |
| Energy Shield active + Mana Shield depleted in same hit | Energy Shield's 25% reduction applies to the full hit. After shield portion is absorbed, remaining damage goes through armor normally. Energy Shield deactivates when shield reaches 0. | The 25% reduction is active for the entire hit that breaks the shield, not just the absorbed portion. |
| Arcane Mark with 0 current mana shield | No damage bonus. Condition is "while Mana Shield is active." | Incentivizes keeping shield up. |
| Philosopher's Stone (-1000 HP) with mana shield active | HP reduced, shield unaffected. Philosopher's Stone targets HP, not shield. | They are separate pools. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Tower Entity** | Internal | Mana Shield is a tower stat |
| **Damage Calculation** | Upstream (hard) | Invoked in damage pipeline |
| **Enemy System** | Upstream (soft) | Kill events for on-kill restore |
| **Weapon System** | Upstream (soft) | Hit events for on-hit restore |
| **Shop System** | Upstream (hard) | Upgrades modify max/current shield |
| **HUD** | Downstream (soft) | Displays shield bar |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Mana Shield base values (per upgrade) | 1000-10000 | — | Larger shield pool; more raw damage absorbed | Smaller pool; shield breaks faster |
| On-kill restore (Maw of Death) | 15 per kill | 5–50 | Shield sustains better during heavy combat | Shield depletes during lulls |
| On-hit restore (Soulstealer) | 3 per enemy hit | 1–10 | Shield sustains from multi-target weapons | Shield drains faster |
| Energy Shield damage reduction | 25% | 10–40% | Significantly extends both shield and HP | Marginal benefit |

**Knob interactions**: Shield pool size × raw enemy DPS = shield uptime. On-kill
restore × kill rate = shield sustain. If enemies deal 100 raw DPS and shield
is 5000, shield lasts ~50 seconds without restoration. With Maw of Death at 15
mana per kill and 2 kills/sec, that's 30 mana/sec restoration.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Shield absorbs hit | Blue/purple flash on tower (instead of red) | Magical absorption sound | High |
| Shield breaks (reaches 0) | Shield shatter effect around tower | Glass break / energy disperse sound | High |
| Shield restores (from 0 to > 0) | Shield shimmer reappears around tower | Subtle energy hum | Medium |
| On-kill restore tick | Brief blue sparkle at tower | None (too frequent) | Low |

## UI Requirements

| Information | Display Location | Update Frequency | Condition |
|-------------|-----------------|-----------------|-----------|
| Current / Max Mana Shield | HUD — overlaid on HP bar or separate bar above it | Every frame | When max mana shield > 0 |
| Shield active indicator | Blue glow or overlay on HP bar area | On state change | When current > 0 |

## Acceptance Criteria

- [ ] Mana Shield starts at 0 and is built from upgrades
- [ ] Mana Shield absorbs raw damage before armor is applied
- [ ] Overflow damage (shield < hit) passes through armor → HP pipeline
- [ ] On-kill restoration works (Maw of Death)
- [ ] On-hit restoration works (Soulstealer)
- [ ] Current mana shield cannot exceed max
- [ ] Energy Shield 25% reduction applies while shield is > 0
- [ ] Arcane Mark damage bonus scales with current mana shield while active
- [ ] Shield break triggers visual/audio feedback
- [ ] Mana Shield bar displayed in HUD when max > 0
- [ ] Shield state correctly tracked across pause/unpause

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should mana shield have a passive regen rate (e.g., 1% per second)? | Game Designer | During balancing | Currently no natural regen; on-kill/on-hit only |
| How does Energy Shield's 25% reduction interact with the armor formula? | Damage Calculation | Implementation | Reduction applies to raw damage before shield absorbs, so both shield and overflow benefit |
