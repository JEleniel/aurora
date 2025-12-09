import React, { useState } from 'react';
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

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100">
      <div className="max-w-7xl mx-auto p-4">
        <header className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-semibold">Aurora Designer</h1>
          <div className="text-sm opacity-80">Dark • Desktop • Tauri</div>
        </header>

        <Toolbar
          source={text}
          onOpen={setText}
          onStatus={setStatus}
          onLoadWorkspace={(w) => setWorkspace(w)}
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
