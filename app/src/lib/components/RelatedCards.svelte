<script lang="ts">
	import CardItem from './CardItem.svelte';
	import type { Card } from '$lib/types';

	export let selectedCard: Card | null = null;
	export let relatedCards: Card[] = [];
	export let otherCards: Card[] = [];
	export let onSelect: ((id: string) => void) | undefined = undefined;
	export let onDragstart: ((id: string) => void) | undefined = undefined;
	export let onDragend: (() => void) | undefined = undefined;
	export let onDragover: (() => void) | undefined = undefined;
	export let onDragleave: (() => void) | undefined = undefined;
	export let onDrop: ((id: string) => void) | undefined = undefined;
</script>

<aside class="cards-related">
	{#if selectedCard}
		<h3>Related Cards</h3>
		{#if relatedCards.length === 0}
			<p class="placeholder-related">No related cards</p>
		{:else}
			<div class="related-cards-list">
				{#each relatedCards as card (card.id)}
					<CardItem
						{card}
						isSelected={false}
						isDragOver={false}
						small
						onSelect={(id) => onSelect?.(id)}
						onDragstart={() => onDragstart?.(card.id)}
						{onDragend}
						{onDragover}
						{onDragleave}
						onDrop={() => onDrop?.(card.id)}
					/>
				{/each}
			</div>
		{/if}

		<div class="other-cards-section">
			<h4>Other Cards</h4>
			{#if otherCards.length === 0}
				<p class="placeholder-related">No other cards</p>
			{:else}
				<div class="other-cards-list">
					{#each otherCards.slice(0, 5) as card (card.id)}
						<CardItem
							{card}
							isSelected={false}
							isDragOver={false}
							small
							onSelect={(id) => onSelect?.(id)}
							onDragstart={() => onDragstart?.(card.id)}
							{onDragend}
							{onDragover}
							{onDragleave}
							onDrop={() => onDrop?.(card.id)}
						/>
					{/each}
					{#if otherCards.length > 5}
						<p class="placeholder-related" style="font-size: 0.85rem;">
							+{otherCards.length - 5} more cards
						</p>
					{/if}
				</div>
			{/if}
		</div>
	{:else}
		<p class="placeholder-related">Select a card to see related cards</p>
	{/if}
</aside>

<style>
	.cards-related {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 12px;
		padding: 1.5rem;
		overflow-y: auto;
		max-height: calc(100vh - 200px);
	}

	.cards-related h3 {
		font-size: 1rem;
		font-weight: 600;
		margin: 0 0 1rem 0;
		color: var(--md-sys-color-on-surface);
	}

	.cards-related h4 {
		font-size: 0.9rem;
		font-weight: 600;
		margin: 0 0 0.5rem 0;
		color: var(--md-sys-color-on-surface);
	}

	.other-cards-section {
		margin-top: 1.5rem;
		padding-top: 1.5rem;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}

	.related-cards-list,
	.other-cards-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.placeholder-related {
		font-size: 0.85rem;
		color: var(--md-sys-color-on-surface-variant);
		text-align: center;
		padding: 0.75rem 0;
		margin: 0;
	}
</style>
