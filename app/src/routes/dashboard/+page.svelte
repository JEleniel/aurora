<script lang="ts">
	import * as d3 from 'd3';
	import { onMount } from 'svelte';
	import { allCards, allLinks, selectedCard, architecture } from '$lib/stores/architecture';
	import { CardType as _CardTypeEnum, CardStatus as CardStatusEnum } from '$lib/types';

	let svgElement: SVGSVGElement | null = $state(null);
	let zoomBehavior: d3.ZoomBehavior<SVGSVGElement, unknown> | null = $state(null);
	let simulationNodes: any[] = $state([]);
	let simulationLinks: any[] = $state([]);
	let simulation: d3.Simulation<any, undefined> | null = $state(null);

	const statusColors: Record<string, string> = {
		[CardStatusEnum.Approved]: '#4CAF50',
		[CardStatusEnum.Implemented]: '#2196F3',
		[CardStatusEnum.Verified]: '#9C27B0',
		[CardStatusEnum.Deprecated]: '#FF9800',
		[CardStatusEnum.Retired]: '#F44336',
	};

	function getStatusColor(status?: string): string {
		return (status && status in statusColors ? statusColors[status] : '#999') as string;
	}

	function getTypeColor(type: string): string {
		const colors: Record<string, string> = {
			mission: '#1f77b4',
			driver: '#ff7f0e',
			requirement: '#2ca02c',
			behavior: '#d62728',
			interface: '#9467bd',
			constraint: '#8c564b',
			'logical-component': '#e377c2',
			'deployable-node': '#7f7f7f',
			actor: '#bcbd22',
			test: '#17becf',
			artifact: '#1f77b4',
			view: '#ff7f0e',
			note: '#2ca02c',
		};
		return colors[type] || '#1f77b4';
	}

	function getTypeLabel(type: string): string {
		const labels: Record<string, string> = {
			mission: '🎯',
			driver: '🚀',
			requirement: '✓',
			behavior: '⚙️',
			interface: '🔌',
			constraint: '🛑',
			'logical-component': '📦',
			'deployable-node': '🖥️',
			actor: '👤',
			test: '🧪',
			artifact: '📄',
			view: '👁️',
			note: '📝',
		};
		return labels[type] || '•';
	}

	function renderGraph() {
		if (!svgElement || $allCards.length === 0) return;

		// Build simulation data
		simulationNodes = $allCards.map((card) => ({
			id: card.id,
			name: card.name,
			type: card.type,
			status: card.status,
			fx: undefined,
			fy: undefined,
		}));

		simulationLinks = $allLinks.map((link) => ({
			source: link.source_id,
			target: link.target_id,
		}));

		// Clear previous content
		d3.select(svgElement).selectAll('*').remove();

		const width = svgElement.clientWidth;
		const height = svgElement.clientHeight;

		// Create simulation
		simulation = d3
			.forceSimulation(simulationNodes as any[])
			.force(
				'link',
				d3
					.forceLink(simulationLinks as any[])
					.id((d: any) => d.id)
					.distance(80),
			)
			.force('charge', d3.forceManyBody().strength(-400))
			.force('center', d3.forceCenter(width / 2, height / 2))
			.force('collision', d3.forceCollide().radius(35));

		// Create SVG elements
		const svg = d3.select(svgElement);
		const g = svg.append('g');

		const zoom = d3
			.zoom()
			.scaleExtent([0.1, 4])
			.on('zoom', (event: d3.D3ZoomEvent<SVGSVGElement, unknown>) => {
				g.attr('transform', event.transform.toString());
			});

		zoomBehavior = zoom as any as typeof zoomBehavior;
		svg.call(zoom as any);

		// Draw links
		g.append('g')
			.selectAll('line')
			.data(simulationLinks)
			.join('line')
			.attr('stroke', '#aaa')
			.attr('stroke-opacity', 0.5)
			.attr('stroke-width', 2)
			.attr('marker-end', 'url(#arrowhead)');

		// Add arrow markers
		svg.append('defs')
			.append('marker')
			.attr('id', 'arrowhead')
			.attr('markerWidth', 10)
			.attr('markerHeight', 10)
			.attr('refX', 28)
			.attr('refY', 3)
			.attr('orient', 'auto')
			.append('polygon')
			.attr('points', '0 0, 10 3, 0 6')
			.attr('fill', '#aaa');

		// Draw nodes
		const node = g
			.append('g')
			.selectAll('circle')
			.data(simulationNodes)
			.join('circle')
			.attr('r', 20)
			.attr('fill', (d: any) => getTypeColor(d.type))
			.attr('stroke', (d: any) => (d.id === $selectedCard?.id ? '#000' : '#fff'))
			.attr('stroke-width', (d: any) => (d.id === $selectedCard?.id ? 3 : 2))
			.style('cursor', 'pointer')
			.on('click', (_event: any, d: any) => {
				architecture.selectCard(d.id);
			})
			.call(
				d3
					.drag<SVGCircleElement, any>()
					.on('start', dragStarted)
					.on('drag', dragged)
					.on('end', dragEnded) as any,
			);

		// Add labels (emoji + first 2 letters)
		g.append('g')
			.selectAll('text')
			.data(simulationNodes)
			.join('text')
			.attr('x', 0)
			.attr('y', 0)
			.attr('text-anchor', 'middle')
			.attr('dominant-baseline', 'central')
			.attr('font-size', '16px')
			.attr('fill', '#fff')
			.attr('font-weight', 'bold')
			.attr('pointer-events', 'none')
			.text((d: any) => getTypeLabel(d.type));

		// Update positions on simulation tick
		const link = g.selectAll('line');
		const labels = g.selectAll('text');

		simulation.on('tick', () => {
			link.attr('x1', (d: any) => d.source.x)
				.attr('y1', (d: any) => d.source.y)
				.attr('x2', (d: any) => d.target.x)
				.attr('y2', (d: any) => d.target.y);

			node.attr('cx', (d: any) => d.x)
				.attr('cy', (d: any) => d.y)
				.attr('stroke', (d: any) => (d.id === $selectedCard?.id ? '#000' : '#fff'))
				.attr('stroke-width', (d: any) => (d.id === $selectedCard?.id ? 3 : 2));

			labels.attr('x', (d: any) => d.x).attr('y', (d: any) => d.y);
		});

		function dragStarted(event: any, d: any) {
			if (!event.active) simulation?.alphaTarget(0.3).restart();
			d.fx = d.x;
			d.fy = d.y;
		}

		function dragged(event: any, d: any) {
			d.fx = event.x;
			d.fy = event.y;
		}

		function dragEnded(event: any, d: any) {
			if (!event.active) simulation?.alphaTarget(0);
			d.fx = null;
			d.fy = null;
		}

		// Fit to view after simulation
		setTimeout(() => {
			fitToView();
		}, 1000);
	}

	function fitToView() {
		if (!svgElement || !simulation || !zoomBehavior) return;

		const svg = d3.select(svgElement);
		const g = svg.select('g');
		const bounds = (g.node() as SVGGElement).getBBox();
		const width = svgElement.clientWidth;
		const height = svgElement.clientHeight;

		const midX = bounds.x + bounds.width / 2;
		const midY = bounds.y + bounds.height / 2;

		const scale = Math.min(width / bounds.width, height / bounds.height, 2) * 0.85;
		const translate: [number, number] = [width / 2 - scale * midX, height / 2 - scale * midY];

		svg.transition()
			.duration(300)
			.call(zoomBehavior!.transform as any, d3.zoomIdentity.translate(translate[0], translate[1]).scale(scale));
	}

	onMount(() => {
		renderGraph();

		const unsubscribe1 = allCards.subscribe(() => renderGraph());
		const unsubscribe2 = allLinks.subscribe(() => renderGraph());
		const unsubscribe3 = selectedCard.subscribe(() => renderGraph());

		return () => {
			unsubscribe1();
			unsubscribe2();
			unsubscribe3();
		};
	});
