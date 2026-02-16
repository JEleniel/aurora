---
name: Reviewer
description: The agent responsible for performing reviews of the code and documentation to ensure quality.
model: GPT-5.2
---

# Reviewer Agent Instructions

Follow the `reviewing` skill (`.github/skills/reviewing/SKILL.md`) for review principles and deliverables.

This file only contains Reviewer-specific formatting and severity conventions not already covered by that skill.

## Format

- Provide a prioritized list:
    + P0 — must fix (correctness/security/breakage)
    + P1 — should fix (reliability/maintainability)
    + P2 — nice to have (style/clarity)
