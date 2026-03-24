# Systems Index: Tower of Doom

> **Status**: Draft
> **Created**: 2026-03-22
> **Last Updated**: 2026-03-22
> **Source Concept**: design/gdd/game-concept.md

---

## Overview

Tower of Doom is a 23-system game built around a tight core loop: a stationary
tower auto-fires weapons at approaching enemies while the player manages a live
shop economy. The systems divide into a data-driven foundation (content loading,
game state), a gameplay layer (weapons, enemies, damage, economy), a presentation
layer (HUD, shop UI, end-of-run), and a polish layer (audio, VFX, menus).

The core pillars — Greed Kills, Your Build Your Story, Fifteen Minutes of Escalation
— mean the Shop System and Weapon System are disproportionately important. The
shop UI in particular is the highest-risk system: it must be fast, readable, and
usable under combat pressure.

---

## Systems Enumeration

| # | System Name | Category | Priority | Status | Design Doc | Depends On |
|---|-------------|----------|----------|--------|------------|------------|
| 1 | Content Database | Core | MVP | Designed | design/gdd/content-database.md | (none) |
| 2 | Run Manager | Core | MVP | Designed | design/gdd/run-manager.md | (none) |
| 3 | Arena | Core | MVP | Designed | design/gdd/arena.md | (none) |
| 4 | Isometric Camera | Core | MVP | Designed | design/gdd/isometric-camera.md | (none) |
| 5 | Enemy Data | Core | MVP | Designed | design/gdd/enemy-data.md | Content Database |
| 6 | Tower Entity | Gameplay | MVP | Designed | design/gdd/tower-entity.md | Content Database, Arena |
| 7 | Gold Economy | Economy | MVP | Designed | design/gdd/gold-economy.md | Run Manager |
| 8 | Enemy System | Gameplay | MVP | Designed | design/gdd/enemy-system.md | Enemy Data, Arena, Run Manager |
| 9 | Weapon System | Gameplay | MVP | Designed | design/gdd/weapon-system.md | Content Database, Tower Entity, Enemy System |
| 10 | Projectile System | Gameplay | MVP | Designed | design/gdd/projectile-system.md | Weapon System |
| 11 | Damage Calculation | Gameplay | MVP | Designed | design/gdd/damage-calculation.md | Content Database, Enemy Data, Tower Entity |
| 12 | Wave Escalation | Gameplay | MVP | Designed | design/gdd/wave-escalation.md | Enemy Data, Enemy System, Run Manager |
| 13 | Shop System | Economy | MVP | Designed | design/gdd/shop-system.md | Content Database, Gold Economy, Weapon System |
| 14 | HUD | UI | MVP | Designed | design/gdd/hud.md | Tower Entity, Gold Economy, Run Manager, Weapon System |
| 15 | Shop UI | UI | MVP | Designed | design/gdd/shop-ui.md | Shop System, Gold Economy |
| 16 | Status Effects | Gameplay | Vertical Slice | Designed | design/gdd/status-effects.md | Damage Calculation, Enemy System |
| 17 | Mana Shield | Gameplay | Vertical Slice | Designed | design/gdd/mana-shield.md | Tower Entity, Damage Calculation |
| 18 | Boss Encounter | Gameplay | Vertical Slice | Designed | design/gdd/boss-encounter.md | Enemy Data, Enemy System, Wave Escalation, Run Manager, Damage Calculation |
| 19 | Run Statistics | Meta | Vertical Slice | Designed | design/gdd/run-statistics.md | Gold Economy, Weapon System, Enemy System, Run Manager |
| 20 | End-of-Run Screen | UI | Vertical Slice | Designed | design/gdd/end-of-run-screen.md | Run Manager, Run Statistics |
| 21 | Audio System | Audio | Alpha | Not Started | — | Weapon System, Enemy System, Shop System, Run Manager |
| 22 | VFX / Juice | Meta | Alpha | Not Started | — | Projectile System, Damage Calculation, Enemy System |
| 23 | Main Menu | UI | Full Vision | Not Started | — | Run Manager |

---

## Categories

| Category | Description |
|----------|-------------|
| **Core** | Foundation systems everything depends on — data loading, game state, rendering, arena |
| **Gameplay** | Systems that drive the moment-to-moment game — weapons, enemies, damage, status effects |
| **Economy** | Resource creation and consumption — gold, shop, pricing |
| **UI** | Player-facing displays — HUD, shop interface, menus, end-of-run |
| **Audio** | Sound effects and music |
| **Meta** | Systems outside the core loop — VFX polish, statistics tracking |

---

## Priority Tiers

| Tier | Definition | Systems | Design Urgency |
|------|------------|---------|----------------|
| **MVP** | Required for the core loop to function. Tests: "Is buying weapons from a live shop during auto-combat fun?" | 15 systems | Design FIRST |
| **Vertical Slice** | Complete experience with boss, status effects, and end-of-run feedback | 5 systems | Design SECOND |
| **Alpha** | Audio and visual polish | 2 systems | Design THIRD |
| **Full Vision** | Main menu and any remaining polish | 1 system | Design as needed |

---

## Dependency Map

### Foundation Layer (no dependencies)

1. **Content Database** — Everything reads weapon, upgrade, and enemy data from it
2. **Run Manager** — Game state machine; other systems check run phase
3. **Isometric Camera** — Rendering foundation; nothing displays without it
4. **Arena** — Playfield with boundaries and spawn zones

### Core Layer (depends on Foundation)

5. **Enemy Data** — depends on: Content Database
6. **Tower Entity** — depends on: Content Database, Arena
7. **Gold Economy** — depends on: Run Manager
8. **Enemy System** — depends on: Enemy Data, Arena, Run Manager

### Feature Layer (depends on Core)

