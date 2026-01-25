<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let label: string;
	export let description: string;
	export let disabled = false;

	const dispatch = createEventDispatcher<{ click: MouseEvent }>();

	function handleClick(event: MouseEvent): void {
		if (disabled) {
			return;
		}
		dispatch('click', event);
	}
</script>

<button class="action" type="button" {disabled} aria-label={label} on:click={handleClick}>
	<div class="action__label">{label}</div>
	<div class="action__desc">{description}</div>
</button>

<style>
	.action {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		text-align: left;
		padding: 0.85rem 1rem;
		border-radius: 1rem;
		border: 1px solid rgba(99, 102, 241, 0.3);
		background: rgba(15, 23, 42, 0.7);
		color: #e2e8f0;
		cursor: pointer;
		transition:
			border 0.2s ease,
			box-shadow 0.2s ease,
			transform 0.2s ease;
	}

	.action:hover:not(:disabled) {
		border-color: rgba(129, 140, 248, 0.7);
		box-shadow: 0 12px 30px rgba(59, 130, 246, 0.25);
		transform: translateY(-1px);
	}

	.action:focus-visible {
		outline: 2px solid rgba(129, 140, 248, 0.8);
		outline-offset: 2px;
	}

	.action:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.action__label {
		font-weight: 600;
	}

	.action__desc {
		font-size: 0.85rem;
		color: #c7d2fe;
	}
</style>
