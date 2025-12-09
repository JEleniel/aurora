#!/usr/bin/env python3
"""Generate human-readable Markdown docs for cards and links.

Usage: python3 tools/generate_docs.py

This script reads JSON files from `examples/cards/` and `examples/links/`
and writes corresponding Markdown files under `docs/cards/` and `docs/links/`.
"""
import json
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
CARDS_DIR = ROOT / "examples" / "cards"
LINKS_DIR = ROOT / "examples" / "links"
OUT_CARDS = ROOT / "docs" / "cards"
OUT_LINKS = ROOT / "docs" / "links"
SCHEMAS_DIR = ROOT / "schemas"


def load_schemas():
    """Load JSON Schema files from `schemas/` and return a map keyed by type name.

    Keys are derived from filenames like `driver.schema.json` -> `driver`.
    """
    schemas = {}
    if not SCHEMAS_DIR.exists():
        return schemas
    for p in sorted(SCHEMAS_DIR.glob("*.json")):
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
            key = p.name.split(".")[0].lower()
            schemas[key] = data
        except Exception:
            # ignore bad schema files
            continue
    return schemas


def generate_aurora_definition_card(schemas_map):
    """Create an `aurora-definition` card JSON derived from the available schemas.

    The generated card is written to `examples/cards/aurora-definition.json`.
    It contains a summary list of each schema (filename, title, description, properties).
    """
    card = {}
    card_id = "aurora:definition"
    card["id"] = card_id
    card["type"] = "definition"
    card["name"] = "Aurora Definition"
    card["description"] = (
        "Canonical Aurora definition generated from the repository JSON Schemas. "
        "This card is auto-generated from `schemas/*.json`. Treat schemas as the single source of truth."
    )
    # Use timezone-aware UTC timestamps
    now = datetime.now(timezone.utc)
    card["version"] = now.strftime("0.0.0+generated.%Y%m%d%H%M%S")
    card["status"] = "proposed"
    card["metadata"] = {"serialization": ["json"], "generated_by": "tools/generate_docs.py"}
    card["provenance"] = {"generated_at": now.isoformat()}

    # Include full inline schemas as the canonical definition
    schemas_out = {}
    for key, s in sorted(schemas_map.items()):
        schemas_out[key + ".json"] = s

    card["schemas"] = schemas_out

    out_path = CARDS_DIR / "aurora-definition.json"
    out_path.write_text(json.dumps(card, indent=2), encoding="utf-8")
    return out_path


def safe_filename(s: str) -> str:
    return s.replace(":", "-").replace("/", "-")


def mk_dirs():
    OUT_CARDS.mkdir(parents=True, exist_ok=True)
    OUT_LINKS.mkdir(parents=True, exist_ok=True)


