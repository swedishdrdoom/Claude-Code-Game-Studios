# Enemy Data

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story

## Overview

Enemy Data defines the static properties of every enemy type in the game —
HP, damage, speed, armor type, gold bounty, and visual identity. It also
owns the damage type vs. armor type interaction matrix, which determines how
much damage each weapon type deals to each enemy. Enemy Data is loaded through
the Content Database from `assets/content/enemies.json` and the damage matrix
is defined in `assets/content/damage-matrix.json`.

## Player Fantasy

Enemies are the escalating pressure that makes the shop matter. Each armor type
creates a puzzle: "What weapons do I need to deal with this?" When Heavy-armored
tanks start appearing mid-run and your Piercing weapons bounce off, the player
feels the urgency to adapt. This serves "Your Build, Your Story" — the enemy
composition shapes which builds are viable and forces adaptation.

## Detailed Design

### Core Rules

1. **Armor Types** (6 total):

   | Armor Type | Typical Enemy | Design Role |
   |------------|--------------|-------------|
   | Light | Fast swarmers | Piercing food. Die fast, come in numbers. |
   | Medium | Standard enemies | The baseline. Normal damage excels here. |
   | Heavy | Slow tanks | High HP, Magic-vulnerable. Punish single-target builds. |
   | Fortified | Elite/armored units | Resist almost everything. Siege is the answer. |
   | Hero | Boss only | Resists Piercing/Siege/Magic. Normal and Chaos work. |
   | Unarmored | Weak fodder | Easy kills. Piercing and Siege get bonus damage. |

2. **Damage Multiplier Matrix**:

   | | Light | Medium | Heavy | Fortified | Hero | Unarmored |
   |---|---|---|---|---|---|---|
   | **Normal** | 100% | 150% | 100% | 70% | 100% | 100% |
   | **Piercing** | 200% | 75% | 100% | 35% | 50% | 150% |
   | **Siege** | 100% | 50% | 100% | 150% | 50% | 150% |
   | **Magic** | 125% | 75% | 200% | 35% | 50% | 100% |
   | **Chaos** | 100% | 100% | 100% | 100% | 100% | 100% |

   Final damage = weapon_damage × matrix_multiplier × other_modifiers

3. **Enemy Data Schema** (JSON):
   - `Name`: string — display name
   - `ArmorType`: enum — Light, Medium, Heavy, Fortified, Hero, Unarmored
   - `MaxHP`: integer — hit points
   - `Damage`: integer — damage per attack to tower
   - `AttackCooldown`: float — seconds between attacks
   - `MoveSpeed`: float — units per second toward tower
   - `GoldBounty`: integer — gold awarded on kill
   - `FrostResistance`: float — multiplier on Frost stack application (0.0–1.0)
   - `StunResistance`: float — multiplier on Stun duration (0.0–1.0)
   - `Image`: string — sprite path

4. **Hero Armor Special Properties**:
   - Significantly reduced Frost/Slow effectiveness (high FrostResistance)
   - Significantly reduced Stun duration (high StunResistance)
   - Only one Hero-armor enemy exists: the boss

5. **Enemy types are defined in data, not code**. Adding a new enemy type
   means adding a JSON entry, not writing new systems. Enemy behaviors
   (special abilities, movement patterns) are composed from shared
   behavior components, not per-enemy code.

### States and Transitions

Enemy Data is static — loaded at startup, immutable at runtime. No states.

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Content Database** | Upstream | Enemy Data is loaded through the Content Database pipeline |
| **Enemy System** | Downstream | Reads enemy definitions to spawn enemies with correct stats |
| **Damage Calculation** | Downstream | Reads armor type to look up damage multiplier from the matrix |
| **Wave Escalation** | Downstream | Reads enemy type list to compose waves (e.g., "wave 5 introduces Heavy enemies") |
| **Boss Encounter** | Downstream | Reads the Hero-armor enemy definition for boss stats |
| **Status Effects** | Downstream | Reads FrostResistance and StunResistance for effect application |

## Formulas

### Damage After Armor