9. **Weapon System** — depends on: Content Database, Tower Entity, Enemy System
10. **Projectile System** — depends on: Weapon System
11. **Damage Calculation** — depends on: Content Database, Enemy Data, Tower Entity
12. **Wave Escalation** — depends on: Enemy Data, Enemy System, Run Manager
13. **Shop System** — depends on: Content Database, Gold Economy, Weapon System
14. **Status Effects** — depends on: Damage Calculation, Enemy System
15. **Mana Shield** — depends on: Tower Entity, Damage Calculation

### Presentation Layer (depends on Features)

16. **HUD** — depends on: Tower Entity, Gold Economy, Run Manager, Weapon System
17. **Shop UI** — depends on: Shop System, Gold Economy
18. **Boss Encounter** — depends on: Enemy Data, Enemy System, Wave Escalation, Run Manager, Damage Calculation
19. **Run Statistics** — depends on: Gold Economy, Weapon System, Enemy System, Run Manager
20. **End-of-Run Screen** — depends on: Run Manager, Run Statistics

### Polish Layer

21. **VFX / Juice** — depends on: Projectile System, Damage Calculation, Enemy System
22. **Audio System** — depends on: Weapon System, Enemy System, Shop System, Run Manager
23. **Main Menu** — depends on: Run Manager

---

## Recommended Design Order

### MVP Systems (design first)

| Order | System | Priority | Layer | Est. Effort |
|-------|--------|----------|-------|-------------|
| 1 | Content Database | MVP | Foundation | S |
| 2 | Run Manager | MVP | Foundation | S |
| 3 | Arena | MVP | Foundation | S |
| 4 | Isometric Camera | MVP | Foundation | S |
| 5 | Enemy Data | MVP | Foundation | M |
| 6 | Tower Entity | MVP | Core | S |
| 7 | Gold Economy | MVP | Core | M |
| 8 | Enemy System | MVP | Core | M |
| 9 | Weapon System | MVP | Feature | L |
| 10 | Projectile System | MVP | Feature | M |
| 11 | Damage Calculation | MVP | Feature | M |
| 12 | Wave Escalation | MVP | Feature | M |
| 13 | Shop System | MVP | Feature | L |
| 14 | HUD | MVP | Presentation | M |
| 15 | Shop UI | MVP | Presentation | L |

### Vertical Slice Systems (design second)

| Order | System | Priority | Layer | Est. Effort |
|-------|--------|----------|-------|-------------|
| 16 | Status Effects | Vertical Slice | Feature | M |
| 17 | Mana Shield | Vertical Slice | Feature | S |
| 18 | Boss Encounter | Vertical Slice | Feature | M |
| 19 | Run Statistics | Vertical Slice | Presentation | S |
| 20 | End-of-Run Screen | Vertical Slice | Presentation | S |

### Alpha and Full Vision Systems (design later)

| Order | System | Priority | Layer | Est. Effort |
|-------|--------|----------|-------|-------------|
| 21 | Audio System | Alpha | Polish | M |
| 22 | VFX / Juice | Alpha | Polish | M |
| 23 | Main Menu | Full Vision | Polish | S |

Effort estimates: S = 1 session, M = 2-3 sessions, L = 4+ sessions.

---

## Circular Dependencies

None found. The dependency graph is a clean DAG.

---

## High-Risk Systems

| System | Risk Type | Risk Description | Mitigation |
|--------|-----------|-----------------|------------|
| Shop UI | Technical | The entire game hinges on the shop being fast and usable during combat. Bevy's UI ecosystem is immature — may need custom widgets or a third-party crate. | Prototype shop UI early. Evaluate bevy_egui, Bevy native UI, or custom solution. |
| Weapon System | Scope | 88 weapons with 6 distinct attack patterns (single target, splash, bounce, barrage, area, wave). Each pattern needs unique projectile behavior. Largest system by content volume. | Design the 6 attack pattern archetypes first. Individual weapons are data variants, not code variants. |
| Damage Calculation | Design | 5 base damage types × enemy armor types interaction matrix. Must be correct for balance to work. WC3-inspired but needs its own tuning. | Define the damage/armor matrix in the GDD with concrete numbers. Prototype with 2-3 types first, expand once validated. |
| Isometric Camera | Technical | Bevy has no built-in isometric support. Need custom sprite sorting, z-ordering, and camera setup. | Research existing Bevy iso crates or implement minimal iso camera early in prototype. |

---

## Progress Tracker

| Metric | Count |
|--------|-------|
| Total systems identified | 23 |
| Design docs started | 20 |
| Design docs reviewed | 0 |
| Design docs approved | 0 |
| MVP systems designed | 15/15 |
| Vertical Slice systems designed | 5/5 |

---

## Design Notes

### Weapon "Stacking"

There is no special stacking mechanic. Buying a duplicate weapon adds another
independent instance to the tower's weapon list. Each instance fires at its own
rate, picks its own random target within range, and deals its own damage. 10
copies of Frost Bow = 10 independent Frost Bow projectiles. Power scales linearly
with copies purchased. The economy (gold cost) is the balancing lever, not
diminishing returns.

### Spikes

Spikes is a weapon/damage type, not a separate system. Spikes weapons and upgrades
live within the Weapon System and Damage Calculation systems. Spikes damage is
dealt passively to melee attackers — mechanically it's a weapon with "on hit by
enemy" as its trigger instead of a cooldown timer.

---

## Next Steps

- [ ] Review and approve this systems enumeration
- [ ] Design MVP-tier systems first (use `/design-system [system-name]`)
- [ ] Run `/design-review` on each completed GDD
- [ ] Run `/gate-check pre-production` when MVP systems are designed
- [ ] Prototype the highest-risk system early (`/prototype shop-ui`)
