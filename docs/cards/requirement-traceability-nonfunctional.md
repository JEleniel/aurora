# Provide Traceability Performance Guarantees

---

- **ID**: `requirement:traceability-nonfunctional`
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

Non-functional requirement for traceability: queries and automated analyses should run within acceptable time bounds for repositories of expected size.

## Acceptance Criteria

- Trace queries return results within 5s for repositories with <= 10k cards
- Indexing or caching strategies documented and validated

## Rationale

Derived from `driver:traceability` to ensure traceability is practical at scale.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:traceability](../cards/driver-traceability.md)
**Links**: [link:req-traceability-nf-driver-traceability](../links/link-req-traceability-nf-driver-traceability.md)

## Raw JSON

```json
{
  "id": "requirement:traceability-nonfunctional",
  "type": "requirement",
  "name": "Provide Traceability Performance Guarantees",
  "description": "Non-functional requirement for traceability: queries and automated analyses should run within acceptable time bounds for repositories of expected size.",
  "classification": "non-functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "medium",
  "owner": "platform-team",
  "relations": [
    "driver:traceability"
  ],
  "links": [
    "link:req-traceability-nf-driver-traceability"
  ],
  "acceptance_criteria": [
    "Trace queries return results within 5s for repositories with <= 10k cards",
    "Indexing or caching strategies documented and validated"
  ],
  "rationale": "Derived from `driver:traceability` to ensure traceability is practical at scale.",
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