def render_card(path: Path):
    data = json.loads(path.read_text(encoding="utf-8"))
    card_id = data.get("id", path.stem)
    name = data.get("name", "(no name)")
    out_lines = []
    out_lines.append(f"# {name}")
    out_lines.append("")
    out_lines.append("---")
    out_lines.append("")
    out_lines.append(f"- **ID**: `{card_id}`")
    out_lines.append(f"- **Type**: `{data.get('type', '')}`")
    if data.get("version"):
        out_lines.append(f"- **Version**: `{data.get('version')}`")
    if data.get("status"):
        out_lines.append(f"- **Status**: `{data.get('status')}`")
    if data.get("priority"):
        out_lines.append(f"- **Priority**: `{data.get('priority')}`")
    if data.get("owner"):
        out_lines.append(f"- **Owner**: `{data.get('owner')}`")

    # Field explanations: prefer schema descriptions when available
    schemas = load_schemas()
    schema = None
    t = data.get("type", "").lower()
    if t and t in schemas:
        schema = schemas[t]

    out_lines.append("")
    out_lines.append("## Field Reference")
    props = {}
    if schema and isinstance(schema, dict):
        props = schema.get("properties", {})

    # fallback static explanations
    fallback = {
        "id": "Canonical identifier for the card (namespace:type).",
        "type": "Card type (driver, requirement, actor, behavior, interface, constraint, link, view, etc.).",
        "name": "Human-friendly title using verb+noun where applicable.",
        "description": "Plain-language explanation of the card's intent and scope.",
        "version": "Semver for the card's content (MAJOR.MINOR.PATCH).",
        "status": "Lifecycle state (proposed, accepted, deprecated, retired, etc.).",
        "priority": "Optional top-level priority (high/medium/low).",
        "owner": "Optional top-level owner or team responsible for the card.",
        "relations": "References to other card IDs indicating logical relationships.",
        "links": "Explicit link artifact IDs that encode richer relationship metadata.",
        "acceptance_criteria": "Machine-or-human-verifiable criteria for satisfying a requirement.",
        "rationale": "Why this card exists; derivation or justification.",
        "provenance": "Source and origin metadata (source, owner, version).",
        "audit_history": "Chronological events describing create/update actions with timestamps.",
        "metadata": "Format and serialization metadata for tooling (format, serialization).",
    }

    # Use schema property descriptions when present
    if props:
        for k, v in props.items():
            desc = v.get("description") if isinstance(v, dict) else None
            if not desc:
                desc = fallback.get(k, "")
            # indicate required
            req = " (required)" if k in schema.get("required", []) else ""
            out_lines.append(f"- **{k}**: {desc}{req}")
    else:
        for k, v in fallback.items():
            out_lines.append(f"- **{k}**: {v}")
    out_lines.append("")

    out_lines.append("## Description")
    out_lines.append("")
    out_lines.append(data.get("description", "(no description)"))
    out_lines.append("")

    if data.get("acceptance_criteria"):
        out_lines.append("## Acceptance Criteria")
        out_lines.append("")
        for c in data.get("acceptance_criteria", []):
            out_lines.append(f"- {c}")
        out_lines.append("")

    if data.get("rationale"):
        out_lines.append("## Rationale")
        out_lines.append("")
        out_lines.append(data.get("rationale"))
        out_lines.append("")

    if data.get("provenance"):
        out_lines.append("## Provenance")
        out_lines.append("")
        prov = data.get("provenance", {})
        for k, v in prov.items():
            out_lines.append(f"- **{k}**: `{v}`")
        out_lines.append("")

    if data.get("audit_history"):
        out_lines.append("## Audit History")
        out_lines.append("")
        for e in data.get("audit_history", []):
            ev = e.get("event", "")
            by = e.get("by", "")
            at = e.get("event_time", "")
            out_lines.append(f"- {ev} — by {by} at {at}")
        out_lines.append("")

    # Related cards/links (provide doc links)
    related = []
    if data.get("relations"):
        rel_links = []
        for r in data.get("relations", []):
            fname = safe_filename(r) + ".md"
            rel_links.append(f"[{r}](../cards/{fname})")
        related.append(("Relations", rel_links))
    if data.get("links"):
        link_links = []
        for l in data.get("links", []):
            fname = safe_filename(l) + ".md"
            link_links.append(f"[{l}](../links/{fname})")
        related.append(("Links", link_links))

    if related:
        out_lines.append("## Related")
        out_lines.append("")
        for title, items in related:
            out_lines.append(f"**{title}**: {', '.join(items)}")
        out_lines.append("")

    out_lines.append("## Raw JSON")
    out_lines.append("")
    out_lines.append("```json")
    out_lines.append(json.dumps(data, indent=2))
    out_lines.append("```")

    filename = safe_filename(card_id) + ".md"
    content = normalize_and_write(out_lines)
    (OUT_CARDS / filename).write_text(content, encoding="utf-8")


def render_link(path: Path):
    data = json.loads(path.read_text(encoding="utf-8"))
    link_id = data.get("id", path.stem)
    out_lines = []
    out_lines.append(f"# Link `{link_id}`")
    out_lines.append("")
    out_lines.append(f"- **Type**: `{data.get('type', '')}`")
    out_lines.append(f"- **Link Type**: `{data.get('link_type', '')}`")
    out_lines.append(f"- **Source**: `{data.get('source', '')}`")
    out_lines.append(f"- **Target**: `{data.get('target', '')}`")
    if data.get("strength"):
        out_lines.append(f"- **Strength**: `{data.get('strength')}`")
    out_lines.append("")
    if data.get("rationale"):
        out_lines.append("## Rationale")
        out_lines.append("")
        out_lines.append(data.get("rationale"))
        out_lines.append("")

    if data.get("audit_history"):
        out_lines.append("## Audit History")
        out_lines.append("")
        for e in data.get("audit_history", []):
            ev = e.get("event", "")
            by = e.get("by", "")
            at = e.get("event_time", "")
            out_lines.append(f"- {ev} — by {by} at {at}")
        out_lines.append("")

    out_lines.append("## Raw JSON")
    out_lines.append("")
    out_lines.append("```json")
    out_lines.append(json.dumps(data, indent=2))
    out_lines.append("```")

    filename = safe_filename(link_id) + ".md"
    content = normalize_and_write(out_lines)
    (OUT_LINKS / filename).write_text(content, encoding="utf-8")


