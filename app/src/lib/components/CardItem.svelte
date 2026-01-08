<script lang="ts">
	import type { Card } from '$lib/types';
	import { cardTypeLabel, cardStatusLabel } from '$lib/types';
	import { getStatusClass } from '$lib/utils/cardUtils';
	import { getCardColor } from '$lib/utils/cardStyling';
	import CardIcon from '$lib/components/CardIcon.svelte';

	interface Props {
		card: Card;
		isSelected?: boolean;
		isDragOver?: boolean;
		onSelect?: (id: string) => void;
		onEdit?: (id: string) => void;
		onDelete?: (id: string) => void;
		onDragstart?: () => void;
		onDragend?: () => void;
		onDragover?: () => void;
		onDragleave?: () => void;
		onDrop?: () => void;
		small?: boolean;
	}

	const {
		card,
		isSelected = false,
		isDragOver = false,
		onSelect,
		onEdit,
		onDelete,
		onDragstart,
		onDragend,
		onDragover,
		onDragleave,
		onDrop,
		small = false,
	}: Props = $props();

	let isSmall: boolean = $derived(small);
	let cardColor: string = $derived(getCardColor(card.type));

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

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		onDragover?.();
	}

	function handleDragLeave() {
		onDragleave?.();
	}

	function handleDropEvent(event: DragEvent) {
		event.preventDefault();
		onDrop?.();
	}
</script>

<div
	class="card-item"
	class:selected={isSelected}
	class:drag-over={isDragOver}
	class:small={isSmall}
	class:sysml={card.type === 'mission' || card.type === 'requirement'}
	draggable="true"
	onclick={handleClick}
	onkeydown={(e) => e.key === 'Enter' && handleClick()}
	ondragstart={handleDragStart}
	ondragend={handleDragEnd}
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDropEvent}
	role="button"
	tabindex="0"
	title={card.description || ''}
>
	{#if !isSmall}
		<div class="card-buttons">
			<button
				class="action-btn edit-btn"
				onclick={handleEdit}
				title="Edit card"
				aria-label="Edit card"
				tabindex="-1"
			>
				✎
			</button>
			<button
				class="action-btn delete-btn"
				onclick={handleDelete}
				title="Delete card"
				aria-label="Delete card"
				tabindex="-1"
			>
				✕
			</button>
		</div>
	{/if}

	<div class="lcars-header" style="--card-color: {cardColor}">
		<div class="header-left">
			<div class="card-icon">
				<CardIcon type={card.type} size={20} />
			</div>
			<span class="card-type-label">{cardTypeLabel(card.type)}</span>
			<h4 class="card-title-header">{card.name}</h4>
		</div>
		{#if card.status && !isSmall}
			<span class="card-status-badge {getStatusClass(card.status)}">
				{cardStatusLabel(card.status)}
			</span>
		{/if}
	</div>

	<div class="card-content">
		<p class="card-id" class:card-id-sm={isSmall}>{card.id}</p>

		{#if !isSmall && card.description}
			<p class="card-description">{card.description.substring(0, 150)}...</p>
		{/if}

		{#if !isSmall}
			<div class="card-footer">
				<small>v{card.version || '1.0.0'}</small>
			</div>
		{/if}
	</div>
</div>

<style>
	.card-item {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 2rem 0.5rem 0.5rem 2rem;
		border-left: 2rem solid var(--card-color, var(--md-sys-color-primary));
		border-right: 0.5rem solid #ffffff;
		border-bottom: 0.25rem solid var(--card-color, var(--md-sys-color-primary));
		padding: 0.75rem 0.75rem 0.25rem 0.75rem;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		user-select: none;
		position: relative;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
	}

	.card-item.sysml {
		border-radius: 0.75rem;
		border-left: 2rem solid var(--card-color, var(--md-sys-color-primary));
		border-right: 0.5rem solid #ffffff;
		border-bottom: 0.25rem solid var(--card-color, var(--md-sys-color-primary));
		clip-path: polygon(1.5rem 0, 100% 0, 100% calc(100% - 0.75rem), calc(100% - 0.75rem) 100%, 0 100%, 0 0.75rem);
	}

	.card-buttons {
		position: absolute;
		left: 0.25rem;
		top: 0.25rem;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		z-index: 10;
	}

	.card-item.small {
		padding: 0.6rem;
		padding-left: 0.5rem;
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
		border-left-color: var(--md-sys-color-primary);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
		border-right-color: #f5f5f5;
	}

	.card-item.selected {
		background-color: var(--md-sys-color-primary-container);
		border-left-color: var(--md-sys-color-primary);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
	}

	.card-item.drag-over {
		background-color: color-mix(in srgb, var(--md-sys-color-primary) 10%, transparent);
		border-left: 2rem solid var(--md-sys-color-primary);
		box-shadow:
			inset 0 0 8px rgba(var(--md-sys-color-primary), 0.3),
			0 4px 12px rgba(0, 0, 0, 0.12);
	}

	.lcars-header {
		background: linear-gradient(
			90deg,
			var(--card-color, var(--md-sys-color-primary)) 0%,
			rgba(var(--card-color, var(--md-sys-color-primary)), 0.9) 100%
		);
		color: #ffffff;
		padding: 0.6rem 0.8rem;
		margin: -0.75rem -0.75rem 0.6rem -2rem;
		border-radius: 1.5rem 0 0 0;
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 0.5rem;
		font-family: 'Antonio', 'Noto Sans', sans-serif;
	}

	.header-left {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
		flex: 1;
		min-width: 0;
	}

	.card-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.2rem;
		height: 1.2rem;
		flex-shrink: 0;
		color: currentColor;
	}

	.card-type-label {
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		opacity: 0.9;
	}

	.card-title-header {
		font-size: 0.95rem;
		font-weight: 600;
		margin: 0;
		color: #ffffff;
		word-break: break-word;
		line-height: 1.2;
	}

	.card-status-badge {
		background-color: rgba(255, 255, 255, 0.2);
		color: #ffffff;
		padding: 0.2rem 0.4rem;
		border-radius: 0.2rem;
		font-size: 0.7rem;
		font-weight: 600;
	}

	.card-status-badge.status-proposed {
		background-color: rgba(255, 255, 255, 0.2);
		color: #ffffff;
	}

	.card-status-badge.status-approved {
		background-color: rgba(255, 255, 255, 0.2);
		color: #ffffff;
	}

	.card-status-badge.status-deprecated {
		background-color: rgba(255, 255, 255, 0.2);
		color: #ffffff;
	}

	.card-content {
		display: flex;
		flex-direction: column;
		flex: 1;
	}

	.card-id {
		display: none;
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
		margin: 0.4rem 0;
		line-height: 1.4;
	}

	.card-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: auto;
		padding-top: 0.4rem;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}

	.card-footer small {
		color: var(--md-sys-color-on-surface-variant);
	}

	.action-btn {
		background: rgba(255, 255, 255, 0.2);
		border: 1px solid rgba(255, 255, 255, 0.3);
		cursor: pointer;
		padding: 0.3rem 0.4rem;
		border-radius: 0.3rem;
		color: #ffffff;
		font-size: 0.9rem;
		transition: all 0.2s ease;
		min-width: 2rem;
		min-height: 1.6rem;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.action-btn:hover {
		background: rgba(255, 255, 255, 0.35);
		border-color: rgba(255, 255, 255, 0.5);
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.delete-btn:hover {
		background: #d32f2f;
		color: #ffffff;
		border-color: #d32f2f;
	}
</style>
