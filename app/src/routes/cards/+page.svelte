<script lang="ts">
	import TextField from '$lib/components/TextField.svelte';
	import TextArea from '$lib/components/TextArea.svelte';
	import Select from '$lib/components/Select.svelte';
	import { architecture, allCards, selectedCard } from '$lib/stores/architecture';
	import type { CardStatus, CardType } from '$lib/types';
	import { CardStatus as CardStatusEnum, CardType as CardTypeEnum, cardTypeLabel, cardStatusLabel } from '$lib/types';

	// Form state
	let cardType: CardType = CardTypeEnum.Requirement;
	let cardId = '';
	let cardName = '';
	let cardDescription = '';
	let cardStatus: CardStatus = CardStatusEnum.Proposed;
	let editingId: string | null = null;

	let isLoading = false;
	let errorMessage = '';

	const cardTypes = [
		{ value: CardTypeEnum.Driver, label: 'Driver' },
		{ value: CardTypeEnum.Requirement, label: 'Requirement' },
		{ value: CardTypeEnum.Behavior, label: 'Behavior' },
		{ value: CardTypeEnum.Interface, label: 'Interface' },
		{ value: CardTypeEnum.Constraint, label: 'Constraint' },
		{ value: CardTypeEnum.LogicalComponent, label: 'Logical Component' },
		{ value: CardTypeEnum.DeployableNode, label: 'Deployable Node' },
		{ value: CardTypeEnum.Actor, label: 'Actor' },
		{ value: CardTypeEnum.Test, label: 'Test' },
		{ value: CardTypeEnum.Artifact, label: 'Artifact' },
		{ value: CardTypeEnum.View, label: 'View' },
		{ value: CardTypeEnum.Note, label: 'Note' },
	];

	const statuses = [
		{ value: CardStatusEnum.Proposed, label: 'Proposed' },
		{ value: CardStatusEnum.Approved, label: 'Approved' },
		{ value: CardStatusEnum.Implemented, label: 'Implemented' },
		{ value: CardStatusEnum.Verified, label: 'Verified' },
		{ value: CardStatusEnum.Deprecated, label: 'Deprecated' },
		{ value: CardStatusEnum.Retired, label: 'Retired' },
	];

	function getStatusClass(status: CardStatus): string {
		return `status-${status}`;
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		isLoading = true;
		errorMessage = '';

		try {
			if (editingId) {
				// Update existing card
				await architecture.updateCard(editingId, cardName, cardDescription, cardStatus);
			} else {
				// Create new card
				if (!cardId.trim()) {
					throw new Error('Card ID is required');
				}
				await architecture.createCard(cardId, cardType, cardName, cardDescription);
			}
			resetForm();
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'An error occurred';
		} finally {
			isLoading = false;
		}
	}

	function resetForm() {
		cardType = CardTypeEnum.Requirement;
		cardId = '';
		cardName = '';
		cardDescription = '';
		cardStatus = CardStatusEnum.Proposed;
		editingId = null;
	}

	function editCard(cardId: string) {
		const card = $allCards.find((c) => c.id === cardId);
		if (card) {
			cardType = card.type;
			cardId = card.id;
			cardName = card.name;
			cardDescription = card.description || '';
			cardStatus = card.status || CardStatusEnum.Proposed;
			editingId = card.id;
		}
	}

	async function deleteCard(id: string) {
		if (!confirm(`Delete card "${id}"? This cannot be undone.`)) return;

		isLoading = true;
		errorMessage = '';
		try {
			await architecture.deleteCard(id);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Failed to delete card';
		} finally {
			isLoading = false;
		}
	}

	function selectCard(id: string) {
		architecture.selectCard(id);
	}

	$: isLoading = $architecture.loading;
	$: errorMessage = $architecture.error || '';
</script>

