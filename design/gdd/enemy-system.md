# Enemy System

> **Status**: In Design
> **Author**: user + game-designer
> **Last Updated**: 2026-03-22
> **Implements Pillar**: Fifteen Minutes of Escalation

## Overview

The Enemy System handles the runtime behavior of all enemies — spawning them
at the arena's spawn ring, moving them toward the tower, attacking the tower
on arrival, and processing their death. It does not define what enemies *are*
(that's Enemy Data) or how many spawn (that's Wave Escalation). The Enemy
System is the executor: given an enemy type and a spawn command, it creates
the entity, moves it, makes it fight, and cleans it up when it dies.

## Player Fantasy

Enemies are the doom. They walk toward you, relentlessly, from every direction.
The player feels the pressure as the screen fills — first a trickle, then a
stream, then a flood. Each enemy that reaches the tower chips away at survival.
The satisfaction comes from watching your weapons mow them down before they
arrive, and the tension comes from the ones that get through.

## Detailed Design

### Core Rules

1. **Spawning**: Enemies spawn at positions on the spawn ring (off-screen,
   defined by Arena). Spawn position is a random angle on the ring. The Enemy
   System does not decide *when* or *what* to spawn — it receives spawn
   commands from Wave Escalation.

2. **Movement**: All enemies move in a straight line toward the tower at
   world origin (0,0). Movement speed is defined per enemy type in Enemy Data.
   ```
   direction = normalize(tower_position - enemy_position)
   enemy_position += direction * move_speed * delta_time
   ```

3. **Separation Force**: Enemies apply a light push-apart force to nearby
   enemies to prevent full overlap. This is a soft separation, not hard
   collision — enemies can still clump but don't perfectly stack.
   ```
   for each nearby enemy within separation_radius:
       push_direction = normalize(self_position - other_position)
       self_position += push_direction * separation_strength * delta_time
   ```
   The separation force is weaker than the movement force, so enemies
   still converge on the tower but spread slightly.

4. **Attack**: When an enemy reaches the tower (distance < attack_range),
   it stops moving and attacks. Attacks deal the enemy's damage value to
   the tower on a cooldown timer (defined per enemy type).
   ```
   if distance_to_tower < attack_range:
       stop moving
       if attack_cooldown_timer <= 0:
           deal damage to tower (raw damage, no damage type matrix)
           reset attack_cooldown_timer
   ```

