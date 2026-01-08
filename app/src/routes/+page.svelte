<script lang="ts">
	import { allCards, architecture, cardToCreate } from '$lib/stores/architecture';
	import { CardType as CardTypeEnum, CardStatus as CardStatusEnum } from '$lib/types';
	import { goto } from '$app/navigation';
	import { invoke } from '@tauri-apps/api/core';

	let isLoading: boolean = $state(false);
	let errorMessage: string = $state('');

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

	async function handleLoadArchitecture() {
		isLoading = true;
		errorMessage = '';
		try {
			const selected = await invoke<string | null>('select_file');
			if (selected) {
				await architecture.loadFromZip(selected);
				goto('/dashboard');
			}
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			isLoading = false;
		}
	}

	function handleCreateCard() {
		const isFirstCard = $allCards.length === 0;
		const cardType = isFirstCard ? CardTypeEnum.Mission : CardTypeEnum.Requirement;
		const cardName = isFirstCard ? 'Mission' : undefined;
		cardToCreate.set({ type: cardType, name: cardName });
		goto('/cards');
	}

	function handleEditCard(cardId: string) {
		const card = $allCards.find((c) => c.id === cardId);
		if (card) {
			cardToCreate.set(card);
			goto('/cards');
		}
	}
</script>

<div class="startup-container">
	{#if $allCards.length === 0}
		<section class="startup-section">
			<div class="startup-header">
				<h2>Welcome to AURORA</h2>
				<p>Agent-Unified Representation of Requirements and Architecture</p>
			</div>

			<div class="startup-grid">
				<div class="startup-card">
					<div class="startup-card-icon">📂</div>
					<h3>Load Existing Architecture</h3>
					<p>Open a previously saved architecture file (ZIP format).</p>
					<button class="md-button md-button--primary" onclick={handleLoadArchitecture} disabled={isLoading}>
						{isLoading ? 'Loading...' : 'Load Architecture'}
					</button>
				</div>

				<div class="startup-card">
					<div class="startup-card-icon">✨</div>
					<h3>Start New Architecture</h3>
					<p>Begin with a new architecture by creating your first card.</p>
					<button class="md-button md-button--primary" onclick={handleCreateCard}> Create First Card </button>
				</div>
			</div>

			{#if errorMessage}
				<div class="error-message">
					<p>⚠️ {errorMessage}</p>
				</div>
			{/if}
		</section>
	{:else}
		<section class="cards-grid">
			<div class="cards-header">
				<h2>Architecture Cards</h2>
				<p>{$allCards.length} card{$allCards.length !== 1 ? 's' : ''} defined</p>
				<button class="md-button md-button--primary" onclick={handleCreateCard}> + Create Card </button>
			</div>

			<div class="cards-table-container">
				<table class="cards-table">
					<thead>
						<tr>
							<th>Type</th>
							<th>Name</th>
							<th>Description</th>
							<th>Status</th>
							<th>Modified</th>
							<th>Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each $allCards as card (card.id)}
							<tr class="card-row">
								<td class="col-type">
									<span class="type-badge">{getTypeLabel(card.type)}</span>
								</td>
								<td class="col-name"><strong>{card.name}</strong></td>
								<td class="col-description">{card.description || '—'}</td>
								<td class="col-status">
									{#if card.status}
										<span
											class="status-badge"
											style={`--status-color: ${getStatusColor(card.status)}`}
										>
											{card.status}
										</span>
									{:else}
										<span class="status-badge">Draft</span>
									{/if}
								</td>
								<td class="col-modified">{card.modified_at || '—'}</td>
								<td class="col-actions">
									<button
										class="action-button"
										title="Edit card"
										onclick={() => handleEditCard(card.id)}
									>
										✏️
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</section>
	{/if}
</div>

<style>
	.startup-container {
		width: 100%;
		min-height: calc(100vh - 100px);
		padding: 2rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.startup-section {
		max-width: 900px;
		width: 100%;
	}

	.startup-header {
		text-align: center;
		margin-bottom: 3rem;
	}

	.startup-header h2 {
		font-size: 2.5rem;
		font-weight: 600;
		color: var(--md-sys-color-primary);
		margin: 0 0 0.5rem 0;
	}

	.startup-header p {
		font-size: 1.125rem;
		color: var(--md-sys-color-on-surface-variant);
		margin: 0;
	}

	.startup-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
		gap: 2rem;
	}

	.startup-card {
		background: linear-gradient(135deg, var(--md-sys-color-surface-container) 0%, var(--md-sys-color-surface) 100%);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 16px;
		padding: 2rem;
		text-align: center;
		transition: all 0.3s cubic-bezier(0.2, 0, 0, 1);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
	}

	.startup-card:hover {
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
		border-color: var(--md-sys-color-primary);
		transform: translateY(-4px);
	}

	.startup-card-icon {
		font-size: 3rem;
		margin-bottom: 1rem;
	}

	.startup-card h3 {
		font-size: 1.25rem;
		font-weight: 600;
		margin: 0 0 0.75rem 0;
		color: var(--md-sys-color-on-surface);
	}

	.startup-card p {
		font-size: 0.95rem;
		color: var(--md-sys-color-on-surface-variant);
		margin: 0 0 1.5rem 0;
		line-height: 1.5;
	}

	.startup-card button {
		width: 100%;
	}

	.error-message {
		margin-top: 2rem;
		background-color: color-mix(in srgb, var(--md-sys-color-error) 12%, transparent);
		border: 1px solid var(--md-sys-color-error);
		border-radius: 8px;
		padding: 1rem;
		color: var(--md-sys-color-error);
		text-align: center;
	}

	.cards-grid {
		width: 100%;
		padding: 0;
	}

	.cards-header {
		margin-bottom: 2rem;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.cards-header h2 {
		margin: 0;
		font-size: 1.75rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	.cards-header p {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.95rem;
	}

	.cards-header button {
		white-space: nowrap;
	}

	.cards-table-container {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		overflow: hidden;
	}

	.cards-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.95rem;
	}

	.cards-table thead {
		background-color: var(--md-sys-color-surface-container);
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}

	.cards-table th {
		padding: 1rem;
		text-align: left;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
		white-space: nowrap;
	}

	.cards-table td {
		padding: 1rem;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface);
	}

	.card-row:hover {
		background-color: var(--md-sys-color-surface-container);
	}

	.col-type {
		width: 80px;
	}

	.col-name {
		width: 20%;
		max-width: 250px;
	}

	.col-description {
		flex: 1;
		max-width: 40%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--md-sys-color-on-surface-variant);
	}

	.col-status {
		width: 120px;
	}

	.col-modified {
		width: 140px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}

	.col-actions {
		width: 60px;
		text-align: center;
	}

	.type-badge {
		font-size: 1.25rem;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.status-badge {
		display: inline-block;
		padding: 0.25rem 0.75rem;
		border-radius: 6px;
		font-size: 0.8rem;
		font-weight: 500;
		text-transform: capitalize;
		background-color: color-mix(in srgb, var(--status-color, var(--md-sys-color-primary)) 15%, transparent);
		color: var(--status-color, var(--md-sys-color-primary));
	}

	.action-button {
		background: none;
		border: none;
		cursor: pointer;
		font-size: 1rem;
		padding: 0.25rem;
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.action-button:hover {
		background-color: color-mix(in srgb, var(--md-sys-color-primary) 15%, transparent);
	}

	@media (max-width: 1200px) {
		.col-description {
			display: none;
		}

		.cards-table th:nth-child(3),
		.cards-table td:nth-child(3) {
			display: none;
		}
	}

	@media (max-width: 768px) {
		.startup-container {
			padding: 1.5rem;
		}

		.startup-header h2 {
			font-size: 1.75rem;
		}

		.startup-grid {
			gap: 1rem;
		}

		.startup-card {
			padding: 1.5rem;
		}

		.cards-header {
			flex-direction: column;
			align-items: flex-start;
		}

		.col-modified {
			display: none;
		}

		.cards-table th:nth-child(5),
		.cards-table td:nth-child(5) {
			display: none;
		}
	}
</style>
