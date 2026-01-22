<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onMount } from 'svelte';

	type ValidationReport = {
		valid: boolean;
		messages: string[];
	};

	type CardSummary = {
		id: string;
		missionId: string;
		name: string;
		cardType: string;
		cardSubtype?: string;
		status?: string;
		relativePath: string;
	};

	type MissionSummary = {
		id: string;
		name: string;
		cards: CardSummary[];
	};

	type ViewFilterSummary = {
		id: string;
		label: string;
		description: string;
		rootCardTypes: string[];
		includeCardTypes: string[];
		cardCount: number;
	};

	type FilteredCards = {
		filterId: string;
		filterLabel: string;
		cardCount: number;
		cards: CardSummary[];
	};

	type MissionViewModel = {
		id: string;
		name: string;
		totalCards: number;
		visibleCards: CardSummary[];
	};

	type ModelHomeSummary = {
		root: string;
		missions: MissionSummary[];
		cardCount: number;
		filters: ViewFilterSummary[];
	};

	type AuroraCard = {
		$schema?: string;
		id: string;
		card_type: string;
		card_subtype?: string;
		name: string;
		description: string;
		status?: string;
	};

	type LoadedCard = {
		path: string;
		raw: string;
		card: AuroraCard;
	};

	type ModelChangedPayload = {
		root: string;
		summary: ModelHomeSummary;
	};

	type GraphEdgeSummary = {
		sourceId: string;
		targetId: string;
		relationship: string;
	};

	type GraphNodeSummary = {
		id: string;
		missionId: string;
		cardType: string;
		cardSubtype?: string;
		name: string;
		status?: string;
		relativePath: string;
		distance: number;
	};

	type GraphNeighborhood = {
		filterId: string;
		filterLabel: string;
		maxDepth: number;
		center: GraphNodeSummary;
		nodes: GraphNodeSummary[];
		edges: GraphEdgeSummary[];
	};

	type GraphOrientation = 'upstream' | 'downstream' | 'lateral';

	type GraphLane = {
		label: string;
		distance: number;
		nodes: GraphNodeSummary[];
	};

	type GraphEdgeDetail = {
		id: string;
		sourceLabel: string;
		targetLabel: string;
		relationship: string;
	};

	const defaultValidationMessage =
		'Paste or edit an Aurora card JSON payload, then select "Validate" to run the shared Rust library.';

	let modelHomePath = '';
	let summary: ModelHomeSummary | null = null;
	let missionViews: MissionViewModel[] = [];
	let selectedCard: CardSummary | null = null;
	let loadedCard: LoadedCard | null = null;
	let filteredCards: FilteredCards | null = null;
	let cardsByMission: Record<string, CardSummary[]> = {};
	let activeFilterId: string | null = null;
	let activeFilterSummary: ViewFilterSummary | null = null;
	let filterBusy = false;
	let filterError = '';
	let isLoadingModel = false;
	let loadError = '';
	let loadStatus = 'Select a model home to begin.';
	let cardJson = `{
"$schema": "./Aurora.schema.json",
"id": "MIS-001",
"card_type": "Mission",
"name": "Enable Deterministic Aurora CLI Tooling",
"description": "Demo card used by the Aurora Editor",
"links": [],
"audit_trail": {
	"version": "1.0.0",
	"hash": null,
	"history": [
		{
			"editor": "System",
			"timestamp": "2024-01-01T00:00:00Z",
			"event": "created",
			"hash": null
		}
	]
}
}`;
	let status: 'idle' | 'valid' | 'invalid' | 'error' = 'idle';
	let validationMessage = defaultValidationMessage;
	let busy = false;
	let unlistenModelChanges: null | (() => void) = null;
	let graph: GraphNeighborhood | null = null;
	let graphBusy = false;
	let graphError = '';
	let graphDepth = 2;
	const minGraphDepth = 1;
	const maxGraphDepth = 3;
	let graphUpstreamLanes: GraphLane[] = [];
	let graphDownstreamLanes: GraphLane[] = [];
	let graphLateralNodes: GraphNodeSummary[] = [];
	let graphEdgeDetails: GraphEdgeDetail[] = [];
	let graphRequestId = 0;

	onMount(() => {
		const register = async () => {
			unlistenModelChanges = await listen<ModelChangedPayload>('model-home-changed', (event) => {
				void handleModelChanged(event.payload);
			});
		};
		register();
		return () => {
			unlistenModelChanges?.();
		};
	});

	function formatError(error: unknown): string {
		return error instanceof Error ? error.message : String(error);
	}

	async function handleLoadModelHome() {
		const trimmed = modelHomePath.trim();
		if (!trimmed) {
			loadError = 'Enter a model home path to continue.';
			return;
		}

		isLoadingModel = true;
		loadError = '';
		try {
			const result = await invoke<ModelHomeSummary>('load_model_home', { path: trimmed });
			summary = result;
			selectedCard = null;
			loadedCard = null;
			filteredCards = null;
			cardsByMission = {};
			activeFilterId = null;
			activeFilterSummary = null;
			graphRequestId += 1;
			graph = null;
			graphError = '';
			graphBusy = false;
			graphUpstreamLanes = [];
			graphDownstreamLanes = [];
			graphLateralNodes = [];
			graphEdgeDetails = [];
			loadStatus = `Loaded ${result.cardCount} cards from ${result.root}`;
			status = 'idle';
			validationMessage = 'Select a card to inspect its JSON payload.';
			filterError = '';
			const initialFilter = result.filters?.[0]?.id ?? null;
			await applyFilter(initialFilter);
		} catch (error) {
			console.error('Failed to load model home', error);
			loadError = formatError(error);
		} finally {
			isLoadingModel = false;
		}
	}

	async function browseForModelHome() {
		const defaultPath = summary?.root ?? (modelHomePath.trim() || undefined);
		try {
			const selection = await open({
				directory: true,
				multiple: false,
				defaultPath,
				title: 'Select Aurora model home',
			});
			const resolved =
				selection == null ? null : Array.isArray(selection) ? selection[0] ?? null : selection;
			if (!resolved) {
				return;
			}
			modelHomePath = resolved;
			loadError = '';
			loadStatus = `Scanning ${resolved}…`;
			await handleLoadModelHome();
		} catch (error) {
			console.error('Folder picker failed', error);
			loadError = `Failed to open folder chooser: ${formatError(error)}`;
		}
	}

	async function copyLoadError() {
		if (!loadError) {
			return;
		}
		try {
			await navigator.clipboard.writeText(loadError);
			loadStatus = 'Copied the most recent error to the clipboard.';
		} catch (error) {
			console.error('Failed to copy load error', error);
		}
	}

	async function selectCard(card: CardSummary) {
		if (!summary) {
			return;
		}

		try {
			const response = await invoke<LoadedCard>('load_card_file', {
				rootPath: summary.root,
				relativePath: card.relativePath,
			});
			selectedCard = card;
			loadedCard = response;
			cardJson = response.raw;
			status = 'idle';
			validationMessage = `Loaded ${card.id} from ${response.path}`;
			graph = null;
			graphError = '';
			void refreshGraph(card.id);
		} catch (error) {
			status = 'error';
			validationMessage = `Failed to load ${card.id}: ${formatError(error)}`;
		}
	}

	async function validateCard() {
		busy = true;
		try {
			const response = await invoke<ValidationReport>('validate_card_json', { cardJson });
			if (response.valid) {
				status = 'valid';
				validationMessage = 'Card passed deterministic validation checks ✅';
			} else {
				status = 'invalid';
				validationMessage = response.messages.join('\n');
			}
		} catch (error) {
			status = 'error';
			validationMessage = formatError(error);
		} finally {
			busy = false;
		}
	}

	async function applyFilter(filterId: string | null) {
		if (!summary || summary.filters.length === 0) {
			filteredCards = null;
			activeFilterId = null;
			filterError = '';
			return;
		}

		filterBusy = true;
		filterError = '';
		try {
			const response = await invoke<FilteredCards>('filter_cards', {
				filter_id: filterId ?? undefined,
			});
			filteredCards = response;
			activeFilterId = response.filterId;
		} catch (error) {
			filterError = formatError(error);
			filteredCards = null;
			activeFilterId = filterId;
		} finally {
			filterBusy = false;
		}
	}

	function findCardSummary(cardId: string): CardSummary | null {
		if (!summary) {
			return null;
		}
		for (const mission of summary.missions) {
			const match = mission.cards.find((card) => card.id === cardId);
			if (match) {
				return match;
			}
		}
		return null;
	}

	function cardSummaryFromGraphNode(node: GraphNodeSummary): CardSummary {
		return {
			id: node.id,
			missionId: node.missionId,
			cardType: node.cardType,
			cardSubtype: node.cardSubtype,
			name: node.name,
			status: node.status,
			relativePath: node.relativePath,
		};
	}

	function classifyGraphNodes(graphData: GraphNeighborhood): Record<string, GraphOrientation> {
		const orientation: Record<string, GraphOrientation> = {};
		const upstreamVisited = new Set<string>([graphData.center.id]);
		const upstreamQueue: string[] = [graphData.center.id];
		while (upstreamQueue.length) {
			const current = upstreamQueue.shift()!;
			for (const edge of graphData.edges) {
				if (edge.targetId === current && !upstreamVisited.has(edge.sourceId)) {
					upstreamVisited.add(edge.sourceId);
					if (edge.sourceId !== graphData.center.id) {
						orientation[edge.sourceId] = 'upstream';
					}
					upstreamQueue.push(edge.sourceId);
				}
			}
		}
		const downstreamVisited = new Set<string>([graphData.center.id]);
		const downstreamQueue: string[] = [graphData.center.id];
		while (downstreamQueue.length) {
			const current = downstreamQueue.shift()!;
			for (const edge of graphData.edges) {
				if (edge.sourceId === current && !downstreamVisited.has(edge.targetId)) {
					downstreamVisited.add(edge.targetId);
					if (edge.targetId !== graphData.center.id && orientation[edge.targetId] !== 'upstream') {
						orientation[edge.targetId] = 'downstream';
					}
					downstreamQueue.push(edge.targetId);
				}
			}
		}
		for (const node of graphData.nodes) {
			if (!orientation[node.id]) {
				orientation[node.id] = 'lateral';
			}
		}
		return orientation;
	}

	function buildGraphLanes(graphData: GraphNeighborhood) {
		const orientation = classifyGraphNodes(graphData);
		const upstream = groupNodesByDistance(graphData.nodes.filter((node) => orientation[node.id] === 'upstream'));
		const downstream = groupNodesByDistance(
			graphData.nodes.filter((node) => orientation[node.id] === 'downstream'),
		);
		const lateral = graphData.nodes.filter((node) => orientation[node.id] === 'lateral');
		return { upstream, downstream, lateral };
	}

	function groupNodesByDistance(nodes: GraphNodeSummary[]): GraphLane[] {
		const buckets = new Map<number, GraphNodeSummary[]>();
		for (const node of nodes) {
			const list = buckets.get(node.distance) ?? [];
			list.push(node);
			buckets.set(node.distance, list);
		}
		return Array.from(buckets.entries())
			.sort((a, b) => a[0] - b[0])
			.map(([distance, laneNodes]) => ({
				label: distance === 1 ? 'Direct' : `${distance} hops`,
				distance,
				nodes: laneNodes.sort((a, b) => a.name.localeCompare(b.name)),
			}));
	}

	function describeGraphEdges(graphData: GraphNeighborhood): GraphEdgeDetail[] {
		const lookup = new Map<string, GraphNodeSummary>();
		lookup.set(graphData.center.id, graphData.center);
		for (const node of graphData.nodes) {
			lookup.set(node.id, node);
		}
		return graphData.edges.map((edge, index) => {
			const source = lookup.get(edge.sourceId);
			const target = lookup.get(edge.targetId);
			const sourceLabel = source ? `${source.name} (${edge.sourceId})` : edge.sourceId;
			const targetLabel = target ? `${target.name} (${edge.targetId})` : edge.targetId;
			return {
				id: `${edge.sourceId}-${edge.relationship}-${edge.targetId}-${index}`,
				sourceLabel,
				targetLabel,
				relationship: edge.relationship,
			};
		});
	}

	async function refreshGraph(cardId?: string) {
		if (!summary) {
			graph = null;
			graphError = '';
			return;
		}
		const targetId = cardId ?? selectedCard?.id;
		if (!targetId) {
			graph = null;
			graphError = '';
			return;
		}
		const requestId = ++graphRequestId;
		graphBusy = true;
		graphError = '';
		try {
			const response = await invoke<GraphNeighborhood>('graph_neighborhood', {
				card_id: targetId,
				filter_id: activeFilterId ?? undefined,
				generations: graphDepth,
			});
			if (requestId === graphRequestId) {
				graph = response;
			}
		} catch (error) {
			if (requestId === graphRequestId) {
				graphError = formatError(error);
				graph = null;
			}
		} finally {
			if (requestId === graphRequestId) {
				graphBusy = false;
			}
		}
	}

	function handleGraphDepthChange(depth: number) {
		if (depth === graphDepth) {
			return;
		}
		const clamped = Math.min(Math.max(depth, minGraphDepth), maxGraphDepth);
		graphDepth = clamped;
		void refreshGraph();
	}

	function handleGraphNodeRecenter(node: GraphNodeSummary) {
		void selectCard(cardSummaryFromGraphNode(node));
	}

	async function handleFilterSelect(filterId: string) {
		if (!summary || filterBusy) {
			return;
		}
		if (filterId === activeFilterId && !filterError) {
			return;
		}
		await applyFilter(filterId);
		const currentSelection = selectedCard;
		if (currentSelection && filteredCards && filteredCards.cards.some((card) => card.id === currentSelection.id)) {
			void refreshGraph(currentSelection.id);
		}
	}

	async function handleModelChanged(payload: ModelChangedPayload) {
		if (!summary || summary.root !== payload.root) {
			return;
		}
		const previouslySelectedId = selectedCard?.id ?? null;
		summary = payload.summary;
		loadStatus = `Detected changes on disk - reloaded ${summary.cardCount} cards.`;
		const nextFilterId =
			summary.filters.find((filter) => filter.id === activeFilterId) ?
				activeFilterId
			:	(summary.filters[0]?.id ?? null);
		await applyFilter(nextFilterId);
		if (!previouslySelectedId) {
			return;
		}
		const refreshedSelection =
			filteredCards?.cards.find((card) => card.id === previouslySelectedId) ??
			findCardSummary(previouslySelectedId);
		if (refreshedSelection) {
			await selectCard(refreshedSelection);
			validationMessage = `Reloaded ${refreshedSelection.id} after detecting file changes.`;
		} else {
			selectedCard = null;
			loadedCard = null;
			status = 'idle';
			validationMessage = `Previously selected card ${previouslySelectedId} is no longer available.`;
		}
	}

	$: activeFilterSummary = summary ? (summary.filters.find((filter) => filter.id === activeFilterId) ?? null) : null;

	$: if (filteredCards) {
		const grouped: Record<string, CardSummary[]> = {};
		for (const card of filteredCards.cards) {
			const missionCards = grouped[card.missionId] ?? [];
			missionCards.push(card);
			grouped[card.missionId] = missionCards;
		}
		cardsByMission = grouped;
	} else {
		cardsByMission = {};
	}

	$: if (summary) {
		missionViews = summary.missions.map((mission) => ({
			id: mission.id,
			name: mission.name,
			totalCards: mission.cards.length,
			visibleCards: filteredCards ? (cardsByMission[mission.id] ?? []) : mission.cards,
		}));
	} else {
		missionViews = [];
	}

	$: if (selectedCard && filteredCards) {
		const currentSelection = selectedCard;
		const stillVisible = filteredCards.cards.some((card) => card.id === currentSelection.id);
		if (!stillVisible) {
			selectedCard = null;
			loadedCard = null;
			status = 'idle';
			validationMessage = 'Select a card to inspect its JSON payload.';
		}
	}

	$: if (!selectedCard) {
		graph = null;
		graphError = '';
	}

	$: if (graph) {
		const { upstream, downstream, lateral } = buildGraphLanes(graph);
		graphUpstreamLanes = upstream;
		graphDownstreamLanes = downstream;
		graphLateralNodes = lateral;
		graphEdgeDetails = describeGraphEdges(graph);
	} else {
		graphUpstreamLanes = [];
		graphDownstreamLanes = [];
		graphLateralNodes = [];
		graphEdgeDetails = [];
	}