def normalize_and_write(lines: list) -> str:
    """Normalize markdown lines to comply with markdownlint rules in .markdownlint.json.

    - Trim trailing spaces
    - Collapse multiple blank lines to a single blank line
    - Ensure there is a blank line before lists and fenced code blocks
    - Ensure fenced code blocks have surrounding blank lines
    - Return the final content string with a single trailing newline
    """
    stripped = [ln.rstrip() for ln in lines]
    out = []
    prev_blank = False
    in_fence = False

    def last_nonblank():
        for x in reversed(out):
            if x.strip() != '':
                return x
        return None

    for i, ln in enumerate(stripped):
        s = ln
        # detect fence start/end
        if s.startswith('```'):
            # ensure blank line before fence
            if out and out[-1].strip() != '':
                out.append('')
            out.append(s)
            prev_blank = False
            in_fence = not in_fence
            continue

        if in_fence:
            # inside code block - preserve content as-is (already rstripped)
            out.append(s)
            prev_blank = False
            continue

        # collapse multiple blank lines
        if s.strip() == '':
            if not prev_blank:
                out.append('')
                prev_blank = True
            # else skip extra blank
            continue

        # headings: ensure a blank line AFTER headings (not before)
        if s.lstrip().startswith('#'):
            # append heading
            out.append(s)
            prev_blank = False
            # ensure next non-blank content will be preceded by a single blank
            # we implement by inserting a blank now if next input line is non-blank
            # but to avoid peeking, set flag by appending a blank only if next line
            # is not blank in the original input
            next_line = stripped[i+1] if i+1 < len(stripped) else ''
            if next_line.strip() != '':
                out.append('')
                prev_blank = True
            continue

        # list items handling: ensure one blank before a list when previous
        # non-blank is a paragraph, but ensure no blank between consecutive list items
        if s.startswith(('- ', '* ')):
            last_nb = last_nonblank()
            if last_nb is None:
                # at top of file, no blank needed
                pass
            elif last_nb.lstrip().startswith('#'):
                # if last nonblank is a heading, ensure exactly one blank exists
                if out and out[-1].strip() != '':
                    out.append('')
            elif last_nb.startswith(('- ', '* ')):
                # previous nonblank is also a list item; ensure no blank between
                if out and out[-1].strip() == '':
                    # remove accidental blank between list items
                    out.pop()
            else:
                # previous nonblank is a paragraph or other block: ensure blank
                if out and out[-1].strip() != '':
                    out.append('')
            out.append(s)
            prev_blank = False
            continue

        # default: normal paragraph/content
        out.append(s)
        prev_blank = False

    # Remove leading blank lines
    while out and out[0].strip() == '':
        out.pop(0)
    # Ensure single trailing newline
    content = '\n'.join(out).rstrip() + '\n'
    return content


def main():
    mk_dirs()
    # Load schemas and (re)generate aurora-definition card
    schemas = load_schemas()
    try:
        gen_path = generate_aurora_definition_card(schemas)
        print(f"Generated aurora definition card at {gen_path}")
    except Exception as e:
        print(f"Skipping aurora-definition generation: {e}")

    card_files = sorted(CARDS_DIR.glob("*.json"))
    link_files = sorted(LINKS_DIR.glob("*.json")) if LINKS_DIR.exists() else []

    for p in card_files:
        try:
            render_card(p)
            print(f"Wrote card doc for {p.name}")
        except Exception as e:
            print(f"Failed to render {p}: {e}")

    for p in link_files:
        try:
            render_link(p)
            print(f"Wrote link doc for {p.name}")
        except Exception as e:
            print(f"Failed to render link {p}: {e}")

    # Generate a simple docs index linking to the aurora-definition and all card/link docs
    try:
        idx_lines = []
        idx_lines.append("# Aurora Documentation Index")
        idx_lines.append("")
        idx_lines.append("This index was generated by `tools/generate_docs.py`. The `aurora-definition` card contains the canonical JSON Schemas.")
        idx_lines.append("")
        # Link to aurora-definition
        ad_name = safe_filename('aurora:definition') + '.md'
        idx_lines.append("## Canonical Definition")
        idx_lines.append("")
        idx_lines.append(f"- [Aurora Definition](cards/{ad_name})")
        idx_lines.append("")
        # Cards
        idx_lines.append("## Cards")
        idx_lines.append("")
        for p in card_files:
            fid = json.loads(p.read_text(encoding='utf-8')).get('id', p.stem)
            fname = safe_filename(fid) + '.md'
            idx_lines.append(f"- [{fid}](cards/{fname})")
        idx_lines.append("")
        # Links
        if link_files:
            idx_lines.append("## Links")
            idx_lines.append("")
            for p in link_files:
                fid = json.loads(p.read_text(encoding='utf-8')).get('id', p.stem)
                fname = safe_filename(fid) + '.md'
                idx_lines.append(f"- [{fid}](links/{fname})")
            idx_lines.append("")

        index_path = ROOT / 'docs' / 'index.md'
        content = '\n'.join([ln.rstrip() for ln in idx_lines]).rstrip() + '\n'
        index_path.write_text(content, encoding='utf-8')
        print(f"Wrote docs index to {index_path}")
    except Exception as e:
        print(f"Failed to write docs index: {e}")


if __name__ == "__main__":
    main()
