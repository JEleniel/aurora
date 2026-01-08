<script lang="ts">
	import { onMount } from 'svelte';
	import TextField from '$lib/components/TextField.svelte';
	import CardSelect from '$lib/components/CardSelect.svelte';
	import RadioGroup from '$lib/components/RadioGroup.svelte';
	import LinkEditModal from '$lib/components/LinkEditModal.svelte';
	import { architecture } from '$lib/stores/architecture';
	import type { Link } from '$lib/types';

	let linkTarget: string = $state('card');
	let sourceCardId: string = $state('');
	let targetCardId: string = $state('');
	let targetUrl: string = $state('');
	let linkTitle: string = $state('');
	let error: string = $state('');
	let successMessage: string = $state('');
	let isLoading: boolean = $state(false);

	let editingLink: Link | null = $state(null);
	let showEditModal: boolean = $state(false);

	let allCards = $derived($architecture.cards);
	let allCardsArray = $derived(Array.from(allCards.values()));
	let allLinks = $derived($architecture.links);
	let loading = $derived($architecture.loading);

	const linkTargets = [
		{ value: 'card', label: 'Link to Card' },
		{ value: 'url', label: 'Link to External URL' },
	];

	onMount(async () => {
		if (allCards.size === 0 && !loading) {
			await architecture.refresh();
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		successMessage = '';
		isLoading = true;

		try {
			if (!sourceCardId.trim()) {
				throw new Error('Source card ID is required');
			}

			if (linkTarget === 'card') {
				if (!targetCardId.trim()) {
					throw new Error('Target card ID is required');
				}
				// Verify both cards exist
				if (!allCards.has(sourceCardId)) {
					throw new Error(`Source card "${sourceCardId}" not found`);
				}
				if (!allCards.has(targetCardId)) {
					throw new Error(`Target card "${targetCardId}" not found`);
				}
				if (sourceCardId === targetCardId) {
					throw new Error('Source and target cards cannot be the same');
				}
			} else {
				if (!targetUrl.trim()) {
					throw new Error('Target URL is required');
				}
				// Basic URL validation
				try {
					new URL(targetUrl);
				} catch {
					throw new Error('Invalid URL format');
				}
			}

			// Create the link
			if (linkTarget === 'card') {
				await architecture.createLink(sourceCardId, targetCardId);
			} else {
				await architecture.createLink(sourceCardId, undefined, targetUrl);
			}

			successMessage = 'Link created successfully';
			sourceCardId = '';
			targetCardId = '';
			targetUrl = '';
			linkTitle = '';

			// Clear success message after 3 seconds
			setTimeout(() => {
				successMessage = '';
			}, 3000);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to create link';
		} finally {
			isLoading = false;
		}
	}

	function getSourceCard(link: Link) {
		return allCards.get(link.source_id);
	}

	function getTargetCard(link: Link) {
		return link.target_id ? allCards.get(link.target_id) : null;
	}

	function openEditModal(link: Link) {
		editingLink = link;
		showEditModal = true;
	}

	function closeEditModal() {
		editingLink = null;
		showEditModal = false;
	}
</script>

<div class="links-container">
	<h2>Links</h2>
	<p>Create relationships between cards and external URLs for complete traceability.</p>

	{#if error}
		<div class="error-message" role="alert">
			<p><strong>Error:</strong> {error}</p>
			<button onclick={() => (error = '')} class="dismiss-btn" aria-label="Dismiss error message">✕</button>
		</div>
	{/if}

	{#if successMessage}
		<div class="success-message" role="status" aria-live="polite" aria-atomic="true">
			<p>{successMessage}</p>
		</div>
	{/if}

	<div class="links-layout">
		<section class="links-form" class:disabled={isLoading || loading}>
			<h3>Create New Link</h3>
			<form onsubmit={handleSubmit}>
				<CardSelect
					label="Source Card"
					name="sourceCardId"
					cards={allCardsArray}
					bind:value={sourceCardId}
					required
					disabled={isLoading || loading}
				/>

				<RadioGroup
					label="Target Type"
					name="linkTarget"
					bind:value={linkTarget}
					options={linkTargets}
					required
				/>

				{#if linkTarget === 'card'}
					<CardSelect
						label="Target Card"
						name="targetCardId"
						cards={allCardsArray}
						bind:value={targetCardId}
						required
						disabled={isLoading || loading}
					/>
				{:else}
					<TextField
						label="Target URL"
						name="targetUrl"
						type="url"
						placeholder="https://example.com"
						bind:value={targetUrl}
						required
						disabled={isLoading || loading}
					/>

					<TextField
						label="Link Title (Optional)"
						name="linkTitle"
						placeholder="Display name for the link"
						bind:value={linkTitle}
						disabled={isLoading || loading}
					/>
				{/if}

				<button type="submit" class="md-button md-button--primary" disabled={isLoading || loading}>
					{isLoading || loading ? 'Creating...' : 'Create Link'}
				</button>
				<button type="reset" class="md-button md-button--outlined" disabled={isLoading || loading}>
					Clear
				</button>
			</form>
		</section>

		<section class="links-list">
			<h3>Existing Links ({allLinks.length})</h3>
			{#if allLinks.length === 0}
				<div class="placeholder-message">
					<p>🔗 No links created yet. Use the form to create your first link.</p>
				</div>
			{:else}
				<div class="links-grid">
					{#each allLinks as link (link.source_id + link.target_id + link.target_url)}
						{@const sourceCard = getSourceCard(link)}
						{@const targetCard = getTargetCard(link)}
						<div class="link-item">
							<div class="link-header">
								<div class="link-source">
									<div class="link-label">From</div>
									<div class="link-value">{sourceCard?.name || link.source_id}</div>
									<div class="link-id">{link.source_id}</div>
								</div>
								<div class="link-arrow">→</div>
								<div class="link-target">
									<div class="link-label">To</div>
									{#if targetCard}
										<div class="link-value">{targetCard.name}</div>
										<div class="link-id">{link.target_id}</div>
									{:else if link.target_url}
										<div class="link-value link-url">{link.target_url}</div>
										{#if link.metadata?.link_title}
											<div class="link-title">{link.metadata.link_title}</div>
										{/if}
									{:else}
										<div class="link-value">Unknown</div>
									{/if}
								</div>
							</div>
							<div class="link-footer">
								<span class="link-date">Created {new Date(link.created_at).toLocaleDateString()}</span>
								<div class="link-actions">
									<button
										class="edit-btn"
										onclick={() => openEditModal(link)}
										disabled={loading}
										title="Edit link metadata"
									>
										✎
									</button>
									<button
										class="delete-btn"
										onclick={() => {
											if (confirm('Are you sure you want to delete this link?')) {
												architecture.deleteLink(
													link.source_id,
													link.target_id || undefined,
													link.target_url || undefined,
												);
											}
										}}
										disabled={loading}
										title="Delete link"
									>
										🗑️
									</button>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>
	</div>

	<LinkEditModal isOpen={showEditModal} link={editingLink} onClose={closeEditModal} />
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

	.error-message {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		border: 1px solid var(--md-sys-color-error);
		border-radius: 8px;
		padding: 1rem;
		margin-bottom: 1.5rem;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.error-message p {
		margin: 0;
		font-size: 0.95rem;
	}

	.dismiss-btn {
		background: none;
		border: none;
		color: inherit;
		cursor: pointer;
		font-size: 1.2rem;
		padding: 0;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.success-message {
		background-color: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		border: 1px solid var(--md-sys-color-tertiary);
		border-radius: 8px;
		padding: 1rem;
		margin-bottom: 1.5rem;
		animation: slideDown 0.3s ease-out;
	}

	@keyframes slideDown {
		from {
			opacity: 0;
			transform: translateY(-10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
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
		height: fit-content;
		position: sticky;
		top: 20px;
	}

	.links-form.disabled {
		opacity: 0.6;
		pointer-events: none;
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
		padding: 0.75rem 1.5rem;
		border-radius: 6px;
		font-weight: 600;
		font-size: 0.95rem;
		border: none;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	button:first-of-type {
		margin-top: 0;
	}

	.md-button {
		display: inline-block;
		padding: 0.75rem 1.5rem;
		border-radius: 6px;
		font-weight: 600;
		border: none;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.md-button--primary {
		background-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}

	.md-button--primary:hover:not(:disabled) {
		background-color: var(--md-sys-color-primary-dark);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
	}

	.md-button--primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.md-button--outlined {
		background-color: transparent;
		color: var(--md-sys-color-primary);
		border: 1px solid var(--md-sys-color-outline);
	}

	.md-button--outlined:hover:not(:disabled) {
		background-color: var(--md-sys-color-primary-light);
	}

	.md-button--outlined:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.placeholder-message {
		text-align: center;
		padding: 3rem 2rem;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 1.1rem;
	}

	.links-grid {
		display: grid;
		grid-template-columns: 1fr;
		gap: 1rem;
	}

	.link-item {
		background-color: var(--md-sys-color-surface-variant);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		padding: 1.5rem;
		transition: all 0.2s ease;
	}

	.link-item:hover {
		background-color: var(--md-sys-color-surface);
		border-color: var(--md-sys-color-outline);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.link-header {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		gap: 1rem;
		align-items: center;
		margin-bottom: 1rem;
	}

	.link-source,
	.link-target {
		display: flex;
		flex-direction: column;
	}

	.link-label {
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface-variant);
		margin-bottom: 0.25rem;
	}

	.link-value {
		font-size: 1rem;
		font-weight: 600;
		color: var(--md-sys-color-on-background);
		word-break: break-word;
	}

	.link-url {
		font-size: 0.9rem;
		color: var(--md-sys-color-primary);
		font-family: monospace;
	}

	.link-id {
		font-size: 0.8rem;
		color: var(--md-sys-color-on-surface-variant);
		font-family: monospace;
		margin-top: 0.25rem;
	}

	.link-title {
		font-size: 0.8rem;
		color: var(--md-sys-color-on-surface-variant);
		font-style: italic;
		margin-top: 0.25rem;
	}

	.link-arrow {
		font-size: 1.5rem;
		color: var(--md-sys-color-primary);
		text-align: center;
	}

	.link-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-top: 1rem;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		font-size: 0.85rem;
	}

	.link-date {
		color: var(--md-sys-color-on-surface-variant);
	}

	.delete-btn {
		background: none;
		border: none;
		font-size: 1.1rem;
		cursor: pointer;
		padding: 0.5rem;
		border-radius: 4px;
		transition: all 0.2s ease;
		color: var(--md-sys-color-error);
	}

	.delete-btn:hover:not(:disabled) {
		background-color: var(--md-sys-color-error-container);
	}

	.delete-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.link-actions {
		display: flex;
		gap: 0.5rem;
	}

	.edit-btn {
		background: none;
		border: none;
		font-size: 1.1rem;
		cursor: pointer;
		padding: 0.5rem;
		border-radius: 4px;
		transition: all 0.2s ease;
		color: var(--md-sys-color-primary);
	}

	.edit-btn:hover:not(:disabled) {
		background-color: var(--md-sys-color-primary-container);
	}

	.edit-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