</script>

<main>
	<header class="hero">
		<div class="hero-copy">
			<h1>Aurora Viewer + Editor</h1>
			<p>Load an Aurora model home, browse mission trees, and validate cards using the shared Rust library.</p>
		</div>
		<div class="context-panel">
			<label for="model-home">Model home path</label>
			<div class="input-row">
				<input
					id="model-home"
					type="text"
					placeholder="/path/to/aurora/model"
					spellcheck="false"
					bind:value={modelHomePath}
				/>
				<button
					type="button"
					class="ghost"
					on:click|preventDefault={() => browseForModelHome()}
					disabled={isLoadingModel}
					aria-label="Browse for Aurora model home"
				>
					Browse…
				</button>
				<button
					type="button"
					class="primary"
					on:click|preventDefault={handleLoadModelHome}
					disabled={isLoadingModel}
				>
					{isLoadingModel ? 'Loading…' : 'Load Model Home'}
				</button>
			</div>
			<p class="status-line" aria-live="polite">{loadStatus}</p>
			{#if loadError}
				<div class="error-banner" role="alert" aria-live="assertive">
					<pre>{loadError}</pre>
					<button
						type="button"
						class="ghost compact"
						on:click|preventDefault={copyLoadError}
					>
						Copy error
					</button>
				</div>
			{/if}
		</div>
		<div class="metrics">
			<div class="metric">
				<p>Total Cards</p>
				<strong>{summary ? summary.cardCount : '—'}</strong>
			</div>
			<div class="metric">
				<p>Missions</p>
				<strong>{summary ? summary.missions.length : '—'}</strong>
			</div>
			<div class="metric">
				<p>Selected Card</p>
				<strong>{selectedCard ? selectedCard.id : '—'}</strong>
			</div>
		</div>
	</header>

	<section class="workspace">
		<aside class="navigator">
			<div class="panel-header">
				<h2>Model Tree</h2>
				<p>Mission → cards, sorted deterministically.</p>
			</div>
			{#if summary}
				<div class="filter-toolbar">
					<h3>View Filters</h3>
					{#if summary.filters.length}
						<div class="filter-chips">
							{#each summary.filters as filter (filter.id)}
								<button
									type="button"
									class:selected={filter.id === activeFilterId}
									on:click={() => handleFilterSelect(filter.id)}
									disabled={filterBusy}
								>
									<span>{filter.label}</span>
									<small>{filter.cardCount}</small>
								</button>
							{/each}
						</div>
						{#if activeFilterSummary}
							<p class="filter-description">{activeFilterSummary.description}</p>
						{/if}
						{#if filteredCards}
							<p class="filter-status">
								{#if filterBusy}
									Filtering…
								{:else}
									<strong>{filteredCards.cardCount}</strong>
									&nbsp;cards visible
									{#if activeFilterSummary}
										&nbsp;•&nbsp;{activeFilterSummary.label}
									{/if}
								{/if}
							</p>
						{:else if filterBusy}
							<p class="filter-status">Filtering…</p>
						{/if}
					{:else}
						<p class="filter-status">This model does not define view filters.</p>
					{/if}
					{#if filterError}
						<p class="warning">{filterError}</p>
					{/if}
				</div>
				<div class="mission-list">
					{#each missionViews as mission (mission.id)}
						<div class="mission-card">
							<div class="mission-header">
								<p class="mission-id">{mission.id}</p>
								<h3>{mission.name}</h3>
								<span>
									{#if filteredCards}
										{mission.visibleCards.length} / {mission.totalCards} cards
									{:else}
										{mission.totalCards} cards
									{/if}
								</span>
							</div>
							{#if mission.visibleCards.length}
								<ul>
									{#each mission.visibleCards as card (card.relativePath)}
										<li>
											<button
												type="button"
												class:selected={selectedCard?.id === card.id}
												on:click={() => selectCard(card)}
											>
												<span class="card-id">{card.id}</span>
												<span class="card-name">{card.name}</span>
												<span class="card-meta">
													{card.cardType}
													{#if card.status}
														&nbsp;•&nbsp;{card.status}
													{/if}
												</span>
											</button>
										</li>
									{/each}
								</ul>
							{:else if filteredCards}
								<p class="no-matches">No cards match this filter.</p>
							{:else}
								<p class="no-matches">This mission does not contain any cards yet.</p>
							{/if}
						</div>
					{/each}
				</div>
			{:else}
				<p class="placeholder">Load a model home to explore missions, requirements, and capabilities.</p>
			{/if}
		</aside>

		<section class="editor">
			<div class="editor-header">
				<div>
					<h2>{selectedCard ? selectedCard.name : 'Card JSON Editor'}</h2>
					<p>
						{#if selectedCard}
							<span>{selectedCard.id}</span>
							&nbsp;•&nbsp;
							<span>{selectedCard.cardType}</span>
							{#if selectedCard.status}
								&nbsp;•&nbsp;<span>{selectedCard.status}</span>
							{/if}
						{:else}
							Choose a card from the navigator to load its JSON payload.
						{/if}
					</p>
				</div>
				{#if loadedCard}
					<div class="path-chip">{loadedCard.path}</div>
				{/if}
			</div>

			<label for="card-json">Aurora card JSON</label>
			<textarea id="card-json" bind:value={cardJson} rows={18} spellcheck="false" class="code"></textarea>
			<div class="editor-actions">
				<button type="button" class="primary" on:click|preventDefault={validateCard} disabled={busy}>
					{busy ? 'Validating…' : 'Validate'}
				</button>
			</div>
		</section>
	</section>

	<section class="graph-panel">
		<div class="panel-header">
			<h2>Graph Explorer</h2>
			<p>Visualize upstream and downstream relationships, and click any node to recenter.</p>
		</div>
		{#if !summary}
			<p class="placeholder">Load a model home to explore graph neighborhoods.</p>
		{:else if !selectedCard}
			<p class="placeholder">Select a card to inspect its graph neighborhood.</p>
		{:else}
			<div class="graph-toolbar">
				<div class="graph-depth-control">
					<label for="graph-depth">Generations</label>
					<input
						id="graph-depth"
						type="range"
						min={minGraphDepth}
						max={maxGraphDepth}
						step={1}
						value={graphDepth}
						aria-label="Graph depth"
						aria-valuemin={minGraphDepth}
						aria-valuemax={maxGraphDepth}
						aria-valuenow={graphDepth}
						on:input={(event) =>
							handleGraphDepthChange(Number((event.currentTarget as HTMLInputElement).value))}
						disabled={graphBusy}
					/>
					<span class="graph-depth-value">{graphDepth}</span>
				</div>
				<div class="graph-status">
					{#if graphBusy}
						<p>Loading neighborhood…</p>
					{:else if graph}
						<p>Showing {graph.nodes.length + 1} cards • {graph.filterLabel}</p>
					{:else}
						<p>Graph view ready for {selectedCard.name}</p>
					{/if}
				</div>
			</div>
			{#if graphError}
				<p class="warning">{graphError}</p>
			{/if}
			{#if graph}
				<div class={`graph-view ${graphBusy ? 'loading' : ''}`}>
					<div class="graph-column upstream">
						<h3>Incoming</h3>
						{#if graphUpstreamLanes.length}
							{#each graphUpstreamLanes as lane (lane.distance)}
								<div class="graph-lane">
									<p class="lane-label">{lane.label}</p>
									<div class="node-list">
										{#each lane.nodes as node (node.id)}
											<button
												type="button"
												class="graph-node"
												on:click={() => handleGraphNodeRecenter(node)}
												disabled={graphBusy}
												title={`${node.cardType}${node.status ? ` • ${node.status}` : ''}`}
											>
												<span class="node-id">{node.id}</span>
												<span class="node-name">{node.name}</span>
												<span class="node-meta">{node.cardType}</span>
											</button>
										{/each}
									</div>
								</div>
							{/each}
						{:else}
							<p class="placeholder">No upstream links in this depth.</p>
						{/if}
					</div>
					<div class="graph-center">
						<p class="lane-label">Center</p>
						<button
							type="button"
							class="graph-node center"
							on:click={() => selectedCard && refreshGraph(selectedCard.id)}
							disabled={graphBusy}
							title="Reload neighborhood"
						>
							<span class="node-id">{graph.center.id}</span>
							<span class="node-name">{graph.center.name}</span>
							<span class="node-meta">{graph.center.cardType}</span>
						</button>
					</div>
					<div class="graph-column downstream">
						<h3>Outgoing</h3>
						{#if graphDownstreamLanes.length}
							{#each graphDownstreamLanes as lane (lane.distance)}
								<div class="graph-lane">
									<p class="lane-label">{lane.label}</p>
									<div class="node-list">
										{#each lane.nodes as node (node.id)}
											<button
												type="button"
												class="graph-node"
												on:click={() => handleGraphNodeRecenter(node)}
												disabled={graphBusy}
												title={`${node.cardType}${node.status ? ` • ${node.status}` : ''}`}
											>
												<span class="node-id">{node.id}</span>
												<span class="node-name">{node.name}</span>
												<span class="node-meta">{node.cardType}</span>
											</button>
										{/each}
									</div>
								</div>
							{/each}
						{:else}
							<p class="placeholder">No downstream links in this depth.</p>
						{/if}
					</div>
				</div>
				{#if graphLateralNodes.length}
					<div class="graph-lateral">
						<h3>Cross-links</h3>
						<div class="node-list">
							{#each graphLateralNodes as node (node.id)}
								<button
									type="button"
									class="graph-node"
									on:click={() => handleGraphNodeRecenter(node)}
									disabled={graphBusy}
									title={`${node.cardType}${node.status ? ` • ${node.status}` : ''}`}
								>
									<span class="node-id">{node.id}</span>
									<span class="node-name">{node.name}</span>
									<span class="node-meta">{node.cardType}</span>
								</button>
							{/each}
						</div>
					</div>
				{/if}
				{#if graphEdgeDetails.length}
					<div class="graph-edges">
						<h3>Relationships</h3>
						<ul>
							{#each graphEdgeDetails as detail (detail.id)}
								<li>
									<strong>{detail.sourceLabel}</strong>
									<span aria-hidden="true">&nbsp;—{detail.relationship}→&nbsp;</span>
									<strong>{detail.targetLabel}</strong>
								</li>
							{/each}
						</ul>
					</div>
				{/if}
			{:else if graphBusy}
				<p class="placeholder">Loading graph neighborhood…</p>
			{:else}
				<p class="placeholder">Graph neighborhood data is not available for this card yet.</p>
			{/if}
		{/if}
	</section>

	<section class={`status-panel ${status}`}>
		<h2>Validation status</h2>
		<pre>{validationMessage}</pre>
	</section>
</main>

<style>
	:global(:root) {
		--panel-bg: rgba(3, 22, 36, 0.92);
		--panel-border: rgba(111, 255, 229, 0.35);
		--accent: #5ff8de;
		--accent-strong: #1fb9ff;
		--text-muted: #a7c2d1;
		--danger: #ff9d8e;
	}

	:global(body) {
		margin: 0;
		font-family:
			'Space Grotesk',
			'IBM Plex Sans',
			'Segoe UI',
			system-ui,
			-apple-system,
			sans-serif;
		background: radial-gradient(circle at 25% 20%, rgba(32, 190, 214, 0.25), transparent 55%),
			linear-gradient(130deg, #020c13, #01060c 55%, #03101a);
		color: #f4fdff;
		min-height: 100vh;
	}

	main {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem 1.5rem 4rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.hero {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: 1.5rem;
		background: var(--panel-bg);
		border: 1px solid var(--panel-border);
		border-radius: 1.5rem;
		padding: 1.75rem;
		box-shadow: 0 30px 60px rgba(0, 0, 0, 0.35);
	}

	.hero-copy h1 {
		margin: 0 0 0.35rem;
		font-size: 2.25rem;
	}

	.hero-copy p {
		margin: 0;
		color: var(--text-muted);
	}

	.context-panel {
		display: flex;
		flex-direction: column;
		gap: 0.65rem;
	}

	label {
		font-weight: 600;
		letter-spacing: 0.02em;
	}

	.input-row {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
		align-items: center;
	}

	input[type='text'] {
		width: 100%;
		border-radius: 0.75rem;
		border: 1px solid rgba(255, 255, 255, 0.15);
		background: rgba(2, 11, 18, 0.8);
		color: #f6feff;
		padding: 0.75rem 1rem;
		font-size: 0.95rem;
	}

	.input-row input[type='text'] {
		flex: 1 1 260px;
		min-width: 220px;
	}

	.input-row button {
		flex: 0 0 auto;
	}

	.status-line {
		margin: 0;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.warning {
		margin: 0;
		color: var(--danger);
		font-weight: 600;
	}

	.error-banner {
		margin: 0.35rem 0 0;
		border: 1px solid rgba(255, 157, 142, 0.9);
		border-radius: 0.9rem;
		background: rgba(56, 6, 18, 0.75);
		padding: 0.75rem 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.65rem;
	}

	.error-banner pre {
		margin: 0;
		white-space: pre-wrap;
		word-break: break-word;
		font-family: 'IBM Plex Mono', 'Space Mono', 'Fira Code', monospace;
		font-size: 0.9rem;
		color: #ffd9d1;
	}

	.metrics {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
		gap: 0.75rem;
	}

	.metric {
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 1rem;
		padding: 1rem;
	}

	.metric p {
		margin: 0;
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.metric strong {
		display: block;
		margin-top: 0.25rem;
		font-size: 1.5rem;
	}

	.workspace {
		display: grid;
		grid-template-columns: 360px 1fr;
		gap: 1.5rem;
	}

	.graph-panel {
		background: var(--panel-bg);
		border: 1px solid var(--panel-border);
		border-radius: 1.5rem;
		padding: 1.5rem;
		box-shadow: 0 30px 60px rgba(0, 0, 0, 0.3);
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.navigator,
	.editor,
	.status-panel {
		background: var(--panel-bg);
		border: 1px solid var(--panel-border);
		border-radius: 1.5rem;
		padding: 1.5rem;
		box-shadow: 0 30px 60px rgba(0, 0, 0, 0.3);
	}

	.panel-header {
		margin-bottom: 1rem;
	}

	.panel-header h2 {
		margin: 0;
	}

	.panel-header p {
		margin: 0.25rem 0 0;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.graph-toolbar {
		display: flex;
		flex-wrap: wrap;
		gap: 1rem;
		justify-content: space-between;
		align-items: center;
	}

	.graph-depth-control {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.graph-depth-control input[type='range'] {
		width: 180px;
		accent-color: var(--accent);
	}

	.graph-depth-value {
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}

	.graph-status p {
		margin: 0;
		color: var(--text-muted);
	}

	.filter-toolbar {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 1rem;
		padding: 0.75rem 1rem 1rem;
		background: rgba(2, 14, 20, 0.45);
		margin-bottom: 1rem;
	}

	.filter-toolbar h3 {
		margin: 0;
		font-size: 0.95rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.filter-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 0.4rem;
	}

	.filter-chips button {
		border-radius: 999px;
		border: 1px solid rgba(255, 255, 255, 0.12);
		background: rgba(3, 18, 27, 0.7);
		color: inherit;
		padding: 0.35rem 0.85rem;
		font-size: 0.85rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 0.35rem;
		transition:
			border-color 140ms ease,
			background 140ms ease;
	}

	.filter-chips button small {
		font-size: 0.75rem;
		color: var(--text-muted);
	}

	.filter-chips button.selected {
		border-color: var(--accent);
		background: rgba(3, 40, 50, 0.9);
		color: #5ff8de;
	}

	.filter-chips button:disabled {
		opacity: 0.7;
		cursor: progress;
	}

	.filter-status,
	.filter-description {
		margin: 0;
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.filter-description {
		font-size: 0.8rem;
	}

	.graph-view {
		display: grid;
		grid-template-columns: minmax(220px, 1fr) minmax(200px, 240px) minmax(220px, 1fr);
		gap: 1rem;
		align-items: start;
	}

	.graph-column h3 {
		margin: 0 0 0.35rem;
		font-size: 0.95rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.graph-center {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
	}

	.graph-lane {
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 1rem;
		padding: 0.75rem;
		background: rgba(2, 14, 20, 0.5);
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 0.75rem;
	}

	.lane-label {
		margin: 0;
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.node-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.graph-node {
		border-radius: 1rem;
		border: 1px solid rgba(255, 255, 255, 0.12);
		background: rgba(3, 18, 27, 0.75);
		color: inherit;
		padding: 0.65rem 0.85rem;
		text-align: left;
		cursor: pointer;
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
		transition:
			border-color 140ms ease,
			background 140ms ease;
	}

	.graph-node.center {
		border-color: var(--accent);
		background: rgba(7, 54, 66, 0.85);
		width: 100%;
	}

	.graph-node:focus-visible {
		outline: 2px solid var(--accent-strong);
		outline-offset: 2px;
	}

	.graph-node:disabled {
		cursor: progress;
		opacity: 0.8;
	}

	.node-id {
		font-size: 0.8rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.node-name {
		font-weight: 600;
	}

	.node-meta {
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.graph-lateral {
		border: 1px dashed rgba(255, 255, 255, 0.2);
		border-radius: 1rem;
		padding: 1rem;
	}

	.graph-edges ul {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}

	.graph-edges li {
		font-size: 0.9rem;
		color: var(--text-muted);
	}

	.mission-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		max-height: 520px;
		overflow-y: auto;
		padding-right: 0.25rem;
	}

	.mission-card {
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 1rem;
		padding: 1rem;
		background: rgba(2, 14, 20, 0.6);
	}

	.mission-header {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
		margin-bottom: 0.35rem;
	}

	.mission-id {
		margin: 0;
		font-size: 0.85rem;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.mission-header h3 {
		margin: 0;
		font-size: 1.1rem;
	}

	.mission-header span {
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	.mission-card ul {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}

	.mission-card button {
		width: 100%;
		text-align: left;
		border: 1px solid rgba(255, 255, 255, 0.07);
		background: rgba(4, 12, 18, 0.65);
		border-radius: 0.9rem;
		padding: 0.6rem 0.8rem;
		color: inherit;
		cursor: pointer;
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 0.35rem 0.6rem;
		transition:
			border-color 160ms ease,
			background 160ms ease;
		font-family: inherit;
	}

	.mission-card button .card-id {
		font-weight: 600;
	}

	.mission-card button .card-name {
		grid-column: span 2;
	}

	.mission-card button .card-meta {
		grid-column: span 2;
		font-size: 0.85rem;
		color: var(--text-muted);
	}

	@media (max-width: 1080px) {
		.workspace {
			grid-template-columns: 1fr;
		}

		.graph-view {
			grid-template-columns: 1fr;
		}

		.graph-center {
			order: -1;
		}
	}

	@media (max-width: 720px) {
		.input-row {
			flex-direction: column;
			align-items: stretch;
		}

		.input-row button {
			width: 100%;
		}
	}

	.mission-card button:hover,
	.mission-card button.selected {
		border-color: var(--accent);
		background: rgba(3, 40, 50, 0.85);
	}

	.no-matches {
		margin: 0.35rem 0 0;
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.placeholder {
		color: var(--text-muted);
		margin: 2rem 0 0;
	}

	.editor {
		display: flex;
		flex-direction: column;
		gap: 0.85rem;
	}

	.editor-header {
		display: flex;
		justify-content: space-between;
		gap: 1rem;
		align-items: flex-start;
	}

	.editor-header h2 {
		margin: 0;
	}

	.editor-header p {
		margin: 0.2rem 0 0;
		color: var(--text-muted);
	}

	.path-chip {
		font-size: 0.85rem;
		padding: 0.4rem 0.75rem;
		border-radius: 999px;
		border: 1px solid rgba(255, 255, 255, 0.2);
		background: rgba(255, 255, 255, 0.05);
	}

	textarea {
		border-radius: 1rem;
		border: 1px solid rgba(255, 255, 255, 0.15);
		background: rgba(2, 11, 18, 0.85);
		color: #f4fdff;
		padding: 1rem;
		font-family: 'IBM Plex Mono', 'Space Mono', 'Fira Code', monospace;
		font-size: 0.9rem;
		line-height: 1.5;
		resize: vertical;
	}

	.editor-actions {
		display: flex;
		gap: 0.75rem;
	}

	button.primary {
		background: linear-gradient(110deg, var(--accent), var(--accent-strong));
		border: none;
		border-radius: 999px;
		padding: 0.75rem 1.75rem;
		color: #041a22;
		font-weight: 600;
		cursor: pointer;
		transition:
			transform 160ms ease,
			box-shadow 160ms ease;
	}

	button.primary:disabled {
		opacity: 0.6;
		cursor: progress;
	}

	button.primary:not(:disabled):hover {
		transform: translateY(-1px) scale(1.01);
		box-shadow: 0 8px 18px rgba(31, 185, 255, 0.35);
	}

	button.ghost {
		border-radius: 999px;
		border: 1px dashed rgba(255, 255, 255, 0.35);
		background: rgba(4, 16, 24, 0.65);
		color: #f4fdff;
		padding: 0.75rem 1.25rem;
		font-weight: 600;
		cursor: pointer;
		transition:
			border-color 160ms ease,
			color 160ms ease;
	}

	button.ghost:disabled {
		opacity: 0.55;
		cursor: progress;
	}

	button.ghost:not(:disabled):hover,
	button.ghost:focus-visible {
		border-color: var(--accent);
		color: var(--accent);
	}

	button.ghost:focus-visible {
		outline: 2px solid var(--accent-strong);
		outline-offset: 2px;
	}

	button.ghost.compact {
		padding: 0.5rem 1rem;
		font-size: 0.85rem;
	}

	.status-panel {
		white-space: pre-wrap;
		word-break: break-word;
	}

	.status-panel h2 {
		margin-top: 0;
	}

	.status-panel pre {
		margin: 0;
		font-family: 'IBM Plex Mono', 'Space Mono', monospace;
		font-size: 0.95rem;
	}

	.status-panel.valid {
		border-color: rgba(95, 248, 222, 0.65);
	}

	.status-panel.invalid {
		border-color: rgba(255, 174, 91, 0.65);
	}

	.status-panel.error {
		border-color: rgba(255, 140, 140, 0.75);
	}

	@media (max-width: 1100px) {
		.workspace {
			grid-template-columns: 1fr;
		}

		.mission-list {
			max-height: 360px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		* {
			transition: none !important;
			animation: none !important;
		}
	}
</style>
