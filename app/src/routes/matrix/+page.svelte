<script lang="ts">
	import type { PageData } from './$types';
	import { allCards } from '$lib/stores/architecture';
	import { generateTraceabilityMatrix } from '$lib/services/architecture';
	import { parseTraceabilityMatrix, type TraceabilityMatrix } from '$lib/traceabilityTypes';
	import Select from '$lib/components/Select.svelte';

	export const data: PageData = {};

	let sourceType: string = 'Driver';
	let targetType: string = 'Requirement';
	let matrix: TraceabilityMatrix | null = null;
	let loading = false;
	let errorMessage = '';

	const cardTypeOptions = [
		{ value: 'Driver', label: 'Driver' },
		{ value: 'Requirement', label: 'Requirement' },
		{ value: 'Behavior', label: 'Behavior' },
		{ value: 'Constraint', label: 'Constraint' },
		{ value: 'Interface', label: 'Interface' },
		{ value: 'Actor', label: 'Actor' },
		{ value: 'LogicalComponent', label: 'Logical Component' },
		{ value: 'DeployableNode', label: 'Deployable Node' },
		{ value: 'Artifact', label: 'Artifact' },
		{ value: 'Test', label: 'Test' },
		{ value: 'View', label: 'View' },
		{ value: 'Note', label: 'Note' },
	];

	// Build card map for lookup
	$: cardMap = new Map($allCards.map((c) => [c.id, c]));

	async function generateMatrix() {
		errorMessage = '';
		loading = true;
		try {
			const result = await generateTraceabilityMatrix(sourceType, targetType);
			matrix = parseTraceabilityMatrix(result);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
			matrix = null;
		} finally {
			loading = false;
		}
	}

	function swapTypes() {
		[sourceType, targetType] = [targetType, sourceType];
		matrix = null;
	}
</script>

