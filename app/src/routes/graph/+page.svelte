<script lang="ts">
	import { onMount } from 'svelte';
	import type { PageData } from './$types';
	import { allCards } from '$lib/stores/architecture';
	import { generateDependencyGraph } from '$lib/services/architecture';
	import { parseDependencyGraph, type DependencyGraph, type GraphMetrics } from '$lib/dependencyGraphTypes';
	import { getCardColor } from '$lib/utils/cardStyling';
	import { CardTypeDisplay, getCardTypesByDisplay } from '$lib/types';
	import * as d3 from 'd3';

	export const data: PageData = {};

	// Precompute shape groupings using the display enum
	const beveled = getCardTypesByDisplay(CardTypeDisplay.Mission, CardTypeDisplay.Requirement, CardTypeDisplay.Driver);
	const beveledSet = new Set(beveled.map((t) => t as string));

	let svgElement = $state<SVGSVGElement | null>(null);
	let graph = $state<DependencyGraph | null>(null);
	let metrics = $state<GraphMetrics | null>(null);
	let loading = $state(false);
	let errorMessage = $state('');
	let selectedNode = $state<string | null>(null);
	let zoomLevel = $state(1);
	let zoomBehavior: d3.ZoomBehavior<SVGSVGElement, unknown> | null = null;

	onMount(() => {
		// Auto-load graph on component mount
		loadGraph();
	});

	// Automatically regenerate graph when cards change
	$effect(() => {
		if ($allCards.length > 0) {
			loadGraph();
		}
	});

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

	function renderNodeShape(selection: any) {
		// Mission, Requirement, Driver: Beveled square
		selection
			.filter((d: any) => beveledSet.has(d.card_type))
			.append('polygon')
			.attr('points', `${-12},${-10} ${12},${-10} ${16},${10} ${-12},${10}`)
			.attr('fill', (d: any) => getCardColor(d.card_type as any) || '#999999')
			.attr('stroke', '#fff')
			.attr('stroke-width', 1.5);

		// Behavior: Oval
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.Behavior)
			.append('ellipse')
			.attr('rx', 12)
			.attr('ry', 8)
			.attr('fill', (d: any) => getCardColor(d.card_type as any) || '#999999')
			.attr('stroke', '#fff')
			.attr('stroke-width', 1.5);

		// Constraint: Elongated octagon
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.Constraint)
			.append('polygon')
			.attr('points', `${-4},${-10} ${4},${-10} ${14},${0} ${4},${10} ${-4},${10} ${-14},${0}`)
			.attr('fill', (d: any) => getCardColor(d.card_type as any) || '#999999')
			.attr('stroke', '#fff')
			.attr('stroke-width', 1.5);

		// LogicalComponent: 3D shadow
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.LogicalComponent)
			.append('g')
			.each(function (this: SVGGElement, d: any) {
				d3.select(this)
					.append('rect')
					.attr('x', -8)
					.attr('y', 4)
					.attr('width', 12)
					.attr('height', 6)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('opacity', 0.3);
				d3.select(this)
					.append('rect')
					.attr('x', -10)
					.attr('y', -5)
					.attr('width', 13)
					.attr('height', 11)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke', '#fff')
					.attr('stroke-width', 1.5);
			});

		// DeployableNode: Server with legs
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.DeployableNode)
			.append('g')
			.each(function (this: SVGGElement, d: any) {
				d3.select(this)
					.append('rect')
					.attr('x', -6)
					.attr('y', -4)
					.attr('width', 12)
					.attr('height', 8)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke', '#fff')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('line')
					.attr('x1', -3)
					.attr('y1', 4)
					.attr('x2', -5)
					.attr('y2', 10)
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('line')
					.attr('x1', 3)
					.attr('y1', 4)
					.attr('x2', 5)
					.attr('y2', 10)
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1.5);
			});

		// Actor: Stick figure with computer head
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.Actor)
			.append('g')
			.each(function (this: SVGGElement, d: any) {
				d3.select(this)
					.append('circle')
					.attr('cx', 0)
					.attr('cy', -6)
					.attr('r', 2.5)
					.attr('fill', getCardColor(d.card_type as any) || '#999999');
				d3.select(this)
					.append('rect')
					.attr('x', -4)
					.attr('y', -9)
					.attr('width', 8)
					.attr('height', 5)
					.attr('fill', 'none')
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1);
				d3.select(this)
					.append('line')
					.attr('x1', 0)
					.attr('y1', -3.5)
					.attr('x2', 0)
					.attr('y2', 2)
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('line')
					.attr('x1', -4)
					.attr('y1', 0)
					.attr('x2', 4)
					.attr('y2', 0)
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('line')
					.attr('x1', -2)
					.attr('y1', 2)
					.attr('x2', -5)
					.attr('y2', 8)
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('line')
					.attr('x1', 2)
					.attr('y1', 2)
					.attr('x2', 5)
					.attr('y2', 8)
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1.5);
			});

		// Test: Elongated hexagon
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.Test)
			.append('polygon')
			.attr('points', `${-4},${-10} ${4},${-10} ${14},${0} ${4},${10} ${-4},${10} ${-14},${0}`)
			.attr('fill', (d: any) => getCardColor(d.card_type as any) || '#999999')
			.attr('stroke', '#fff')
			.attr('stroke-width', 1.5);

		// Artifact: Document
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.Artifact)
			.append('g')
			.each(function (this: SVGGElement, d: any) {
				d3.select(this)
					.append('rect')
					.attr('x', -4)
					.attr('y', -7)
					.attr('width', 10)
					.attr('height', 14)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke', '#fff')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('line')
					.attr('x1', -2)
					.attr('y1', -3)
					.attr('x2', 2)
					.attr('y2', -3)
					.attr('stroke', 'white')
					.attr('stroke-width', 1);
				d3.select(this)
					.append('line')
					.attr('x1', -2)
					.attr('y1', 0)
					.attr('x2', 2)
					.attr('y2', 0)
					.attr('stroke', 'white')
					.attr('stroke-width', 1);
				d3.select(this)
					.append('line')
					.attr('x1', -2)
					.attr('y1', 3)
					.attr('x2', 2)
					.attr('y2', 3)
					.attr('stroke', 'white')
					.attr('stroke-width', 1);
			});

		// View: Double circle
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.View)
			.append('g')
			.each(function (this: SVGGElement, d: any) {
				d3.select(this)
					.append('circle')
					.attr('cx', -4)
					.attr('cy', 0)
					.attr('r', 4)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke', '#fff')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('circle')
					.attr('cx', 4)
					.attr('cy', 0)
					.attr('r', 4)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke', '#fff')
					.attr('stroke-width', 1.5);
			});

		// Interface: Elongated circle on stick
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.Interface)
			.append('g')
			.each(function (this: SVGGElement, d: any) {
				d3.select(this)
					.append('ellipse')
					.attr('cx', 0)
					.attr('cy', -4)
					.attr('rx', 5)
					.attr('ry', 4)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke', '#fff')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('line')
					.attr('x1', 0)
					.attr('y1', 0)
					.attr('x2', 0)
					.attr('y2', 10)
					.attr('stroke', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke-width', 1.5);
			});

		// Note: Note card with fold
		selection
			.filter((d: any) => d.card_type === CardTypeDisplay.Note)
			.append('g')
			.each(function (this: SVGGElement, d: any) {
				d3.select(this)
					.append('path')
					.attr('d', `M ${-8} ${-6} L ${8} ${-6} L ${8} ${-2} L ${10} ${-2} L ${10} ${8} L ${-8} ${8} Z`)
					.attr('fill', getCardColor(d.card_type as any) || '#999999')
					.attr('stroke', '#fff')
					.attr('stroke-width', 1.5);
				d3.select(this)
					.append('polygon')
					.attr('points', `${4},${-6} ${10},${-2} ${8},${-2}`)
					.attr('fill', 'white')
					.attr('opacity', 0.3);
			});
	}

	function renderGraph() {
		if (!graph || !svgElement) return;

		// Handle empty graph
		if (graph.nodes.length === 0) {
			d3.select(svgElement).selectAll('*').remove();
			const svg = d3.select(svgElement);
			const width = svgElement.clientWidth;
			const height = svgElement.clientHeight;

			svg.append('text')
				.attr('x', width / 2)
				.attr('y', height / 2)
				.attr('text-anchor', 'middle')
				.attr('font-size', '16px')
				.attr('fill', '#999')
				.text('No cards in architecture. Create some cards to see the dependency graph.');

			return;
		}

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

		zoomBehavior = zoom;
		svg.call(zoom as any);

		// Draw links
		const link = g
			.append('g')
			.selectAll('line')
			.data(graph.links)
			.join('line')
			.attr('class', 'link')
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
		const nodeContainer = g.append('g').selectAll('g').data(graph.nodes).join('g');

		// Render custom shapes for each card type
		renderNodeShape(nodeContainer);

		// Add click handler to all nodes
		nodeContainer
			.on('click', (_event: any, d: any) => {
				selectedNode = d.id;
				highlightNode(d.id);
			})
			.call(d3.drag<SVGGElement, any>().on('start', dragStarted).on('drag', dragged).on('end', dragEnded) as any);

		// Add labels
		const labels = g
			.append('g')
			.selectAll('text')
			.data(graph.nodes)
			.join('text')
			.attr('x', 0)
			.attr('y', 0)
			.attr('text-anchor', 'middle')
			.attr('dominant-baseline', 'central')
			.attr('font-size', '12px')
			.attr('fill', '#fff')
			.attr('font-weight', 'bold')
			.attr('pointer-events', 'none')
			.text((d: any) => d.name);

		// Update positions on simulation tick
		simulation.on('tick', () => {
			link.attr('x1', (d: any) => d.source.x)
				.attr('y1', (d: any) => d.source.y)
				.attr('x2', (d: any) => d.target.x)
				.attr('y2', (d: any) => d.target.y);

			nodeContainer.attr('transform', (d: any) => `translate(${d.x},${d.y})`);

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

		// Fit to view after simulation has progressed
		setTimeout(() => {
			fitToView();
		}, 1000);
	}

	function highlightNode(nodeId: string) {
		if (!graph || !svgElement) return;

		d3.select(svgElement)
			.selectAll('g > *')
			.attr('opacity', (d: any) => {
				if (d.id === nodeId) return 1;
				const isConnected =
					graph!.links.some((l) => l.source === nodeId && l.target === d.id) ||
					graph!.links.some((l) => l.source === d.id && l.target === nodeId);
				return isConnected ? 0.8 : 0.2;
			});

		d3.select(svgElement)
			.selectAll('line.link')
			.attr('opacity', (d: any) => {
				return d.source.id === nodeId || d.target.id === nodeId ? 0.8 : 0.1;
			});
	}

	function resetView() {
		if (!svgElement) return;
		selectedNode = null;
		d3.select(svgElement).selectAll('g > *').attr('opacity', 1);
		d3.select(svgElement).selectAll('line.link').attr('opacity', 0.6);
	}

	function fitToView() {
		if (!svgElement || !graph || !zoomBehavior) return;

		const svg = d3.select(svgElement);
		const g = svg.select('g');
		const bounds = (g.node() as SVGGElement).getBBox();
		const width = svgElement.clientWidth;
		const height = svgElement.clientHeight;

		const midX = bounds.x + bounds.width / 2;
		const midY = bounds.y + bounds.height / 2;
		const scale = 0.8 / Math.max(bounds.width / width, bounds.height / height);
		const translateX = width / 2 - scale * midX;
		const translateY = height / 2 - scale * midY;

		const transform = d3.zoomIdentity.translate(translateX, translateY).scale(scale);
		svg.transition()
			.duration(750)
			.call(zoomBehavior.transform as any, transform);
		zoomLevel = scale;
	}

	function zoomIn() {
		if (!svgElement || !zoomBehavior) return;
		const svg = d3.select(svgElement);
		svg.transition()
			.duration(300)
			.call(zoomBehavior.scaleBy as any, 1.3);
	}

	function zoomOut() {
		if (!svgElement || !zoomBehavior) return;
		const svg = d3.select(svgElement);
		svg.transition()
			.duration(300)
			.call(zoomBehavior.scaleBy as any, 0.77);
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
		<button type="button" class="md-button md-button--filled" onclick={loadGraph} disabled={loading}>
			{loading ? 'Loading...' : 'Generate Graph'}
		</button>

		{#if graph}
			<button type="button" class="md-button md-button--outlined" onclick={fitToView} title="Fit graph to view">
				⊡ Fit View
			</button>
			<button type="button" class="md-button md-button--outlined" onclick={zoomIn} title="Zoom in">
				+ Zoom In
			</button>
			<button type="button" class="md-button md-button--outlined" onclick={zoomOut} title="Zoom out">
				− Zoom Out
			</button>
			<button type="button" class="md-button md-button--outlined" onclick={resetView}> Reset View </button>
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
