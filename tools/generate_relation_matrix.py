#!/usr/bin/env python3
"""
Generate an element-relationship matrix by scanning JSON card and link files.

Usage:
  python tools/generate_relation_matrix.py

Outputs:
  - docs/matrices/element-relationship-matrix.md

This script is intentionally permissive: it will look for objects with
`id`+`type` to identify cards, `source`+`target` to identify links, and
accept both `type` and `link_type` for link naming because examples vary.
"""
import json
import os
from collections import defaultdict
from pathlib import Path
from typing import Dict, Set, Tuple, List


ROOT = Path(__file__).resolve().parents[1]
SCHEMAS_DIR = ROOT / "schemas"
OUT_DIR = ROOT / "docs" / "matrices"
OUT_FILE = OUT_DIR / "element-relationship-matrix.md"


def load_json(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf8"))
    except Exception:
        return None


def collect_card_types() -> List[str]:
    schema_path = SCHEMAS_DIR / "card.schema.json"
    data = load_json(schema_path)
    if not data:
        return []
    return data.get("properties", {}).get("type", {}).get("enum", [])


def collect_link_types() -> List[str]:
    schema_path = SCHEMAS_DIR / "link.schema.json"
    data = load_json(schema_path)
    if not data:
        return []
    return data.get("properties", {}).get("type", {}).get("enum", [])


def scan_repo_for_cards_and_links(root: Path) -> Tuple[Dict[str, str], List[Dict]]:
    id_to_type: Dict[str, str] = {}
    links: List[Dict] = []

    for dirpath, _, files in os.walk(root):
        # skip .git and node_modules
        if any(part in (".git", "node_modules") for part in Path(dirpath).parts):
            continue
        for fname in files:
            if not fname.endswith(".json"):
                continue
            p = Path(dirpath) / fname
            data = load_json(p)
            if data is None:
                continue

            # Helper: if object contains id+type, treat as card
            def try_register_card(obj):
                if isinstance(obj, dict) and obj.get("id") and obj.get("type"):
                    id_to_type[str(obj.get("id"))] = str(obj.get("type"))

            # If top-level is array, iterate
            if isinstance(data, list):
                for it in data:
                    try_register_card(it)
                    # link detection
                    if isinstance(it, dict) and it.get("source") and it.get("target"):
                        links.append(it)
                continue

            # If object has 'cards' array
            if isinstance(data, dict) and isinstance(data.get("cards"), list):
                for c in data.get("cards", []):
                    try_register_card(c)

            # If object itself is a card
            try_register_card(data)

            # Link detection: object with source+target
            if isinstance(data, dict) and data.get("source") and data.get("target"):
                links.append(data)

            # Some link examples use 'links' array
            if isinstance(data, dict) and isinstance(data.get("links"), list):
                for l in data.get("links", []):
                    if isinstance(l, dict) and l.get("source") and l.get("target"):
                        links.append(l)

    return id_to_type, links


def build_matrix(id_to_type: Dict[str, str], links: List[Dict]) -> Tuple[Dict[Tuple[str, str], Set[str]], List[Dict]]:
    matrix: Dict[Tuple[str, str], Set[str]] = defaultdict(set)
    unknown_links: List[Dict] = []
    # Try to load relationship inverses mapping to normalize link type names
    mapping_path = ROOT / "schemas" / "relationship-inverses.json"
    norm_map = {}
    if mapping_path.exists():
        try:
            raw = load_json(mapping_path)
            for k, v in (raw or {}).items():
                norm_map[k] = k
                # map synonyms to canonical key
                for s in v.get("synonyms", []) if isinstance(v, dict) else []:
                    norm_map[s] = k
        except Exception:
            norm_map = {}

    for l in links:
        src = l.get("source")
        tgt = l.get("target")
        # link type might be stored under 'type' or 'link_type'
        ltype = l.get("type") or l.get("link_type") or l.get("linkType")
        # normalize link type via relationship-inverses.json if present
        if isinstance(ltype, str):
            lt = ltype.strip()
            if lt in norm_map:
                ltype = norm_map[lt]
            else:
                # try lower-cased and trimmed variants
                lc = lt.lower()
                if lc in norm_map:
                    ltype = norm_map[lc]
                else:
                    # if the canonical key equals the displayed token, keep as-is
                    ltype = lt
        if not (src and tgt):
            continue
        src_t = id_to_type.get(str(src))
        tgt_t = id_to_type.get(str(tgt))
        if not src_t or not tgt_t:
            unknown_links.append({"source": src, "target": tgt, "type": ltype})
            continue
        matrix[(src_t, tgt_t)].add(str(ltype) if ltype else "(unspecified)")

    return matrix, unknown_links


def render_markdown(matrix: Dict[Tuple[str, str], Set[str]], card_types: List[str], link_types: List[str], unknown_links: List[Dict]) -> str:
    # Build header
    header = ["Source \\ Target"] + card_types
    lines = []
    lines.append("# Element → Relationship Matrix")
    lines.append("")
    lines.append("This matrix was generated by `tools/generate_relation_matrix.py` by scanning card and link JSON files in the repository.")
    lines.append("")
    # table header
    lines.append("| " + " | ".join(header) + " |")
    lines.append("|" + " --- |" * len(header))

    for src in card_types:
        row = [src]
        for tgt in card_types:
            cell = matrix.get((src, tgt))
            if cell:
                row.append(", ".join(sorted(cell)))
            else:
                row.append("")
        lines.append("| " + " | ".join(row) + " |")

    lines.append("")
    lines.append("## Observed link types")
    lines.append("")
    lines.append("Allowed link types from schema: `" + ", ".join(link_types) + "`.")
    lines.append("")
    if unknown_links:
        lines.append("## Links with unknown endpoints")
        lines.append("")
        lines.append("The following links reference card IDs that were not found when scanning for cards:")
        lines.append("")
        for u in unknown_links:
            lines.append(f"- source: `{u.get('source')}`, target: `{u.get('target')}`, type: `{u.get('type')}`")
        lines.append("")

    return "\n".join(lines)


def main():
    print("Scanning repository for cards and links...")
    card_types = collect_card_types()
    link_types = collect_link_types()
    id_to_type, links = scan_repo_for_cards_and_links(ROOT)
    matrix, unknown = build_matrix(id_to_type, links)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    md = render_markdown(matrix, card_types, link_types, unknown)
    OUT_FILE.write_text(md, encoding="utf8")
    print(f"Wrote matrix to {OUT_FILE}")
    print(f"Found {len(id_to_type)} cards and {len(links)} links (unknown endpoints: {len(unknown)})")


if __name__ == "__main__":
    main()
