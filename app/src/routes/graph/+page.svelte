<script lang="ts">
	import type { PageData } from './$types';
	import { allCards } from '$lib/stores/architecture';
	import { generateDependencyGraph } from '$lib/services/architecture';
	import { parseDependencyGraph, type DependencyGraph, type GraphMetrics } from '$lib/dependencyGraphTypes';
	import * as d3 from 'd3';

	export const data: PageData = {};

	let svgElement: SVGSVGElement;
	let graph: DependencyGraph | null = null;
	let metrics: GraphMetrics | null = null;
	let loading = false;
	let errorMessage = '';
	let selectedNode: string | null = null;
	let zoomLevel = 1;

	async function loadGraph() {
		errorMessage = '';
		loading = true;
		try {
			const result = await generateDependencyGraph();
			graph = parseDependencyGraph(result);
			calculateMetrics();
			renderGraph();
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
			graph = null;
		} finally {
			loading = false;
		}
	}

	function calculateMetrics() {
		if (!graph) return;

		const nodeCount = graph.node_count;
		const linkCount = graph.link_count;

		// Calculate degree for each node
		const degreeMap = new Map<string, number>();
		graph.nodes.forEach((node) => {
			degreeMap.set(node.id, 0);
		});

		graph.links.forEach((link) => {
			degreeMap.set(link.source, (degreeMap.get(link.source) || 0) + 1);
			degreeMap.set(link.target, (degreeMap.get(link.target) || 0) + 1);
		});

		const isolated = graph.nodes.filter((n) => !degreeMap.has(n.id) || degreeMap.get(n.id) === 0);
		const roots = graph.nodes.filter((n) => !graph!.links.some((l) => l.target === n.id));
		const leaves = graph.nodes.filter((n) => !graph!.links.some((l) => l.source === n.id));

		const degrees = Array.from(degreeMap.values());
		const avgDegree = degrees.length > 0 ? degrees.reduce((a, b) => a + b, 0) / degrees.length : 0;
		const maxDegree = Math.max(...degrees, 0);

		metrics = {
			node_count: nodeCount,
			link_count: linkCount,
			density: graph.density,
			isolated_node_count: isolated.length,
			root_node_count: roots.length,
			leaf_node_count: leaves.length,
			avg_degree: avgDegree,
			max_degree: maxDegree,
		};
	}

	function renderGraph() {
		if (!graph || !svgElement) return;

		// Clear previous content
		d3.select(svgElement).selectAll('*').remove();

		const width = svgElement.clientWidth;
		const height = svgElement.clientHeight;

		// Create simulation
		const simulation = d3
			.forceSimulation(graph.nodes as any[])
			.force(
				'link',
				d3
					.forceLink(graph.links as any[])
					.id((d: any) => d.id)
					.distance(50),
			)
			.force('charge', d3.forceManyBody().strength(-300))
			.force('center', d3.forceCenter(width / 2, height / 2))
			.force('collision', d3.forceCollide().radius(25));

		// Create SVG elements
		const svg = d3.select(svgElement);

		// Add zoom behavior
		const g = svg.append('g');

		const zoom = d3
			.zoom<SVGSVGElement, unknown>()
			.scaleExtent([0.1, 4])
			.on('zoom', (event: d3.D3ZoomEvent<SVGSVGElement, unknown>) => {
				zoomLevel = event.transform.k;
				g.attr('transform', event.transform.toString());
			});

		svg.call(zoom as any);

		// Draw links
		const link = g
			.append('g')
			.selectAll('line')
			.data(graph.links)
			.join('line')
			.attr('stroke', '#888')
			.attr('stroke-opacity', 0.6)
			.attr('stroke-width', 2)
			.attr('marker-end', 'url(#arrowhead)');

		// Add arrow markers
		svg.append('defs')
			.append('marker')
			.attr('id', 'arrowhead')
			.attr('markerWidth', 10)
			.attr('markerHeight', 10)
			.attr('refX', 25)
			.attr('refY', 3)
			.attr('orient', 'auto')
			.append('polygon')
			.attr('points', '0 0, 10 3, 0 6')
			.attr('fill', '#888');

		// Draw nodes
		const node = g
			.append('g')
			.selectAll('circle')
			.data(graph.nodes)
			.join('circle')
			.attr('r', (d: any) => {
				const links = graph!.links.filter((l) => l.source === d.id || l.target === d.id).length;
				return Math.max(5, Math.min(15, 5 + links));
			})
			.attr('fill', (d: any) => {
				const colors = [
					'#1f77b4',
					'#ff7f0e',
					'#2ca02c',
					'#d62728',
					'#9467bd',
					'#8c564b',
					'#e377c2',
					'#7f7f7f',
					'#bcbd22',
					'#17becf',
					'#1f77b4',
					'#ff7f0e',
				];
				return colors[d.group % colors.length] || '#1f77b4';
			})
			.attr('stroke', '#fff')
			.attr('stroke-width', 2)
			.style('cursor', 'pointer')
			.on('click', (_event: any, d: any) => {
				selectedNode = d.id;
				highlightNode(d.id);
			})
			.call(
				d3
					.drag<SVGCircleElement, any>()
					.on('start', dragStarted)
					.on('drag', dragged)
					.on('end', dragEnded) as any,
			);

		// Add labels
		const labels = g
			.append('g')
			.selectAll('text')
			.data(graph.nodes)
			.join('text')
			.attr('x', 0)
			.attr('y', 4)
			.attr('text-anchor', 'middle')
			.attr('font-size', '10px')
			.attr('fill', '#fff')
			.attr('font-weight', 'bold')
			.attr('pointer-events', 'none')
			.text((d: any) => d.name.substring(0, 2));

		// Update positions on simulation tick
		simulation.on('tick', () => {
			link.attr('x1', (d: any) => d.source.x)
				.attr('y1', (d: any) => d.source.y)
				.attr('x2', (d: any) => d.target.x)
				.attr('y2', (d: any) => d.target.y);

			node.attr('cx', (d: any) => d.x).attr('cy', (d: any) => d.y);

			labels.attr('x', (d: any) => d.x).attr('y', (d: any) => d.y);
		});

		function dragStarted(event: any, d: any) {
			if (!event.active) simulation.alphaTarget(0.3).restart();
			d.fx = d.x;
			d.fy = d.y;
		}

		function dragged(event: any, d: any) {
			d.fx = event.x;
			d.fy = event.y;
		}

		function dragEnded(event: any, d: any) {
			if (!event.active) simulation.alphaTarget(0);
			d.fx = null;
			d.fy = null;
		}
	}

	function highlightNode(nodeId: string) {
		if (!graph) return;

		d3.select(svgElement)
			.selectAll('circle')
			.attr('opacity', (d: any) => {
				if (d.id === nodeId) return 1;
				const isConnected =
					graph!.links.some((l) => l.source === nodeId && l.target === d.id) ||
					graph!.links.some((l) => l.source === d.id && l.target === nodeId);
				return isConnected ? 0.8 : 0.2;
			});

		d3.select(svgElement)
			.selectAll('line')
			.attr('opacity', (d: any) => {
				return d.source.id === nodeId || d.target.id === nodeId ? 0.8 : 0.1;
			});
	}

	function resetView() {
		selectedNode = null;
		d3.select(svgElement).selectAll('circle').attr('opacity', 1);
		d3.select(svgElement).selectAll('line').attr('opacity', 0.6);
	}
