/* Client-side diagram generators for Aurora
 * Accepts arrays of cards and links and returns Mermaid markdown strings.
 */

export type Card = {
  id: string;
  type?: string;
  name?: string;
  description?: string;
};

export type Link = {
  id?: string;
  source: string;
  target: string;
  link_type?: string;
  description?: string;
};

function sanitizeLabel(s?: string) {
  if (!s) return '';
  return String(s).replace(/\n/g, ' ').replace(/"/g, "'");
}

export function generateAuroraLogical(cards: Card[], links: Link[]): string {
  // Group cards by type for subgraphs
  const groups: Record<string, Card[]> = {};
  cards.forEach((c) => {
    const t = c.type || 'Definition';
    groups[t] = groups[t] || [];
    groups[t].push(c);
  });

  const lines: string[] = [];
  lines.push('```mermaid');
  lines.push('flowchart LR');

  // render subgraphs
  Object.keys(groups).forEach((type) => {
    lines.push(`  subgraph ${type}`);
    groups[type].forEach((c) => {
      const label = sanitizeLabel(c.name || c.id);
      lines.push(`    ${c.id}["${label}"]`);
    });
    lines.push('  end');
  });

  // render links
  links.forEach((l) => {
    const src = l.source;
    const dst = l.target;
    const arrow =
      l.link_type && l.link_type.toLowerCase().includes('inferred')
        ? '-->'
        : '---';
    const label = sanitizeLabel(l.description || l.link_type || '');
    lines.push(`  ${src} ${arrow} ${dst}${label ? ` : ${label}` : ''}`);
  });

  lines.push('```');
  return lines.join('\n') + '\n';
}

export function generateComponentsRequirements(
  cards: Card[],
  links: Link[]
): string {
  // Components are cards with type "Driver" or "Component"; requirements have type "Requirement"
  const components = cards.filter(
    (c) =>
      (c.type || '').toLowerCase().includes('driver') ||
      (c.type || '').toLowerCase().includes('component')
  );
  const requirements = cards.filter((c) =>
    (c.type || '').toLowerCase().includes('requirement')
  );

  const lines: string[] = [];
  lines.push('```mermaid');
  lines.push('flowchart LR');

  lines.push('  subgraph Components');
  components.forEach((c) => {
    lines.push(`    ${c.id}["${sanitizeLabel(c.name || c.id)}"]`);
  });
  lines.push('  end');

  lines.push('  subgraph Requirements');
  requirements.forEach((r) => {
    lines.push(`    ${r.id}["${sanitizeLabel(r.name || r.id)}"]`);
  });
  lines.push('  end');

  // Use explicit links first (source -> target where either side is component/requirement)
  links.forEach((l) => {
    const srcComp = components.find((c) => c.id === l.source);
    const dstReq = requirements.find((r) => r.id === l.target);
    if (srcComp && dstReq) {
      lines.push(
        `  ${l.source} --- ${l.target}${l.link_type ? ` : ${sanitizeLabel(l.link_type)}` : ''}`
      );
    }
  });

  // As a fallback, try simple keyword matching from names (inferred edges)
  components.forEach((c) => {
    requirements.forEach((r) => {
      const ck = (c.name || c.id || '').toLowerCase();
      const rk = (r.name || r.id || '').toLowerCase();
      if (ck && rk && rk.includes(ck.split(/\s+/)[0])) {
        // inferred
        lines.push(`  ${c.id} --> ${r.id} : inferred`);
      }
    });
  });

  lines.push('```');
  return lines.join('\n') + '\n';
}

export default { generateAuroraLogical, generateComponentsRequirements };
