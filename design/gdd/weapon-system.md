# Weapon System

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story

## Overview

The Weapon System controls all offensive output from the tower. The tower
maintains a list of equipped weapons — each an independent instance that fires
automatically at a random enemy within range. Weapons are defined by 6 attack
pattern archetypes (Single Target, Splash, Bounce, Barrage, Area, Wave) plus
Spikes (passive melee retaliation). Buying a duplicate weapon adds another
independent instance; there is no stacking math. 10 copies of Frost Bow = 10
independent projectile streams. The Weapon System reads weapon definitions from
the Content Database and creates projectile entities via the Projectile System.

## Player Fantasy

Your weapons are your identity. When you've stacked five Fire Bows and the
screen fills with flaming arrows, that's YOUR build. The auto-fire means you
never worry about aiming — you're a commander, not a soldier. Your job is
choosing what to buy, not where to shoot. The satisfaction is watching your
build's output scale: one Frost Bomb is a slow thud, three is a blizzard,
five is an extinction event.

## Detailed Design

### Core Rules

1. **Weapon List**: The tower holds an ordered list of weapon instances. Each
   instance is independent — it has its own cooldown timer, picks its own
   target, and fires its own projectiles. The list starts empty.

2. **Adding Weapons**: When a weapon is purchased from the shop, a new weapon
   instance is appended to the tower's weapon list. The instance references the
   weapon definition from the Content Database. Duplicates are separate entries.

3. **Firing Loop**: Each frame, the Weapon System iterates all weapon instances:
   ```
   for each weapon_instance in tower.weapons:
       weapon_instance.cooldown_timer -= delta_time
       if weapon_instance.cooldown_timer <= 0:
           target = pick_random_enemy_in_range(weapon_instance.range)
           if target exists:
               fire(weapon_instance, target)
               weapon_instance.cooldown_timer = weapon_instance.attack_cooldown
   ```

4. **Target Selection**: Each weapon picks a random enemy within its range.
   Range is measured as distance from tower center (0,0) to enemy position.
   If no enemy is in range, the weapon does not fire (cooldown does not reset).

5. **Damage Types**: Each weapon has one or two damage types (e.g., "Piercing",
   "Piercing and Frost"). The primary type determines the damage matrix
   multiplier. Secondary types (Fire, Frost, Poison) trigger status effect
   application handled by the Status Effects system (Vertical Slice).

6. **Attack Patterns**: The weapon's AttackType determines how the projectile
   behaves after firing. There are 6 core patterns plus Spikes:

### Attack Pattern Archetypes

#### Single Target
- Fires one projectile at one random enemy in range.
- Projectile travels to target, deals weapon damage on hit.
- The simplest pattern. Most Common weapons use this.
- Examples: Bow, Magic Missile, Throwing Axes, Chaos Orb

#### Splash (radius)
- Fires one projectile at one random enemy in range.
- On hit, deals weapon damage to the primary target AND all enemies
  within the splash radius (centered on impact point).
- Splash radius is defined per weapon (e.g., "Splash (150)", "Splash (300)").
- All targets in splash take full damage (no falloff).
- Examples: Healing Sprayer (150), Mortar Launcher (300), Fire Bow (150)

#### Bounce (N targets)
- Fires one projectile at one random enemy in range.
- On hit, projectile bounces to the nearest enemy within bounce range.
- Repeats up to N-1 additional times (total N targets hit).
- Each bounce deals full weapon damage (no decay).
- Cannot hit the same enemy twice in one bounce chain.
- If no valid bounce target exists, chain ends.
- Examples: Bouncy Cannonball (8), Seeker Axe (4), Moon Glaive (4)

#### Barrage (N targets)
- Fires N projectiles simultaneously, each at a different random enemy
  in range.
- Each projectile deals full weapon damage independently.
- If fewer than N enemies are in range, fires one projectile per available
  enemy (no wasted shots).
- Examples: Ice Spears (4), Missile Barrage (4), Poisonspitter (4)

#### Area (radius)
- Deals damage to all enemies within a radius centered on a point.
- The point may be centered on a random enemy in range, or centered on
  the tower, depending on the weapon.
- No projectile travels — damage is instant in the area.
- Examples: Flame Pillar (150), Holy Bolt (150), Necromancer's Tome

#### Wave (+range)
- Emits a wave outward from the tower that expands by the listed range.
- The wave damages all enemies it passes through.
- Wave starts at the tower and expands to (weapon range + wave bonus range).
- Examples: Shockwave Axe (+300), Slicerglaive (+300), Chaos Swarm (+300)

