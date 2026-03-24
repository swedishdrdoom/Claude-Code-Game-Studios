# Bevy Breaking Changes — 0.15 → 0.18

*Last verified: 2026-03-22*

This document covers breaking changes across three major versions that are
beyond the LLM's training data. All agents must consult this before suggesting
Bevy API calls.

---

## 0.15 → 0.16 (April 2025)

### Entity Relationships & Hierarchy

Bevy's ECS now has **built-in one-to-many entity relationships**. The parent-child
hierarchy system uses these relationships.

- `despawn_recursive()` and `despawn_descendants()` **removed** from EntityCommands
- `despawn()` is now the primary method
- New alternatives: `despawn_related()` and `remove()`
- `HierarchyQueryExt` methods are now inherent methods on `Query`
- `parent` → `related`
- `children` → `relationship_sources`

### Event Trait

- `Event` no longer requires the `Component` trait bound (it was confusing)
- Events are their own distinct concept now

### WebAssembly

- `wasm32v1-none` target supported
- Browser-specific WASM features now behind `web` feature flag
- If `default-features = false`, enable `web` for browser builds

### MSRV

- Minimum Supported Rust Version is now "latest stable release"

### Math

- Cubic splines API uses `IntoIterator` instead of `Into<Vec<..>>`
- `bevy_reflect` is now a non-default feature of `bevy_math`

---

## 0.16 → 0.17 (September 2025)

### Event System Refactoring (MAJOR)

**`Event` has been split into two traits:**
- `Message` — for buffered events (what was previously `Event`)
- `Event` — for observers

Every `Event` now has an associated `Trigger` implementation. The `Trigger`
trait defines behavior of `world.trigger()`.

### Handle Changes

- `Handle::Weak` **replaced** by `Handle::Uuid`
- `weak_handle!` macro → `uuid_handle!` macro

### OpenGL

- `gles` backend is no longer a default feature of `bevy_render`
- Must explicitly enable `bevy_render/gles` for OpenGL support

### Asset & Reflection

- `ReflectAsset` methods now accept `impl Into<UntypedAssetId>`

### Text API

- `TextFont::from_font` and `TextFont::from_line_height` **removed**
- Use `From` trait implementations instead

### SystemSet Naming

- Bevy's own system sets renamed to `*Systems` suffix convention
- Recommended for user code as well

### Other

- `pixel_size()` now returns `Result` — must handle error

---

## 0.17 → 0.18 (January 2026)

### Entity System (MAJOR)

**"Row" terminology replaced with "Index":**
- `EntityRow` → `EntityIndex`
- `Entity::row` → `Entity::index`
- `Entity::from_row` → `Entity::from_index`

**Flushing removed:**
- Removed: `alloc`, `free`, `reserve`, `reserve_entity`, `reserve_entities`,
  `flush`, `flush_as_invalid`, `total_count`, `used_count`, `total_prospective_count`
- Spawn individual `EntityRow`s instead
- `EntityDoesNotExistError` → `EntityNotSpawnedError`

### EntityEvent

- Mutable methods moved to separate `SetEntityEventTarget` trait
- All `EntityEvent`s are **immutable by default**

### Asset Sources

- `AssetSourceBuilder::with_watcher` changed from `crossbeam_channel::Sender`
  to `async_channel::Sender`

### UI & Picking

- Only text sections of `Text` nodes are pickable (not the whole node)
- For 0.17 behavior: use an intermediate parent node to intercept pointer hits

### Cargo Features

- New high-level feature collections: `2d`, `3d`, `ui`
- Recommended to use these instead of manually enabling specific features

### New Features (non-breaking)

- Built-in fly/pan cameras
- `Popover` component for popup UI positioning
- Variable weight fonts, text strikethroughs, underlines, OpenType features
- Solari raytraced renderer improvements
- Procedural atmosphere customization
