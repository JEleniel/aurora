# Cards and fields

Cards are the primary building block of an Aurora model.

## Navigation

- [Aurora overview](README.md)
- [Links and relationships](links-and-relationships.md)
- [Invariant rules](invariants.md)
- [Design docs index](../design/README.md)

## Core fields

Most cards share the same core fields:

| Field | Purpose |
| --- | --- |
| `$schema` | Relative path to the schema in the model home. |
| `id` | Stable identifier like `REQ-001`. Once assigned, it must not change. |
| `card_type` | The type of element (Title Case), such as `Requirement` or `System`. |
| `card_subtype` | Optional refinement of `card_type` (Title Case). |
| `name` | Human-readable, concise title (Title Case). |
| `description` | The detailed definition of the element. |
| `status` | Lifecycle state (for implementable elements like Features). |
| `links` | Outgoing links to other cards. |
| `audit_trail` | Version and history of changes (created/edited/deleted). |
| `attributes` | Optional free-form metadata for tooling and renderers. |

## The Mission card

A model starts from a single Mission card. The Mission is special:

- It is the root of the model graph.
- It must only have outgoing links.

## Example card (minimal)

This is a minimal illustrative example (fields may vary with schema evolution):

```json
{
	"$schema": "../Aurora.schema.json",
	"id": "CAP-001",
	"card_type": "Capability",
	"name": "Authenticate Identities",
	"description": "The system can establish and verify the identity of callers.",
	"links": [
		{
			"target": "REQ-001",
			"relationship": "satisfies"
		}
	]
}
```

Notes:

- In real models, `audit_trail` is used to track `version` and change history.
- Use `attributes` when you need tool-specific metadata without changing the core schema.
