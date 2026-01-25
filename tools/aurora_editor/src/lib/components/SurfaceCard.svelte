<script lang="ts">
	export let title = '';
	export let subtitle: string | undefined = undefined;
</script>

<section class="surface">
	<div class="surface__header">
		<div>
			<h2>{title}</h2>
			{#if subtitle}
				<p>{subtitle}</p>
			{/if}
		</div>
		<slot name="action" />
	</div>
	<div class="surface__body">
		<slot />
	</div>
</section>

<style>
	.surface {
		background: rgba(15, 23, 42, 0.7);
		border: 1px solid rgba(59, 130, 246, 0.2);
		border-radius: 1.5rem;
		padding: 1.5rem;
		box-shadow: 0 25px 60px rgba(2, 6, 23, 0.35);
		backdrop-filter: blur(18px);
	}

	.surface__header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1rem;
		margin-bottom: 1.25rem;
	}

	.surface__header h2 {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 600;
		color: #e2e8f0;
	}

	.surface__header p {
		margin: 0.35rem 0 0;
		color: #a5b4fc;
		font-size: 0.9rem;
	}

	.surface__body {
		display: grid;
		gap: 1rem;
	}
</style>
