<script lang="ts">
	import CardItem from './CardItem.svelte';
	import type { Card } from '$lib/types';

	export let filteredCards: Card[] = [];
	export let allCards: Card[] = [];
	export let selectedCard: Card | null = null;
	export let searchText = '';
	export let onSelect: ((id: string) => void) | undefined = undefined;
	export let onEdit: ((id: string) => void) | undefined = undefined;
	export let onDelete: ((id: string) => void) | undefined = undefined;
	export let onToggleFilters: (() => void) | undefined = undefined;
	export let onDragstart: ((id: string) => void) | undefined = undefined;
	export let onDragend: (() => void) | undefined = undefined;
	export let onDragover: (() => void) | undefined = undefined;
	export let onDragleave: (() => void) | undefined = undefined;
	export let onDrop: ((id: string) => void) | undefined = undefined;
</script>

<section class="cards-list">
	<div class="cards-list-header">
		<h3>Cards ({filteredCards.length}/{allCards.length})</h3>
		<button type="button" class="md-button md-button--text" on:click={() => onToggleFilters?.()}>
			▼ Filters
		</button>
	</div>

	<div class="search-filters">
		<input
			type="text"
			placeholder="Search by name, description, or ID..."
			bind:value={searchText}
			class="search-input"
		/>
	</div>

	{#if filteredCards.length === 0}
		<div class="placeholder-message">
			{#if allCards.length === 0}
				<p>📭 No cards created yet. Use the form to create your first card.</p>
			{:else}
				<p>🔍 No cards match your search or filters.</p>
			{/if}
		</div>
	{:else}
		<div class="cards-grid">
			{#each filteredCards as card (card.id)}
				<CardItem
					{card}
					isSelected={card.id === selectedCard?.id}
					isDragOver={false}
					{onSelect}
					{onEdit}
					{onDelete}
					onDragstart={() => onDragstart?.(card.id)}
					{onDragend}
					{onDragover}
					{onDragleave}
					onDrop={() => onDrop?.(card.id)}
				/>
			{/each}
		</div>
	{/if}
</section>

<style>
	.cards-list {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 12px;
		padding: 2rem;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.cards-list-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
		width: 100%;
		max-width: 600px;
	}

	.cards-list-header h3 {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 500;
		color: var(--md-sys-color-on-background);
	}

	.search-filters {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0.5rem;
		padding: 1rem;
		margin-bottom: 1rem;
		width: 100%;
		max-width: 600px;
	}

	.search-input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: 0.5rem;
		font-family: inherit;
		font-size: 1rem;
		color: var(--md-sys-color-on-surface);
		background-color: var(--md-sys-color-surface);
	}

	.search-input:focus {
		outline: none;
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 0 0 3px var(--md-sys-color-primary-container);
	}

	.placeholder-message {
		text-align: center;
		padding: 2rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.cards-grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		width: 100%;
		max-width: 600px;
	}
</style>
