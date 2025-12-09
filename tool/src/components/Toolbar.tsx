import React from 'react';
import { saveText, openText, saveSvg } from '../lib/io';
import {
  generateAuroraLogical,
  generateComponentsRequirements,
  Card,
  Link,
} from '../lib/diagramGenerators';
import mermaid from 'mermaid';

export default function Toolbar({
  source,
  onOpen,
  onStatus,
  onLoadWorkspace,
  workspace,
  onLayoutChange,
}: {
  source: string;
  onOpen: (s: string) => void;
  onStatus?: (msg: string) => void;
  onLoadWorkspace?: (w: { cards: Card[]; links: Link[] }) => void;
  workspace?: { cards: Card[]; links: Link[] };
  onLayoutChange?: (layout: string) => void;
}) {
  const [layout, setLayout] = React.useState<string>('dagre');
  // Notify parent of layout changes via callback prop
  // parent should provide `onLayoutChange` in its props if it wants updates
  // (kept local state so select remains controlled locally)

  const onSave = async () => {
    const name = 'aurora-card.json';
    const path = await saveText(name, source);
    if (path && typeof onStatus === 'function') onStatus(`Saved ${path}`);
    else if (typeof onStatus === 'function') onStatus('Saved (download)');
  };

  const onOpenClick = async () => {
    const res = await openText();
    if (res?.contents) {
      onOpen(res.contents);
      if (res.path && typeof onStatus === 'function')
        onStatus(`Opened ${res.path}`);
      else if (typeof onStatus === 'function') onStatus('Opened file');
    }
  };

  const onLoadWorkspaceClick = async () => {
    const res = await openText();
    if (res?.contents) {
      try {
        const parsed = JSON.parse(res.contents);
        const cards = Array.isArray(parsed.cards) ? parsed.cards : [];
        const links = Array.isArray(parsed.links) ? parsed.links : [];
        if (onLoadWorkspace) onLoadWorkspace({ cards, links });
        if (typeof onStatus === 'function') onStatus('Workspace loaded');
      } catch (e) {
        if (typeof onStatus === 'function')
          onStatus('Failed to parse workspace JSON');
      }
    }
  };

  const onGenerateDiagrams = async () => {
    try {
      const cards = workspace?.cards || [];
      const links = workspace?.links || [];
      const aurora = generateAuroraLogical(cards, links);
      const compReq = generateComponentsRequirements(cards, links);
      await saveText('docs/diagrams/aurora-logical.md', aurora);
      await saveText('docs/diagrams/components-to-requirements.md', compReq);
      if (typeof onStatus === 'function')
        onStatus('Generated diagrams and saved to docs/diagrams/');
    } catch (e) {
      if (typeof onStatus === 'function') onStatus('Diagram generation failed');
    }
  };

  const onExportSvg = async () => {
    try {
      const code =
        source && source.trim().length > 0 ? source : 'flowchart TD\n A[Empty]';
      mermaid.initialize({ startOnLoad: false, theme: 'dark' });
      const id = 'm' + Math.random().toString(36).slice(2);
      mermaid.render(id, code, (svg) => {
        saveSvg('diagram.svg', svg);
        if (typeof onStatus === 'function') onStatus('Exported SVG');
      });
    } catch (e) {
      if (typeof onStatus === 'function') onStatus('Export failed');
    }
  };

  const onNewCard = () => {
    const template = {
      id: 'card-1',
      name: 'New Card',
      type: 'logical_component',
      description: 'Describe this component',
    };
    onOpen(JSON.stringify(template, null, 2));
    if (typeof onStatus === 'function') onStatus('New card template created');
  };

  const onLayoutChangeLocal = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const v = e.target.value;
    setLayout(v);
    if (typeof onStatus === 'function') onStatus(`Layout: ${v}`);
    if (typeof onLayoutChange === 'function') onLayoutChange(v);
  };

  return (
    <div className="flex gap-2 mb-3">
      <button onClick={onOpenClick} className="px-3 py-1 rounded bg-gray-700">
        Open
      </button>
      <button onClick={onNewCard} className="px-3 py-1 rounded bg-gray-700">
        New
      </button>
      <button onClick={onSave} className="px-3 py-1 rounded bg-gray-700">
        Save
      </button>
      <button
        onClick={onLoadWorkspaceClick}
        className="px-3 py-1 rounded bg-gray-700"
      >
        Load Workspace
      </button>
      <button
        onClick={onGenerateDiagrams}
        className="px-3 py-1 rounded bg-gray-700"
      >
        Generate Diagrams
      </button>
      <button onClick={onExportSvg} className="px-3 py-1 rounded bg-gray-700">
        Export SVG
      </button>

      <label className="flex items-center gap-2">
        <span className="text-sm opacity-80">Layout</span>
        <select
          value={layout}
          onChange={onLayoutChangeLocal}
          className="px-2 py-1 rounded bg-gray-700"
        >
          <option value="dagre">Dagre (LR)</option>
          <option value="breadthfirst">Breadthfirst (hierarchical)</option>
          <option value="cose">COSE (force-directed)</option>
          <option value="circle">Circle</option>
        </select>
      </label>
    </div>
  );
}
