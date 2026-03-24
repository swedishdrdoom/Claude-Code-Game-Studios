# Projectile System

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Your Build, Your Story (visual expression of the build)

## Overview

The Projectile System handles the lifecycle of projectile entities — spawning
them when weapons fire, moving them toward their targets, detecting collision,
resolving hits, and cleaning up spent projectiles. It is the visual and mechanical
bridge between the Weapon System (which decides what to fire) and the Damage
Calculation system (which resolves the damage). The system must handle hundreds
of simultaneous projectiles at 60fps.

## Player Fantasy

Projectiles are the visible output of your build. When the screen fills with
bouncing cannonballs, frost bombs, and magic bolts, the player sees their
investment paying off. The density of projectiles IS the power fantasy. Each
projectile type should feel distinct — a Mortar Launcher lob looks different
from a Serpent's rapid-fire bolts.

## Detailed Design

### Core Rules

1. **Spawning**: Projectiles are created by the Weapon System when a weapon fires.
   Each projectile carries:
   - Origin position (tower center)
   - Target entity or target position
   - Weapon definition reference (damage, damage type, attack pattern)
   - Speed
   - Visual sprite

2. **Movement**: Projectiles travel from tower center toward their target.
   ```
   direction = normalize(target_position - projectile_position)
   projectile_position += direction * projectile_speed * delta_time
   ```
   Projectiles are homing — they track their target's current position, not
   the position at fire time. If the target dies mid-flight, the projectile
   continues to the last known position and despawns on arrival.

3. **Collision**: A projectile hits its target when within collision radius.
   ```
   if distance(projectile, target) < COLLISION_RADIUS:
       resolve_hit()
   ```

4. **Hit Resolution**: On collision, the projectile triggers its attack pattern:
   - **Single Target**: Deal damage to the target. Despawn projectile.
   - **Splash**: Deal damage to target + all enemies within splash radius. Despawn.
   - **Bounce**: Deal damage to target. Find nearest enemy not yet hit. Create
     a new projectile (or redirect) toward that enemy. Repeat up to N times.
   - **Barrage**: Each barrage projectile is an independent Single Target projectile.
   - **Area**: No projectile — instant damage at the area center. Visual effect only.
   - **Wave**: Expanding ring from tower. Damages all enemies the ring passes
     through. No traditional projectile entity.

5. **Despawn Conditions**:
   - Projectile hits its target (after pattern resolution)
   - Projectile reaches last known target position (target died in flight)
   - Projectile exits the spawn ring radius (cleanup boundary)
   - Projectile exceeds max lifetime (safety timeout: 10 seconds)

6. **No Friendly Fire**: Projectiles only hit enemies, never the tower.

7. **Freeze**: Projectiles freeze during Paused state. They resume on unpause.

### States and Transitions (Per Projectile)

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **InFlight** | Created by Weapon System | Reaches target or despawn condition | Moving toward target, homing |
| **Hit** | Collision detected | Pattern resolved | Triggers damage, splash, bounce, etc. |
| **Despawned** | Despawn condition met | Entity removed | Cleaned up from ECS |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Weapon System** | Upstream | Receives fire commands with target, damage, and pattern data |
| **Enemy System** | Reads | Queries enemy positions for homing. Detects collision with enemies. |
| **Damage Calculation** | Sends | On hit, sends damage event with weapon damage type and value |
| **Arena** | Reads | Uses spawn ring radius as despawn boundary |
| **VFX / Juice** | Sends | Impact events trigger visual effects (explosions, splashes, bounces) |
| **Audio System** | Sends | Impact events trigger hit/explosion sounds |

## Formulas

### Projectile Travel Time

