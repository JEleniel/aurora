# Patterns

Lightweight conventions that keep agent work consistent and low-friction.

## Aurora modeling

- Prefer the canonical registries (card types/relationships/views) as the single source of truth; do not duplicate type matrices in ad-hoc docs.
- Keep every card JSON pretty-printed and include a correct `$schema` relative path for that file’s location.
- Ensure every non-Mission card has at least one incoming link and is reachable from the Mission.

## Agent docs

- Keep `.agents/MAP.md` accurate to what is actually checked into the repo.
- Put short-lived constraints and current blockers into `.agents/CONTEXT.md`.
- Put durable/tooling facts (paths, stacks, commands) into `.agents/TECHNOLOGIES.md`.
