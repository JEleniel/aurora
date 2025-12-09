import React, { useEffect, useRef } from 'react';
import mermaid from 'mermaid';
import cytoscape from 'cytoscape';
import dagre from 'cytoscape-dagre';

// Harden mermaid security: use strict mode to prevent script injection.
// If you later add `dompurify` to dependencies, mermaid will use it for sanitization.
mermaid.initialize({
  startOnLoad: false,
  theme: 'dark',
  securityLevel: 'strict',
});
cytoscape.use(dagre);

type NodeDef = { data: { id: string; label?: string; type?: string } };
type EdgeDef = {
  data: { id?: string; source: string; target: string; label?: string };
};

export default function DiagramPreview({
  source,
  elements,
  onNodeClick,
  layout = 'dagre',
  onStatus,
}: {
  source?: string;
  elements?: { nodes: NodeDef[]; edges: EdgeDef[] };
  onNodeClick?: (nodeId: string) => void;
  layout?: string;
  onStatus?: (msg: string) => void;
}) {
  const ref = useRef<HTMLDivElement | null>(null);
  const cyRef = useRef<any | null>(null);

  // Render mermaid if `source` looks like mermaid (string) and no elements provided
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (elements && elements.nodes && elements.nodes.length > 0) {
      // Render cytoscape graph
      if (cyRef.current) {
        cyRef.current.destroy();
        cyRef.current = null;
      }
      cyRef.current = cytoscape({
        container: el,
        elements: [...elements.nodes, ...elements.edges],
        style: [
          {
            selector: 'node',
            style: {
              label: 'data(label)',
              'text-valign': 'center',
              'background-color': '#2b6cb0',
              color: '#fff',
              'text-outline-width': 0,
            },
          },
          {
            selector: 'edge',
            style: {
              'curve-style': 'bezier',
              'target-arrow-shape': 'triangle',
              'line-color': '#9CA3AF',
              'target-arrow-color': '#9CA3AF',
            },
          },
        ],
      });

      const layoutName = layout || 'dagre';
      const layoutOptions: any =
        layoutName === 'dagre'
          ? {
              name: 'dagre',
              rankDir: 'LR',
              nodeSep: 40,
              rankSep: 50,
              fit: false,
              animate: false,
            }
          : { name: layoutName, fit: false, animate: false };

      const layoutInst = cyRef.current.layout(layoutOptions);

      const onLayoutStart = () => onStatus && onStatus('Layout started');
      const onLayoutStop = () => {
        try {
          cyRef.current.fit();
        } catch (e) {
          /* ignore */
        }
        onStatus && onStatus('Layout complete');
      };

      cyRef.current.on('layoutstart', onLayoutStart);
      cyRef.current.on('layoutstop', onLayoutStop);

      layoutInst.run();

      cyRef.current.on('tap', 'node', (evt: any) => {
        const node = evt.target;
        try {
          cyRef.current.center(node);
        } catch (e) {}
        try {
          cyRef.current.$(':selected').unselect();
          node.select();
        } catch (e) {}
        if (onNodeClick) onNodeClick(node.id());
      });
      return;
    }

    // fallback to mermaid render
    const code =
      source && source.trim().length > 0 ? source : 'flowchart TD\n  A[Empty]';
    try {
      mermaid.parse(code); // validate
      mermaid.render(
        'mermaid-' + Math.random().toString(36).slice(2),
        code,
        (svg) => {
          if (el) el.innerHTML = svg;
        }
      );
    } catch (err) {
      el.innerHTML = `<pre class="text-sm text-red-400">Invalid mermaid or empty input</pre>`;
    }
    // cleanup cytoscape if present
    return () => {
      if (cyRef.current) {
        try {
          cyRef.current.destroy();
        } catch (e) {
          /* ignore */
        }
        cyRef.current = null;
      }
    };
  }, [source, elements, onNodeClick]);

  return <div ref={ref} className="w-full h-full" />;
}
