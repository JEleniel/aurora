<script lang="ts">
	import { allCards } from '$lib/stores/architecture';
	import { analyzeRelationships, calculateImpact, type RelationshipAnalysis } from '$lib/services/relationships';
	import CardSelect from '$lib/components/CardSelect.svelte';

	let selectedCardId = $state('');
	let analysis = $state<RelationshipAnalysis | null>(null);
	let loading = $state(false);
	let errorMessage = $state('');
	let maxDepth = $state(3);

	async function performAnalysis() {
		if (!selectedCardId) {
			errorMessage = 'Please select a card';
			return;
		}

		loading = true;
		errorMessage = '';

		try {
			const depthValue = maxDepth === 0 ? Number.MAX_SAFE_INTEGER : maxDepth;
			analysis = await analyzeRelationships(selectedCardId, depthValue);
		} catch (err) {
			errorMessage = `Failed to analyze relationships: ${err instanceof Error ? err.message : String(err)}`;
		} finally {
			loading = false;
		}
	}

	function getCategoryColor(cardType: string): string {
		const colors: Record<string, string> = {
			Driver: '#3b82f6',
			Requirement: '#6366f1',
			Behavior: '#06b6d4',
			Constraint: '#14b8a6',
			Interface: '#ec4899',
			Actor: '#f59e0b',
			LogicalComponent: '#8b5cf6',
			Test: '#10b981',
			Artifact: '#6b7280',
			View: '#f97316',
			DeployableNode: '#d946ef',
			Mission: '#ef4444',
			Note: '#a78bfa',
		};
		return colors[cardType] || '#6b7280';
	}
</script>

