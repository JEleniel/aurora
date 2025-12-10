import React from 'react';
import { openText, openArchive, saveSvg, saveText } from '../lib/io';
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
  onOpenArchive,
  onSave,
  onAutosaveChange,
}: {
  source: string;
  // onOpen receives contents and optional path (file or archive entry)
  onOpen: (s: string, path?: string | null) => void;
  onStatus?: (msg: string) => void;
  onLoadWorkspace?: (w: { cards: Card[]; links: Link[] }) => void;
  workspace?: { cards: Card[]; links: Link[] };
  onLayoutChange?: (layout: string) => void;
  // Notify parent when an archive is loaded (path + entries)
  onLoadArchive?: (
    arc?: {
      path?: string | null;
      entries?: Array<{ name: string; contents: string }>;
    } | null
  ) => void;
  // Save handler provided by parent (handles archive vs file)
  onSave?: (text: string, fileName?: string) => Promise<void>;
  // Save all modified entries into the current archive
  onSaveAll?: () => Promise<void>;
  // Autosave setting propagated from toolbar
  onAutosaveChange?: (v: boolean) => void;
}) {
  const [layout, setLayout] = React.useState<string>('dagre');
  const [autosave, setAutosave] = React.useState<boolean>(false);
  // Notify parent of layout changes via callback prop
  // parent should provide `onLayoutChange` in its props if it wants updates
  // (kept local state so select remains controlled locally)

  const handleSave = async () => {
    if (typeof onSave === 'function') {
      try {
        await onSave(source);
        if (typeof onStatus === 'function') onStatus('Saved');
      } catch (e) {
        if (typeof onStatus === 'function') onStatus('Save failed');
      }
    }
  };

  const onOpenClick = async () => {
    const res = await openText();
    if (res?.contents) {
      onOpen(res.contents, res.path);
      if (res.path && typeof onStatus === 'function')
        onStatus(`Opened ${res.path}`);
      else if (typeof onStatus === 'function') onStatus('Opened file');
    }
  };

  const onLoadWorkspaceClick = async () => {
    // Prefer opening an archive (ZIP of cards). Fallback to opening a single JSON file.
    try {
      const arc = await openArchive();
      if (arc && arc.entries && arc.entries.length > 0) {
        // try to find a workspace.json first
        let cards: Card[] = [];
        let links: Link[] = [];
        const workspaceEntry = arc.entries.find((e) =>
          e.name.toLowerCase().includes('workspace')
        );
        if (workspaceEntry) {
          try {
            const parsed = JSON.parse(workspaceEntry.contents);
            if (Array.isArray(parsed.cards)) cards = parsed.cards;
            if (Array.isArray(parsed.links)) links = parsed.links;
          } catch (e) {
            /* ignore */
          }
        } else {
          // otherwise, parse all JSON entries and treat objects with `id` as cards
          for (const e of arc.entries) {
            try {
              const parsed = JSON.parse(e.contents);
              if (Array.isArray(parsed.cards)) {
                cards = cards.concat(parsed.cards);
              } else if (Array.isArray(parsed)) {
                // array of cards?
                const possible = parsed.filter((it) => it && it.id);
                if (possible.length > 0) cards = cards.concat(possible);
              } else if (parsed && parsed.id) {
                cards.push(parsed);
              }
              // also detect links arrays
              if (parsed && Array.isArray(parsed.links))
                links = links.concat(parsed.links);
            } catch (e) {
              /* ignore parse errors per-entry */
            }
          }
        }

        if (onLoadWorkspace) onLoadWorkspace({ cards, links });
        if (typeof onStatus === 'function') onStatus('Workspace loaded');
        // notify parent of loaded archive (path + entries)
        if (typeof onLoadArchive === 'function') onLoadArchive(arc);

        // choose a sensible file to open in editor: prefer workspaceEntry, else first card entry
        if (workspaceEntry) {
          // pass entry name separately; parent receives archive path via onOpenArchive
          onOpen(workspaceEntry.contents, workspaceEntry.name);
        } else if (cards.length > 0) {
          // find first matching entry for first card
          const first = arc.entries.find((e) =>
            e.contents.includes(`"id": "${cards[0].id}"`)
          );
          if (first) onOpen(first.contents, first.name);
        }

        return;
      }
    } catch (e) {
      // fall through to single-file open
    }

    // fallback: single JSON file
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
      <button onClick={handleSave} className="px-3 py-1 rounded bg-gray-700">
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
      <button
        onClick={async () => {
          if (typeof onSave === 'function') await onSave(source);
        }}
        className="px-3 py-1 rounded bg-gray-700"
      >
        Save
      </button>
      <button
        onClick={async () => {
          if (typeof onSaveAll === 'function') {
            try {
              await onSaveAll();
              if (typeof onStatus === 'function')
                onStatus('Saved archive (all)');
            } catch (e) {
              if (typeof onStatus === 'function') onStatus('Save all failed');
            }
          }
        }}
        className="px-3 py-1 rounded bg-gray-700"
      >
        Save All
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
      <label className="flex items-center gap-2">
        <input
          type="checkbox"
          checked={autosave}
          onChange={(e) => {
            const v = e.target.checked;
            setAutosave(v);
            if (typeof onAutosaveChange === 'function') onAutosaveChange(v);
            if (typeof onStatus === 'function')
              onStatus(`Autosave: ${v ? 'on' : 'off'}`);
          }}
        />
        <span className="text-sm opacity-80">Autosave</span>
      </label>
    </div>
  );
}
