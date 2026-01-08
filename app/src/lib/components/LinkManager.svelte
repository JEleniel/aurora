<script lang="ts">
	import CardSelect from './CardSelect.svelte';
	import TextField from './TextField.svelte';
	import RadioGroup from './RadioGroup.svelte';
	import type { Card, Link } from '$lib/types';
	import { architecture } from '$lib/stores/architecture';

	interface Props {
		cardId?: string;
		allCards?: Card[];
		allLinks?: Link[];
		isLoading?: boolean;
	}

	const { cardId = '', allCards = [], allLinks = [], isLoading = false }: Props = $props();

	let linkTarget: string = $state('card');
	let targetCardId: string = $state('');
	let targetUrl: string = $state('');
	let error: string = $state('');
	let successMessage: string = $state('');
	let isSubmitting: boolean = $state(false);

	let outgoingLinks: Link[] = $state([]);
	let incomingLinks: Link[] = $state([]);

	const linkTargets = [
		{ value: 'card', label: 'Link to Card' },
		{ value: 'url', label: 'Link to External URL' },
	];

	// Update links whenever cardId or allLinks changes
	$effect(() => {
		if (cardId && allCards.length > 0) {
			outgoingLinks = allLinks.filter((l) => l.source_id === cardId);
			incomingLinks = allLinks.filter((l) => l.target_id === cardId);
		}
	});

	function getCardName(cardId: string): string {
		return allCards.find((c) => c.id === cardId)?.name || cardId;
	}

	function getCardType(cardId: string): string {
		return allCards.find((c) => c.id === cardId)?.type || '';
	}

	async function handleAddLink() {
		error = '';
		successMessage = '';
		isSubmitting = true;

		try {
			if (!cardId.trim()) {
				throw new Error('Card not available');
			}

			if (linkTarget === 'card') {
				if (!targetCardId.trim()) {
					throw new Error('Target card is required');
				}
				if (!allCards.some((c) => c.id === targetCardId)) {
					throw new Error(`Target card not found`);
				}
				if (cardId === targetCardId) {
					throw new Error('Cannot link a card to itself');
				}

				await architecture.createLink(cardId, targetCardId);
			} else {
				if (!targetUrl.trim()) {
					throw new Error('URL is required');
				}
				try {
					new URL(targetUrl);
				} catch {
					throw new Error('Invalid URL format');
				}

				await architecture.createLink(cardId, undefined, targetUrl);
			}

			successMessage = 'Link added successfully';
			targetCardId = '';
			targetUrl = '';
			linkTarget = 'card';

			setTimeout(() => {
				successMessage = '';
			}, 3000);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to add link';
		} finally {
			isSubmitting = false;
		}
	}

	async function handleRemoveLink(link: Link) {
		if (!confirm('Remove this link? This cannot be undone.')) return;

		try {
			await architecture.deleteLink(link.source_id, link.target_id || undefined, link.target_url || undefined);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to remove link';
		}
	}
</script>

