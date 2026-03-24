# Claude Code Game Studios -- Game Studio Agent Architecture

Indie game development managed through 48 coordinated Claude Code subagents.
Each agent owns a specific domain, enforcing separation of concerns and quality.

## Technology Stack

- **Engine**: Bevy 0.18.1 (Rust ECS game engine)
- **Language**: Rust
- **Version Control**: Git with trunk-based development
- **Build System**: Cargo
- **Asset Pipeline**: Bevy Asset System

> **Note**: No Bevy-specific specialist agent exists. General agents
> (gameplay-programmer, engine-programmer, lead-programmer) handle all
> engine work. Always cross-reference `docs/engine-reference/bevy/VERSION.md`
> before suggesting Bevy APIs — the LLM's training data covers up to ~0.15.

## Project Structure

@.claude/docs/directory-structure.md

## Engine Version Reference

@docs/engine-reference/bevy/VERSION.md

## Technical Preferences

@.claude/docs/technical-preferences.md

## Coordination Rules

@.claude/docs/coordination-rules.md

## Collaboration Protocol

**User-driven collaboration, not autonomous execution.**
Every task follows: **Question -> Options -> Decision -> Draft -> Approval**

- Agents MUST ask "May I write this to [filepath]?" before using Write/Edit tools
- Agents MUST show drafts or summaries before requesting approval
- Multi-file changes require explicit approval for the full changeset
- No commits without user instruction

See `docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` for full protocol and examples.

> **First session?** If the project has no engine configured and no game concept,
> run `/start` to begin the guided onboarding flow.

## Coding Standards

@.claude/docs/coding-standards.md

## Context Management

@.claude/docs/context-management.md
