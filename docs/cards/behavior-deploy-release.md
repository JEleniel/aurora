# Deploy Release

---

- **ID**: `behavior:deploy-release`
- **Type**: `behavior`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `high`
- **Owner**: `ops-team`

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

Deploy application releases to target environments with repeatable, auditable processes.

## Provenance

- **source**: `ops playbooks`
- **owner**: `ops-team`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [actor:system-operator](../cards/actor-system-operator.md)

## Raw JSON

```json
{
  "id": "behavior:deploy-release",
  "type": "behavior",
  "name": "Deploy Release",
  "description": "Deploy application releases to target environments with repeatable, auditable processes.",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "ops-team",
  "relations": [
    "actor:system-operator"
  ],
  "audit_history": [
    {
      "event": "created",
      "by": "JEleniel",
      "event_time": "2025-12-09T00:00:00Z"
    }
  ],
  "provenance": {
    "source": "ops playbooks",
    "owner": "ops-team",
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
