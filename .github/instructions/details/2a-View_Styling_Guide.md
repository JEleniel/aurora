# View Styling Guide

## Model-driven Styling

- The Card Definition Guide includes a shape and icon for each card type that should be used when rendering views.
    + The icon should be rendered on the left edge of the card, inside any other markers (like component boxes). It should be scaled to approximately four lines height.
- The text on each card should be centered and not overlap the icon:
    + Line 1: **<card_type> (<card_subtype>**)
    + Line 2: **<id>**
    + Line 3: -blank-
    + Line 4: <description>
- The link to a Note card should be a dotted line.
- Boundary cards should be rendered as dashed lines enclosing the contained cards. When traversing recursively, stop when reaching a card already seen, a leaf card, or a matching Boundary (End) card. The invariants guarantee that one of these will always happen.

## Colors

- Background color: `#FFFFFF`
- Edge line color: `#000000`
- Edge label font color: `#000000`

### Card Type Styling

- BND: stroke-dasharray:5 5,stroke-width:4;
- MIS: fill:#022c22,color:#FFFFFF
- DRI: fill:#064e3b,color:#FFFFFF
- REQ: fill:#065f46,color:#FFFFFF
- ADR: fill:#1f2937,color:#FFFFFF;
- CAP: fill:#052e16,color:#FFFFFF
- FEA: fill:#14532d,color:#FFFFFF
- ACT: fill:#1a2e05,color:#FFFFFF
- STR: fill:#365314,color:#FFFFFF;
- CON: fill:#422006,color:#FFFFFF
- CTL: fill:#713f12,color:#FFFFFF
- CNS: fill:#854d0e,color:#FFFFFF;
- SYS: fill:#172554,color:#FFFFFF
- APP: fill:#1e3a8a,color:#FFFFFF
- COM: fill:#1e40af,color:#FFFFFF
- INT: fill:#082f49,color:#FFFFFF
- ART: fill:#1e293b,color:#FFFFFF
- AST: fill:#334155,color:#FFFFFF;
- DTS: fill:#075985,color:#FFFFFF
- CLS: fill:#0f172a,color:#FFFFFF;
- TES: fill:#022c22,color:#FFFFFF;
- DEP: fill:#1e1b4b,color:#FFFFFF;
- NOD: fill:#312e81,color:#FFFFFF;
- NIN: fill:#3730a3,color:#FFFFFF;
- PRO: fill:#2e1065,color:#FFFFFF
- ATV: fill:#4c1d95,color:#FFFFFF
- EVT: fill:#5b21b6,color:#FFFFFF
- STM: fill:#4a044e,color:#FFFFFF
- STA: fill:#701a75,color:#FFFFFF
- RIS: fill:#881337,color:#FFFFFF;
- THR: fill:#4c0519,color:#FFFFFF;
- NOT: fill:#1f2937,color:#FFFFFF;
