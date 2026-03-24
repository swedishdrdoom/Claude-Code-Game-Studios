# Isometric Camera

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Greed Kills (full visibility enables fast shop decisions)

## Overview

The Isometric Camera is a fixed, slightly-tilted top-down camera that shows
the entire circular arena at all times. It uses Bevy's 2D camera with orthographic
projection. Since the view is near top-down (like Vampire Survivors), z-ordering
is simplified — entities further from the camera (higher Y in world space) render
behind entities closer to the camera. No true isometric diamond tiles are needed;
the "isometric feel" comes from the slight tilt in sprite art, not camera math.

## Player Fantasy

The camera is invisible when it's working. The player should feel like they're
looking down at a battlefield from directly above, with just enough tilt to give
depth and make the tower feel three-dimensional. The fixed camera means the player
never loses track of anything — the entire game state is always visible, which
supports rapid shop decisions under pressure ("Greed Kills").

## Detailed Design

### Core Rules

1. **Projection**: Orthographic 2D camera. No perspective distortion.

2. **Position**: Fixed at world origin, looking down. Camera does not move during
   gameplay. No pan, zoom, or follow behavior.

3. **Zoom Level**: Set once at startup to frame the entire arena plus a small margin.
   Determined by arena radius and window resolution.

4. **Sprite Sorting**: Y-based sorting. Entities with lower Y values (further "up"
   on screen / further from camera) render behind entities with higher Y values.
   Implemented via Bevy's sprite z-ordering using the entity's Y position to
   compute a z-value.

5. **Art Direction**: Sprites are drawn with a slight top-down tilt perspective
   (similar to Vampire Survivors, Brotato). This is an art convention, not a
   camera transform — the camera is purely top-down.

6. **Resolution**: Support arbitrary window sizes. The arena always fits fully
   in view. Letterboxing or pillarboxing as needed to maintain aspect ratio.

7. **Pixel Scaling**: Pixel art must render at integer scaling to avoid
   sub-pixel artifacts. Camera zoom snaps to nearest integer multiple of the
   base pixel size.

### States and Transitions

The camera is static. No runtime state changes.

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Arena** | Reads arena radius | Uses arena radius to calculate zoom level that frames the full arena |
| **All visual entities** | Provides render order | Y-based z-sorting applies to tower, enemies, projectiles, and effects |
| **HUD / Shop UI** | Separate UI camera | Gameplay camera and UI camera are independent. UI renders in screen space on top of the game world. |

## Formulas

### Zoom Calculation

```
camera_scale = (arena_diameter + margin) / window_height
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| arena_diameter | f32 | TBD | Arena config | 2 × arena radius in world units |
| margin | f32 | ~10% of diameter | Camera config | Buffer so arena edge isn't at screen edge |
| window_height | f32 | 480–2160 | OS window | Current window height in pixels |

### Z-Sort Value

```
sprite_z = -world_y * Z_SORT_SCALE
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| world_y | f32 | varies | Entity transform | Entity's Y position in world space |
| Z_SORT_SCALE | f32 | 0.001 | Camera config | Small multiplier to keep z-values in a usable range |

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Window resized during gameplay | Camera recalculates zoom to keep full arena in view. No gameplay impact. | Desktop standard — resizing should just work. |
| Very small window (e.g., 320x240) | Still renders correctly. Pixel art may become hard to read but won't break. | Support minimum viable resolution but don't optimize for it. |
| Two entities at exact same Y position | Render order is undefined but stable frame-to-frame (Bevy's default). | For a bullet-hell, overlapping sprites are expected and harmless. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| (none) | Upstream | No upstream dependencies. |
| **Arena** | Downstream (hard) | Arena needs camera to be visible |
| **All rendering** | Downstream (hard) | Everything visual depends on the camera existing |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Margin (% of arena diameter) | 10% | 5–20% | More empty space around arena; feels zoomed out | Arena fills more of screen; feels more cramped |
| Z_SORT_SCALE | 0.001 | 0.0001–0.01 | More z-separation between entities | Less separation; potential z-fighting |

## Visual/Audio Requirements

N/A — The camera is invisible infrastructure. Visual output is determined by
the sprites and systems it renders.

## UI Requirements

N/A — UI is rendered by a separate screen-space camera, not the gameplay camera.

## Acceptance Criteria

- [ ] Full arena visible at all window sizes without scrolling or panning
- [ ] Pixel art renders at integer scaling (no sub-pixel blur)
- [ ] Entities sort correctly by Y position (higher Y = rendered in front)
- [ ] Camera does not move during gameplay
- [ ] Window resize recalculates zoom correctly
- [ ] UI elements render on top of game world in screen space
- [ ] Performance: camera/sorting overhead < 0.5ms per frame

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should we use Bevy's built-in sprite sorting or a custom z-sort system? | Engine Programmer | During prototype | Evaluate Bevy 0.18's sprite sorting capabilities |
| What base pixel resolution should sprites target? (e.g., 16x16, 32x32 base tile) | Art Director | Before sprite production | Affects camera zoom math and visual density |