</script>

<div class="page">
	<header class="page-header">
		<div class="header-content">
			<h1>Dependency Graph</h1>
			<p>Visualize all card relationships in an interactive force-directed graph</p>
		</div>
	</header>

	<div class="controls">
		<button type="button" class="md-button md-button--filled" on:click={loadGraph} disabled={loading}>
			{loading ? 'Loading...' : 'Generate Graph'}
		</button>

		{#if graph}
			<button type="button" class="md-button md-button--outlined" on:click={resetView}> Reset View </button>
			<span class="zoom-level">Zoom: {zoomLevel.toFixed(1)}x</span>
		{/if}

		{#if errorMessage}
			<div class="error-message">
				<strong>Error:</strong>
				{errorMessage}
			</div>
		{/if}
	</div>

	{#if graph && metrics}
		<div class="graph-container">
			<aside class="graph-sidebar">
				<div class="metrics-section">
					<h3>Graph Metrics</h3>
					<div class="metric">
						<span>Nodes</span>
						<strong>{metrics.node_count}</strong>
					</div>
					<div class="metric">
						<span>Links</span>
						<strong>{metrics.link_count}</strong>
					</div>
					<div class="metric">
						<span>Density</span>
						<strong>{metrics.density.toFixed(2)}%</strong>
					</div>
					<div class="metric">
						<span>Avg Degree</span>
						<strong>{metrics.avg_degree.toFixed(1)}</strong>
					</div>
					<div class="metric">
						<span>Max Degree</span>
						<strong>{metrics.max_degree}</strong>
					</div>
				</div>

				<div class="issues-section">
					{#if metrics.isolated_node_count > 0}
						<div class="issue">
							<span>⚠</span>
							<span>{metrics.isolated_node_count} isolated</span>
						</div>
					{/if}

					{#if metrics.root_node_count > 0}
						<div class="issue">
							<span>📍</span>
							<span>{metrics.root_node_count} roots</span>
						</div>
					{/if}

					{#if metrics.leaf_node_count > 0}
						<div class="issue">
							<span>🍃</span>
							<span>{metrics.leaf_node_count} leaves</span>
						</div>
					{/if}
				</div>

				{#if selectedNode}
					<div class="selected-node">
						<h4>Selected Node</h4>
						<p><strong>{$allCards.find((c) => c.id === selectedNode)?.name || selectedNode}</strong></p>
						<small>{selectedNode}</small>
					</div>
				{/if}
			</aside>

			<svg bind:this={svgElement} class="graph-svg"></svg>
		</div>
	{/if}
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		height: 100vh;
		gap: 1rem;
	}

	.page-header {
		background: linear-gradient(135deg, var(--md-sys-color-primary) 0%, var(--md-sys-color-tertiary) 100%);
		color: var(--md-sys-color-on-primary);
		padding: 1.5rem;
		border-radius: 0.5rem;
	}

	.header-content h1 {
		margin: 0 0 0.5rem 0;
		font-size: 1.75rem;
		font-weight: 600;
	}

	.header-content p {
		margin: 0;
		opacity: 0.9;
	}

	.controls {
		display: flex;
		gap: 1rem;
		align-items: center;
		padding: 1rem;
		background-color: var(--md-sys-color-surface-container);
		border-radius: 0.5rem;
	}

	.zoom-level {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 500;
	}

	.error-message {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		padding: 0.75rem 1rem;
		border-radius: 0.25rem;
		font-size: 0.875rem;
	}

	.graph-container {
		display: flex;
		gap: 1rem;
		flex: 1;
		overflow: hidden;
	}

	.graph-sidebar {
		width: 250px;
		background-color: var(--md-sys-color-surface-container);
		border-radius: 0.5rem;
		padding: 1.5rem;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.metrics-section h3 {
		margin: 0 0 1rem 0;
		font-size: 0.875rem;
		font-weight: 600;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface);
	}

	.metric {
		display: flex;
		justify-content: space-between;
		padding: 0.5rem 0;
		font-size: 0.875rem;
	}

	.metric span {
		color: var(--md-sys-color-on-surface-variant);
	}

	.metric strong {
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}

	.issues-section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.issue {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.selected-node {
		padding: 1rem;
		background-color: var(--md-sys-color-primary-container);
		border-radius: 0.25rem;
		color: var(--md-sys-color-on-primary-container);
	}

	.selected-node h4 {
		margin: 0 0 0.5rem 0;
		font-size: 0.75rem;
		text-transform: uppercase;
		font-weight: 600;
		opacity: 0.7;
	}

	.selected-node p {
		margin: 0;
		word-break: break-word;
	}

	.selected-node small {
		display: block;
		font-size: 0.75rem;
		opacity: 0.7;
		margin-top: 0.25rem;
	}

	.graph-svg {
		flex: 1;
		background-color: var(--md-sys-color-surface);
		border-radius: 0.5rem;
		border: 1px solid var(--md-sys-color-outline-variant);
	}
</style>
