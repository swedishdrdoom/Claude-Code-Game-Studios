# Content Database

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story

## Overview

The Content Database is the data backbone of Tower of Doom. It loads all game
content definitions — weapons, upgrades, and enemy types — from external JSON
files at startup and serves them to gameplay systems at runtime via typed Bevy
resources. Players never interact with this system directly; it exists so that
all gameplay values (damage, cooldowns, costs, abilities) are data-driven rather
than hardcoded, enabling rapid balance iteration without recompilation. Every
weapon the shop offers, every upgrade the player buys, and every enemy that
spawns is defined in the Content Database.

## Player Fantasy

The Content Database is invisible infrastructure — the player never interacts
with it consciously. Its contribution to the player experience is indirect:
it enables the massive build variety (88 weapons × 94 upgrades) that makes
every run feel different. When a player says "I went triple Frost Bomb with
Shatter and it was insane," the Content Database is what made those items
exist as distinct, tunable entities rather than hardcoded special cases.

The designer fantasy this system serves is equally important: any balance
change, new weapon, or new enemy can be added by editing a JSON file and
restarting the game — no recompilation required.

## Detailed Design

### Core Rules

1. **Data Files**: The Content Database loads from three JSON files at startup:
   - `assets/content/weapons.json` — weapon definitions (88 entries)
   - `assets/content/upgrades.json` — upgrade definitions (94 entries)
   - `assets/content/enemies.json` — enemy type definitions (TBD, designed in Enemy Data GDD)

2. **Loading**: All content is loaded once during the app startup phase, before
   the main menu or any run begins. Content is immutable at runtime — no hot-reloading
   during a run. Restart the game to pick up data file changes.

3. **Storage**: Each content type is stored as a Bevy `Resource` containing a
   `Vec` of typed structs. Systems access content via `Res<WeaponDatabase>`,
   `Res<UpgradeDatabase>`, `Res<EnemyDatabase>`.

4. **Rarity-Based Pricing**: Items do not have individual cost fields. Price is
   determined entirely by rarity:

   | Rarity | Gold Cost |
   |--------|-----------|
   | Common | 500 |
   | Uncommon | 2,000 |
   | Rare | 5,000 |
   | Epic | 10,000 |

5. **Weapon Data Schema** (matching existing JSON):
   - `Name`: string — display name
   - `Rarity`: enum — Common, Uncommon, Rare, Epic
   - `Weapon`: string — damage type(s) (e.g., "Piercing and Frost")
   - `AttackType`: string — attack pattern and parameters (e.g., "Splash (300)", "Bounce (8 Targets)")
   - `Damage`: integer — damage per hit
   - `DPS`: integer — theoretical DPS at base attack speed
   - `AttackCooldown`: float — seconds between attacks
   - `Range`: integer — attack range in game units
   - `Ability`: string — special effect description (e.g., "Frost(10 stacks)", "Heal 10 health per enemy hit")
   - `Image`: string — URL/path to icon sprite

6. **Upgrade Data Schema** (matching existing JSON):
   - `Name`: string — display name
   - `Rarity`: enum — Common, Uncommon, Rare, Epic
   - `Type`: string — upgrade category (e.g., "Damage Boost", "Spikes", "Income", "HP")
   - `Description`: string — human-readable effect description
   - `Image`: string — URL/path to icon sprite

7. **Content Validation**: On load, the system validates:
   - All required fields are present and non-empty
   - Rarity values are one of the four valid options
   - Numeric values (Damage, DPS, AttackCooldown, Range) are positive
   - Invalid entries are logged as warnings and skipped (game continues without them)

8. **Lookup**: Downstream systems look up content by name (string key) or by
   filter (e.g., "all weapons of rarity Uncommon", "all upgrades of type Spikes").

### States and Transitions

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| Unloaded | App startup | JSON files parsed successfully | No content available; systems cannot query |
| Loading | Startup system runs | All files parsed + validated | Reads JSON, deserializes, validates |
| Ready | Loading complete | Never (persists for app lifetime) | All content available via Res<> queries |
| Error | Any JSON file missing or unparseable | N/A (fatal) | Log error, prevent game from starting |

The system transitions once: Unloaded → Loading → Ready. There is no runtime
state change. If loading fails, the game does not start.

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Weapon System** | Reads from Content DB | Queries weapon definitions by name to instantiate weapon behavior. Each equipped weapon holds a reference to its definition. |
| **Shop System** | Reads from Content DB | Queries all weapons/upgrades filtered by rarity to generate shop offerings. Uses rarity-to-price mapping for gold costs. |
| **Damage Calculation** | Reads from Content DB | Reads weapon damage types and values. Reads enemy armor types (via Enemy Data). |
| **Tower Entity** | Reads from Content DB | Reads upgrade definitions to apply stat modifications (HP, armor, regen, etc.). |
| **Enemy Data** | Reads from Content DB | Enemy definitions are loaded through the same pipeline. Enemy Data is a content type within the Content Database. |
| **HUD / Shop UI** | Reads from Content DB | Reads item names, descriptions, icons, and rarity for display. |

**Data ownership**: The Content Database owns all static definitions. Runtime
state (e.g., "which weapons does the tower currently have?") is owned by the
respective gameplay system, not the Content Database. The Content Database is
read-only after loading.

## Formulas

### Rarity Price Lookup

```
gold_cost = RARITY_PRICE_TABLE[item.rarity]
```

