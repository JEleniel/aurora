#!/usr/bin/env python3
"""Generate a Mermaid diagram mapping components to requirements.

This script finds component cards (interfaces, behaviors, actors, constraints,
logical_component) and requirement cards, then maps components to requirements
using explicit link artifacts where present and lightweight keyword inference
when explicit links are missing.

Writes `docs/diagrams/components-to-requirements.md`.
"""
from pathlib import Path
import json
import re
from collections import defaultdict

ROOT = Path(__file__).resolve().parents[1]
CARDS_DIR = ROOT / 'examples' / 'cards'
LINKS_DIR = ROOT / 'examples' / 'links'
OUT_DIR = ROOT / 'docs' / 'diagrams'


def load_json_files(directory):
    objs = {}
    for p in sorted(directory.glob('*.json')):
        try:
            objs[p.stem] = json.loads(p.read_text(encoding='utf-8'))
        except Exception:
            continue
    return objs


def sanitize_id(cid: str) -> str:
    return re.sub(r'[^0-9A-Za-z_]', '_', cid)


def keywords(text):
    if not text:
        return set()
    # split on non-word, remove short words
    toks = re.split(r'\W+', text.lower())
    return set(t for t in toks if len(t) > 3)


def find_mappings(cards, links):
    # Identify requirements and components
    requirements = {k: v for k, v in cards.items() if v.get('type') == 'requirement'}
    components = {k: v for k, v in cards.items() if v.get('type') in ('interface', 'behavior', 'actor', 'constraint', 'logical_component', 'deployable_node')}

    # explicit mappings via link artifacts (either direction)
    explicit = defaultdict(list)
    for l in links.values():
        src = l.get('source')
        tgt = l.get('target')
        ltype = l.get('link_type', l.get('type', 'link'))
        if not src or not tgt:
            continue
        # normalize ids by looking up in cards keys (which are filenames stems or explicit id)
        if src in requirements and tgt in components:
            explicit[tgt].append((src, ltype))
        elif tgt in requirements and src in components:
            explicit[src].append((tgt, ltype))

    # inferred mappings via keyword overlap
    inferred = defaultdict(list)
    req_kw = {rid: keywords((r.get('name','') + ' ' + r.get('description',''))) for rid, r in requirements.items()}
    comp_kw = {cid: keywords((c.get('name','') + ' ' + c.get('description',''))) for cid, c in components.items()}

    for cid, cset in comp_kw.items():
        for rid, rset in req_kw.items():
            if not rset or not cset:
                continue
            # compute intersection ratio
            inter = cset & rset
            if inter:
                inferred[cid].append((rid, sorted(list(inter))))

    return components, requirements, explicit, inferred


def build_mermaid(components, requirements, explicit, inferred):
    lines = []
    lines.append('# Components → Requirements')
    lines.append('Generated from `examples/cards/` and `examples/links/`')
    lines.append('')
    lines.append('```mermaid')
    lines.append('flowchart LR')
    lines.append('')

    # Left: components
    lines.append('subgraph Components')
    for cid, c in components.items():
        nid = sanitize_id(cid)
        label = f"{c.get('name','')}\\n({cid})"
        lines.append(f'  {nid}["{label}"]')
    lines.append('end')
    lines.append('')

    # Right: requirements
    lines.append('subgraph Requirements')
    for rid, r in requirements.items():
        nid = sanitize_id(rid)
        label = f"{r.get('name','')}\\n({rid})"
        lines.append(f'  {nid}["{label}"]')
    lines.append('end')
    lines.append('')

    # Draw explicit (solid) and inferred (dashed)
    for comp_id, pairs in explicit.items():
        s = sanitize_id(comp_id)
        for req_id, ltype in pairs:
            t = sanitize_id(req_id)
            lbl = ltype or ''
            lines.append(f'{s} -->|{lbl}| {t}')

    # For inferred, include small label with matched keywords (joined)
    for comp_id, items in inferred.items():
        # skip if already explicit maps to the same requirement
        existing_reqs = {r for r, _ in explicit.get(comp_id, [])}
        s = sanitize_id(comp_id)
        for req_id, kws in items:
            if req_id in existing_reqs:
                continue
            t = sanitize_id(req_id)
            lbl = ','.join(kws[:3])
            lines.append(f'{s} -.->|inferred: {lbl}| {t}')

    lines.append('```')
    return '\n'.join(lines) + '\n'


def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    cards = load_json_files(CARDS_DIR)
    links = load_json_files(LINKS_DIR)
    components, requirements, explicit, inferred = find_mappings(cards, links)
    mermaid = build_mermaid(components, requirements, explicit, inferred)
    out_path = OUT_DIR / 'components-to-requirements.md'
    out_path.write_text(mermaid, encoding='utf-8')
    print(f'Wrote components→requirements diagram to {out_path}')


if __name__ == '__main__':
    main()
