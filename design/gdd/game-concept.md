# Game Concept: Tower of Doom

*Created: 2026-03-22*
*Status: Draft*

---

## Elevator Pitch

> It's an isometric survival-defense game where you are a single stationary
> tower in an open arena, buying weapons and upgrades from a live shop during
> combat to survive a 15-minute run that ends in a final boss. The core tension
> is greed versus survival: spend gold to get stronger, but every purchase is a
> gamble against escalating enemy pressure.

---

## Core Identity

| Aspect | Detail |
| ---- | ---- |
| **Genre** | Isometric survival-defense / bullet-heaven |
| **Platform** | Desktop (macOS development, Windows first demo) |
| **Target Audience** | Mid-core optimizer/achiever players (see Player Profile) |
| **Player Count** | Single-player |
| **Session Length** | 15 minutes per run |
| **Monetization** | Premium (TBD) |
| **Estimated Scope** | Medium (3-9 months) |
| **Comparable Titles** | Vampire Survivors, Tower Survivors (WC3 custom map), Brotato |

---

## Core Fantasy

You are an ancient tower — immovable, immutable, alone — surrounded by an
endless horde. You cannot run. You cannot hide. Your only option is to arm
yourself faster than the doom arrives. Every 30 seconds the merchant appears
with new weapons and you make the same impossible choice: invest in power now
and risk dying to the current wave, or hoard gold and risk being overwhelmed
later. When your build clicks — when five stacked fire bows ignite the entire
screen and the boss melts in seconds — that's the fantasy. You are a fortress
of escalating destruction.

---

## Unique Hook

Like Vampire Survivors, AND ALSO you shop *during* combat instead of between
waves. The shop is always visible, always tempting, and the economy is the
game. You don't choose a build at the start — you construct it live under
pressure, adapting to what the shop offers and what the enemy demands. Duplicate
weapon purchases stack, making committed investment in a weapon type the path
to exponential power.

---

## Player Experience Analysis (MDA Framework)

### Target Aesthetics (What the player FEELS)

| Aesthetic | Priority | How We Deliver It |
| ---- | ---- | ---- |
| **Sensation** (sensory pleasure) | 3 | Screen-filling projectiles, satisfying hit feedback, escalating visual chaos |
| **Fantasy** (make-believe, role-playing) | 5 | "I am the tower" — static power fantasy, fortress identity |
| **Narrative** (drama, story arc) | N/A | No story. The run IS the narrative arc (scramble → build → climax) |
| **Challenge** (obstacle course, mastery) | 1 | Greed-vs-survival tension, boss as skill check, build optimization |
| **Fellowship** (social connection) | N/A | Single-player, no social features in v1 |
| **Discovery** (exploration, secrets) | 4 | Finding weapon synergies, discovering optimal stacking combos |
| **Expression** (self-expression, creativity) | 2 | 88 weapons × 94 upgrades = massive build variety. "My build" identity |
| **Submission** (relaxation, comfort zone) | 6 | Auto-combat provides baseline relaxation; tension comes from economy decisions |

### Key Dynamics (Emergent player behaviors)

- Players will develop favorite weapon "archetypes" and chase specific stacking
  strategies (e.g., "triple Frost Bomb + Shatter build")
- Players will learn to read wave composition and adapt purchases mid-run
- Experienced players will push reroll timing to find rare/epic weapons at the
  right moment in the escalation curve
- Players will develop risk tolerance profiles — some play safe with HP/armor,
  others go glass cannon and rely on kill speed for survival
- The "Philosopher's Stone problem" — high-risk income upgrades that trade HP
  for gold will create clutch moments and memorable deaths

### Core Mechanics (Systems we build)

1. **Auto-Combat System** — Tower fires all equipped weapons automatically at
   enemies within range. No player aiming or targeting. Weapons have distinct
   attack patterns (single target, splash, bounce, barrage, area, wave).
