# Damage Calculation

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story

## Overview

Damage Calculation is the formula engine that resolves all damage in the game.
It sits between the source of damage (weapons hitting enemies, enemies hitting
the tower) and the target's health pool. For weapon-to-enemy damage, it applies
the damage type vs. armor type multiplier from the damage matrix, plus all
applicable upgrade bonuses. For enemy-to-tower damage, it feeds raw damage into
the tower's damage pipeline (mana shield → armor → flat reduction → HP). This
system owns no state — it is a pure calculation that other systems invoke.

## Player Fantasy

The player doesn't think about damage calculation directly — they feel it
through time-to-kill. When their Piercing weapons shred Light enemies instantly
but barely scratch Fortified ones, the damage matrix is doing its job. When they
stack +10% Piercing Damage three times and watch their DPS jump, the modifier
system is delivering. The system is invisible but shapes every combat interaction.

## Detailed Design

### Core Rules

1. **Weapon → Enemy Damage Pipeline**:
   ```
   base_damage (from weapon definition)
   → apply damage type bonuses (from upgrades: +% per type)
   → apply attack type bonuses (from upgrades: Focusfire, Splashfire, etc.)
   → apply global damage bonuses (from upgrades: Improved Attacks, Power Generator)
   → apply critical strike (if proc)
   → apply damage matrix multiplier (weapon type vs enemy armor type)
   → apply enemy-specific debuffs (Chaos damage amp, Poison damage amp)
   → final damage subtracted from enemy HP
   ```

2. **Enemy → Tower Damage Pipeline**:
   ```
   raw_damage (from enemy definition)
   → subtract from Mana Shield (absorbs raw, unmitigated damage)
   → remainder: apply tower armor reduction (WC3 formula)
   → apply flat damage reduction (Deflection: cannot reduce below 25%)
   → final damage subtracted from tower HP
   ```
   (See Tower Entity GDD for full armor formula details.)

3. **Damage Type Bonuses** (additive per type):
   Upgrades like "+10% Piercing Damage" apply only to weapons whose damage
   type includes Piercing. Hybrid weapons (e.g., "Piercing and Frost") benefit
   from both Piercing bonuses AND Frost bonuses.

   ```
   type_bonus = sum of all matching damage type % bonuses
   ```

   Examples from upgrades:
   - Improved Piercing Attacks: +10% Piercing Damage
   - Improved Attacks: +5% Normal/Piercing/Magic/Siege/Chaos Damage
   - Searing Heat: +10% Fire Damage
   - Potent Poison: +10% Poison Damage

4. **Attack Type Bonuses** (from specific upgrades):
   - Focusfire: +25% Damage for Single Target weapons
   - Splashfire: +25% Damage for Splash weapons
   - Wavefire: +25% Damage for Wave weapons
   - Areafire: +20% Area of Effect for Area weapons (not damage, but AoE size)
   - Command Aura: +25% Damage for 300 and 600 Attack Range weapons

5. **Global Damage Bonuses**:
   - Improved Attacks: +5% to all damage types
   - Power Generator: +2% Damage, +1% every 30 seconds
   - Golden Ring: +1% Damage per 50% Kill Bounty

6. **Critical Strike**:
   ```
   if random() < crit_chance:
       damage *= (1 + crit_power)
   ```
   - Base crit chance: 0%
   - Critical Strike upgrade: +5% crit chance
   - Critical Decimation upgrade: +10% crit chance, +10% crit power
   - Default crit power: 100% bonus damage (2x multiplier)

7. **Damage Matrix Lookup**:
   ```
   matrix_multiplier = damage_matrix[weapon_damage_type][enemy_armor_type]
   ```
   Uses the exact matrix defined in Enemy Data GDD. For hybrid weapons,
   use the PRIMARY damage type (first listed) for the matrix lookup. The
   secondary type (Fire, Frost, Poison) triggers status effects, not a
   second matrix lookup.

8. **Enemy Debuffs**:
   - Chaos damage amplification: "Attacks increase Chaos damage taken by 20%
     for 1 second" — this is a per-enemy debuff that stacks from Chaos weapons.
   - Similar debuffs exist for Normal (Thorn), Piercing (Glaive Thrower).
   - These debuffs apply multiplicatively after all other bonuses.

9. **Modifier Stacking Rules**:
   - **Same category**: Additive (e.g., two +10% Piercing = +20% Piercing)
   - **Different categories**: Multiplicative (type bonus × attack type bonus
     × global bonus × crit × matrix × debuffs)
   - This prevents any single upgrade category from being overwhelmingly
     strong while rewarding diverse investment.

### States and Transitions

Damage Calculation is stateless — it is a pure function invoked by other
systems. No states or transitions to document.

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Projectile System** | Upstream | Invokes damage calc when a projectile hits an enemy |
| **Weapon System** | Reads | Gets weapon damage type, base damage, attack pattern |
| **Enemy Data** | Reads | Gets enemy armor type for matrix lookup |
| **Enemy System** | Writes | Subtracts calculated damage from enemy HP |
| **Tower Entity** | Writes | Sends raw enemy damage into tower damage pipeline |
| **Content Database** | Reads | Reads damage matrix values |
| **Status Effects** | Reads/Writes | Reads debuffs on target (Vertical Slice). Triggers status effects from element types. |