</script>

<div class="dashboard-container">
	<div class="graph-container">
		<svg bind:this={svgElement} class="architecture-graph"></svg>
	</div>

	<div class="sidebar">
		<div class="sidebar-header">
			<h3>Graph View</h3>
			<p class="card-count">{$allCards.length} card{$allCards.length !== 1 ? 's' : ''}</p>
		</div>

		{#if $selectedCard}
			<div class="card-detail">
				<h4>{$selectedCard.name}</h4>
				<p class="card-type">{getTypeLabel($selectedCard.type)} {$selectedCard.type}</p>
				<p class="card-description">{$selectedCard.description || '_No description_'}</p>
				{#if $selectedCard.status}
					<p class="card-status" style={`--status-color: ${getStatusColor($selectedCard.status)}`}>
						Status: {$selectedCard.status}
					</p>
				{/if}
			</div>
		{:else}
			<div class="card-detail placeholder">
				<p>Click a card node to view details</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.dashboard-container {
		display: grid;
		grid-template-columns: 1fr 300px;
		gap: 1rem;
		height: 100%;
		padding: 0;
		background-color: var(--md-sys-color-background);
	}

	.graph-container {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 12px;
		overflow: hidden;
		border: 1px solid var(--md-sys-color-outline-variant);
	}

	.architecture-graph {
		width: 100%;
		height: 100%;
		background-color: var(--md-sys-color-surface-container);
	}

	.sidebar {
		background-color: var(--md-sys-color-surface);
		border-radius: 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		padding: 1.5rem;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.sidebar-header {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.sidebar-header h3 {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	.card-count {
		margin: 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.card-detail {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 8px;
		padding: 1rem;
		border: 1px solid var(--md-sys-color-outline-variant);
	}

	.card-detail h4 {
		margin: 0 0 0.5rem 0;
		font-size: 1rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
		word-break: break-word;
	}

	.card-type {
		margin: 0 0 0.5rem 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-primary);
		font-weight: 500;
	}

	.card-description {
		margin: 0 0 0.75rem 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.4;
		max-height: 120px;
		overflow-y: auto;
	}

	.card-status {
		margin: 0.5rem 0 0 0;
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--status-color, var(--md-sys-color-primary));
	}

	.card-detail.placeholder {
		text-align: center;
		color: var(--md-sys-color-on-surface-variant);
		min-height: 100px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 0.9rem;
	}

	@media (max-width: 1024px) {
		.dashboard-container {
			grid-template-columns: 1fr;
			gap: 0;
		}

		.sidebar {
			display: none;
		}
	}
</style>
