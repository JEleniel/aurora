<script lang="ts">
	import TextField from './TextField.svelte';
	import { architecture } from '$lib/stores/architecture';
	import type { Link } from '$lib/types';

	interface Props {
		isOpen?: boolean;
		link?: Link | null;
		onClose?: () => void;
	}

	const { isOpen = false, link = null, onClose = () => {} }: Props = $props();

	let isLoading: boolean = $state(false);
	let weight: number = $state(1);
	let confidence: number = $state(1);
	let viewContext: string = $state('');
	let linkTitle: string = $state('');
	let modalElement = $state<HTMLDivElement | null>(null);
	let firstFocusableElement: HTMLElement | null = $state(null);
	let lastFocusableElement: HTMLElement | null = $state(null);

	$effect(() => {
		if (link && isOpen) {
			weight = link.metadata?.weight ?? 1;
			confidence = link.metadata?.confidence ?? 1;
			viewContext = link.metadata?.view_context ?? '';
			linkTitle = link.metadata?.link_title ?? '';

			// Focus first focusable element and find last
			setTimeout(() => {
				if (modalElement) {
					const focusableElements = modalElement.querySelectorAll(
						'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
					);
					if (focusableElements.length > 0) {
						firstFocusableElement = focusableElements[0] as HTMLElement;
						lastFocusableElement = focusableElements[focusableElements.length - 1] as HTMLElement;
						firstFocusableElement?.focus();
					}
				}
			}, 0);
		}
	});

	async function handleSave() {
		if (!link) return;

		isLoading = true;
		try {
			await architecture.updateLink(link.source_id, link.target_id ?? undefined, link.target_url ?? undefined, {
				weight,
				confidence,
				view_context: viewContext || undefined,
				link_title: linkTitle || undefined,
			});
			onClose();
		} finally {
			isLoading = false;
		}
	}

	function handleCancel() {
		onClose();
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			handleCancel();
		} else if (e.key === 'Tab') {
			// Focus trap
			if (e.shiftKey) {
				// Shift+Tab
				if (document.activeElement === firstFocusableElement) {
					e.preventDefault();
					lastFocusableElement?.focus();
				}
			} else {
				// Tab
				if (document.activeElement === lastFocusableElement) {
					e.preventDefault();
					firstFocusableElement?.focus();
				}
			}
		}
	}
</script>

{#if isOpen && link}
	<div
		class="modal-overlay"
		onclick={handleCancel}
		onkeydown={(e) => e.key === 'Escape' && handleCancel()}
		role="presentation"
		aria-hidden="true"
		tabindex="-1"
	>
		<div
			bind:this={modalElement}
			class="modal-content"
			onclick={(e) => e.stopPropagation()}
			onkeydown={handleKeyDown}
			role="dialog"
			aria-labelledby="modal-title"
			aria-modal="true"
			tabindex="-1"
		>
			<div class="modal-header">
				<h3 id="modal-title">Edit Link Metadata</h3>
				<button
					class="close-btn"
					onclick={handleCancel}
					disabled={isLoading}
					aria-label="Close edit link metadata dialog"
					title="Close (Esc)"
				>
					✕
				</button>
			</div>

			<div class="modal-body">
				<div class="field-section">
					<label for="source-display">Source</label>
					<div class="metadata-value" id="source-display" role="status" aria-live="off">
						{link.source_id}
					</div>
				</div>

				<div class="field-section">
					<label for="target-display">Target</label>
					<div class="metadata-value" id="target-display" role="status" aria-live="off">
						{link.target_id || link.target_url || 'Unknown'}
					</div>
				</div>

				<div class="field-section">
					<label for="weight">Weight</label>
					<input
						id="weight"
						type="number"
						min="0"
						max="10"
						step="0.1"
						bind:value={weight}
						disabled={isLoading}
						class="number-input"
						aria-describedby="weight-help"
					/>
					<small id="weight-help">Strength indicator (0-10)</small>
				</div>

				<div class="field-section">
					<label for="confidence">Confidence</label>
					<input
						id="confidence"
						type="number"
						min="0"
						max="1"
						step="0.1"
						bind:value={confidence}
						disabled={isLoading}
						class="number-input"
						aria-describedby="confidence-help"
					/>
					<small id="confidence-help">Confidence level (0-1)</small>
				</div>

				<div class="field-section">
					<TextField
						id="viewContext"
						label="View Context"
						name="viewContext"
						placeholder="e.g., logical-view, deployment-view"
						bind:value={viewContext}
						disabled={isLoading}
						helpText="Optional context for this link"
					/>
				</div>

				<div class="field-section">
					<TextField
						id="linkTitle"
						label="Link Title"
						name="linkTitle"
						placeholder="Display name for this link"
						bind:value={linkTitle}
						disabled={isLoading}
						helpText="Optional title for external links"
					/>
				</div>
			</div>

			<div class="modal-footer">
				<button class="md-button md-button--outlined" onclick={handleCancel} disabled={isLoading}>
					Cancel
				</button>
				<button class="md-button md-button--primary" onclick={handleSave} disabled={isLoading}>
					{isLoading ? 'Saving...' : 'Save'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		display: flex;
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: rgba(0, 0, 0, 0.5);
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal-content {
		display: flex;
		flex-direction: column;
		background-color: var(--md-sys-color-surface-container);
		border-radius: 12px;
		width: 90%;
		max-width: 500px;
		max-height: 80vh;
		overflow-y: auto;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
		outline: none;
	}

	.modal-content:focus {
		outline: none;
	}

	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.5rem;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}

	.modal-header h3 {
		margin: 0;
		font-size: 1.25rem;
		color: var(--md-sys-color-on-background);
	}

	.close-btn {
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		padding: 0.5rem;
		margin: -0.5rem -0.5rem -0.5rem 0;
		color: var(--md-sys-color-on-surface-variant);
		transition: all 0.2s ease;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		min-width: 44px;
	}

	.close-btn:hover:not(:disabled) {
		color: var(--md-sys-color-on-surface);
		background-color: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}

	.close-btn:focus {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}

	.close-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.modal-body {
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
		flex: 1;
	}

	.field-section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.field-section label {
		font-weight: 500;
		color: var(--md-sys-color-on-surface);
		font-size: 0.95rem;
	}

	.metadata-value {
		padding: 0.75rem;
		background-color: var(--md-sys-color-surface);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.9rem;
		word-break: break-word;
		min-height: 2.5rem;
		display: flex;
		align-items: center;
	}

	.number-input {
		padding: 0.75rem;
		border: 2px solid var(--md-sys-color-outline);
		border-radius: 6px;
		background-color: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		font-family: inherit;
		width: 100%;
		box-sizing: border-box;
		transition: all 0.2s ease;
	}

	.number-input:focus {
		outline: none;
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 12%, transparent);
	}

	.number-input:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	small {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.85rem;
	}

	.modal-footer {
		display: flex;
		gap: 1rem;
		padding: 1.5rem;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		justify-content: flex-end;
	}

	.modal-footer button {
		min-width: 120px;
	}

	.modal-footer button:focus {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}

	@media (max-width: 600px) {
		.modal-content {
			width: 95%;
			max-height: 90vh;
		}

		.modal-header,
		.modal-body,
		.modal-footer {
			padding: 1rem;
		}

		.modal-footer {
			flex-direction: column-reverse;
		}

		.modal-footer button {
			width: 100%;
		}
	}
</style>