<div class="page">
	<header class="page-header">
		<div class="header-content">
			<h1>Traceability Matrix</h1>
			<p>Visualize relationships between different card types</p>
		</div>
	</header>

	<section class="controls">
		<div class="control-group">
			<Select label="From (Source)" bind:value={sourceType} options={cardTypeOptions} disabled={loading} />
			<button
				type="button"
				class="md-button md-button--outlined swap-button"
				on:click={swapTypes}
				disabled={loading}
				title="Swap source and target"
			>
				⇄
			</button>
			<Select label="To (Target)" bind:value={targetType} options={cardTypeOptions} disabled={loading} />
			<button type="button" class="md-button md-button--filled" on:click={generateMatrix} disabled={loading}>
				{loading ? 'Generating...' : 'Generate Matrix'}
			</button>
		</div>

		{#if errorMessage}
			<div class="error-message">
				<strong>Error:</strong>
				{errorMessage}
			</div>
		{/if}
	</section>

	{#if matrix}
		<section class="matrix-section">
			<div class="matrix-stats">
				<div class="stat-card">
					<div class="stat-value">{matrix.rows.length}</div>
					<div class="stat-label">Source Cards</div>
				</div>
				<div class="stat-card">
					<div class="stat-value">{matrix.columns.length}</div>
					<div class="stat-label">Target Cards</div>
				</div>
				<div class="stat-card">
					<div class="stat-value">{matrix.coverage.toFixed(1)}%</div>
					<div class="stat-label">Coverage</div>
				</div>
				<div class="stat-card">
					<div class="stat-value">{matrix.orphanedSources.length}</div>
					<div class="stat-label">Orphaned Sources</div>
				</div>
			</div>

			<div class="matrix-container">
				<table class="traceability-table">
					<thead>
						<tr>
							<th>Source ({matrix.sourceType})</th>
							{#each matrix.columns as columnId}
								<th title={columnId} class="column-header">
									{#if cardMap.get(columnId)?.name}
										{cardMap.get(columnId)?.name.substring(0, 3)}
									{:else}
										{columnId.substring(0, 8)}
									{/if}
								</th>
							{/each}
							<th>Links</th>
						</tr>
					</thead>
					<tbody>
						{#each matrix.rows as row}
							<tr class={row.totalLinks === 0 ? 'orphaned' : ''}>
								<td class="row-header">
									<div class="row-label">
										<strong>{row.name}</strong>
										<small>{row.id}</small>
									</div>
								</td>
								{#each row.cells as cell}
									<td class={cell.hasLink ? 'linked' : 'empty'}>
										{#if cell.hasLink}
											<span class="link-marker">●</span>
										{/if}
									</td>
								{/each}
								<td class="link-count">{row.totalLinks}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			{#if matrix.orphanedSources.length > 0}
				<div class="warning-section">
					<h3>⚠ Orphaned Sources</h3>
					<p>These {matrix.sourceType}s have no links to {matrix.targetType}s:</p>
					<ul>
						{#each matrix.orphanedSources as id}
							<li>
								<strong>{cardMap.get(id)?.name || id}</strong>
								<small>{id}</small>
							</li>
						{/each}
					</ul>
				</div>
			{/if}

			{#if matrix.unreferencedTargets.length > 0}
				<div class="warning-section">
					<h3>⚠ Unreferenced Targets</h3>
					<p>These {matrix.targetType}s are not linked from any source:</p>
					<ul>
						{#each matrix.unreferencedTargets as id}
							<li>
								<strong>{cardMap.get(id)?.name || id}</strong>
								<small>{id}</small>
							</li>
						{/each}
					</ul>
				</div>
			{/if}
		</section>
	{/if}
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.page-header {
		background: linear-gradient(135deg, var(--md-sys-color-primary) 0%, var(--md-sys-color-tertiary) 100%);
		color: var(--md-sys-color-on-primary);
		padding: 2rem;
		border-radius: 0.5rem;
		margin-bottom: 0.5rem;
	}

	.header-content h1 {
		margin: 0 0 0.5rem 0;
		font-size: 2rem;
		font-weight: 600;
	}

	.header-content p {
		margin: 0;
		opacity: 0.9;
	}

	.controls {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 0.5rem;
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.control-group {
		display: flex;
		gap: 1rem;
		align-items: flex-end;
		flex-wrap: wrap;
	}

	.control-group > :nth-child(1),
	.control-group > :nth-child(3) {
		flex: 1;
		min-width: 200px;
	}

	.swap-button {
		min-width: 48px;
		margin-bottom: 0;
	}

	.error-message {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		padding: 1rem;
		border-radius: 0.25rem;
	}

	.matrix-section {
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.matrix-stats {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: 1rem;
	}

	.stat-card {
		background-color: var(--md-sys-color-surface-container);
		padding: 1.5rem;
		border-radius: 0.5rem;
		text-align: center;
		border: 1px solid var(--md-sys-color-outline-variant);
	}

	.stat-value {
		font-size: 2rem;
		font-weight: 600;
		color: var(--md-sys-color-primary);
	}

	.stat-label {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		margin-top: 0.5rem;
	}

	.matrix-container {
		overflow-x: auto;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0.5rem;
		background-color: var(--md-sys-color-surface);
	}

	.traceability-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}

	.traceability-table thead {
		background-color: var(--md-sys-color-surface-container);
		border-bottom: 2px solid var(--md-sys-color-outline-variant);
	}

	.traceability-table th {
		padding: 0.75rem;
		text-align: center;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	.column-header {
		max-width: 60px;
		word-break: break-word;
		vertical-align: bottom;
	}

	.traceability-table td {
		padding: 0.5rem;
		text-align: center;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}

	.row-header {
		position: sticky;
		left: 0;
		background-color: var(--md-sys-color-surface-container);
		text-align: left;
		font-weight: 500;
		min-width: 200px;
		border-right: 2px solid var(--md-sys-color-outline-variant);
	}

	.row-label {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.row-label strong {
		color: var(--md-sys-color-on-surface);
	}

	.row-label small {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}

	.link-marker {
		color: var(--md-sys-color-primary);
		font-size: 1.25rem;
	}

	.linked {
		background-color: var(--md-sys-color-primary-container);
	}

	.link-count {
		font-weight: 600;
		color: var(--md-sys-color-primary);
		border-left: 2px solid var(--md-sys-color-outline-variant);
	}

	tr.orphaned .row-header {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}

	tr.orphaned .row-label strong {
		color: var(--md-sys-color-on-error-container);
	}

	.warning-section {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		padding: 1.5rem;
		border-radius: 0.5rem;
	}

	.warning-section h3 {
		margin: 0 0 0.5rem 0;
	}

	.warning-section p {
		margin: 0 0 1rem 0;
	}

	.warning-section ul {
		margin: 0;
		padding-left: 1.5rem;
	}

	.warning-section li {
		margin: 0.5rem 0;
	}

	.warning-section small {
		display: block;
		font-size: 0.75rem;
		opacity: 0.8;
	}
</style>
