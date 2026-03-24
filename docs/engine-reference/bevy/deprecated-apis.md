# Bevy Deprecated / Removed APIs — Quick Reference

*Last verified: 2026-03-22*

Use this table to catch common mistakes from LLM suggestions based on
older Bevy versions. The LLM knows Bevy ~0.15; this covers 0.16-0.18.

## Don't Use → Use Instead

| Removed / Deprecated | Replacement | Since |
|---|---|---|
| `despawn_recursive()` | `despawn()` (recursive by default) | 0.16 |
| `despawn_descendants()` | `despawn_related()` or `remove()` | 0.16 |
| `HierarchyQueryExt::parent()` | `Query::related()` | 0.16 |
| `HierarchyQueryExt::children()` | `Query::relationship_sources()` | 0.16 |
| `Event` requiring `Component` | `Event` is standalone trait | 0.16 |
| `Handle::Weak` | `Handle::Uuid` | 0.17 |
| `weak_handle!` macro | `uuid_handle!` macro | 0.17 |
| `Event` (for buffered events) | `Message` (buffered) / `Event` (observers) | 0.17 |
| `TextFont::from_font()` | `TextFont::from(font)` (From trait) | 0.17 |
| `TextFont::from_line_height()` | `TextFont::from(line_height)` (From trait) | 0.17 |
| `EntityRow` | `EntityIndex` | 0.18 |
| `Entity::row()` | `Entity::index()` | 0.18 |
| `Entity::from_row()` | `Entity::from_index()` | 0.18 |
| `alloc` / `free` / `reserve` / `flush` | Spawn individual entities | 0.18 |
| `EntityDoesNotExistError` | `EntityNotSpawnedError` | 0.18 |
| Mutable `EntityEvent` methods | `SetEntityEventTarget` trait | 0.18 |
| `crossbeam_channel::Sender` (watcher) | `async_channel::Sender` | 0.18 |

## Convention Changes

| Old Convention | New Convention | Since |
|---|---|---|
| Inconsistent SystemSet names | `*Systems` suffix (e.g., `CombatSystems`) | 0.17 |
| Manual cargo feature selection | High-level collections (`2d`, `3d`, `ui`) | 0.18 |
| `gles` as default render feature | Opt-in via `bevy_render/gles` | 0.17 |

## Safe Patterns (Still Valid in 0.18)

These commonly-used patterns from 0.15 are still valid:

- `App::new().add_plugins(DefaultPlugins)` — still the standard entry point
- `Commands` for spawning/despawning entities
- `Query<>` for system parameters
- `Res<>` / `ResMut<>` for resources
- `Component` derive macro
- `Bundle` derive macro
- `SystemSet` derive macro (but use `*Systems` naming)
- `States` and `NextState` for state management
- `AssetServer::load()` for asset loading
- `Transform` / `GlobalTransform` for positioning
