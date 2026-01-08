# Enforce JSON-Only Serialization

---

- **ID**: `requirement:formats-json-first`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `high`
- **Owner**: `arch-team`

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

Canonical repository artifacts must use deterministic JSON serialization; other formats (YAML) are rejected by tooling.

## Acceptance Criteria

- All canonical cards validate as JSON and are serialized using the agreed schema
- Tooling rejects non-JSON canonical artifacts

## Rationale

Derived from `driver:formats` to keep serialization deterministic and machine-friendly.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:formats](../cards/driver-formats.md)
**Links**: [link:req-formats-functional-driver-formats](../links/link-req-formats-functional-driver-formats.md)

## Raw JSON

```json
{
  "id": "requirement:formats-json-first",
  "type": "requirement",
  "name": "Enforce JSON-Only Serialization",
  "description": "Canonical repository artifacts must use deterministic JSON serialization; other formats (YAML) are rejected by tooling.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "arch-team",
  "relations": [
    "driver:formats"
  ],
  "links": [
    "link:req-formats-functional-driver-formats"
  ],
  "acceptance_criteria": [
    "All canonical cards validate as JSON and are serialized using the agreed schema",
    "Tooling rejects non-JSON canonical artifacts"
  ],
  "rationale": "Derived from `driver:formats` to keep serialization deterministic and machine-friendly.",
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
