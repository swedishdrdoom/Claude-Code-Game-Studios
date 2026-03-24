# Tower Entity

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: All — the tower IS the player

## Overview

The Tower Entity is the player — a stationary structure at the center of the
arena with an HP pool, armor stat, mana shield, and health regeneration. It
holds the list of equipped weapons and applied upgrades. The tower does not
move. All damage from enemies flows into the tower; all weapons fire from it.
When its HP reaches 0, the run ends in defeat.

## Player Fantasy

You ARE the tower. Every weapon you buy extends from you. Every hit you take
chips away at your walls. The tower's HP bar is the run's heartbeat — watching
it drop creates the tension that makes shop decisions urgent. When you stack
armor and regen and watch enemies futilely claw at your fortified walls, that's
the defensive power fantasy. When your HP is at 200 and you buy a Philosopher's
Stone (-1000 HP, +2000 gold), that's the greed fantasy.

## Detailed Design

### Core Rules

1. **Position**: Fixed at world origin (0,0). Cannot move. Ever.

2. **Starting Stats**:
   - HP: 1500
   - Armor: 0
   - HP Regen: 0 per second
   - Mana Shield: 0
   - Gold: (defined by Gold Economy)
   - Weapons: empty list
   - Upgrades: empty list

3. **Health**: Tower has current HP and max HP. Current HP cannot exceed max HP.
   Current HP cannot go below 0 (clamp). When current HP reaches 0, the tower
   is destroyed and the Run Manager transitions to Defeat.

4. **Armor** (WC3 formula):

   For positive armor:
   ```
   damage_reduction = (armor * 0.06) / (1 + armor * 0.06)
   ```

   For negative armor:
   ```
   damage_multiplier = 2 - 0.94^(-armor)
   ```

   - 0 armor = 0% reduction (starting state)
   - 5 armor = 23% reduction
   - 10 armor = 37.5% reduction
   - 20 armor = 54.5% reduction
   - Diminishing returns — each point is worth slightly less
   - Negative armor increases damage taken (capped at 2x)

5. **Incoming Damage Pipeline**:
   ```
   raw_damage (from enemy)
   → subtract from Mana Shield first (if active, absorbs raw unmitigated damage)
   → remainder: apply armor damage reduction (WC3 formula)
   → apply flat damage reduction (from upgrades like Deflection)
   → subtract from HP
   ```

   Mana Shield absorbs damage BEFORE armor. This means Mana Shield takes the
   full brunt of enemy attacks without any reduction, but it protects the tower's
   HP pool completely while active.

6. **HP Regeneration**: Tower regenerates HP per second based on regen stat
   (starting at 0). Regen is modified by upgrades (Repair Crew: +20 regen,
   Rejuvenating Petal: +40 regen + burst heal, etc.). Regen ticks every frame
   (regen_per_second * delta_time).

7. **Weapon List**: The tower maintains an ordered list of equipped weapons.
   Each entry is an independent weapon instance referencing a weapon definition.
   Buying a duplicate adds another entry — no stacking logic. Each weapon fires
   independently.

8. **Upgrade List**: Applied upgrades modify tower stats (HP, armor, regen,
   damage bonuses, etc.). Upgrades are applied immediately on purchase. Some
   upgrades have ongoing effects (scaling over time).

9. **Run Reset**: On restart, all tower state resets to starting stats. Weapon
   list empties. Upgrade list empties. HP resets to starting max HP.

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Alive** | Run starts | HP reaches 0 | All systems active. Takes damage, fires weapons, applies upgrades. |
| **Destroyed** | HP reaches 0 | Run Manager transitions to Defeat | Stop all weapon firing. Tower visually destroyed. Trigger Defeat. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Content Database** | Reads | Reads upgrade definitions to apply stat modifications |
| **Arena** | Reads | Spawns at world origin (0,0) |
| **Weapon System** | Provides weapon list | Weapon System iterates the tower's weapon list to fire each weapon |
| **Damage Calculation** | Receives damage | Enemy attacks send raw damage to the tower. Tower applies mana shield → armor → flat reduction → HP. |
| **Shop System** | Receives purchases | Shop adds weapons/upgrades to the tower's lists and modifies stats |
| **Run Manager** | Sends death event | Tower HP reaching 0 triggers Defeat state |
| **HUD** | Exposes stats | HUD reads current HP, max HP, armor, mana shield, regen for display |

