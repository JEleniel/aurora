# Record Complete Provenance

---

- **ID**: `requirement:governance-provenance`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `high`
- **Owner**: `governance-team`

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

All artifacts must include provenance metadata (source, owner, version) and retain audit history to support governance and decision records.

## Acceptance Criteria

- Cards include `provenance` with source, owner, and version
- `audit_history` entries exist for create/update actions

## Rationale

Derived from `driver:governance-collaboration` to provide accountability and collaboration records.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:governance-collaboration](../cards/driver-governance-collaboration.md)
**Links**: [link:req-governance-functional-driver-governance-collaboration](../links/link-req-governance-functional-driver-governance-collaboration.md)

## Raw JSON

```json
{
  "id": "requirement:governance-provenance",
  "type": "requirement",
  "name": "Record Complete Provenance",
  "description": "All artifacts must include provenance metadata (source, owner, version) and retain audit history to support governance and decision records.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "governance-team",
  "relations": [
    "driver:governance-collaboration"
  ],
  "links": [
    "link:req-governance-functional-driver-governance-collaboration"
  ],
  "acceptance_criteria": [
    "Cards include `provenance` with source, owner, and version",
    "`audit_history` entries exist for create/update actions"
  ],
  "rationale": "Derived from `driver:governance-collaboration` to provide accountability and collaboration records.",
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
