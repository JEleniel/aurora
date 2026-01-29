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

## Views
