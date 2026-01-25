<script lang="ts">
	export let tone: 'good' | 'warn' | 'error' | 'neutral' = 'neutral';
</script>

<span class={`pill pill--${tone}`}>
	<slot />
</span>

<style>
	.pill {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.25rem 0.75rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		border: 1px solid transparent;
	}

	.pill--neutral {
		background: rgba(30, 64, 175, 0.25);
		border-color: rgba(99, 102, 241, 0.35);
		color: #c7d2fe;
	}

	.pill--good {
		background: rgba(34, 197, 94, 0.18);
		border-color: rgba(34, 197, 94, 0.4);
		color: #bbf7d0;
	}

	.pill--warn {
		background: rgba(245, 158, 11, 0.2);
		border-color: rgba(245, 158, 11, 0.4);
		color: #fde68a;
	}

	.pill--error {
		background: rgba(248, 113, 113, 0.2);
		border-color: rgba(248, 113, 113, 0.45);
		color: #fecaca;
	}
</style>