{#if cardId}
	<div class="link-manager">
		<div class="section-header">
			<h4>Links</h4>
			<span class="section-count">
				{outgoingLinks.length + incomingLinks.length}
			</span>
		</div>

		{#if error}
			<div class="error-message" role="alert">
				<p>{error}</p>
				<button class="close-btn" onclick={() => (error = '')} aria-label="Dismiss error message"
					>&times;</button
				>
			</div>
		{/if}

		{#if successMessage}
			<div class="success-message">
				<p>{successMessage}</p>
			</div>
		{/if}

		<!-- Add New Link -->
		<div class="add-link-form">
			<details>
				<summary>+ Add Link</summary>

				<div class="form-content">
					<RadioGroup label="Target Type" name="linkTarget" bind:value={linkTarget} options={linkTargets} />

					{#if linkTarget === 'card'}
						<CardSelect
							label="Target Card"
							name="targetCardId"
							cards={allCards}
							bind:value={targetCardId}
							disabled={isSubmitting || isLoading}
						/>
					{:else}
						<TextField
							label="External URL"
							name="targetUrl"
							placeholder="https://example.com"
							type="url"
							bind:value={targetUrl}
							disabled={isSubmitting || isLoading}
						/>
					{/if}

					<button
						type="button"
						class="md-button md-button--primary"
						onclick={handleAddLink}
						disabled={isSubmitting || isLoading}
					>
						{isSubmitting ? '...' : 'Add Link'}
					</button>
				</div>
			</details>
		</div>

		<!-- Outgoing Links -->
		{#if outgoingLinks.length > 0}
			<div class="links-list">
				<h5>Linked To ({outgoingLinks.length})</h5>

				<div class="links-grid">
					{#each outgoingLinks as link (link.source_id + (link.target_id || link.target_url))}
						<div class="link-item">
							<div class="link-content">
								{#if link.target_id}
									<span class="link-type card-link">Card</span>
									<span class="link-target">
										→ {getCardName(link.target_id)}
										<code>{getCardType(link.target_id)}</code>
									</span>
								{:else if link.target_url}
									<span class="link-type url-link">URL</span>
									<a
										href={link.target_url}
										target="_blank"
										rel="noopener noreferrer"
										class="link-target"
									>
										{link.metadata?.link_title || link.target_url}
									</a>
								{/if}
							</div>

							<button
								type="button"
								class="remove-btn"
								onclick={() => handleRemoveLink(link)}
								title="Remove link"
								disabled={isLoading}
							>
								✕
							</button>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Incoming Links -->
		{#if incomingLinks.length > 0}
			<div class="links-list">
				<h5>Referenced By ({incomingLinks.length})</h5>

				<div class="links-grid">
					{#each incomingLinks as link (link.source_id + (link.target_id || link.target_url))}
						<div class="link-item read-only">
							<div class="link-content">
								<span class="link-type card-link">Card</span>
								<span class="link-target">
									← {getCardName(link.source_id)}
									<code>{getCardType(link.source_id)}</code>
								</span>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Empty State -->
		{#if outgoingLinks.length === 0 && incomingLinks.length === 0}
			<div class="empty-state">
				<p>No links yet. Add a link to connect this card to other cards or resources.</p>
			</div>
		{/if}
	</div>
{/if}

<style>
	.link-manager {
		padding: 1.5rem 0;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		margin-top: 1.5rem;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;
	}

	.section-header h4 {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	.section-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 2rem;
		height: 2rem;
		padding: 0 0.5rem;
		background-color: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		border-radius: 1rem;
		font-size: 0.875rem;
		font-weight: 600;
	}

	.add-link-form {
		margin-bottom: 1.5rem;
	}

	.add-link-form details {
		cursor: pointer;
	}

	.add-link-form summary {
		padding: 0.75rem 1rem;
		background-color: var(--md-sys-color-surface-variant);
		border-radius: 0.5rem;
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 500;
		transition: background-color 0.2s;
		user-select: none;
	}

	.add-link-form summary:hover {
		background-color: var(--md-sys-color-outline-variant);
	}

	.add-link-form details[open] summary {
		background-color: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}

	.form-content {
		padding: 1rem;
		background-color: var(--md-sys-color-surface-variant);
		border-radius: 0 0 0.5rem 0.5rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.links-list {
		margin-bottom: 1.5rem;
	}

	.links-list h5 {
		margin: 0 0 0.75rem 0;
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	.links-grid {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.link-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		background-color: var(--md-sys-color-surface-variant);
		border-radius: 0.5rem;
		border-left: 3px solid var(--md-sys-color-primary);
		transition: background-color 0.2s;
	}

	.link-item:hover {
		background-color: var(--md-sys-color-surface-dim);
	}

	.link-item.read-only {
		border-left-color: var(--md-sys-color-outline-variant);
		opacity: 0.8;
	}

	.link-content {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		flex: 1;
		min-width: 0;
	}

	.link-type {
		display: inline-flex;
		align-items: center;
		padding: 0.25rem 0.5rem;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}

	.link-type.card-link {
		background-color: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}

	.link-type.url-link {
		background-color: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}

	.link-target {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		min-width: 0;
		word-break: break-word;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9rem;
	}

	.link-target code {
		padding: 0.125rem 0.375rem;
		background-color: var(--md-sys-color-outline-variant);
		border-radius: 0.25rem;
		font-size: 0.8rem;
		color: var(--md-sys-color-on-surface-variant);
		white-space: nowrap;
	}

	:global(.link-target a) {
		color: var(--md-sys-color-primary);
		text-decoration: none;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	:global(.link-target a:hover) {
		text-decoration: underline;
	}

	.remove-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 2.5rem;
		min-height: 2.5rem;
		padding: 0;
		margin: -0.25rem;
		background: none;
		border: none;
		color: var(--md-sys-color-error);
		cursor: pointer;
		font-size: 1.25rem;
		border-radius: 50%;
		transition: background-color 0.2s;
	}

	.remove-btn:hover:not(:disabled) {
		background-color: var(--md-sys-color-error-container);
	}

	.remove-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.empty-state {
		padding: 1.5rem;
		text-align: center;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.9rem;
		background-color: var(--md-sys-color-surface-variant);
		border-radius: 0.5rem;
	}

	.error-message {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem;
		margin-bottom: 1rem;
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		border-radius: 0.5rem;
	}

	.error-message p {
		margin: 0;
		flex: 1;
	}

	.close-btn {
		background: none;
		border: none;
		color: inherit;
		cursor: pointer;
		font-size: 1.5rem;
		padding: 0;
		margin-left: 1rem;
	}

	.success-message {
		padding: 1rem;
		margin-bottom: 1rem;
		background-color: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		border-radius: 0.5rem;
		animation: slideIn 0.3s ease-out;
	}

	@keyframes slideIn {
		from {
			opacity: 0;
			transform: translateY(-0.5rem);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@media (max-width: 768px) {
		.link-item {
			flex-direction: column;
			align-items: flex-start;
			gap: 0.75rem;
		}

		.remove-btn {
			align-self: flex-end;
			margin-top: 0.5rem;
		}
	}
</style>