2. **Live Shop Economy** — Always-visible shop refreshes every 30 seconds.
   Players buy weapons and upgrades with gold earned from kills. Rerolls cost
   gold with escalating prices. Duplicate weapons stack for increased output.
3. **Wave Escalation** — Enemy pressure ramps continuously over 15 minutes.
   Difficulty scaling is invisible to the player. Final boss appears at the end.
4. **Damage Type System** — 5 base damage types (Normal, Piercing, Magic, Siege,
   Chaos) with 4 element subtypes (Fire, Frost, Poison, Spikes). Enemy
   resistances and weapon synergies create build depth.
5. **Tower Defense Layer** — HP bar, armor, mana shield, HP regen, spikes damage.
   Defensive investment competes with offensive investment for gold.

---

## Player Motivation Profile

### Primary Psychological Needs Served

| Need | How This Game Satisfies It | Strength |
| ---- | ---- | ---- |
| **Autonomy** (freedom, meaningful choice) | Every shop refresh is a meaningful decision. Build identity emerges from player choices, not prescribed paths. Reroll vs. save vs. buy creates constant agency. | Core |
| **Competence** (mastery, skill growth) | Players learn damage types, stacking thresholds, economy timing, and boss patterns. Knowledge directly translates to survival time and build efficiency. | Core |
| **Relatedness** (connection, belonging) | Minimal in v1. Future: leaderboards, shared build codes, community meta discussion. | Minimal |

### Player Type Appeal (Bartle Taxonomy)

- [x] **Achievers** (goal completion, collection, progression) — Beat the boss,
  optimize DPS, discover all weapon combos, push gold efficiency
- [x] **Explorers** (discovery, understanding systems, finding secrets) —
  Discover synergies between damage types and upgrades, find edge-case builds
  (e.g., full spikes + armor-plated defense)
- [ ] **Socializers** (relationships, cooperation, community) — Not served in v1
- [ ] **Killers/Competitors** (domination, PvP, leaderboards) — Future:
  leaderboards and score competition

### Flow State Design

- **Onboarding curve**: First run teaches through doing. Shop is immediately
  visible, gold appears from kills, weapons are self-explanatory. No tutorial
  screens — the 15-minute structure IS the tutorial.
- **Difficulty scaling**: Invisible ramp. Early waves let players experiment with
  the shop. Mid-run pressure forces commitment to a build strategy. Late-run
  tests whether the build can handle density. Boss tests peak DPS.
- **Feedback clarity**: Damage numbers, projectile density, gold counter, HP bar.
  The player sees their build's output in real-time. Stacking a weapon visibly
  increases fire rate / projectile count.
- **Recovery from failure**: Instant restart. 15-minute runs mean failure costs
  minutes, not hours. Death screen shows build summary so the player knows what
  to try differently.

---

## Core Loop

### Moment-to-Moment (30 seconds)

