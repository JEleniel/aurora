# AURORA Link Semantics and View-Dependent Interpretation

This document explains how AURORA links work and how their semantic meaning is determined by the view context.

---

## Core Principle: Links Are Untyped and View-Dependent

In AURORA, links are **directional, untyped relationships**. A link from Card A to Card B has no intrinsic semantic meaning; instead, its meaning depends on the **context and view** being used to interpret the model.

The same link can mean different things in different contexts:

- In a **Requirements Traceability View**: the link means "satisfies" (a requirement satisfies a driver)
- In a **Component Architecture View**: the same link means "implements" (a component implements a requirement)
- In a **Logical Design View**: it may mean "refines" (a detailed design refines an abstract specification)
- In a **Test Coverage View**: it means "verified-by" (a requirement is verified by a test)

This view-dependent semantics is powerful because it allows a single, simple model to be interpreted from multiple architectural perspectives without duplication or complex typing systems.

---

## Link Directionality and the Root Hierarchy

All links point **toward the Root Driver**, forming a complete derivation hierarchy:

- The **Root Driver** has no outgoing links (it is the apex)
- All other cards have at least one outgoing link pointing toward Root (directly or indirectly)
- This ensures every card is connected and traceable back to the project's authoritative source

**Example hierarchy**:

```text
[Card: Specific Implementation Detail]
        ↓ (link)
[Card: Logical Component]
        ↓ (link)
[Card: Requirement]
        ↓ (link)
[Card: Driver: Security]
        ↓ (link)
[Card: Root Driver]
```

All arrows point upward toward Root, establishing a complete derivation chain.

---

## Common View Interpretations

While links themselves are untyped, here are common semantic interpretations used by views and tools:

### Traceability & Derivation Views

When traversing upward toward Root, links typically mean:

- **satisfies**: A lower-level element satisfies or fulfills a higher-level requirement
- **derives-from**: An element is derived from or originates from a higher-level concept
- **refines**: A detailed element refines or specifies a higher-level abstraction
- **traces-to**: An element traces back to its source or driver

### Architecture & Structure Views

When examining component relationships, the same links may be interpreted as:

- **implements**: A component implements a requirement or specification
- **realizes**: A concrete design realizes an abstract architecture
- **is-composed-of**: A higher-level system is composed of lower-level components
- **allocates**: A logical component allocates to a physical deployment node

### Behavior & Interaction Views

When modeling behavior flows, links may mean:

- **triggers**: One behavior triggers or causes another
- **depends-on**: One behavior depends on another being complete
- **precedes**: One step precedes another in a sequence
- **data-flows**: Data flows from one component to another

### Validation & Test Views

When connecting to verification methods, links may mean:

- **verified-by**: A requirement is verified or validated by a test
- **tested-by**: An implementation is tested by a test case
- **validates**: A test validates a design or requirement

---

## Modeling Best Practices

1. **Ensure Connectivity**: Every card (except Root) must have at least one outgoing link pointing toward Root, either directly or indirectly.

2. **Use Meaningful Card Names**: Since links have no labels, the names of the source and target cards should be clear enough that the relationship is self-evident in the model.

3. **Document Context**: Document which views and interpretations apply to your model. This helps both humans and tools understand how to interpret the links.

4. **Consistency**: Be consistent within a given view. If all links in a requirements view mean "satisfies", maintain that consistency throughout the model.

5. **Multiple Paths**: A card can have multiple outgoing links, allowing it to be interpreted differently in different views. For example, a card might link to both a driver (traceability) and to a test (verification).

---

## Comparison with Typed Link Systems

Traditional architectural frameworks (UML, SysML) use **typed edges** where each link has a semantic label (e.g., "satisfy", "refine", "depend-on"). This requires:

- Defining all possible link types upfront
- Schema extensions for custom types
- Tools must understand link semantics

AURORA's **untyped approach**:

- Simpler schema: links are just connections
- Flexible interpretation: one model, many views
- Tool-agnostic: views define the semantics, not the data
- Extensible: new views can interpret links differently without changing the model

This is a fundamental design choice that simplifies the specification while increasing flexibility.

---

## Constraint & Conflict Link Types

### `conflicts-with`

- **Direction**: Source ↔ Target (bidirectional implication)
- **Meaning**: Source and target are in tension, contradiction, or mutual exclusion.
- **Typical Use**: Performance goals conflict with security requirements; two implementation approaches conflict.
- **Example**: Low-latency caching conflicts with data freshness constraints.
- **Note**: Use sparingly; indicates design tension requiring resolution.

### `refines-into`

- **Direction**: Source → Target
- **Meaning**: Source is refined into, decomposed into, or breaks down into target(s).
- **Typical Use**: A high-level requirement is refined into multiple detailed ones; an epic breaks down into stories.
- **Example**: "Ensure system security" refines-into "encrypt data in transit" and "encrypt data at rest".
- **Distinction from `refines`**: `refines` is detail in place; `refines-into` is breakdown/decomposition.

### `related-to`

- **Direction**: Source ↔ Target
- **Meaning**: Generic relationship; no specific semantic meaning. Use only when no other type fits.
- **Typical Use**: Cards that are topically or contextually related but not formally related by another link type.
- **Example**: A note related to a requirement; a reference document related to a behavior.
- **Note**: Minimize use; prefer explicit, typed relationships.

---

## Link Metadata

Beyond the link type, links can include:

### `strength`

- **Valid Values**: `weak`, `normal`, `strong`
- **Meaning**: Indicates the strength or tightness of the relationship.
    + `weak`: Informational; not critical to the relationship's validity.
    + `normal`: Standard relationship; expected and documented.
    + `strong`: Critical; breaking this link affects both source and target; changes must be carefully coordinated.
- **Example**: A design verification link might be `normal` or `strong`; a historical reference link might be `weak`.

### `rationale`

- **Type**: String
- **Meaning**: Explanation of why the relationship exists or what it means in context.
- **Example**: "This requirement derives from the security driver because data protection is essential to customer trust."

### `metadata`

- **Type**: Object (custom)
- **Meaning**: Additional tooling or context-specific metadata.
- **Example**: `{ "confidence": "high", "last_reviewed": "2025-12-10", "review_note": "..." }`

---

## See Also

- [Card Field Reference](card-field-reference.md)
- [JSON Schema: link.schema.json](../schemas/link.schema.json)
- [Tooling Guide](tooling.md)
