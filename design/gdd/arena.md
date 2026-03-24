# Arena

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: All (foundational stage)

## Overview

The Arena is the static playfield where all gameplay occurs — a circular open
area with the tower at its center. The camera is fixed and shows the entire arena
at all times. Enemies spawn off-screen and walk inward toward the tower. The arena
has no obstacles, terrain features, or interactive elements — it is a clean,
readable stage designed to keep the player's attention on the shop and the
approaching horde.

## Player Fantasy

The arena reinforces the tower's isolation. You are alone in the center of an open
field, surrounded on all sides, with nowhere to hide. The circular shape means
threats come from every direction equally — there's no safe side, no chokepoint
to exploit. This serves "Greed Kills": you can't turtle behind geometry, so your
only defense is your build.

## Detailed Design

### Core Rules

1. **Shape**: Circular arena with the tower at the exact center (world origin 0,0).

2. **Size**: The arena radius defines the visible play area. Enemies are considered
   "in the arena" when they cross the arena boundary. The arena must be large enough
   that the longest-range weapons (1200 units) can fire at enemies before they reach
   the tower, but small enough that the entire arena fits on a single fixed camera.

3. **Camera**: Fixed isometric camera, centered on the tower, showing the entire
   arena at all times. No pan, no zoom, no camera movement.

4. **Spawn Zone**: Enemies spawn in a ring outside the visible camera area (off-screen)
   and walk inward. The spawn ring radius is larger than the camera's visible radius,
   so enemies appear to walk in from beyond the player's view.

5. **No Obstacles**: The arena contains no walls, terrain, or obstacles. Enemies walk
   in straight lines toward the tower. The only collision is enemy-to-tower.

6. **Boundaries**: Enemies that somehow move away from the tower (e.g., knockback in
   a future version) are despawned if they exit the spawn ring radius (cleanup zone).

7. **Visual Ground**: The arena floor is a tiled isometric surface. The boundary may
   be visually indicated (edge glow, fade to black, terrain edge) but is not a
   physical wall.

### States and Transitions

The Arena is static. It has no runtime state changes — it exists identically
from run start to run end. No states or transitions to document.

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Tower Entity** | Arena provides position | Tower spawns at world origin (0,0). Arena defines the coordinate system. |
| **Enemy System** | Arena provides spawn zone | Enemies spawn at random points on the spawn ring (off-screen circle). Arena provides spawn radius and arena radius. |
| **Weapon System** | Arena provides spatial context | Weapon range values are in arena coordinate units. Range of 900 means 900 units from tower center. |
| **Isometric Camera** | Arena defines camera bounds | Camera is positioned to show the full arena diameter. Arena radius determines camera zoom level. |
| **Projectile System** | Arena provides despawn bounds | Projectiles that exit the spawn ring radius are despawned (cleanup). |

## Formulas

### Spawn Position

```
angle = random(0, 2π)
spawn_x = cos(angle) * SPAWN_RING_RADIUS
spawn_y = sin(angle) * SPAWN_RING_RADIUS
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| angle | f32 | 0–2π | Random | Random direction around the arena |
| SPAWN_RING_RADIUS | f32 | TBD | Arena config | Distance from center where enemies appear (off-screen) |

**Note**: Actual spawn ring radius and arena radius values will be determined
during prototyping based on camera field-of-view and weapon range testing.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Enemy spawns exactly at 0,0 (tower position) | Should never happen — spawn ring is far from center. If it does, enemy immediately attacks tower. | Spawn ring radius prevents this by construction. |
| Hundreds of enemies clump at tower center | No special handling. Enemies overlap. Splash/area weapons are the counter. | Keep rules simple. Clumping is a gameplay feature, not a bug — it rewards AoE builds. |
| Projectile flies forever (no target hit) | Despawn when exiting spawn ring radius. | Prevents entity leak from missed projectiles. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| (none) | Upstream | Arena has no upstream dependencies. |
| **Tower Entity** | Downstream (hard) | Needs arena origin for tower placement |
| **Enemy System** | Downstream (hard) | Needs spawn ring for enemy spawn positions |
| **Isometric Camera** | Downstream (hard) | Needs arena radius for camera framing |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Arena radius | TBD (prototype) | — | Enemies take longer to reach tower; more time to shoot; longer-range weapons favored | Enemies arrive faster; close-range/AoE weapons favored; more frantic |
| Spawn ring radius | TBD (prototype) | Must be > camera visible radius | Enemies walk further before visible; more warning time | Enemies appear suddenly at screen edge; more surprise |

**Knob interactions**: Arena radius and spawn ring radius interact with weapon
range values (max 1200 units in current data). The arena must be sized so that
the longest-range weapons can engage enemies before they reach the tower.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Arena ground | Isometric tiled floor. Readable, low-detail — must not compete with enemies/projectiles for visual attention. | None | Medium |
| Arena edge | Subtle visual boundary (fade, glow, or terrain edge). Not a wall. | None | Low |

## UI Requirements

N/A — The Arena has no UI elements. All gameplay UI (HUD, shop) is overlaid by
other systems.

## Acceptance Criteria

- [ ] Tower spawns at world origin (0,0)
- [ ] Fixed camera shows the entire arena without pan/zoom
- [ ] Enemies spawn off-screen at the spawn ring and walk toward center
- [ ] Spawn positions are uniformly distributed around the full 360-degree perimeter
- [ ] Projectiles are despawned when exiting the spawn ring radius
- [ ] No invisible walls or collision barriers at arena edge
- [ ] Arena renders correctly in isometric perspective
- [ ] Weapon range of 1200 units allows engagement before enemies reach the tower

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| What are the exact arena radius and spawn ring radius values? | Prototype | During core loop prototype | Requires testing with camera FOV and weapon ranges |
| Should the arena ground have any visual variation (e.g., darkening toward edges)? | Art Director | Alpha | Cosmetic, not gameplay-critical |
