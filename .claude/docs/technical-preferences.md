# Technical Preferences

<!-- Populated by /setup-engine. Updated as the user makes decisions throughout development. -->
<!-- All agents reference this file for project-specific standards and conventions. -->

## Engine & Language

- **Engine**: Bevy 0.18.1
- **Language**: Rust (latest stable)
- **Rendering**: Bevy 2D renderer (sprite-based isometric)
- **Physics**: TBD — evaluate bevy_rapier vs. custom simple collision

## Naming Conventions

- **Types / Structs / Enums**: PascalCase (e.g., `WeaponStats`, `ShopState`)
- **Functions / Methods**: snake_case (e.g., `spawn_enemy`, `calculate_dps`)
- **Variables / Fields**: snake_case (e.g., `attack_cooldown`, `max_health`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_WAVE_DURATION`, `SHOP_REFRESH_INTERVAL`)
- **Modules / Files**: snake_case (e.g., `weapon_system.rs`, `shop_ui.rs`)
- **Components**: PascalCase noun (e.g., `Health`, `Weapon`, `GoldReward`)
- **Resources**: PascalCase noun (e.g., `ShopInventory`, `WaveState`, `GameEconomy`)
- **Systems**: snake_case verb phrase (e.g., `fire_weapons`, `spawn_wave`, `update_shop`)
- **SystemSets**: PascalCase with `Systems` suffix (e.g., `CombatSystems`, `ShopSystems`)
- **Plugins**: PascalCase with `Plugin` suffix (e.g., `WeaponPlugin`, `EconomyPlugin`)
- **Events / Messages**: PascalCase past tense (e.g., `EnemyKilled`, `WeaponPurchased`)

## Performance Budgets

- **Target Framerate**: 60fps
- **Frame Budget**: 16.6ms
- **Draw Calls**: [TO BE CONFIGURED — profile after first prototype]
- **Memory Ceiling**: [TO BE CONFIGURED — profile after first prototype]
- **Entity Budget**: Must sustain 500+ simultaneous entities (enemies + projectiles) at 60fps

## Testing

- **Framework**: `cargo test` + Bevy's built-in test utilities
- **Minimum Coverage**: [TO BE CONFIGURED]
- **Required Tests**: Balance formulas, economy math, damage calculations, weapon stacking logic

## Forbidden Patterns

- Hardcoded gameplay values — all balance data must live in external files (JSON/RON)
- `unsafe` blocks without documented justification
- Blocking I/O on the main thread
- Direct system-to-system coupling — use events/messages for cross-system communication

## Allowed Libraries / Addons

<!-- Add approved third-party dependencies here -->
- [None configured yet — add as dependencies are approved]
- Candidates to evaluate: `bevy_rapier` (physics), `bevy_kira_audio` (audio), `bevy_egui` (debug UI)

## Architecture Decisions Log

<!-- Quick reference linking to full ADRs in docs/architecture/ -->
- **ADR-0001**: [Bevy Plugin Structure](../../docs/architecture/adr-0001-bevy-plugin-structure.md) — 7 domain-based plugins (Content, Run, Core, Enemy, Combat, Economy, UI)
