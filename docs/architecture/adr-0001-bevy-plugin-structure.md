# ADR-0001: Bevy Plugin Structure

## Status
Accepted

## Date
2026-03-22

## Context

### Problem Statement
Tower of Doom has 23 designed game systems that need to be organized into a
maintainable Bevy codebase. We need to decide how to structure the code into
Bevy plugins, what module hierarchy to use, and how plugins communicate. This
decision shapes the entire codebase and is hard to change later.

### Constraints
- Bevy 0.18.1 ECS architecture (entities, components, resources, systems, events)
- Single developer — must be navigable by one person
- 23 game systems mapped in `design/gdd/systems-index.md`
- Systems have clear dependency relationships (DAG, no cycles)
- Need to support incremental development (MVP → Vertical Slice → Alpha)

### Requirements
- Must support adding systems incrementally without refactoring existing code
- Must enforce separation of concerns between gameplay domains
- Must allow parallel system execution where possible (Bevy's default)
- Must use Bevy-idiomatic patterns (plugins, SystemSets, states, events)
- All inter-plugin communication via Bevy primitives (no direct function calls)

## Decision

Organize the codebase into **7 domain-based plugins** with a shared component
module. Each plugin owns a gameplay domain and registers its own systems,
events, and setup logic. Plugins communicate exclusively through Bevy's
built-in mechanisms: Components, Resources, Events/Messages, and States.

### Plugin Registry

| Plugin | Systems Owned | Priority Tier |
|--------|--------------|---------------|
| **ContentPlugin** | Content Database | MVP |
| **RunPlugin** | Run Manager, Run Statistics | MVP |
| **CorePlugin** | Tower Entity, Arena, Isometric Camera, Mana Shield | MVP |
| **EnemyPlugin** | Enemy Data, Enemy System, Wave Escalation, Boss Encounter | MVP |
| **CombatPlugin** | Weapon System, Projectile System, Damage Calculation, Status Effects | MVP |
| **EconomyPlugin** | Gold Economy, Shop System | MVP |
| **UiPlugin** | HUD, Shop UI, End-of-Run Screen, Main Menu | MVP |

### Module Structure

```
src/
├── main.rs                     # App::new(), plugin registration
├── components/                 # Shared component & resource definitions
│   ├── mod.rs
│   ├── tower.rs                # Health, MaxHealth, Armor, ManaShield,
│   │                           # WeaponList, UpgradeList, TowerMarker
│   ├── enemy.rs                # EnemyType, ArmorType, MovementSpeed,
│   │                           # GoldBounty, FrostStacks, StatusEffects
│   ├── combat.rs               # Projectile, DamageEvent, WeaponInstance,
│   │                           # AttackPattern, DamageType
│   ├── economy.rs              # Gold, ShopInventory, ShopSlot, RerollState
│   └── run.rs                  # GameState (Bevy States enum), RunTimer,
│                               # RunStats
├── data/                       # JSON deserialization structs
│   ├── mod.rs
│   ├── weapons.rs              # WeaponDefinition, serde structs
│   ├── upgrades.rs             # UpgradeDefinition, serde structs
│   ├── enemies.rs              # EnemyDefinition, serde structs
│   └── damage_matrix.rs        # DamageMatrix, ArmorType enum
├── plugins/                    # Plugin implementations
│   ├── mod.rs                  # Re-exports all plugins
│   ├── content.rs              # ContentPlugin
│   ├── run.rs                  # RunPlugin
│   ├── core.rs                 # CorePlugin
│   ├── enemy.rs                # EnemyPlugin
│   ├── combat.rs               # CombatPlugin
│   ├── economy.rs              # EconomyPlugin
│   └── ui.rs                   # UiPlugin
└── systems/                    # System functions (organized by domain)
    ├── mod.rs
    ├── content/                # load_weapons, load_upgrades, validate_content
    ├── run/                    # update_timer, check_boss_trigger, track_stats
    ├── core/                   # setup_tower, setup_arena, setup_camera,
    │                           # apply_damage_to_tower, mana_shield_absorb
    ├── enemy/                  # spawn_enemies, move_enemies, enemy_attack,
    │                           # enemy_death, wave_escalation, spawn_boss
    ├── combat/                 # fire_weapons, move_projectiles,
    │                           # resolve_hits, apply_damage, apply_status
    ├── economy/                # award_bounty, tick_income, process_purchase,
    │                           # process_reroll, refresh_shop
    └── ui/                     # render_hud, render_shop, render_end_screen
```

### Communication Patterns

Plugins communicate via these Bevy primitives:

| Mechanism | Use Case | Example |
|-----------|----------|---------|
| **States** (`GameState`) | Run lifecycle | `GameState::Playing`, `GameState::Boss`, `GameState::Paused` |
| **Resources** | Shared read-only data | `Res<WeaponDatabase>`, `Res<RunTimer>`, `Res<Gold>` |
| **Events/Messages** | Cross-domain notifications | `EnemyKilledEvent`, `WeaponPurchasedEvent`, `DamageEvent` |
| **Components** | Per-entity data | `Health`, `ArmorType`, `FrostStacks` on enemy entities |
| **SystemSets** | Execution ordering | `CombatSystems` runs before `EconomySystems` (bounty after kill) |

### GameState Enum (Bevy States)

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Loading,        // ContentPlugin loading JSON
    MainMenu,       // Title screen
    GracePeriod,    // Pre-run shop time
    Playing,        // Active gameplay
    Boss,           // Boss spawned, timer frozen
    Paused,         // Gameplay frozen (sub-state)
    Victory,        // Boss killed
    Defeat,         // Tower destroyed
}
```

### SystemSet Ordering

```rust
// Defined in each plugin, ordered in main.rs
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSystems {
    ContentSystems,     // Load data (startup only)
    RunSystems,         // Timer, state transitions
    EnemySystems,       // Spawn, move, attack
    CombatSystems,      // Fire weapons, move projectiles, resolve damage
    StatusSystems,      // Apply/tick status effects
    EconomySystems,     // Gold, shop refresh, purchases
    UiSystems,          // Render all UI
}

