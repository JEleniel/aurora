<script lang="ts">
	import CardForm from '$lib/components/CardForm.svelte';
	import CardList from '$lib/components/CardList.svelte';
	import RelatedCards from '$lib/components/RelatedCards.svelte';
	import { architecture, allCards, allLinks, selectedCard } from '$lib/stores/architecture';
	import { searchCards } from '$lib/searchFilter';
	import { getRelatedCards, getOtherCards } from '$lib/utils/cardUtils';
	import type { Card, CardStatus, CardType } from '$lib/types';

	// Form state
	let editingCard: Card | null = null;
	let errorMessage = '';

	// Search and filter state
	let searchText = '';
	let selectedCardTypes: CardType[] = [];
	let selectedStatuses: CardStatus[] = [];
	let selectedTags: string[] = [];
	let filteredCards: Card[] = [];
	let relatedCards: Card[] = [];
	let allOtherCards: Card[] = [];
	let showFilters = false;

	// Drag state
	let draggedFromCard: string | null = null;

	$: isLoading = $architecture.loading;
	$: errorMessage = $architecture.error || '';

	// Update filtered cards when search or filters change
	$: filteredCards = searchCards($allCards, {
		searchText,
		cardTypes: selectedCardTypes.length > 0 ? selectedCardTypes : undefined,
		statuses: selectedStatuses.length > 0 ? selectedStatuses : undefined,
		tags: selectedTags.length > 0 ? selectedTags : undefined,
	});

	// Update related cards when selection changes
	$: relatedCards = getRelatedCards($selectedCard?.id, $allCards, $allLinks);
	$: allOtherCards = getOtherCards($selectedCard?.id, $allCards, relatedCards);

	async function handleFormSubmit(detail: {
		editingId: string | null;
		cardId: string;
		cardType: CardType;
		cardName: string;
		cardDescription: string;
		cardStatus: CardStatus;
	}) {
		const { editingId, cardId, cardType, cardName, cardDescription, cardStatus } = detail;

		try {
			if (editingId) {
				await architecture.updateCard(editingId, cardName, cardDescription, cardStatus);
			} else {
				await architecture.createCard(cardId, cardType, cardName, cardDescription);
			}
			editingCard = null;
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'An error occurred';
		}
	}

	function handleFormReset() {
		editingCard = null;
	}

	function handleSelectCard(id: string) {
		architecture.selectCard(id);
	}

	async function handleEditCard(id: string) {
		const card = $allCards.find((c) => c.id === id);
		if (card) {
			editingCard = card;
		}
	}

	async function handleDeleteCard(id: string) {
		if (!confirm(`Delete card "${id}"? This cannot be undone.`)) return;

		try {
			await architecture.deleteCard(id);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Failed to delete card';
		}
	}

	async function createLinkDragDrop(targetId: string) {
		if (!draggedFromCard || draggedFromCard === targetId) return;

		try {
			await architecture.createLink(draggedFromCard, targetId);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Failed to create link';
		} finally {
			draggedFromCard = null;
		}
	}
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
		<CardForm {editingCard} {isLoading} onSubmit={handleFormSubmit} onReset={handleFormReset} />

		<CardList
			{filteredCards}
			allCards={$allCards}
			selectedCard={$selectedCard}
			bind:searchText
			onSelect={handleSelectCard}
			onEdit={handleEditCard}
			onDelete={handleDeleteCard}
			onToggleFilters={() => (showFilters = !showFilters)}
		/>

		<RelatedCards
			selectedCard={$selectedCard}
			{relatedCards}
			otherCards={allOtherCards}
			onSelect={handleSelectCard}
			onDragstart={(id) => (draggedFromCard = id)}
			onDragend={() => (draggedFromCard = null)}
			onDrop={(id) => createLinkDragDrop(id)}
		/>
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
		grid-template-columns: 320px 1fr 280px;
		gap: 1.5rem;
		height: calc(100vh - 250px);
	}

	@media (max-width: 1200px) {
		.cards-layout {
			grid-template-columns: 1fr;
			height: auto;
		}
	}
</style>