```
effective_damage = base_damage * damage_matrix[weapon_type][armor_type] * bonus_modifiers
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| base_damage | int | 75–6000 | Weapon definition | Raw weapon damage per hit |
| damage_matrix | float | 0.35–2.0 | Damage matrix JSON | Multiplier for this weapon type vs this armor type |
| bonus_modifiers | float | 1.0+ | Upgrades, status effects | Stacked % damage bonuses |

**Expected output range**: ~26 (weakest weapon vs Fortified at 35%) to 12,000+
(strongest weapon vs Heavy at 200% with upgrades)

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Enemy with armor type not in matrix | Default to 100% for all damage types. Log warning. | Fail safe, don't crash. |
| Enemy with 0 HP | Skip spawning. Log warning. | Dead on arrival makes no sense. |
| Enemy with 0 MoveSpeed | Spawns but never reaches tower. Still targetable. | Could be used for stationary enemies in future. Log warning for now. |
| Enemy with 0 GoldBounty | Valid — some enemies might not give gold (adds design space). | Zero bounty enemies as a difficulty mechanic. |
| Multiple Hero-armor enemies defined | Allowed but warned — design intent is one boss only. | Don't prevent it technically; warn at data validation. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Content Database** | Upstream (hard) | Enemy definitions loaded through Content DB pipeline |
| **Enemy System** | Downstream (hard) | Needs enemy stats for spawning |
| **Damage Calculation** | Downstream (hard) | Needs armor type and damage matrix |
| **Wave Escalation** | Downstream (hard) | Needs enemy type list for wave composition |
| **Boss Encounter** | Downstream (hard) | Needs Hero-armor enemy definition |
| **Status Effects** | Downstream (soft) | Needs resistance values for Frost/Stun |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Damage matrix multipliers | See matrix above | 0.25–3.0 | Stronger type advantages; more build dependency on matching types | Flatter damage; type choice matters less |
| Enemy HP values | TBD per enemy type | — | Spongier enemies; longer time-to-kill; AoE/DPS builds favored | Enemies die fast; stacking fewer weapons works |
| Enemy MoveSpeed | TBD per enemy type | — | Less reaction time; close-range builds punished | More shooting time; long-range builds less necessary |
| GoldBounty per enemy | TBD per enemy type | — | More gold income; faster build scaling | Tighter economy; greed-vs-survival tension increases |
| Boss FrostResistance | TBD (high) | 0.5–1.0 | Boss nearly immune to slow/freeze; pure DPS required | Frost builds can lock boss down; trivializes encounter |
| Boss StunResistance | TBD (high) | 0.5–1.0 | Boss nearly immune to stun; stun-lock boss possible; trivializes encounter | Stun builds viable against boss |

## Visual/Audio Requirements

N/A — Enemy Data defines stats, not visuals. Sprite rendering is handled by the
Enemy System. Sound effects are handled by the Audio System.

## UI Requirements

N/A — Enemy stats are not displayed directly to the player. The HUD may show
enemy health bars, but that is owned by the HUD system.

## Acceptance Criteria

- [ ] All enemy types load from enemies.json via Content Database
- [ ] Damage matrix loads from damage-matrix.json
- [ ] Damage matrix lookup returns correct multiplier for all 30 cells (5 damage types × 6 armor types)
- [ ] Hero-armor enemy has reduced Frost and Stun effectiveness
- [ ] Invalid armor types default to 100% multiplier with a warning
- [ ] Enemy definitions with missing required fields are skipped with a warning
- [ ] At least 3 enemy types defined for MVP (covering at least 3 different armor types)
- [ ] Boss enemy uses Hero armor type

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| What are the specific enemy types for the demo (names, stats, armor types)? | Game Designer | Before prototype | Need 3-4 concrete enemy definitions |
| How do element subtypes (Fire, Frost, Poison) interact with armor? Same matrix or separate? | Damage Calculation GDD | When Damage Calc is designed | Element damage may bypass armor or use its own multipliers |
| Should enemy stats scale with wave escalation, or are they fixed per type with harder types introduced later? | Wave Escalation GDD | When Wave Escalation is designed | Both approaches have trade-offs |