| Variable | Type | Values | Source | Description |
|----------|------|--------|--------|-------------|
| item.rarity | enum | Common, Uncommon, Rare, Epic | JSON data | The item's rarity tier |
| RARITY_PRICE_TABLE | map | {Common: 500, Uncommon: 2000, Rare: 5000, Epic: 10000} | Content DB config | Fixed price per rarity |

**Expected output range**: 500 to 10,000 gold.

No other formulas live in this system. Damage formulas, economy curves, and
scaling calculations are owned by their respective systems (Damage Calculation,
Gold Economy, Wave Escalation) and reference Content Database values as inputs.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| JSON file missing from disk | Fatal error at startup. Log which file is missing. Game does not start. | The game cannot function without content definitions. Fail fast and clear. |
| JSON file is empty or contains `[]` | Fatal error. Log "no entries found in [filename]". | An empty weapon or upgrade pool means the shop has nothing to sell. |
| Entry has unknown rarity value (e.g., "Legendary") | Skip the entry, log a warning with the entry name and invalid rarity. | Allows partial content loads for development. Don't crash on a typo. |
| Duplicate entry names in the same file | Load both. Log a warning. First match wins for name-based lookups. | During development, duplicates happen. Don't crash, but warn loudly. |
| Entry has negative or zero Damage/DPS/Range | Skip the entry, log a warning. | Negative values would break downstream calculations. |
| Entry has AttackCooldown of 0 or negative | Skip the entry, log a warning. | Zero cooldown = infinite fire rate = system crash. |
| Ability field is empty string or "None" | Treat as "no special ability." Weapon fires but has no status effect or on-hit behavior. | Common weapons have "None" as ability — this is valid. |
| Image path is missing or broken | Load the entry normally, use a placeholder "missing texture" icon. | Art can be added later; missing icons shouldn't block gameplay. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| (none) | Upstream | Content Database has no upstream dependencies. It is the root of the dependency graph. |
| **Enemy Data** | Downstream (hard) | Enemy Data reads enemy type definitions from the Content Database. Cannot function without it. |
| **Tower Entity** | Downstream (hard) | Reads upgrade definitions to know what stat modifications to apply. |
| **Weapon System** | Downstream (hard) | Reads weapon definitions for all attack behavior. Cannot function without it. |
| **Damage Calculation** | Downstream (hard) | Reads damage type values from weapons and armor type values from enemies. |
| **Shop System** | Downstream (hard) | Reads full weapon/upgrade catalogs to generate shop offerings. Uses rarity-price mapping. |
| **HUD** | Downstream (soft) | Reads names and icons for display. HUD could function with placeholder text without it. |
| **Shop UI** | Downstream (soft) | Reads names, descriptions, icons, and rarity for item cards. |

All downstream dependencies are read-only. No system writes back to the Content
Database at runtime.

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Common price | 500 gold | 200–1,000 | Common items feel expensive; players diversify less | Flood of cheap items; builds converge on stacked commons |
| Uncommon price | 2,000 gold | 1,000–3,000 | Uncommons feel like real investments; fewer purchased per run | Uncommons become trivial buys; less tension in shop decisions |
| Rare price | 5,000 gold | 3,000–8,000 | Rares become late-game only; more commitment required | Rares accessible too early; build power spikes sooner |
| Epic price | 10,000 gold | 7,000–15,000 | Epics feel like once-per-run decisions | Epics accessible mid-run; diminishes their impact |

**Knob interactions**: All prices interact with the Gold Economy system's income
rate. Changing prices here without adjusting gold income will shift the entire
economy. The ratio between income and prices matters more than absolute values.

All other tunable values (weapon damage, cooldowns, ranges, enemy stats) live in
the JSON data files themselves — they are tuning knobs by nature. The Content
Database just loads whatever is in the files.

## Visual/Audio Requirements

N/A — The Content Database is invisible infrastructure with no direct visual or
audio output. Item icons referenced in the data are loaded and displayed by the
Shop UI and HUD systems.

## UI Requirements

N/A — No direct UI. Downstream systems (Shop UI, HUD) are responsible for
displaying content data to the player.

## Acceptance Criteria

- [ ] All three JSON files (weapons, upgrades, enemies) load successfully at startup
- [ ] `Res<WeaponDatabase>` contains 88 entries after loading weapons.json
- [ ] `Res<UpgradeDatabase>` contains 94 entries after loading upgrades.json
- [ ] Rarity-to-price lookup returns correct values: Common=500, Uncommon=2000, Rare=5000, Epic=10000
- [ ] Querying weapons by rarity returns the correct subset (e.g., 5 Common weapons)
- [ ] Querying upgrades by type returns the correct subset (e.g., 29 Damage Boost upgrades)
- [ ] Invalid entries (missing fields, bad rarity, zero cooldown) are skipped with a logged warning
- [ ] Game refuses to start if a JSON file is missing, with a clear error message
- [ ] Game refuses to start if a JSON file contains zero valid entries
- [ ] Content loading completes in under 100ms on target hardware
- [ ] No hardcoded gameplay values in the Content Database implementation — all values come from JSON
- [ ] Content is immutable after loading — no system can modify definitions at runtime

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should the Ability field be parsed into structured data (enum + params) or kept as a freeform string for the Weapon System to interpret? | Weapon System GDD | When Weapon System is designed | Depends on how ability effects are implemented |
| What fields does enemies.json need? | Enemy Data GDD | When Enemy Data is designed | Schema TBD in Enemy Data system design |
| Should we use Bevy's built-in asset system or raw file I/O for JSON loading? | Architecture decision | Before implementation | Bevy AssetServer supports custom asset types; evaluate complexity vs. raw serde_json |
