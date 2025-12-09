#!/usr/bin/env python3
"""Generate a Mermaid logical diagram of Aurora from card and link artifacts.

Writes `docs/diagrams/aurora-logical.md` with a Mermaid `flowchart LR` diagram
showing nodes grouped by card `type` and edges from explicit link artifacts and
relations arrays.
"""
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
CARDS_DIR = ROOT / 'examples' / 'cards'
LINKS_DIR = ROOT / 'examples' / 'links'
OUT = ROOT / 'docs' / 'diagrams'


def sanitize_id(cid: str) -> str:
    # Mermaid node ids must be alphanumeric and not start with a digit;
    # replace non-alphanum with underscore and prefix with 'n' if starting digit
    s = re.sub(r'[^0-9A-Za-z_]', '_', cid)
    if re.match(r'^[0-9]', s):
        s = 'n' + s
    return s


def load_cards():
    cards = {}
    for p in sorted(CARDS_DIR.glob('*.json')):
        try:
            data = json.loads(p.read_text(encoding='utf-8'))
            cid = data.get('id', p.stem)
            cards[cid] = {
                'type': data.get('type', 'unknown'),
                'name': data.get('name', cid),
                'relations': data.get('relations', []),
            }
        except Exception:
            continue
    return cards


def load_links():
    links = []
    if not LINKS_DIR.exists():
        return links
    for p in sorted(LINKS_DIR.glob('*.json')):
        try:
            data = json.loads(p.read_text(encoding='utf-8'))
            links.append({
                'id': data.get('id', p.stem),
                'type': data.get('link_type', data.get('type', 'link')),
                'source': data.get('source'),
                'target': data.get('target'),
            })
        except Exception:
            continue
    return links


def build_mermaid(cards, links):
    types = {}
    for cid, c in cards.items():
        types.setdefault(c['type'], []).append((cid, c))

    lines = []
    lines.append('# Aurora Logical Diagram')
    lines.append('This diagram is generated from `examples/cards/` and `examples/links/`.')
    lines.append('')
    lines.append('```mermaid')
    lines.append('flowchart LR')
    lines.append('')

    # Create subgraphs for common types in a stable order
    order = ['driver', 'requirement', 'interface', 'actor', 'behavior', 'constraint', 'definition', 'unknown']
    for t in order:
        if t in types:
            lines.append(f'subgraph {t.capitalize()}s')
            for cid, c in types[t]:
                nid = sanitize_id(cid)
                label = f"{c['name']}\\n({cid})"
                lines.append(f'  {nid}["{label}"]')
            lines.append('end')
            lines.append('')

    # Any remaining types not in order
    for t, entries in types.items():
        if t in order:
            continue
        lines.append(f'subgraph {t.capitalize()}s')
        for cid, c in entries:
            nid = sanitize_id(cid)
            label = f"{c['name']}\\n({cid})"
            lines.append(f'  {nid}["{label}"]')
        lines.append('end')
        lines.append('')

    # Edges from links (solid arrows)
    for l in links:
        src = l.get('source')
        tgt = l.get('target')
        if not src or not tgt:
            continue
        s_n = sanitize_id(src)
        t_n = sanitize_id(tgt)
        label = l.get('type', '')
        lines.append(f'{s_n} -->|{label}| {t_n}')

    # Edges from relations arrays (dashed lines)
    for cid, c in cards.items():
        for r in c.get('relations', []):
            s_n = sanitize_id(cid)
            t_n = sanitize_id(r)
            lines.append(f'{s_n} -.-> {t_n}')

    lines.append('```')
    return '\n'.join(lines) + '\n'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    cards = load_cards()
    links = load_links()
    mermaid = build_mermaid(cards, links)
    out_path = OUT / 'aurora-logical.md'
    out_path.write_text(mermaid, encoding='utf-8')
    print(f'Wrote logical diagram to {out_path}')


if __name__ == '__main__':
    main()
