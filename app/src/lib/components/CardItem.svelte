<script lang="ts">
	import type { Card } from '$lib/types';
	import { cardTypeLabel, cardStatusLabel } from '$lib/types';
	import { getStatusClass } from '$lib/utils/cardUtils';

	export let card: Card;
	export let isSelected = false;
	export let isDragOver = false;
	export let onSelect: ((id: string) => void) | undefined = undefined;
	export let onEdit: ((id: string) => void) | undefined = undefined;
	export let onDelete: ((id: string) => void) | undefined = undefined;
	export let onDragstart: (() => void) | undefined = undefined;
	export let onDragend: (() => void) | undefined = undefined;
	export let onDragover: (() => void) | undefined = undefined;
	export let onDragleave: (() => void) | undefined = undefined;
	export let onDrop: (() => void) | undefined = undefined;

	let isSmall = false; // Set to true when used in sidebar

	export { isSmall as small };

	function handleClick() {
		onSelect?.(card.id);
	}

	function handleEdit(e: Event) {
		e.stopPropagation();
		onEdit?.(card.id);
	}

	function handleDelete(e: Event) {
		e.stopPropagation();
		onDelete?.(card.id);
	}

	function handleDragStart() {
		onDragstart?.();
	}

	function handleDragEnd() {
		onDragend?.();
	}

	function handleDragOver() {
		onDragover?.();
	}

	function handleDragLeave() {
		onDragleave?.();
	}

	function handleDropEvent() {
		onDrop?.();
	}
</script>

<div
	class="card-item"
	class:selected={isSelected}
	class:drag-over={isDragOver}
	class:small={isSmall}
	draggable="true"
	on:click={handleClick}
	on:keydown={(e) => e.key === 'Enter' && handleClick()}
	on:dragstart={handleDragStart}
	on:dragend={handleDragEnd}
	on:dragover|preventDefault={handleDragOver}
	on:dragleave={handleDragLeave}
	on:drop|preventDefault={handleDropEvent}
	role="button"
	tabindex="0"
	title={card.description || ''}
>
	<div class="card-header" class:card-header-sm={isSmall}>
		<span class="card-type-badge" class:card-type-badge-sm={isSmall}>
			{cardTypeLabel(card.type)}
		</span>
		{#if card.status && !isSmall}
			<span class="card-status-badge {getStatusClass(card.status)}">
				{cardStatusLabel(card.status)}
			</span>
		{/if}
	</div>

	{#if isSmall}
		<h5 class="card-title">{card.name}</h5>
	{:else}
		<h4 class="card-title">{card.name}</h4>
	{/if}
	<p class="card-id" class:card-id-sm={isSmall}>{card.id}</p>

	{#if !isSmall && card.description}
		<p class="card-description">{card.description.substring(0, 150)}...</p>
	{/if}

	{#if !isSmall}
		<div class="card-footer">
			<small>v{card.version || '1.0.0'}</small>
			<div class="card-actions">
				<button class="action-btn edit-btn" on:click={handleEdit} title="Edit" tabindex="-1"> ✎ </button>
				<button class="action-btn delete-btn" on:click={handleDelete} title="Delete" tabindex="-1"> ✕ </button>
			</div>
		</div>
	{/if}
</div>

<style>
	.card-item {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0.5rem;
		padding: 1rem;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		user-select: none;
	}

	.card-item.small {
		padding: 0.75rem;
		cursor: grab;
	}

	.card-item[draggable='true'] {
		cursor: grab;
	}

	.card-item[draggable='true']:active {
		cursor: grabbing;
		opacity: 0.7;
	}

	.card-item:hover {
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.card-item.selected {
		background-color: var(--md-sys-color-primary-container);
		border-color: var(--md-sys-color-primary);
	}

	.card-item.drag-over {
		background-color: color-mix(in srgb, var(--md-sys-color-primary) 10%, transparent);
		border: 2px dashed var(--md-sys-color-primary);
		box-shadow: inset 0 0 8px rgba(var(--md-sys-color-primary), 0.3);
	}

	.card-header {
		display: flex;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
		flex-wrap: wrap;
	}

	.card-header-sm {
		margin-bottom: 0.25rem;
	}

	.card-type-badge {
		background-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		padding: 0.25rem 0.5rem;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.card-type-badge-sm {
		padding: 0.15rem 0.35rem;
		border-radius: 0.2rem;
		font-size: 0.65rem;
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

	.card-title {
		font-size: 1rem;
		font-weight: 600;
		margin: 0.5rem 0;
		color: var(--md-sys-color-on-surface);
	}

	.card-item.small .card-title {
		font-size: 0.85rem;
		margin: 0.25rem 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.card-id {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		margin: 0;
		font-family: monospace;
	}

	.card-id-sm {
		font-size: 0.7rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
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
</style>