// Ordering:
// RunSystems → EnemySystems → CombatSystems → StatusSystems → EconomySystems → UiSystems
```

### Key Interfaces (Cross-Plugin Events)

```rust
// Enemy killed — sent by CombatPlugin, received by EconomyPlugin + RunPlugin
struct EnemyKilledEvent {
    entity: Entity,
    enemy_type: EnemyType,
    gold_bounty: u32,
    position: Vec2,
    had_burning: bool,       // For fire explosion chain
    fire_damage: f32,        // Stored fire damage value
}

// Weapon purchased — sent by EconomyPlugin, received by CombatPlugin
struct WeaponPurchasedEvent {
    weapon_name: String,
    weapon_index: usize,     // Index into WeaponDatabase
}

// Upgrade purchased — sent by EconomyPlugin, received by CorePlugin
struct UpgradePurchasedEvent {
    upgrade_name: String,
    upgrade_index: usize,    // Index into UpgradeDatabase
}

// Damage to tower — sent by EnemyPlugin, received by CorePlugin
struct TowerDamageEvent {
    raw_damage: f32,
    source_entity: Entity,
}

// Tower destroyed — sent by CorePlugin, received by RunPlugin
struct TowerDestroyedEvent;

// Boss killed — sent by CombatPlugin, received by RunPlugin
struct BossKilledEvent;
```

## Alternatives Considered

### Alternative 1: One Plugin Per System (23 plugins)
- **Description**: Each of the 23 systems gets its own plugin file.
- **Pros**: Maximum separation. Each system is fully independent.
- **Cons**: Too many plugins for one developer. Excessive boilerplate.
  Tightly coupled systems (Weapon + Projectile + Damage) artificially split.
  Communication overhead between plugins that should share internal state.
- **Rejection Reason**: Over-engineered for a solo developer. The coupling
  between e.g., weapons/projectiles/damage is so tight that splitting them
  creates artificial boundaries that harm more than help.

### Alternative 2: Three Plugins (Core, Gameplay, UI)
- **Description**: Three broad plugins: CorePlugin (all non-UI game logic),
  GameplayPlugin (all gameplay systems), UiPlugin (all UI).
- **Pros**: Simple. Few files. Easy to navigate.
- **Cons**: CorePlugin would be massive (15+ systems). No internal boundaries.
  As the game grows, the single gameplay plugin becomes a monolith.
- **Rejection Reason**: Too coarse. Loses the organizational benefit of plugins
  entirely. A 3000-line plugin is worse than 7 focused plugins.

### Alternative 3: Layer-Based Plugins (Foundation, Core, Feature, Presentation, Polish)
- **Description**: Plugins mirror the dependency layers from the systems index.
- **Pros**: Maps directly to the design document. Clear dependency direction.
- **Cons**: Layers don't reflect gameplay domains. "Feature layer" contains
  weapons, damage, waves, shop — unrelated concerns. Developers think in
  domains ("I'm working on combat"), not layers ("I'm working on features").
- **Rejection Reason**: Layers are useful for design ordering, not code
  organization. Domain-based grouping is more natural for development.

## Consequences

### Positive
- Clear ownership: every system belongs to exactly one plugin
- Domain-based organization matches how a developer thinks about the game
- Incremental development: each plugin can be built and tested independently
- Bevy-idiomatic: uses plugins, SystemSets, states, and events as intended
- 7 plugins is manageable for one person while providing real structure

### Negative
- Shared `components/` module creates a dependency every plugin imports
- Some systems span domain boundaries (Mana Shield is tower-related but
  affects damage flow) — placement is a judgment call
- Event-based communication adds indirection compared to direct function calls

### Risks
- **Risk**: Components module becomes a dumping ground for all shared types
  - **Mitigation**: Keep components module organized by domain (tower.rs,
    enemy.rs, combat.rs). Review periodically.
- **Risk**: SystemSet ordering becomes complex as systems grow
  - **Mitigation**: Document ordering constraints in each plugin. Use Bevy's
    `.before()` and `.after()` for explicit ordering.
- **Risk**: Event storms (too many events per frame) hurt performance
  - **Mitigation**: Profile event throughput during stress tests. Batch
    events if needed (e.g., one kill-summary event vs. one per kill).

## Performance Implications
- **CPU**: Bevy's parallel system execution benefits from this structure.
  Independent plugins run in parallel automatically.
- **Memory**: Minimal overhead. Plugin registration is negligible. Components
  use Bevy's archetype storage (efficient for our entity types).
- **Load Time**: ContentPlugin loads all JSON at startup. Expected < 100ms
  for current data volume.
- **Network**: N/A (single-player).

## Migration Plan
N/A — this is the initial architecture for a greenfield project.

## Validation Criteria
- Each plugin can be disabled (not registered) without crashing other plugins
  (they may lose functionality but should not panic)
- Adding a new system to an existing plugin requires changes only within that
  plugin's module (+ possibly a new event/component in the shared module)
- Bevy's system ordering graph has no ambiguities (verified by Bevy's
  built-in ambiguity detection in debug builds)
- Performance target: all systems complete within 16.6ms frame budget at
  500+ entities

## Related Decisions
- ADR-0002 (pending): Data Loading Pipeline (ContentPlugin implementation details)
- ADR-0003 (pending): UI Approach (UiPlugin technology choice)
- Game concept: `design/gdd/game-concept.md`
- Systems index: `design/gdd/systems-index.md`
- Engine reference: `docs/engine-reference/bevy/VERSION.md`