5. **Death**: When an enemy's HP reaches 0:
   - Remove the enemy entity
   - Award kill bounty gold to the player (via Gold Economy)
   - Trigger death event (for VFX, audio, status effect propagation)
   - Leave a corpse marker briefly (for Necromancer's Tome ability)

6. **Despawn**: Enemies that somehow move beyond the spawn ring radius
   (e.g., from a future knockback mechanic) are despawned without
   awarding bounty.

7. **Freeze During Non-Active States**: Enemies freeze during Paused
   state. Enemies stop spawning and freeze during Victory/Defeat. During
   Boss state, existing enemies remain active but no new enemies spawn.

### States and Transitions (Per Enemy)

| State | Entry Condition | Exit Condition | Behavior |
|-------|----------------|----------------|----------|
| **Moving** | Spawned | Reaches tower attack range OR HP reaches 0 | Walks toward tower. Targetable by weapons. |
| **Attacking** | Within attack range of tower | HP reaches 0 | Stationary. Deals damage to tower on cooldown. Still targetable. |
| **Dead** | HP reaches 0 | Corpse timer expires | Awards bounty. Triggers death VFX/audio. Corpse lingers briefly. |

### Interactions with Other Systems

| System | Direction | Interface |
|--------|-----------|-----------|
| **Enemy Data** | Reads | Gets enemy type definitions (HP, damage, speed, armor type, bounty) |
| **Arena** | Reads | Gets spawn ring radius and arena boundaries |
| **Run Manager** | Reads | Checks run state for freeze/despawn behavior |
| **Wave Escalation** | Receives commands | Wave Escalation tells Enemy System to spawn X enemies of type Y |
| **Weapon System** | Is targeted by | Weapons select enemies within range as targets |
| **Projectile System** | Receives hits | Projectiles collide with enemies and deal damage |
| **Damage Calculation** | Receives damage | Processes incoming damage through armor type multiplier |
| **Gold Economy** | Sends bounty | Awards kill bounty gold on enemy death |
| **Status Effects** | Receives effects | Enemies can be slowed (Frost), poisoned, stunned, etc. |
| **VFX / Juice** | Sends events | Death events trigger VFX (explosion, particles) |
| **Audio System** | Sends events | Death and attack events trigger SFX |
| **Run Statistics** | Sends events | Tracks total enemies killed, enemies that reached tower |

## Formulas

### Movement

```
direction = normalize((0, 0) - enemy_position)
new_position = enemy_position + direction * move_speed * delta_time
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| enemy_position | Vec2 | spawn ring to (0,0) | Entity transform | Current position |
| move_speed | f32 | TBD per enemy type | Enemy Data | Units per second |
| delta_time | f32 | ~0.016 | Bevy Time | Frame delta |

### Separation

```
for each neighbor within SEPARATION_RADIUS:
    push = normalize(self - neighbor) * SEPARATION_STRENGTH * delta_time
    self_position += push
```

| Variable | Type | Range | Source | Description |
|----------|------|-------|--------|-------------|
| SEPARATION_RADIUS | f32 | TBD | Enemy System config | Distance at which enemies push apart |
| SEPARATION_STRENGTH | f32 | TBD | Enemy System config | Force of push (must be < move_speed) |

### Time to Reach Tower

```
time_to_tower = spawn_ring_radius / move_speed
```

This is useful for tuning: if spawn ring radius is 2000 units and move speed
is 200 units/sec, enemies take 10 seconds to reach the tower.

## Edge Cases

| Scenario | Expected Behavior | Rationale |
|----------|------------------|-----------|
| Enemy spawns directly on top of another | Both exist. Separation force pushes them apart over time. | Spawn overlap is expected at high spawn rates. |
| Enemy HP reduced to 0 by multiple hits in same frame | Die once. Award bounty once. Process first lethal hit, ignore remainder. | No double-death, no double-bounty. |
| Enemy reaches tower but tower has mana shield | Enemy attacks tower normally. Mana shield absorbs raw damage per tower damage pipeline. | Enemy doesn't know or care about mana shield. |
| Hundreds of enemies on screen simultaneously | Must sustain 60fps. ECS architecture handles this. Separation force uses spatial hashing for performance. | Entity budget of 500+ is a performance requirement. |
| Enemy killed by Frost (frozen then shattered) | Standard death — bounty awarded, death event fired. | Death cause doesn't matter for the basic death flow. |
| Stunned enemy | Enemy stops moving and attacking for stun duration. Remains targetable. | Stun pauses enemy behavior, doesn't remove it. |
| Slowed enemy (Frost stacks) | move_speed multiplied by slow factor. Attack speed also reduced. | Frost reduces both movement and attack speed per the Frost mechanic. |
| Boss state: existing enemies still alive | They continue moving and attacking. They are not despawned or frozen. | Boss phase doesn't grant safety from remaining enemies. |

## Dependencies

| System | Direction | Nature of Dependency |
|--------|-----------|---------------------|
| **Enemy Data** | Upstream (hard) | Needs enemy type definitions |
| **Arena** | Upstream (hard) | Needs spawn ring radius |
| **Run Manager** | Upstream (hard) | Needs run state for freeze/active behavior |
| **Wave Escalation** | Upstream (hard) | Receives spawn commands |
| **Weapon System** | Downstream (hard) | Enemies are weapon targets |
| **Projectile System** | Downstream (hard) | Projectiles hit enemies |
| **Damage Calculation** | Downstream (hard) | Processes damage to enemies |
| **Gold Economy** | Downstream (hard) | Awards bounty on death |
| **Status Effects** | Downstream (soft) | Enemies receive status effects |
| **VFX / Juice** | Downstream (soft) | Death triggers visual effects |
| **Audio System** | Downstream (soft) | Death/attack triggers sound |

## Tuning Knobs

| Parameter | Current Value | Safe Range | Effect of Increase | Effect of Decrease |
|-----------|--------------|------------|-------------------|-------------------|
| Separation radius | TBD | 10–100 units | Enemies spread further apart; less clumping | Tighter clumps; splash/area weapons more effective |
| Separation strength | TBD | 10–100% of move speed | More visible spreading; less stacking | Enemies pile up more; feels more like a horde |
| Attack range | TBD | 10–50 units | Enemies start attacking further from tower center | Enemies must get very close before attacking |
| Corpse linger time | TBD | 0.5–3.0 seconds | Corpses stay longer (Necromancer's Tome has more to work with) | Corpses vanish quickly; less visual clutter |

**Knob interactions**: Separation strength vs. move speed determines how
much enemies spread. If separation is too strong relative to speed, enemies
form a ring instead of clumping. If too weak, they fully overlap.

## Visual/Audio Requirements

| Event | Visual Feedback | Audio Feedback | Priority |
|-------|----------------|---------------|----------|
| Enemy spawns | Enemy appears at spawn ring, walks in | None (too frequent at scale) | N/A |
| Enemy reaches tower | None (attack animations handle this) | None | N/A |
| Enemy attacks tower | Attack animation on enemy | Melee hit sound | Medium |
| Enemy takes damage | Brief flash/hit effect on enemy sprite | Impact sound | High |
| Enemy dies | Death animation / explosion particles | Death sound (varies by type) | High |
| Enemy corpse fades | Sprite fades over corpse linger time | None | Low |

## UI Requirements

N/A — Enemies have no dedicated UI. Health bars on enemies are optional
polish (not MVP). Enemy count may be shown in Run Statistics.

## Acceptance Criteria

- [ ] Enemies spawn at random positions on the spawn ring (off-screen)
- [ ] Enemies move in a straight line toward (0,0) at their defined speed
- [ ] Enemies apply separation force to avoid full overlap
- [ ] Enemies stop and attack when within attack range of tower
- [ ] Enemy attacks deal raw damage to tower (processed through tower damage pipeline)
- [ ] Enemy death awards correct bounty to Gold Economy
- [ ] Enemy death fires event for VFX, audio, and statistics
- [ ] Enemies freeze during Paused state
- [ ] No new enemies spawn during Boss, Victory, or Defeat states
- [ ] Existing enemies remain active during Boss state
- [ ] System sustains 500+ simultaneous enemies at 60fps
- [ ] Stunned enemies stop moving and attacking
- [ ] Slowed enemies (Frost) have reduced move speed and attack speed
- [ ] Enemies beyond spawn ring radius are despawned without bounty

## Open Questions

| Question | Owner | Deadline | Resolution |
|----------|-------|----------|-----------|
| Should enemies have visible health bars? | HUD / UX Designer | Alpha | Adds info but also visual clutter at scale |
| How long should corpses linger for Necromancer's Tome? | Weapon System GDD | When Necromancer's Tome is implemented | Needs testing with the summon mechanic |
| Should there be a spawn animation (enemies materialize) or do they just appear off-screen and walk in? | Art Director | Alpha | Walking in from off-screen may be sufficient |
