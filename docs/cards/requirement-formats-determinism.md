# Guarantee Serialization Determinism

---

- **ID**: `requirement:formats-determinism`
- **Type**: `requirement`
- **Version**: `1.0.0`
- **Status**: `proposed`
- **Priority**: `medium`
- **Owner**: `platform-team`

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

Non-functional requirement: serialization order, field presence and canonicalization must be deterministic across tools producing canonical JSON.

## Acceptance Criteria

- Two independent tooling runs on the same logical model produce byte-for-byte identical canonical JSON outputs
- Canonical sort, stable key ordering and consistent timestamp formats documented

## Rationale

Derived from `driver:formats` to enable reliable diffing, signing, and caching.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:formats](../cards/driver-formats.md)
**Links**: [link:req-formats-nf-driver-formats](../links/link-req-formats-nf-driver-formats.md)

## Raw JSON

```json
{
  "id": "requirement:formats-determinism",
  "type": "requirement",
  "name": "Guarantee Serialization Determinism",
  "description": "Non-functional requirement: serialization order, field presence and canonicalization must be deterministic across tools producing canonical JSON.",
  "classification": "non-functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "medium",
  "owner": "platform-team",
  "relations": [
    "driver:formats"
  ],
  "links": [
    "link:req-formats-nf-driver-formats"
  ],
  "acceptance_criteria": [
    "Two independent tooling runs on the same logical model produce byte-for-byte identical canonical JSON outputs",
    "Canonical sort, stable key ordering and consistent timestamp formats documented"
  ],
  "rationale": "Derived from `driver:formats` to enable reliable diffing, signing, and caching.",
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
