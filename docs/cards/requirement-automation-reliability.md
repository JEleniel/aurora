# Ensure Automation Reliability

---

- **ID**: `requirement:automation-reliability`
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

Non-functional: automated pipelines must be resilient and recoverable; failures should present clear diagnostics for human operators.

## Acceptance Criteria

- Pipelines produce actionable logs and meaningful exit codes
- Transient errors are retried automatically and humans alerted when manual intervention is required

## Rationale

Derived from `driver:automation` to make automation practical and maintainable.

## Provenance

- **source**: `derived from drivers`
- **owner**: `JEleniel`
- **version**: `1.0`

## Audit History

- created — by JEleniel at 2025-12-09T00:00:00Z

## Related

**Relations**: [driver:automation](../cards/driver-automation.md)
**Links**: [link:req-automation-nf-driver-automation](../links/link-req-automation-nf-driver-automation.md)

## Raw JSON

```json
{
  "id": "requirement:automation-reliability",
  "type": "requirement",
  "name": "Ensure Automation Reliability",
  "description": "Non-functional: automated pipelines must be resilient and recoverable; failures should present clear diagnostics for human operators.",
  "classification": "non-functional",
  "version": "1.0.0",
  "status": "proposed",
  "priority": "medium",
  "owner": "platform-team",
  "relations": [
    "driver:automation"
  ],
  "links": [
    "link:req-automation-nf-driver-automation"
  ],
  "acceptance_criteria": [
    "Pipelines produce actionable logs and meaningful exit codes",
    "Transient errors are retried automatically and humans alerted when manual intervention is required"
  ],
  "rationale": "Derived from `driver:automation` to make automation practical and maintainable.",
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