## Formulas

### Full Weapon → Enemy Damage Formula

```
// Step 1: Base damage with type bonuses (additive within category)
typed_damage = base_damage * (1 + type_bonus_percent)

// Step 2: Attack type bonus
attack_typed_damage = typed_damage * (1 + attack_type_bonus_percent)

// Step 3: Global bonus
global_damage = attack_typed_damage * (1 + global_bonus_percent)

// Step 4: Critical strike
if random() < crit_chance:
    crit_damage = global_damage * (1 + crit_power)
else:
    crit_damage = global_damage

// Step 5: Damage matrix
matrix_damage = crit_damage * damage_matrix[weapon_type][armor_type]

// Step 6: Target debuffs
final_damage = matrix_damage * (1 + target_debuff_percent)

// Apply to enemy HP
enemy.hp -= final_damage
```

### Example Calculation

**Setup**: Frost Bow (Piercing, 225 damage) hitting a Light enemy.
**Upgrades**: 2× Improved Piercing Attacks (+20% total), Focusfire (+25%),
Power Generator (+5% global), Critical Strike (5% crit, 100% crit power).
**No crit this hit.**

```
typed_damage    = 225 * (1 + 0.20) = 270
attack_typed    = 270 * (1 + 0.25) = 337.5
global_damage   = 337.5 * (1 + 0.05) = 354.4
crit_damage     = 354.4 (no crit)
matrix_damage   = 354.4 * 2.0 (Piercing vs Light) = 708.8
final_damage    = 708.8 (no debuffs)
```

Enemy takes **709 damage** (rounded).

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Damage rounds to 0 (e.g., very weak weapon vs Fortified at 35%) | Minimum 1 damage. No attack deals 0 damage. | Every hit should feel like it does something, even if barely. |
| Hybrid weapon type (e.g., "Piercing and Frost") — which type for matrix? | Use first listed type (Piercing) for matrix lookup. Frost triggers status effect. | Clear priority. First type = damage, second type = effect. |
| Multiple damage type bonuses apply to hybrid weapon | Both type bonuses apply additively. "+10% Piercing" and "+10% Frost" both boost a "Piercing and Frost" weapon for +20% type bonus. | Hybrid weapons benefit from investing in either (or both) element types. |
| Crit on Splash/Bounce/Barrage — does crit apply to all targets? | Crit is rolled once per weapon fire. If it crits, ALL targets in that fire event get crit damage. | One roll per attack, not per target. Splash crit feels powerful. |
| 100%+ crit chance | Cap at 100%. Guaranteed crits. | Possible with enough Critical Decimation stacks. |
| Negative damage after all modifiers | Clamp to 1. | Should never happen with current math, but safe floor. |
| Debuff expires between hit calculation and damage application | Apply debuff state at time of hit calculation. | Frame-level timing is good enough. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Content Database** | Upstream (hard) | Damage matrix, weapon definitions |
| **Enemy Data** | Upstream (hard) | Armor types |
| **Tower Entity** | Downstream (hard) | Tower damage pipeline |
| **Projectile System** | Upstream (hard) | Invokes on hit |
| **Weapon System** | Upstream (hard) | Weapon stats |
| **Enemy System** | Downstream (hard) | Applies damage to enemies |
| **Status Effects** | Bidirectional (soft) | Reads debuffs, triggers effects |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Minimum damage | 1 | 0–5 | Every hit does at least N damage | Weapons can deal effectively 0 vs hard targets |
| Crit chance cap | 100% | 50–100% | Guaranteed crits possible | Soft cap on crit stacking |
| Default crit power | 100% (2x damage) | 50–200% | Crits hit harder | Crits less impactful |
| Damage matrix values | See Enemy Data GDD | 0.25–3.0 | Stronger type matchups | Flatter type matchups |

## Visual/Audio Requirements

N/A — Damage Calculation is invisible. Damage numbers, hit effects, and crit
indicators are owned by VFX / Juice and HUD systems.

## UI Requirements

N/A — Damage values are not shown to the player in MVP. Damage numbers floating
above enemies are a polish feature for VFX / Juice.

## Acceptance Criteria

- [ ] Weapon damage correctly applies type bonuses additively within category
- [ ] Different bonus categories multiply together (type × attack type × global × crit × matrix)
- [ ] Damage matrix returns correct multiplier for all 30 cells
- [ ] Hybrid weapons use first damage type for matrix, benefit from both type bonuses
- [ ] Critical strike rolls once per fire event, applies to all targets
- [ ] Minimum damage is 1 (no zero-damage hits)
- [ ] Enemy-to-tower damage flows through mana shield → armor → flat reduction → HP
- [ ] Mana shield absorbs raw damage before armor
- [ ] Tower armor uses WC3 formula correctly
- [ ] Debuff amplification (Chaos amp, etc.) stacks correctly
- [ ] No hardcoded damage values — all values from Content Database and upgrades

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| How do Fire/Frost/Poison element damage types interact with the matrix? | Status Effects GDD | Vertical Slice | Element damage may use its own multiplier or bypass armor entirely |
| Should damage numbers float above enemies? | VFX / Juice GDD | Alpha | Common in the genre but adds visual noise at scale |
| Should the player be able to see DPS stats for their current build? | HUD GDD | Alpha | Useful for optimization-minded players |
