<script lang="ts">
	// @ts-ignore - Card type used for documentation purposes
	import type { Card } from '$lib/types';

	let { label = '', cards = [], value = $bindable(''), name = '', required = false, disabled = false } = $props();

	// Display selected card
	let selectedCard = $derived(cards.find((c) => c.id === value));
</script>

<div class="card-select-wrapper">
	{#if label}
		<label for={name}>{label}{required ? ' *' : ''}</label>
	{/if}
	<select {name} bind:value {required} {disabled} id={name} class="card-select">
		<option value="">-- Select a card --</option>
		{#each cards as card (card.id)}
			<option value={card.id}>
				{card.name}
			</option>
		{/each}
	</select>
	{#if selectedCard}
		<div class="selected-info">
			<span class="card-type">{selectedCard.type}</span>
			<span class="card-id">{selectedCard.id}</span>
		</div>
	{/if}
</div>

<style>
	.card-select-wrapper {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--md-sys-color-on-surface);
	}

	.card-select {
		padding: 0.75rem 0.875rem;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0.375rem;
		background-color: var(--md-sys-color-surface-variant);
		color: var(--md-sys-color-on-surface);
		font-family: 'Noto Sans', sans-serif;
		font-size: 0.875rem;
		transition: all 0.2s ease;
		cursor: pointer;
	}

	.card-select:hover {
		border-color: var(--md-sys-color-outline);
		background-color: color-mix(in srgb, var(--md-sys-color-surface-variant) 80%, var(--md-sys-color-primary) 20%);
	}

	.card-select:focus {
		outline: none;
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 20%, transparent);
	}

	.card-select:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.selected-info {
		display: flex;
		gap: 0.5rem;
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.card-type {
		background-color: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		padding: 0.25rem 0.5rem;
		border-radius: 3px;
		font-weight: 600;
		text-transform: uppercase;
	}

	.card-id {
		background-color: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		padding: 0.25rem 0.5rem;
		border-radius: 3px;
		font-family: 'Noto Sans Mono', monospace;
	}
</style>
