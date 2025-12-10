import React, { useState, useEffect, useRef } from 'react';
import DiagramPreview from './components/DiagramPreview';
import MonacoEditor from './components/MonacoEditor';
import Toolbar from './components/Toolbar';
import StatusBar from './components/StatusBar';
import schemaIndex from '../../schemas/card.schema.json';
import type { Card, Link } from './lib/diagramGenerators';

export default function App(): JSX.Element {
  const [text, setText] = useState<string>('');
  const [workspace, setWorkspace] = useState<{ cards: Card[]; links: Link[] }>({
    cards: [],
    links: [],
  });

  // Track current archive path and current open file/entry name (for saves)
  const [currentArchive, setCurrentArchive] = useState<string | null>(null);
  const [currentFile, setCurrentFile] = useState<string | null>(null);
  const [autosave, setAutosave] = useState<boolean>(false);
  const [archiveEntries, setArchiveEntries] = useState<
    Array<{ name: string; contents: string }>
  >([]);
  const [idToFilename, setIdToFilename] = useState<Record<string, string>>({});
  const [modifiedMap, setModifiedMap] = useState<Record<string, string>>({});
  const saveTimer = useRef<number | null>(null);

  const [status, setStatus] = useState<string | undefined>(undefined);
  const [layout, setLayout] = useState<string>('dagre');
  // layout state is driven by Toolbar via `onLayoutChange`

  const elements = React.useMemo(() => {
    // map workspace to cytoscape elements
    const nodes = (workspace.cards || []).map((c) => ({
      data: { id: c.id, label: c.name || c.id, type: c.type },
    }));
    const edges = (workspace.links || []).map((l, i) => ({
      data: {
        id: l.id || `e${i}`,
        source: l.source,
        target: l.target,
        label: l.link_type,
      },
    }));
    return { nodes, edges };
  }, [workspace]);

  const onNodeClick = (id: string) => {
    // find card and load its JSON into the editor
    const card = workspace.cards.find((c) => c.id === id);
    if (card) setText(JSON.stringify(card, null, 2));
    setStatus(`Centered on ${id}`);
  };

  // save handler: writes to archive entry if open, otherwise uses download/save dialog
  const saveCurrent = async (contents?: string, fileName?: string) => {
    const body = typeof contents === 'undefined' ? text : contents;
    let name = fileName || currentFile || `aurora-card.json`;
    // If saving into an archive but we don't have a filename, try to derive
    // a sensible path using the card's type and id: cards/<type>/<id>.json
    if (!fileName && !currentFile) {
      try {
        const parsed = JSON.parse(body || '{}');
        if (parsed && parsed.id) {
          const t = parsed.type || 'unknown';
          name = `cards/${t}/${parsed.id}.json`;
          // update id->filename map so future saves use the same path
          setIdToFilename((m) => ({ ...m, [parsed.id]: name }));
        }
      } catch (e) {
        // leave name as-is if parse fails
      }
    }
    try {
      const { saveArchiveEntry, saveText } = await import('./lib/io');
      if (currentArchive) {
        await saveArchiveEntry(currentArchive, name, body);
        setStatus(`Saved ${name} into ${currentArchive}`);
        return;
      }
      // fallback to saveText (dialog/download)
      await saveText(name, body);
      setStatus(`Saved ${name}`);
    } catch (e) {
      setStatus('Save failed');
    }
  };

  // Save all modified entries (workspace + modified cards) into archive
  const saveAll = async () => {
    try {
      const { saveArchive } = await import('./lib/io');
      const entries: Array<{ name: string; contents: string }> = [];
      // workspace.json
      entries.push({
        name: 'workspace.json',
        contents: JSON.stringify(workspace, null, 2),
      });
      // modified cards
      Object.keys(modifiedMap).forEach((id) => {
        const existing = idToFilename[id];
        let fname = existing || `cards/${id}.json`;
        // Try to parse the modified content to get the card type and build
        // a foldered path when no existing mapping is present.
        if (!existing) {
          try {
            const parsed = JSON.parse(modifiedMap[id]);
            if (parsed && parsed.type && parsed.id) {
              fname = `cards/${parsed.type}/${parsed.id}.json`;
            }
          } catch (e) {
            // ignore parse errors
          }
        }
        entries.push({ name: fname, contents: modifiedMap[id] });
      });
      await saveArchive(currentArchive, entries);
      setStatus('Saved archive');
    } catch (e) {
      setStatus('Save all failed');
    }
  };

  // autosave: debounce text changes
  useEffect(() => {
    if (!autosave) return undefined;
    if (saveTimer.current) window.clearTimeout(saveTimer.current);
    // delay save by 800ms
    saveTimer.current = window.setTimeout(() => {
      void saveCurrent();
    }, 800);
    return () => {
      if (saveTimer.current) window.clearTimeout(saveTimer.current);
    };
  }, [text, autosave]);

  // update modifiedMap when editor text looks like a card with `id`
  useEffect(() => {
    try {
      const parsed = JSON.parse(text || '{}');
      if (parsed && parsed.id) {
        setModifiedMap((m) => ({
          ...m,
          [parsed.id]: JSON.stringify(parsed, null, 2),
        }));
      }
    } catch (e) {
      // ignore invalid JSON while editing
    }
  }, [text]);

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100">
      <div className="max-w-7xl mx-auto p-4">
        <header className="flex items-center justify-between mb-4">
          <div>
            <h1 className="text-2xl font-semibold">Aurora Designer</h1>
            <div className="text-sm opacity-70">
              {currentArchive
                ? `Archive: ${currentArchive}`
                : 'No archive open'}
              {currentFile ? ` • File: ${currentFile}` : ''}
            </div>
          </div>
          <div className="text-sm opacity-80">Dark • Desktop • Tauri</div>
        </header>

        <Toolbar
          source={text}
          onOpen={(s: string, path?: string | null) => {
            setText(s);
            setCurrentFile(path || null);
          }}
          onStatus={setStatus}
          onLoadWorkspace={(w) => setWorkspace(w)}
          onLoadArchive={(arc) => {
            if (!arc) return;
            setCurrentArchive(arc.path || null);
            setArchiveEntries(arc.entries || []);
            // build id->filename map and a workspace from entries
            const map: Record<string, string> = {};
            const cards: Card[] = [];
            const links: Link[] = [];

            // First pass: parse entries that are individual card files or arrays
            for (const e of arc.entries || []) {
              try {
                const parsed = JSON.parse(e.contents);
                const isWorkspaceFile = e.name
                  .toLowerCase()
                  .includes('workspace');

                // If this is the workspace container file, collect its cards but
                // do NOT map their ids to the workspace filename. Individual
                // card files (including those placed under folders by type)
                // will provide their correct paths.
                if (isWorkspaceFile) {
                  if (Array.isArray(parsed.cards)) {
                    parsed.cards.forEach((c: Card) => cards.push(c));
                  }
                  if (Array.isArray(parsed.links))
                    parsed.links.forEach((l: Link) => links.push(l));
                  continue;
                }

                // If entry is an object representing a single card
                if (parsed && parsed.id && parsed.type) {
                  cards.push(parsed);
                  map[parsed.id] = e.name;
                } else if (Array.isArray(parsed)) {
                  // entry is an array of cards
                  parsed.forEach((it: any) => {
                    if (it && it.id) {
                      cards.push(it);
                      map[it.id] = e.name;
                    }
                  });
                } else if (parsed && Array.isArray(parsed.cards)) {
                  // Some files may contain a cards array (not workspace.json)
                  parsed.cards.forEach((c: Card) => {
                    cards.push(c);
                    if (c.id) map[c.id] = e.name;
                  });
                }

                // collect links if present in this file
                if (parsed && Array.isArray(parsed.links)) {
                  parsed.links.forEach((l: Link) => links.push(l));
                }
              } catch (err) {
                // ignore parse errors for non-json entries
              }
            }

            setIdToFilename(map);
            setWorkspace({ cards, links });

            // Prefer opening workspace.json if present, otherwise open first
            // individual card file (not the workspace container).
            const workspaceEntry = (arc.entries || []).find((e) =>
              e.name.toLowerCase().includes('workspace')
            );
            if (workspaceEntry) {
              setText(workspaceEntry.contents);
              setCurrentFile(workspaceEntry.name);
            } else if (cards.length > 0) {
              // find the entry that contains the first card by id
              const firstCard = cards[0];
              const first = (arc.entries || []).find((e) =>
                e.contents.includes(`"id": "${firstCard.id}"`)
              );
              if (first) {
                setText(first.contents);
                setCurrentFile(first.name);
              }
            }
          }}
          onSave={async (s: string, fname?: string) =>
            await saveCurrent(s, fname)
          }
          onSaveAll={async () => await saveAll()}
          onAutosaveChange={(v: boolean) => setAutosave(v)}
          workspace={workspace}
          onLayoutChange={setLayout}
        />

        <div className="grid grid-cols-12 gap-4">
          <div className="col-span-5">
            <label className="block text-sm mb-2">Card / Mermaid Source</label>
            <div className="w-full h-[60vh] rounded bg-gray-800 text-gray-100 border border-gray-700">
              <MonacoEditor
                value={text}
                onChange={setText}
                schema={schemaIndex}
              />
            </div>
          </div>

          <div className="col-span-7">
            <label className="block text-sm mb-2">Diagram Preview</label>
            <div className="p-2 rounded bg-gray-800 border border-gray-700 h-[60vh] overflow-auto">
              <DiagramPreview
                source={text}
                elements={elements}
                onNodeClick={onNodeClick}
                layout={layout}
                onStatus={setStatus}
              />
            </div>
          </div>
        </div>
      </div>
      <StatusBar message={status} />
    </div>
  );
}
