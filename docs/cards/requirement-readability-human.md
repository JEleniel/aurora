# Ensure Human-Readable Descriptions

---

- **ID**: `requirement:readability-human`
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

Provide clear, plain-language descriptions and examples on every card so human readers understand intent and usage without needing tools.

## Acceptance Criteria

- Each card includes a concise description (<= 200 words)
- Examples or minimal usage snippets included when applicable

## Rationale

Derived from `driver:readability` to ensure artifacts are human consumable.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:readability](../cards/driver-readability.md)
**Links**: [link:req-readability-human-driver-readability](../links/link-req-readability-human-driver-readability.md)

## Raw JSON

```json
{
  "id": "requirement:readability-human",
  "type": "requirement",
  "name": "Ensure Human-Readable Descriptions",
  "description": "Provide clear, plain-language descriptions and examples on every card so human readers understand intent and usage without needing tools.",
  "classification": "functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "high",
  "owner": "arch-team",
  "relations": [
    "driver:readability"
  ],
  "links": [
    "link:req-readability-human-driver-readability"
  ],
  "acceptance_criteria": [
    "Each card includes a concise description (<= 200 words)",
    "Examples or minimal usage snippets included when applicable"
  ],
  "rationale": "Derived from `driver:readability` to ensure artifacts are human consumable.",
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