<div class="relationship-browser">
	<header class="browser-header">
		<div class="header-content">
			<h1>Relationship Browser</h1>
			<p>Explore card dependencies and impact analysis</p>
		</div>
	</header>

	<div class="controls-section">
		<div class="control-group">
			<label for="card-select">Select Card:</label>
			<CardSelect bind:value={selectedCardId} cards={$allCards} />
		</div>

		<div class="control-group">
			<label for="depth-select">Analysis Depth:</label>
			<div class="depth-controls">
				<select id="depth-select" bind:value={maxDepth} class="depth-select">
					<option value={1}>1-hop (immediate)</option>
					<option value={2}>2-hops</option>
					<option value={3}>3-hops (default)</option>
					<option value={5}>5-hops</option>
					<option value={0}>All (unlimited)</option>
				</select>
			</div>
		</div>

		<button
			onclick={performAnalysis}
			disabled={!selectedCardId || loading}
			class="analyze-button"
		>
			{loading ? 'Analyzing...' : 'Analyze'}
		</button>
	</div>

	{#if errorMessage}
		<div role="alert" class="error-message">
			{errorMessage}
			<button
				onclick={() => (errorMessage = '')}
				class="dismiss-button"
				aria-label="Dismiss error message"
			>
				×
			</button>
		</div>
	{/if}

	{#if analysis}
		<div class="analysis-container">
			<div class="impact-assessment">
				<h2>Impact Assessment</h2>
				<div class="impact-metrics">
					<div class="metric">
						<span class="metric-label">Direct Dependents:</span>
						<span class="metric-value">{calculateImpact(analysis).direct_dependents}</span>
					</div>
					<div class="metric">
						<span class="metric-label">Transitive Dependents:</span>
						<span class="metric-value">{calculateImpact(analysis).transitive_dependents}</span>
					</div>
					<div class="metric">
						<span class="metric-label">Direct Dependencies:</span>
						<span class="metric-value">{calculateImpact(analysis).direct_dependencies}</span>
					</div>
					<div class="metric">
						<span class="metric-label">Transitive Dependencies:</span>
						<span class="metric-value">{calculateImpact(analysis).transitive_dependencies}</span>
					</div>
					<div class="metric highlight">
						<span class="metric-label">Total Impact Scope:</span>
						<span class="metric-value">{analysis.total_affected} cards</span>
					</div>
					{#if calculateImpact(analysis).circular_dependency_risk}
						<div class="metric warning">
							<span class="metric-label">⚠️ Circular Dependencies:</span>
							<span class="metric-value">{analysis.circular_refs.length}</span>
						</div>
					{/if}
				</div>
			</div>

			<div class="relationships-container">
				<div class="relationship-section">
					<h2>Upstream Dependencies ({analysis.upstream.length})</h2>
					{#if analysis.upstream.length === 0}
						<p class="empty-state">No upstream dependencies</p>
					{:else}
						<div class="relationship-tree">
							{#each analysis.upstream as entry (entry.id)}
								<div class="tree-entry" style="margin-left: {entry.depth * 1.5}rem">
									<div class="entry-header">
										<span
											class="type-badge"
											style="background-color: {getCategoryColor(entry.card_type)}"
										>
											{entry.card_type}
										</span>
										<span class="entry-name">{entry.name}</span>
										<span class="depth-indicator">D{entry.depth}</span>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>

				<div class="relationship-section">
					<h2>Downstream Dependencies ({analysis.downstream.length})</h2>
					{#if analysis.downstream.length === 0}
						<p class="empty-state">No downstream dependencies</p>
					{:else}
						<div class="relationship-tree">
							{#each analysis.downstream as entry (entry.id)}
								<div class="tree-entry" style="margin-left: {entry.depth * 1.5}rem">
									<div class="entry-header">
										<span
											class="type-badge"
											style="background-color: {getCategoryColor(entry.card_type)}"
										>
											{entry.card_type}
										</span>
										<span class="entry-name">{entry.name}</span>
										<span class="depth-indicator">D{entry.depth}</span>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.relationship-browser {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		padding: 1.5rem 2rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.browser-header {
		background: linear-gradient(135deg, var(--color-primary), var(--color-secondary));
		padding: 2rem;
		border-radius: 0.75rem;
		color: white;
		margin-bottom: 1rem;
	}

	.header-content h1 {
		margin: 0 0 0.5rem 0;
		font-size: 2rem;
		font-weight: 600;
	}

	.header-content p {
		margin: 0;
		opacity: 0.9;
		font-size: 0.95rem;
	}

	.controls-section {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		align-items: flex-end;
		padding: 1rem;
		background: var(--color-surface-dim);
		border-radius: 0.5rem;
		margin-bottom: 1rem;
	}

	.control-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		flex: 1;
		min-width: 200px;
	}

	.control-group label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-on-surface-variant);
	}

	.depth-controls {
		display: flex;
		gap: 0.5rem;
	}

	.depth-select {
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--color-outline);
		border-radius: 0.4rem;
		background: var(--color-surface);
		color: var(--color-on-surface);
		font-size: 0.875rem;
	}

	.analyze-button {
		padding: 0.625rem 1.5rem;
		background: var(--color-primary);
		color: white;
		border: none;
		border-radius: 0.4rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.analyze-button:hover:not(:disabled) {
		background: var(--color-primary-dark);
	}

	.analyze-button:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.error-message {
		padding: 1rem;
		background: var(--color-error-container);
		color: var(--color-error);
		border-radius: 0.5rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.dismiss-button {
		background: none;
		border: none;
		color: var(--color-error);
		font-size: 1.5rem;
		cursor: pointer;
		padding: 0;
		width: 1.5rem;
		height: 1.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.analysis-container {
		display: grid;
		grid-template-columns: 300px 1fr;
		gap: 1.5rem;
	}

	.impact-assessment {
		background: var(--color-surface-bright);
		border: 1px solid var(--color-outline);
		border-radius: 0.5rem;
		padding: 1.5rem;
		height: fit-content;
		position: sticky;
		top: 1rem;
	}

	.impact-assessment h2 {
		margin: 0 0 1rem 0;
		font-size: 1.1rem;
		color: var(--color-on-surface);
	}

	.impact-metrics {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.metric {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem;
		background: var(--color-surface);
		border-radius: 0.4rem;
		font-size: 0.875rem;
	}

	.metric.highlight {
		background: var(--color-primary-container);
		color: var(--color-on-primary-container);
	}

	.metric.warning {
		background: var(--color-warning-container);
		color: var(--color-on-warning-container);
	}

	.metric-label {
		font-weight: 500;
		color: var(--color-on-surface-variant);
	}

	.metric.highlight .metric-label,
	.metric.warning .metric-label {
		color: inherit;
	}

	.metric-value {
		font-weight: 600;
		font-size: 1rem;
	}

	.relationships-container {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1.5rem;
	}

	.relationship-section {
		background: var(--color-surface-bright);
		border: 1px solid var(--color-outline);
		border-radius: 0.5rem;
		padding: 1.5rem;
	}

	.relationship-section h2 {
		margin: 0 0 1rem 0;
		font-size: 1.1rem;
		color: var(--color-on-surface);
	}

	.empty-state {
		color: var(--color-on-surface-variant);
		font-style: italic;
		margin: 0;
		padding: 1rem;
		text-align: center;
	}

	.relationship-tree {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.tree-entry {
		padding: 0.75rem;
		background: var(--color-surface);
		border-radius: 0.4rem;
		border-left: 3px solid var(--color-outline);
		transition: background-color 0.2s;
	}

	.tree-entry:hover {
		background: var(--color-surface-dim);
	}

	.entry-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		flex-wrap: wrap;
	}

	.type-badge {
		display: inline-block;
		padding: 0.25rem 0.6rem;
		border-radius: 0.3rem;
		color: white;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}

	.entry-name {
		flex: 1;
		color: var(--color-on-surface);
		font-weight: 500;
		min-width: 150px;
	}

	.depth-indicator {
		color: var(--color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
		padding: 0.25rem 0.5rem;
		background: var(--color-surface-dim);
		border-radius: 0.3rem;
		white-space: nowrap;
	}

	@media (max-width: 1024px) {
		.analysis-container {
			grid-template-columns: 1fr;
		}

		.impact-assessment {
			position: static;
		}

		.relationships-container {
			grid-template-columns: 1fr;
		}
	}
</style>