## Formulas

### Armor Damage Reduction (Positive Armor)

```
damage_reduction = (armor * 0.06) / (1 + armor * 0.06)
damage_after_armor = raw_damage * (1 - damage_reduction)
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| armor | f32 | 0–100+ | Tower stat (upgrades) | Current armor value |
| raw_damage | f32 | 1–1000+ | Enemy attack | Incoming damage after mana shield |
| damage_reduction | f32 | 0.0–~0.86 | Calculated | % of damage absorbed by armor |

**Example outputs**:

| Armor | Reduction | 100 raw → takes |
|-------|-----------|-----------------|
| 0 | 0% | 100 |
| 5 | 23.1% | 77 |
| 10 | 37.5% | 63 |
| 20 | 54.5% | 46 |
| 30 | 64.3% | 36 |
| 50 | 75.0% | 25 |

### Armor Damage Amplification (Negative Armor)

```
damage_multiplier = 2 - 0.94^(-armor)
damage_after_armor = raw_damage * damage_multiplier
```

| Armor | Multiplier | 100 raw → takes |
|-------|-----------|-----------------|
| -5 | 1.27x | 127 |
| -10 | 1.46x | 146 |
| -20 | 1.71x | 171 |
| Cap | 2.0x | 200 |

### Full Damage Pipeline

```
// Step 1: Mana Shield absorbs raw damage (no armor reduction)
if mana_shield > 0:
    absorbed = min(mana_shield, raw_damage)
    mana_shield -= absorbed
    remaining_damage = raw_damage - absorbed
else:
    remaining_damage = raw_damage

// Step 2: Armor reduction on remainder
if armor >= 0:
    reduction = (armor * 0.06) / (1 + armor * 0.06)
    damage_after_armor = remaining_damage * (1 - reduction)
else:
    multiplier = 2 - 0.94^(-armor)
    damage_after_armor = remaining_damage * multiplier

// Step 3: Flat damage reduction (Deflection upgrade)
if flat_reduction > 0:
    damage_final = max(damage_after_armor - flat_reduction, damage_after_armor * 0.25)
else:
    damage_final = damage_after_armor

