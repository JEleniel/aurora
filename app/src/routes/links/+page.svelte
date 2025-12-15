<script lang="ts">
	import TextField from '$lib/components/TextField.svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';

	let linkTarget = 'card';
	let sourceCardId = '';
	let targetCardId = '';
	let targetUrl = '';
	let linkTitle = '';

	const linkTargets = [
		{ value: 'card', label: 'Link to Card' },
		{ value: 'url', label: 'Link to External URL' },
	];

	function handleSubmit(e: Event) {
		e.preventDefault();
		console.log({
			linkTarget,
			sourceCardId,
			targetCardId,
			targetUrl,
			linkTitle,
		});
	}
</script>

<div class="links-container">
	<h2>Links</h2>
	<p>Create relationships between cards and external URLs for complete traceability.</p>

	<div class="links-layout">
		<section class="links-form">
			<h3>Create New Link</h3>
			<form on:submit={handleSubmit}>
				<TextField
					label="Source Card ID"
					name="sourceCardId"
					placeholder="e.g., requirement-automation"
					bind:value={sourceCardId}
					required
				/>

				<RadioGroup
					label="Target Type"
					name="linkTarget"
					bind:value={linkTarget}
					options={linkTargets}
					required
				/>

				{#if linkTarget === 'card'}
					<TextField
						label="Target Card ID"
						name="targetCardId"
						placeholder="e.g., driver-automation"
						bind:value={targetCardId}
						required
					/>
				{:else}
					<TextField
						label="Target URL"
						name="targetUrl"
						type="url"
						placeholder="https://example.com"
						bind:value={targetUrl}
						required
					/>

					<TextField
						label="Link Title (Optional)"
						name="linkTitle"
						placeholder="Display name for the link"
						bind:value={linkTitle}
					/>
				{/if}

				<button type="submit" class="md-button md-button--primary">Create Link</button>
				<button type="reset" class="md-button md-button--outlined">Clear</button>
			</form>
		</section>

		<section class="links-list">
			<h3>Existing Links</h3>
			<div class="placeholder-message">
				<p>🔗 No links created yet. Use the form to create your first link.</p>
			</div>
		</section>
	</div>
</div>

<style>
	.links-container {
		max-width: 1400px;
		margin: 0 auto;
	}

	h2 {
		font-size: 2rem;
		font-weight: 500;
		color: var(--md-sys-color-primary);
		margin-bottom: 0.5rem;
	}

	.links-container > p {
		color: var(--md-sys-color-on-surface-variant);
		margin-bottom: 2rem;
	}

	h3 {
		font-size: 1.25rem;
		font-weight: 500;
		margin-bottom: 1.5rem;
		color: var(--md-sys-color-on-background);
	}

	.links-layout {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2rem;
	}

	@media (max-width: 1024px) {
		.links-layout {
			grid-template-columns: 1fr;
		}
	}

	.links-form {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 2rem;
	}

	.links-list {
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 2rem;
	}

	form {
		display: flex;
		flex-direction: column;
	}

	button {
		margin-top: 1rem;
		margin-right: 0.5rem;
	}

	button:first-of-type {
		margin-top: 0;
	}

	.placeholder-message {
		text-align: center;
		padding: 2rem;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