```
travel_time = distance(tower, target) / projectile_speed
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| projectile_speed | f32 | TBD | Projectile config (per pattern) | Units per second |
| distance | f32 | 0–spawn ring radius | Calculated | Distance from tower to target |

### Wave Expansion

```
wave_radius += wave_expansion_speed * delta_time
damage all enemies where: distance(enemy, tower) >= wave_inner_edge AND distance(enemy, tower) <= wave_radius
```

Each enemy can only be hit once per wave. Track hit enemies per wave instance.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Target dies while projectile is in flight | Projectile continues to last known position, despawns on arrival. No damage dealt. | Don't redirect — target is dead, shot is wasted. Feels natural. |
| Bounce target dies before bounce reaches it | Bounce chain ends. Remaining bounces are lost. | Dead targets can't be bounced to. |
| Bounce: all nearby enemies already hit | Chain ends early. Remaining bounces are lost. | No double-hitting in one chain. |
| Hundreds of projectiles on screen | ECS handles this. Projectiles are simple entities (position + velocity + sprite). | Must sustain 500+ projectiles at 60fps. |
| Projectile spawned with 0 speed | Stays at tower forever. Safety timeout (10s) despawns it. | Shouldn't happen (validation), but handle gracefully. |
| Splash radius larger than arena | All enemies on screen take damage. Valid. | Big splash is a feature, not a bug. |
| Wave and enemy moving in same direction | Enemy can outrun a slow wave. Wave damage only applies at the moment of contact. | Creates interesting range dynamics. |
| Two projectiles hit same enemy on same frame | Both deal damage independently. | No collision between projectiles. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Weapon System** | Upstream (hard) | Creates projectiles |
| **Enemy System** | Upstream (hard) | Projectiles need enemies to hit |
| **Damage Calculation** | Downstream (hard) | Sends damage events on hit |
| **Arena** | Upstream (soft) | Despawn boundary |
| **VFX / Juice** | Downstream (soft) | Impact visual effects |
| **Audio System** | Downstream (soft) | Impact sound effects |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Projectile speed | TBD (per pattern) | 200–2000 units/sec | Faster hits; less visual travel time; feels snappier | Slower projectiles; more visual density on screen; more "bullet hell" feel |
| Collision radius | TBD | 5–30 units | Easier to hit; more forgiving | Tighter collision; projectiles can "miss" fast enemies (unlikely with homing) |
| Max lifetime | 10 seconds | 5–30 | More time for slow projectiles to reach far targets | Faster cleanup of stray projectiles |
| Wave expansion speed | TBD | 100–1000 units/sec | Wave reaches arena edge faster; harder for enemies to outrun | Slower wave; more enemies can walk past it |

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Projectile in flight | Projectile sprite with rotation toward target | None (fire sound covers it) | High |
| Single Target hit | Small impact flash | Thud/impact | High |
| Splash impact | Circular flash at impact radius | Explosion | High |
| Bounce hop | Projectile arc between targets with trail | Ricochet ping | Medium |
| Wave expansion | Expanding ring visual from tower | Whoosh | High |
| Area activation | Circular area highlight + damage flash | Area effect sound | High |

## UI Requirements

N/A — Projectiles have no UI elements. They are purely visual gameplay entities.

## Acceptance Criteria

- [ ] Single Target projectiles travel to target and deal damage on hit
- [ ] Splash projectiles deal damage to all enemies within splash radius on hit
- [ ] Bounce projectiles chain to N targets without hitting the same enemy twice
- [ ] Barrage fires N independent projectiles at N different enemies
- [ ] Area deals instant damage to all enemies in radius (no projectile)
- [ ] Wave expands from tower and damages enemies it passes through (once each)
- [ ] Projectiles home on moving targets
- [ ] Projectiles despawn when target dies in flight (reach last position, no damage)
- [ ] Projectiles despawn when exiting spawn ring radius
- [ ] Projectiles freeze during Paused state
- [ ] System sustains 500+ simultaneous projectiles at 60fps
- [ ] No friendly fire — projectiles never damage the tower

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| What are the projectile speeds for each attack pattern? | Prototype | During core loop prototype | Needs feel testing — too fast looks snappy, too slow creates visual chaos |
| Should projectiles have trail effects (particle trails)? | Art Director | Alpha | Adds visual clarity but increases render cost |
| Should bounce projectiles arc visually or travel in straight lines? | Art Director | Alpha | Arcs look better but are more complex to render |