#### Spikes (passive)
- Not a fired weapon. Spikes deal damage to enemies that melee attack
  the tower.
- When an enemy attacks the tower, the tower deals spikes damage back
  to the attacker.
- Spikes damage is the sum of all spikes damage sources (base spikes
  upgrades + weapon-based spikes).
- Spikes damage is its own damage type for the damage matrix.
- Examples: Spiked Barricades (+20 spikes damage), Spikewheel Launcher

### Weapon Stats Reference

From Content Database, each weapon instance uses:

| Stat | Type | Description |
|------|------|-------------|
| Name | string | Display name |
| Rarity | enum | Common, Uncommon, Rare, Epic |
| Weapon | string | Damage type(s) — e.g., "Piercing and Frost" |
| AttackType | string | Pattern + params — e.g., "Splash (300)" |
| Damage | int | Damage per hit |
| DPS | int | Theoretical DPS (informational, not used in calculations) |
| AttackCooldown | float | Seconds between attacks |
| Range | int | Attack range from tower center |
| Ability | string | Special ability text (deferred to Vertical Slice) |

### States and Transitions

Weapon instances have no complex states. They are either:
- **Active**: Cycling through cooldown → target → fire.
- **Inactive**: Run is not in an active state (Paused, Victory, Defeat, MainMenu).

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Content Database** | Reads | Gets weapon definitions (stats, attack type, damage type) |
| **Tower Entity** | Reads/Writes | Reads weapon list. Shop System writes to weapon list on purchase. |
| **Enemy System** | Reads | Queries enemies within range for target selection |
| **Projectile System** | Sends commands | Creates projectile entities for each fired weapon |
| **Damage Calculation** | Sends damage events | When a projectile hits, sends damage to Damage Calculation with weapon damage type |
| **Shop System** | Receives additions | Shop adds weapons to the tower's weapon list |
| **Status Effects** | Sends trigger | Weapons with element subtypes (Fire, Frost, Poison) trigger status effect application on hit (Vertical Slice) |
| **HUD** | Exposes data | HUD may display equipped weapons list |
| **Audio System** | Sends fire events | Each weapon fire triggers an SFX event |
| **VFX / Juice** | Sends fire events | Each weapon fire triggers visual effects |

## Formulas

### Effective DPS (per weapon instance)

```
effective_dps = damage_per_hit / attack_cooldown
```

This matches the DPS field in the JSON for base values. With upgrades:

```
modified_dps = (damage_per_hit * (1 + damage_bonuses)) / (attack_cooldown / (1 + attack_speed_bonus))
```

### Total Tower DPS

```
total_dps = sum(effective_dps for each weapon_instance)
```

Since each weapon fires independently, total DPS is simply the sum of all
individual weapon DPS values. Duplicate weapons contribute linearly.

### Spikes Damage Per Hit

```
spikes_damage = base_spikes + sum(all spikes upgrade values)
spikes_damage_modified = spikes_damage * (1 + spikes_damage_bonus_percent)
```

Applied each time an enemy attacks the tower.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| No enemies in range | Weapon does not fire. Cooldown does not reset. Weapon waits. | Don't waste shots into empty space. |
| More weapons than enemies | Each weapon independently picks a random target. Multiple weapons may target the same enemy. | Random targeting is simpler than smart distribution and creates natural focus fire. |
| Barrage with fewer enemies than N | Fire one projectile per available enemy. No wasted projectiles. | Barrage adapts to available targets. |
| Bounce target dies mid-chain | Chain continues from dead target's position to nearest alive enemy. | Don't break the chain on a kill — that's satisfying. |
| Bounce with fewer than N enemies in range | Chain hits available enemies and stops early. Cannot hit same enemy twice. | Natural chain termination. |
| Wave hits enemy at max range edge | Damage applies if enemy is within the wave's area at the time of expansion. | Generous hitbox — waves should feel powerful. |
| Splash on enemy at tower center with 100 enemies stacked | All 100 enemies in splash radius take full damage. | This is intentional — splash is the counter to clumping. |
| Attack cooldown of 0 | Should never occur (Content Database validation skips these). If it does, treat as 0.1. | Infinite fire rate would crash the system. |
| Weapon with 0 damage | Weapon fires but deals 0 damage. May still apply status effects. | Zero-damage weapons could exist for pure utility (future design space). |
| 88 weapons all equipped (theoretical max) | System must handle it. Each fires independently. | Unlikely in practice but must not crash. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Content Database** | Upstream (hard) | Weapon definitions |
| **Tower Entity** | Upstream (hard) | Weapon list lives on the tower |
| **Enemy System** | Upstream (hard) | Needs enemies to target |
| **Projectile System** | Downstream (hard) | Creates projectiles |
| **Damage Calculation** | Downstream (hard) | Sends damage events |
| **Shop System** | Upstream (hard) | Receives weapon additions |
| **Status Effects** | Downstream (soft) | Triggers element effects (Vertical Slice) |
| **HUD** | Downstream (soft) | Displays equipped weapons |
| **Audio System** | Downstream (soft) | Fire SFX |
| **VFX / Juice** | Downstream (soft) | Fire VFX |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| All weapon damage values | Per weapon (see JSON) | — | Higher DPS; enemies die faster; runs feel easier | Lower DPS; runs feel harder; more weapons needed |
| All weapon cooldowns | Per weapon (see JSON) | 0.1–5.0 | Slower fire rate; each shot matters more | Faster fire rate; more projectiles on screen |
| All weapon ranges | Per weapon (300–1200) | 100–2000 | Weapons engage earlier; more time to kill before enemies arrive | Weapons engage later; more enemies reach tower |
| Bounce decay | None (full damage) | 0%–50% per bounce | Later bounces deal less; first target matters most | Even damage distribution across chain |
| Splash falloff | None (full damage) | 0%–50% at edge | Edge targets take less; rewards precise targeting (N/A with random targeting) | Even damage in splash zone |

