# Ensure Machine-Readable Examples

---

- **ID**: `requirement:readability-machine`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `medium`
- **Owner**: `tooling-team`

## Field Reference

- **id**: Canonical identifier for the card (namespace:type).
- **type**: Card type (driver, requirement, actor, behavior, interface, constraint, link, view, etc.).
- **name**: Human-friendly title using verb+noun where applicable.
- **description**: Plain-language explanation of the card's intent and scope.
- **version**: Semver for the card's content (MAJOR.MINOR.PATCH).
- **status**: Lifecycle state (proposed, accepted, deprecated, retired, etc.).
- **priority**: Optional top-level priority (high/medium/low).
- **owner**: Optional top-level owner or team responsible for the card.
- **relations**: References to other card IDs indicating logical relationships.
- **links**: Explicit link artifact IDs that encode richer relationship metadata.
- **acceptance_criteria**: Machine-or-human-verifiable criteria for satisfying a requirement.
- **rationale**: Why this card exists; derivation or justification.
- **provenance**: Source and origin metadata (source, owner, version).
- **audit_history**: Chronological events describing create/update actions with timestamps.
- **metadata**: Format and serialization metadata for tooling (format, serialization).

## Description

Provide minimal machine-oriented examples and consistent field naming to enable automated parsing and tool assistance.

## Acceptance Criteria

- Field names use deterministic, snake_case or kebab-case conventions
- Examples include minimal JSON snippets that validate against the schema

## Rationale

Derived from `driver:readability` to make artifacts friendly for processing by agents.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:readability](../cards/driver-readability.md)
**Links**: [link:req-readability-machine-driver-readability](../links/link-req-readability-machine-driver-readability.md)

## Raw JSON

```json
{
  "id": "requirement:readability-machine",
  "type": "requirement",
  "name": "Ensure Machine-Readable Examples",
  "description": "Provide minimal machine-oriented examples and consistent field naming to enable automated parsing and tool assistance.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "medium",
  "owner": "tooling-team",
  "relations": [
    "driver:readability"
  ],
  "links": [
    "link:req-readability-machine-driver-readability"
  ],
  "acceptance_criteria": [
    "Field names use deterministic, snake_case or kebab-case conventions",
    "Examples include minimal JSON snippets that validate against the schema"
  ],
  "rationale": "Derived from `driver:readability` to make artifacts friendly for processing by agents.",
  "audit_history": [
    {
      "event": "created",
      "by": "JEleniel",
      "event_time": "2025-12-09T00:00:00Z"
    }
  ],
  "provenance": {
    "source": "derived from drivers",
    "owner": "JEleniel",
    "version": "1.0"
  },
  "metadata": {
    "format": "aurora-card-v1",
    "serialization": [
      "json"
    ]
  }
}

```