<div class="cards-container">
	<h2>Cards</h2>
	<p>Create and manage AURORA cards (drivers, requirements, behaviors, interfaces, constraints, and more).</p>

	{#if errorMessage}
		<div class="error-message">
			{errorMessage}
			<button class="close-btn" on:click={() => (errorMessage = '')}>&times;</button>
		</div>
	{/if}

	<div class="cards-layout">
		<section class="cards-form">
			<h3>{editingId ? 'Edit Card' : 'Create New Card'}</h3>
			<form on:submit={handleSubmit}>
				{#if !editingId}
					<Select
						label="Card Type"
						name="cardType"
						bind:value={cardType}
						options={cardTypes}
						required
						disabled={editingId !== null}
					/>
				{/if}

				<TextField
					label={editingId ? 'Card ID (read-only)' : 'Card ID'}
					name="cardId"
					placeholder="e.g., driver-automation"
					bind:value={cardId}
					required
					disabled={editingId !== null}
				/>

				<TextField label="Name" name="cardName" placeholder="Enter card name" bind:value={cardName} required />

				<TextArea
					label="Description"
					name="cardDescription"
					placeholder="Enter detailed description..."
					bind:value={cardDescription}
					rows={6}
				/>

				{#if editingId}
					<Select label="Status" name="cardStatus" bind:value={cardStatus} options={statuses} />
				{/if}

				<div class="form-buttons">
					<button type="submit" class="md-button md-button--primary" disabled={isLoading}>
						{isLoading ? '...'
						: editingId ? 'Save'
						: 'Create'}
					</button>
					<button
						type="button"
						class="md-button md-button--outlined"
						on:click={resetForm}
						disabled={isLoading}
					>
						{editingId ? 'Cancel' : 'Clear'}
					</button>
				</div>
			</form>
		</section>

		<section class="cards-list">
			<h3>Cards ({$allCards.length})</h3>
			{#if $allCards.length === 0}
				<div class="placeholder-message">
					<p>📭 No cards created yet. Use the form to create your first card.</p>
				</div>
			{:else}
				<div class="cards-grid">
					{#each $allCards as card (card.id)}
						<div
							class="card-item"
							class:selected={card.id === $selectedCard?.id}
							on:click={() => selectCard(card.id)}
							on:keydown={(e) => e.key === 'Enter' && selectCard(card.id)}
							role="button"
							tabindex="0"
						>
							<div class="card-header">
								<span class="card-type-badge">{cardTypeLabel(card.type)}</span>
								{#if card.status}
									<span class="card-status-badge {getStatusClass(card.status)}"
										>{cardStatusLabel(card.status)}</span
									>
								{/if}
							</div>
							<h4>{card.name}</h4>
							<p class="card-id">{card.id}</p>
							{#if card.description}
								<p class="card-description">{card.description.substring(0, 150)}...</p>
							{/if}
							<div class="card-footer">
								<small>v{card.version || '1.0.0'}</small>
								<div class="card-actions">
									<button
										class="action-btn edit-btn"
										on:click|stopPropagation={() => editCard(card.id)}
										title="Edit">✎</button
									>
									<button
										class="action-btn delete-btn"
										on:click|stopPropagation={() => deleteCard(card.id)}
										title="Delete"
										disabled={isLoading}>✕</button
									>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>
	</div>
</div>

<style>
	.cards-container {
		max-width: 1400px;
		margin: 0 auto;
	}

	h2 {
		font-size: 2rem;
		font-weight: 500;
		color: var(--md-sys-color-primary);
		margin-bottom: 0.5rem;
	}

	.cards-container > p {
		color: var(--md-sys-color-on-surface-variant);
		margin-bottom: 2rem;
	}

	h3 {
		font-size: 1.25rem;
		font-weight: 500;
		margin-bottom: 1.5rem;
		color: var(--md-sys-color-on-background);
	}

	.error-message {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem;
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		border-radius: 0.5rem;
		margin-bottom: 1.5rem;
	}

	.close-btn {
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		padding: 0;
		margin-left: 1rem;
		color: inherit;
	}

	.cards-layout {
		display: grid;
		grid-template-columns: 1fr 2fr;
		gap: 2rem;
	}

	.cards-form {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 0.5rem;
		padding: 1.5rem;
		height: fit-content;
		position: sticky;
		top: 1rem;
	}

	.cards-form form {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.form-buttons {
		display: flex;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.form-buttons button {
		flex: 1;
	}

	.cards-list {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 0.5rem;
		padding: 1.5rem;
	}

	.placeholder-message {
		text-align: center;
		padding: 2rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.cards-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 1rem;
	}

	.card-item {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0.5rem;
		padding: 1rem;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
	}

	.card-item:hover {
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.card-item.selected {
		background-color: var(--md-sys-color-primary-container);
		border-color: var(--md-sys-color-primary);
	}

	.card-header {
		display: flex;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
		flex-wrap: wrap;
	}

	.card-type-badge {
		background-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		padding: 0.25rem 0.5rem;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.card-status-badge {
		background-color: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		padding: 0.25rem 0.5rem;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.card-status-badge.status-proposed {
		background-color: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}

	.card-status-badge.status-approved {
		background-color: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}

	.card-status-badge.status-deprecated {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}

	.card-item h4 {
		font-size: 1rem;
		font-weight: 600;
		margin: 0.5rem 0;
		color: var(--md-sys-color-on-surface);
	}

	.card-id {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		margin: 0;
		font-family: monospace;
	}

	.card-description {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		margin: 0.5rem 0;
		line-height: 1.4;
	}

	.card-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: auto;
		padding-top: 0.5rem;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}

	.card-footer small {
		color: var(--md-sys-color-on-surface-variant);
	}

	.card-actions {
		display: flex;
		gap: 0.5rem;
	}

	.action-btn {
		background: none;
		border: none;
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		border-radius: 0.25rem;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1rem;
		transition: all 0.2s ease;
	}

	.action-btn:hover {
		background-color: var(--md-sys-color-surface-variant);
		color: var(--md-sys-color-on-surface);
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.delete-btn:hover {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}

	@media (max-width: 1024px) {
		.cards-layout {
			grid-template-columns: 1fr;
		}

		.cards-form {
			position: relative;
			top: auto;
		}

		.cards-grid {
			grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
		}
	}

	@media (max-width: 1024px) {
		.cards-layout {
			grid-template-columns: 1fr;
		}
	}

	.cards-form {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 2rem;
	}

	.cards-list {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 2rem;
	}

	form {
		display: flex;
		flex-direction: column;
	}

	button {
		margin-top: 1rem;
		margin-right: 0.5rem;
	}

	button:first-of-type {
		margin-top: 0;
	}

	.placeholder-message {
		text-align: center;
		padding: 2rem;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
