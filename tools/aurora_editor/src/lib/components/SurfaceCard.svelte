<script lang="ts">
	export let title = '';
	export let subtitle: string | undefined = undefined;
</script>

<section class="surface">
	<div class="surface__header">
		<h2>{title}</h2>
		{#if subtitle}
			<p>{subtitle}</p>
		{/if}
		<slot name="action" />
	</div>
	<div class="surface__body">
		<slot />
	</div>
</section>