Watch the battlefield, scan the shop, make a buy/save/reroll decision. The tower
auto-attacks; the player's job is economic — reading the shop offerings against
the current threat and deciding where to invest. The satisfying crunch of this
loop is: see item → evaluate against build → buy (or don't) → see immediate
impact on combat output.

### Short-Term (5-15 minutes)

A single 15-minute run. The arc: scramble for first weapons (0-3 min), establish
a build identity (3-7 min), optimize and stack (7-12 min), survive the final push
and kill the boss (12-15 min). Each shop refresh (every 30 seconds) is a decision
point — roughly 30 decision points per run.

### Session-Level (30-120 minutes)

2-4 runs per session. Each run experiments with a different weapon archetype or
stacking strategy. Natural stopping point is after a successful boss kill or
after a particularly satisfying (or frustrating) death. Session variety comes
from the shop's randomized offerings forcing adaptation.

### Long-Term Progression

V1: Clean slate. Mastery is the player's knowledge of weapon synergies, economy
timing, and damage type interactions. The 88 weapons and 94 upgrades provide
enormous combinatorial depth to explore.

Future: Light meta-progression — unlock new weapons for the pool, cosmetic tower
skins, additional arenas, harder difficulty modifiers.

### Retention Hooks

- **Curiosity**: "What if I stacked 5 Flamecasters?" / "Can a pure spikes build
  actually beat the boss?"
- **Investment**: Knowledge of the system — once you understand damage types and
  stacking, you want to test your theories
- **Social**: Future — leaderboards, build sharing, community theorycrafting
- **Mastery**: Tighter gold management, better reroll timing, faster boss kills

---

## Game Pillars

### Pillar 1: Greed Kills

Every decision is a risk/reward trade-off. The shop tempts you to optimize your
build while enemies are actively threatening you. The player who plays it safe
survives but underperforms; the player who gets greedy might die or might pop off.

*Design test*: "Should we add a pause menu during shop?" — No. Greed Kills means
pressure is always on.

### Pillar 2: Your Build, Your Story

No two runs should play the same way. The combination of weapons, passives, and
stacking creates emergent builds that feel personal. The player should be able to
say "I went triple Frost Bomb with Shatter and froze the entire arena."

*Design test*: "Should we add a 'recommended build' tooltip?" — No. Discovery is
the point.

### Pillar 3: Fifteen Minutes of Escalation

Every second of the run should feel different from the last. The difficulty curve
is a smooth ramp that ends in a spike. No filler waves, no downtime, no grinding.
Fifteen minutes, one arc, done.

*Design test*: "Should we add optional side objectives mid-run?" — Only if they
feed the escalation, never if they break the tempo.

### Anti-Pillars (What This Game Is NOT)

- **NOT a tower defense game** — You don't build mazes, place multiple towers, or
  manage pathing. You ARE the tower.
- **NOT a story game** — No cutscenes, no lore dumps, no dialogue. Narrative
  lives in the run itself.
- **NOT a long-session game** — If a run takes more than 18 minutes, something is
  wrong. Respect the player's time.
- **NOT a meta-grind** — V1 has no permanent progression. Skill is the unlock.

---

## Inspiration and References

| Reference | What We Take From It | What We Do Differently | Why It Matters |
| ---- | ---- | ---- | ---- |
| Vampire Survivors | Auto-combat, escalating hordes, bullet-heaven density, short runs | Shop during combat (not level-up picks), stationary tower instead of mobile character, gold economy instead of XP | Validates the auto-combat survival genre has massive audience (10M+ copies) |
| Tower Survivors (WC3) | Stationary tower fantasy, damage type system, weapon stacking, WC3-style ability design | Real-time shop instead of build phase, continuous escalation instead of discrete waves | Proves the "you are the tower" concept is compelling in a horde context |
| Brotato | Shop between waves, weapon stacking, stat-based builds, short runs | Shop is live during combat, not between rounds — the economy IS the gameplay | Validates shop + weapon stacking as a satisfying build system |

**Non-game inspirations**: Warcraft III's damage type / armor type interaction
matrix. The "one more spin" psychology of slot machines applied to shop rerolls.
The economic tension of poker — knowing when to go all-in vs. fold.

---

## Target Player Profile

| Attribute | Detail |
| ---- | ---- |
| **Age range** | 18-35 |
| **Gaming experience** | Mid-core to hardcore. Familiar with roguelite/survivor genre conventions |
| **Time availability** | 15-60 minute sessions. Perfect for "one more run" between other activities |
| **Platform preference** | PC (Steam). Desktop-first players who also play indie games |
| **Current games they play** | Vampire Survivors, Brotato, Slay the Spire, Risk of Rain 2 |
| **What they're looking for** | A survivor-like with more strategic depth. Economy decisions, not just dodge-and-collect |
| **What would turn them away** | Mandatory grinding, pay-to-win, runs longer than 20 minutes, lack of build variety |

---

## Technical Considerations

| Consideration | Assessment |
| ---- | ---- |
| **Engine** | Bevy (Rust) — ECS architecture ideal for hundreds of entities (projectiles, enemies). Data-driven design is native to ECS. Strong performance for bullet-hell density. |
| **Key Technical Challenges** | Live shop UI during gameplay (Bevy UI ecosystem is immature). Balancing 88 weapons × 94 upgrades. Isometric rendering with pixel art. Audio system (Bevy audio is basic). |
| **Art Style** | Pixel art isometric. WC3-inspired icon aesthetic for weapons/upgrades (existing Cloudinary-hosted sprites). |
| **Art Pipeline Complexity** | Low-Medium. Existing weapon/upgrade icons from WC3-style sprites. Need: tower sprite, enemy sprites (3-4 types + boss), arena tileset, projectile effects, UI elements. |
| **Audio Needs** | Moderate. Combat SFX (weapon fire, impacts, abilities), ambient arena music, boss music, shop interaction sounds. Adaptive music that escalates with difficulty. |
| **Networking** | None (single-player) |
| **Content Volume** | 88 weapons, 94 upgrades, 3-4 enemy types, 1 boss, 1 arena (demo). Data already defined in JSON. |
| **Procedural Systems** | Shop offering randomization, enemy wave composition, spawn patterns. No procedural level generation. |

---

## Existing Content Inventory

Content data already exists in JSON format from prior iteration (tower-of-doom-v3).

### Weapons (88 total)

| Rarity | Count |
| ---- | ---- |
| Common | 5 |
| Uncommon | 35 |
| Rare | 34 |
| Epic | 14 |

**Damage types**: Normal (12), Piercing (9), Magic (11), Siege (13), Chaos (11),
plus hybrid types with Fire, Frost, Poison, and Spikes subtypes.

**Attack patterns**: Single Target (26), Splash (19), Bounce (14), Area (12),
Wave (9), Barrage (8).

**Notable mechanics**: Frost stacking (slow → freeze at 50 stacks), Poison DoT,
Fire DoT, Chaos damage amplification, Stun, Heal-on-hit, summons (Necromancer's
Tome), rotating patterns (Flamecaster).

### Upgrades (94 total)

| Rarity | Count |
| ---- | ---- |
| Common | 13 |
| Uncommon | 34 |
| Rare | 34 |
| Epic | 13 |

**Categories**: Damage Boost (29), Spikes (18), Income (8), HP (7), Mana Shield
(7), Armor (6), Regen (4), Bounty (3), Utility (3), Critical (2), Frost (2),
Stun (2), Attack Boost (1), Healing (1), Dodge (1).

**Notable mechanics**: Scaling upgrades (+X every 30 seconds), risk/reward trades
(Philosopher's Stone: -1000 HP, +2000 Gold; Cursed Treasure: -100 Regen, +5000
Gold), weapon-specific synergies (Chaos Legion: +5% Chaos per Chaos Orb; Blessed
Steel: +5% Normal per Throwing Axe), Duplicator (copy weapons/upgrades), Black
Market (choose an uncommon).

**Source files**: `tower-of-doom-v3/assets/content/tower_survivors/weapons.json`,
`tower-of-doom-v3/assets/content/tower_survivors/upgrades.json`

---

## Risks and Open Questions

### Design Risks

- **Shop overwhelm** — Managing a live shop while under attack could feel
  stressful rather than exciting. UX must make scanning and buying near-instant.
- **Stacking dominance** — If stacking one weapon type always dominates, build
  variety collapses. Need diminishing returns or opportunity costs.
- **Passive play** — Auto-combat means the player could disengage. Economy
  pressure and boss threat must keep attention active.

### Technical Risks

- **Bevy UI maturity** — The shop UI is the entire game. Bevy's UI ecosystem is
  young. May need custom widget work or a third-party UI crate.
- **Bevy audio** — Basic audio support. May need `kira` or another audio backend
  for adaptive music and layered SFX.
- **Entity scale** — 88 weapon types × stacking × hundreds of enemies ×
  projectiles. Must validate ECS performance under peak load early.
- **Isometric rendering** — Bevy doesn't have built-in isometric support. Need
  to evaluate sprite sorting, z-ordering, and camera setup approaches.

### Market Risks

- **Genre saturation** — The survivor/bullet-heaven space is crowded post-
  Vampire Survivors. Differentiation through live shop economy is genuine but
  must be immediately visible in marketing.
- **Bevy perception** — Players don't care about engine, but Bevy's ecosystem
  gaps could delay polish and platform support.

### Scope Risks

- **88 weapons need balancing** — Large content volume requires systematic
  balance testing. Data-driven design helps but doesn't eliminate the work.
- **Art production** — Pixel art iso sprites for tower, enemies, arena, and
  effects are all net-new. This is the primary content bottleneck.

### Open Questions

- What are the enemy types and their behaviors? (Need enemy design doc)
- What is the boss design? (Need boss encounter doc)
- How exactly does weapon stacking work mechanically? (Linear? Multiplicative?
  New projectile per stack? Increased stats per stack?)
- What is the gold economy curve? (Starting gold, kill bounty scaling, shop
  prices by rarity, reroll cost escalation)
- How does the damage type vs. armor type interaction matrix work? (WC3-style
  multipliers? Simpler system?)
- What is the exact difficulty scaling formula? (Enemy HP/damage/density over
  the 15-minute run)

---

## MVP Definition

**Core hypothesis**: "Players find the live-shop economy during auto-combat
engaging enough to play multiple 15-minute runs."

**Required for MVP**:
1. Stationary tower with HP in an open arena
2. Auto-combat with 3-5 weapon types (representative of different attack
   patterns: single target, splash, area)
3. Live shop with buy, reroll, and weapon stacking
4. 2-3 enemy types with basic pathfinding toward tower
5. 15-minute difficulty ramp
6. Gold from kills, shop prices, reroll cost escalation
7. Win/lose condition (boss or timer)

**Explicitly NOT in MVP** (defer to later):
- All 88 weapons and 94 upgrades (ship with a balanced subset)
- Damage type / armor type interaction matrix
- Status effects (Frost, Poison, Fire, Stun)
- Mana shield system
- Spikes damage system
- Meta-progression
- Audio beyond placeholder
- Polish, particles, screen shake

### Scope Tiers (if budget/time shrinks)

| Tier | Content | Features | Timeline |
| ---- | ---- | ---- | ---- |
| **MVP** | 5 weapons, 5 upgrades, 2 enemies, no boss | Core loop: shop + auto-combat + gold economy | 4-6 weeks |
| **Vertical Slice** | 15-20 weapons, 15-20 upgrades, 3 enemies, 1 boss | Full damage types, stacking, status effects, win condition | 8-12 weeks |
| **Alpha** | All 88 weapons, all 94 upgrades, 4+ enemies, boss | Full content, rough balance, basic audio/VFX | 16-24 weeks |
| **Demo (v1)** | All content, balanced | Polished UX, audio, VFX, title screen, Windows build | 24-36 weeks |

---

## Next Steps

- [ ] Configure Bevy engine and Rust toolchain (`/setup-engine`)
- [ ] Validate concept completeness (`/design-review design/gdd/game-concept.md`)
- [ ] Decompose concept into systems (`/map-systems`)
- [ ] Author per-system GDDs starting with shop economy (`/design-system`)
- [ ] First architecture decisions — ECS structure, data format, UI approach (`/architecture-decision`)
- [ ] Prototype core loop — tower + enemies + shop (`/prototype core-loop`)
- [ ] Playtest the prototype (`/playtest-report`)
- [ ] Plan first sprint (`/sprint-plan new`)
