# Bevy Engine — Version Reference

| Field | Value |
|-------|-------|
| **Engine Version** | Bevy 0.18.1 |
| **Release Date** | March 4, 2026 (0.18.1 patch); January 13, 2026 (0.18.0) |
| **Project Pinned** | 2026-03-22 |
| **Last Docs Verified** | 2026-03-22 |
| **LLM Knowledge Cutoff** | May 2025 |

## Knowledge Gap Warning

The LLM's training data likely covers Bevy up to ~0.15. Versions 0.16, 0.17,
and 0.18 introduced **major breaking changes** that the model does NOT know
about. Always cross-reference this directory before suggesting Bevy APIs.

## Post-Cutoff Version Timeline

| Version | Release | Risk Level | Key Theme |
|---------|---------|------------|-----------|
| 0.16 | Apr 2025 | MEDIUM | Entity relationships, despawn refactor, Event/Component split |
| 0.17 | Sep 2025 | HIGH | Event→Message/Event split, Handle::Weak→Handle::Uuid, SystemSet `*Systems` naming |
| 0.18 | Jan 2026 | HIGH | Entity row→index rename, EntityEvent immutability, cargo feature collections, Solari raytracing |

## Verified Sources

- Official docs: https://docs.rs/bevy/latest/bevy/
- 0.15→0.16 migration: https://bevy.org/learn/migration-guides/0-15-to-0-16/
- 0.16→0.17 migration: https://bevy.org/learn/migration-guides/0-16-to-0-17/
- 0.17→0.18 migration: https://bevy.org/learn/migration-guides/0-17-to-0-18/
- Release notes: https://bevy.org/news/bevy-0-18/
- Bevy book: https://bevy.org/learn/
