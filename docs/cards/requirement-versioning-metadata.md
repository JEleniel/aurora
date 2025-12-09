# Record Version and Lifecycle Metadata

---

- **ID**: `requirement:versioning-metadata`
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

Every card must include a `version` (semver) and `status` to support lifecycle automation and compatibility checks.

## Acceptance Criteria

- `version` follows semver (MAJOR.MINOR.PATCH)
- `status` is one of the allowed lifecycle states

## Rationale

Derived from `driver:versioning-lifecycle` to enable lifecycle management and compatibility checks.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:versioning-lifecycle](../cards/driver-versioning-lifecycle.md)
**Links**: [link:req-versioning-functional-driver-versioning-lifecycle](../links/link-req-versioning-functional-driver-versioning-lifecycle.md)

## Raw JSON

```json
{
  "id": "requirement:versioning-metadata",
  "type": "requirement",
  "name": "Record Version and Lifecycle Metadata",
  "description": "Every card must include a `version` (semver) and `status` to support lifecycle automation and compatibility checks.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "arch-team",
  "relations": [
    "driver:versioning-lifecycle"
  ],
  "links": [
    "link:req-versioning-functional-driver-versioning-lifecycle"
  ],
  "acceptance_criteria": [
    "`version` follows semver (MAJOR.MINOR.PATCH)",
    "`status` is one of the allowed lifecycle states"
  ],
  "rationale": "Derived from `driver:versioning-lifecycle` to enable lifecycle management and compatibility checks.",
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
