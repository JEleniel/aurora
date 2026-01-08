# Drivers (SysML Requirements View)

This diagram shows the AURORA Root Driver and the primary derived drivers as a SysML-style requirements view. Each node corresponds to a driver card (one element = one card). Links are typed as `derives` to indicate derivation from the Root Driver.

```mermaid
graph LR
  %% nodes (IDs use underscores to be Mermaid-safe; labels show card ids)
  driver_root["driver:root<br/>Root Driver"]
  driver_read["driver:readability<br/>Readability"]
  driver_trace["driver:traceability<br/>Traceability"]
  driver_formats["driver:formats<br/>Formats: JSON-first"]
  driver_auto["driver:automation<br/>Automation"]
  driver_simp["driver:simplicity<br/>Simplicity"]
  driver_interop["driver:interoperability<br/>Interoperability"]
  driver_sec["driver:security-and-privacy<br/>Security & Privacy"]
  driver_ver["driver:versioning-lifecycle<br/>Versioning & Lifecycle"]
  driver_test["driver:testability<br/>Testability"]
  driver_gov["driver:governance-collaboration<br/>Governance & Collaboration"]

  %% derivation links
  driver_root -->|derives| driver_read
  driver_root -->|derives| driver_trace
  driver_root -->|derives| driver_formats
  driver_root -->|derives| driver_auto
  driver_root -->|derives| driver_simp
  driver_root -->|derives| driver_interop
  driver_root -->|derives| driver_sec
  driver_root -->|derives| driver_ver
  driver_root -->|derives| driver_test
  driver_root -->|derives| driver_gov

  %% (no styling applied)
```

Notes:

- Node labels show the card `id` and a short name; update labels if you rename or add driver cards.
- This is a documentation view only; the canonical relationships live on the card `relations` and/or explicit `link` artifacts under `examples/cards/`.
- To render locally, use a Markdown viewer that supports Mermaid, or preview via GitHub Pages.