// Step 4: Apply to HP
current_hp = max(current_hp - damage_final, 0)
```

### HP Regeneration

```
current_hp = min(current_hp + regen_per_second * delta_time, max_hp)
```

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| HP drops below 0 | Clamp to 0. Trigger Defeat once. | No negative HP; no double-death. |
| Max HP reduced below current HP (Philosopher's Stone: -1000 HP) | Current HP clamped to new max HP. If new max HP ≤ 0, tower dies. | Buying a risky upgrade can kill you. That's the design. |
| Armor goes negative (future: armor shred mechanic) | WC3 negative armor formula applies. Damage increases up to 2x cap. | Consistent with WC3 reference. |
| Regen while at max HP | No effect. HP stays at max. | Standard behavior. |
| Mana shield absorbs more than remaining mana shield value | Absorb up to remaining mana shield, overflow passes to armor → HP. | Mana shield is a buffer, not a hard gate. |
| Multiple damage instances in one frame | Each processed independently through the full pipeline. | No batching. Each hit checks mana shield, armor, and HP separately. |
| Healing (from Healing Sprayer, Holy Bolt, etc.) | Add to current HP, clamped to max HP. Healing is not affected by armor or mana shield. | Healing bypasses damage pipeline entirely. |
| Mana Shield takes raw damage (100 damage with 50 armor) | Mana Shield absorbs the full 100. If shield runs out, remainder goes through armor. | Mana Shield is a trade-off: it protects HP but burns faster because armor doesn't help it. |
| Tower at 1 HP, enemy deals 1000 damage, tower has 500 mana shield | Shield absorbs 500, remaining 500 goes through armor, reduced amount kills tower. | Pipeline processes in order. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Content Database** | Upstream (hard) | Reads upgrade definitions |
| **Arena** | Upstream (hard) | Provides spawn position |
| **Weapon System** | Downstream (hard) | Reads tower's weapon list |
| **Damage Calculation** | Bidirectional (hard) | Sends damage to tower; tower applies armor/shield reduction |
| **Shop System** | Downstream (hard) | Modifies tower stats on purchase |
| **Run Manager** | Downstream (hard) | Death triggers Defeat |
| **HUD** | Downstream (soft) | Displays tower stats |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Starting HP | 1500 | 500–5000 | More forgiving early game; greed is safer | Punishing early game; first weapon purchase is critical |
| Starting armor | 0 | 0–10 | Free damage reduction from start; early defensive upgrades less impactful | Every point of armor from upgrades matters more |
| Armor constant | 0.06 | 0.03–0.10 | Armor is more effective per point; fewer upgrades needed to tank | Armor is less effective; need more investment for meaningful reduction |

**Knob interactions**: Starting HP interacts with economy (Philosopher's Stone
trades -1000 HP for +2000 gold — at 1500 starting HP, that's 2/3 of your health).
Armor constant interacts with all armor upgrade values.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Tower takes damage | Brief flash/shake on tower sprite | Impact thud sound | High |
| Mana shield absorbs hit | Blue/purple flash instead of red | Magical absorption sound | Medium |
| Mana shield breaks (reaches 0) | Shield shatter effect | Glass break sound | High |
| Tower destroyed | Destruction animation, rubble | Collapse/explosion sound | High |
| HP regen tick | Subtle green particles (optional) | None (too frequent) | Low |

## UI Requirements

| Information | Display Location | Update Frequency | Condition |
|-------------|-----------------|-----------------|-----------|
| Current HP / Max HP | HUD — prominent bar | Every frame | Always during run |
| Mana Shield | HUD — overlaid on HP bar or separate bar | Every frame | When mana shield > 0 |
| Armor value | HUD — near HP bar | On change | When armor > 0 |
| HP Regen | HUD — small indicator | On change | When regen > 0 |

## Acceptance Criteria

- [ ] Tower spawns at (0,0) with 1500 HP, 0 armor, 0 regen, 0 mana shield
- [ ] Armor reduces damage using WC3 formula: 10 armor reduces 100 damage to 63
- [ ] Negative armor amplifies damage up to 2x cap
- [ ] Mana Shield absorbs raw damage BEFORE armor is applied
- [ ] When Mana Shield runs out, remaining damage passes through armor normally
- [ ] HP cannot exceed max HP
- [ ] HP reaching 0 triggers Defeat exactly once
- [ ] Buying a weapon adds an independent instance to the weapon list
- [ ] Buying a duplicate weapon results in two entries, not a merged/stacked entry
- [ ] Upgrades that reduce max HP can kill the tower if new max ≤ 0
- [ ] HP regen ticks per frame using delta_time (not per-second intervals)
- [ ] All stats reset to starting values on run restart
- [ ] Flat damage reduction cannot reduce a hit below 25% of its post-armor value

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Does mana shield regenerate over time, or only through upgrades/on-kill effects? | Mana Shield GDD | When Mana Shield is designed | Current upgrades suggest on-kill and flat amounts only |
| Should armor be displayed as a number or as a % reduction? | HUD GDD | When HUD is designed | % reduction is more intuitive for players |
| Can the tower be healed above its original 1500 HP by max HP upgrades? | Confirmed | — | Yes — upgrades like Improved Masonry (+500 Max HP) increase the cap |