**Knob interactions**: Weapon damage interacts with enemy HP (time-to-kill),
weapon cooldowns interact with attack speed upgrades (Rapidfire: +10% attack
speed), and weapon ranges interact with arena radius and enemy move speed.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Weapon fires | Muzzle flash / launch animation at tower | Weapon-specific fire sound | High |
| Projectile in flight | Projectile sprite moving toward target | None (fire sound covers it) | High |
| Projectile hits enemy | Impact flash on enemy | Impact sound | High |
| Splash damage area | Brief circular flash at impact point | Explosion/splash sound | High |
| Wave expansion | Expanding ring/arc visual from tower | Whoosh/wave sound | High |
| Bounce chain | Projectile visibly arcs between targets | Bounce/ricochet sound per hop | Medium |
| Spikes damage | Brief thorns/spike visual on attacking enemy | Spike/crunch sound | Medium |

## UI Requirements

| Information | Display Location | Update Frequency | Condition |
|-------------|-----------------|-----------------|-----------|
| Equipped weapons list | HUD or inventory panel | On weapon purchase | Always during run |
| Weapon count per type | HUD — grouped by weapon name with count | On weapon purchase | When duplicates exist |

## Acceptance Criteria

- [ ] Each weapon instance fires independently with its own cooldown timer
- [ ] Buying a duplicate weapon adds a new independent instance (not merged)
- [ ] Weapons pick a random enemy within range as target
- [ ] Weapons do not fire when no enemy is in range
- [ ] Single Target pattern: one projectile, one target, correct damage
- [ ] Splash pattern: one projectile, damages all enemies within splash radius on hit
- [ ] Bounce pattern: projectile bounces to N targets, no same-enemy repeats
- [ ] Barrage pattern: fires N projectiles at N different enemies simultaneously
- [ ] Area pattern: instant damage to all enemies in radius
- [ ] Wave pattern: expanding wave from tower, damages all enemies in path
- [ ] Spikes: deals damage to enemies that attack the tower
- [ ] Weapon damage type is passed to Damage Calculation for armor multiplier
- [ ] All weapon stats come from Content Database (no hardcoded values)
- [ ] System handles 20+ simultaneous weapon instances at 60fps
- [ ] Weapons freeze during Paused state

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| How should special weapon behaviors (Flamecaster rotation, Necromancer summon, Ice Beam lock-on) be implemented? | Weapon System v2 | Vertical Slice | Deferred — MVP uses only the 6 standard attack patterns |
| Should the Ability field be parsed into structured data or kept as freeform text? | Architecture Decision | Before implementation | Structured data is cleaner but requires defining all ability types upfront |
| How do damage bonuses from upgrades apply — per weapon type, per damage type, or globally? | Damage Calculation GDD | When Damage Calc is designed | Upgrade descriptions suggest per-type bonuses (e.g., "+10% Piercing Damage") |
| Should weapons have a visual representation on the tower sprite (mounted weapons)? | Art Director | Alpha | Would look cool but adds significant art scope |
| What is the projectile speed for each attack pattern? | Projectile System GDD | When Projectile System is designed | Likely fixed per pattern, not per weapon |
