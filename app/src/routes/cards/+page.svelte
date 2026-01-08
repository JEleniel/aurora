<script lang="ts">
	import CardForm from '$lib/components/CardForm.svelte';
	import CardList from '$lib/components/CardList.svelte';
	import RelatedCards from '$lib/components/RelatedCards.svelte';
	import { architecture, allCards, allLinks, selectedCard } from '$lib/stores/architecture';
	import { searchCards } from '$lib/searchFilter';
	import { getRelatedCards, getOtherCards } from '$lib/utils/cardUtils';
	import { getTemplatesForType, applyTemplate } from '$lib/cardTemplates';
	import type { Card, CardStatus, CardType } from '$lib/types';
	import { CardType as CardTypeEnum } from '$lib/types';

	// Form state
	let editingCard: Card | null = $state(null);
	let errorMessage: string = $state('');
	let currentCardType: CardType = $state(CardTypeEnum.Requirement);

	// Search and filter state
	let searchText: string = $state('');
	let selectedCardTypes: CardType[] = $state([]);
	let selectedStatuses: CardStatus[] = $state([]);
	let selectedTags: string[] = $state([]);
	let filteredCards: Card[] = $state([]);
	let relatedCards: Card[] = $state([]);
	let allOtherCards: Card[] = $state([]);
	let showFilters: boolean = $state(false);

	// Drag state
	let draggedFromCard: string | null = $state(null);

	let isLoading = $derived($architecture.loading);
	$effect(() => {
		errorMessage = $architecture.error || '';
	});

	// Update filtered cards when search or filters change
	$effect(() => {
		filteredCards = searchCards($allCards, {
			searchText,
			cardTypes: selectedCardTypes.length > 0 ? selectedCardTypes : undefined,
			statuses: selectedStatuses.length > 0 ? selectedStatuses : undefined,
			tags: selectedTags.length > 0 ? selectedTags : undefined,
		});
	});

	// Update related cards when selection changes
	let relatedCards_temp = $derived(getRelatedCards($selectedCard?.id, $allCards, $allLinks));
	$effect(() => {
		relatedCards = relatedCards_temp;
	});

	let allOtherCards_temp = $derived(getOtherCards($selectedCard?.id, $allCards, relatedCards));
	$effect(() => {
		allOtherCards = allOtherCards_temp;
	});

	let currentTemplates = $derived(editingCard ? [] : getTemplatesForType(currentCardType));

	async function handleFormSubmit(detail: {
		editingId: string | null;
		cardId: string;
		cardType: CardType;
		cardName: string;
		cardDescription: string;
		cardStatus: CardStatus;
		linkToCardId?: string;
	}) {
		const { editingId, cardId, cardType, cardName, cardDescription, cardStatus, linkToCardId } = detail;

		try {
			if (editingId) {
				await architecture.updateCard(editingId, cardName, cardDescription, cardStatus);
			} else {
				await architecture.createCard(cardId, cardType, cardName, cardDescription);

				// If a link target was specified, create the link
				if (linkToCardId && linkToCardId.trim()) {
					await architecture.createLink(cardId, linkToCardId);
				}
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

	function handleApplyTemplate(templateName: string) {
		const templateData = applyTemplate(currentCardType, templateName);
		if (templateData.name) {
			// Update form with template data
			editingCard = null;
		}
	}
</script>

<div class="cards-container">
	<h2>Cards</h2>
	<p>Create and manage AURORA cards (drivers, requirements, behaviors, interfaces, constraints, and more).</p>

	{#if errorMessage}
		<div class="error-message" role="alert">
			{errorMessage}
			<button class="close-btn" onclick={() => (errorMessage = '')} aria-label="Dismiss error message"
				>&times;</button
			>
		</div>
	{/if}

	<div class="cards-layout">
		<CardForm
			{editingCard}
			{isLoading}
			availableCards={$allCards}
			allLinks={$allLinks}
			onSubmit={handleFormSubmit}
			onReset={handleFormReset}
		/>

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
			onDragstart={(id: string) => (draggedFromCard = id)}
			onDragend={() => (draggedFromCard = null)}
			onDrop={(id: string) => createLinkDragDrop(id)}
		/>
	</div>

	{#if !editingCard && currentTemplates.length > 0}
		<div class="templates-carousel">
			<div class="carousel-label">Quick Templates for {currentCardType}</div>
			<div class="carousel-container">
				{#each currentTemplates as template (template.name)}
					<button
						class="carousel-button"
						onclick={() => handleApplyTemplate(template.name)}
						title={template.description}
					>
						<span class="carousel-icon">{template.icon}</span>
						<span class="carousel-name">{template.name}</span>
					</button>
				{/each}
			</div>
		</div>
	{/if}
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

	.templates-carousel {
		margin-top: 1.5rem;
		padding: 1rem;
		background-color: var(--md-sys-color-surface);
		border-top: 2px solid var(--md-sys-color-outline-variant);
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
	}

	.carousel-label {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
		margin-bottom: 0.75rem;
		text-transform: capitalize;
	}

	.carousel-container {
		display: flex;
		gap: 0.5rem;
		overflow-x: auto;
		padding-bottom: 0.5rem;
	}

	.carousel-button {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.6rem 0.9rem;
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
		flex-shrink: 0;
		font-size: 0.9rem;
		color: var(--md-sys-color-on-surface);
		font-family: inherit;
	}

	.carousel-button:hover {
		background-color: var(--md-sys-color-secondary-container);
		border-color: var(--md-sys-color-primary);
		transform: translateY(-2px);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.carousel-button:active {
		transform: translateY(0);
	}

	.carousel-icon {
		font-size: 1.1rem;
	}

	.carousel-name {
		font-weight: 500;
	}

	@media (max-width: 1200px) {
		.cards-layout {
			grid-template-columns: 1fr;
			height: auto;
			gap: 1rem;
		}
	}

	@media (max-width: 768px) {
		.cards-layout {
			gap: 0.75rem;
		}

		.templates-carousel {
			margin-top: 1rem;
			padding: 0.75rem;
		}

		.carousel-container {
			gap: 0.4rem;
		}

		.carousel-button {
			padding: 0.5rem 0.7rem;
			font-size: 0.85rem;
		}

		.carousel-icon {
			font-size: 1rem;
		}

		.carousel-label {
			font-size: 0.8rem;
			margin-bottom: 0.5rem;
		}
	}
</style>
