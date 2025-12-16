<script lang="ts">
	import TextField from '$lib/components/TextField.svelte';
	import TextArea from '$lib/components/TextArea.svelte';
	import Select from '$lib/components/Select.svelte';
	import * as archService from '$lib/services/architecture';
	import { getTemplatesForType, applyTemplate } from '$lib/cardTemplates';
	import type { Card, CardStatus, CardType } from '$lib/types';
	import { CardStatus as CardStatusEnum, CardType as CardTypeEnum } from '$lib/types';

	export let editingCard: Card | null = null;
	export let isLoading = false;
	export let onReset: (() => void) | undefined = undefined;
	export let onSubmit:
		| ((detail: {
				editingId: string | null;
				cardId: string;
				cardType: CardType;
				cardName: string;
				cardDescription: string;
				cardStatus: CardStatus;
		  }) => void)
		| undefined = undefined;

	let cardType: CardType = CardTypeEnum.Requirement;
	let cardId = '';
	let cardName = '';
	let cardDescription = '';
	let cardStatus: CardStatus = CardStatusEnum.Proposed;

	let validationMessage = '';
	let validationPass = false;

	$: if (editingCard) {
		cardType = editingCard.type;
		cardId = editingCard.id;
		cardName = editingCard.name;
		cardDescription = editingCard.description || '';
		cardStatus = editingCard.status || CardStatusEnum.Proposed;
	}

	const cardTypes = [
		{ value: CardTypeEnum.Driver, label: 'Driver' },
		{ value: CardTypeEnum.Requirement, label: 'Requirement' },
		{ value: CardTypeEnum.Behavior, label: 'Behavior' },
		{ value: CardTypeEnum.Interface, label: 'Interface' },
		{ value: CardTypeEnum.Constraint, label: 'Constraint' },
		{ value: CardTypeEnum.LogicalComponent, label: 'Logical Component' },
		{ value: CardTypeEnum.DeployableNode, label: 'Deployable Node' },
		{ value: CardTypeEnum.Actor, label: 'Actor' },
		{ value: CardTypeEnum.Test, label: 'Test' },
		{ value: CardTypeEnum.Artifact, label: 'Artifact' },
		{ value: CardTypeEnum.View, label: 'View' },
		{ value: CardTypeEnum.Note, label: 'Note' },
	];

	const statuses = [
		{ value: CardStatusEnum.Proposed, label: 'Proposed' },
		{ value: CardStatusEnum.Approved, label: 'Approved' },
		{ value: CardStatusEnum.Implemented, label: 'Implemented' },
		{ value: CardStatusEnum.Verified, label: 'Verified' },
		{ value: CardStatusEnum.Deprecated, label: 'Deprecated' },
		{ value: CardStatusEnum.Retired, label: 'Retired' },
	];

	async function validateCard() {
		validationMessage = '';
		validationPass = false;

		try {
			const card = {
				id: cardId || 'temp-id',
				type: cardType,
				name: cardName,
				description: cardDescription || undefined,
				version: '1.0.0',
				status: cardStatus,
				attributes: undefined,
				created_at: new Date().toISOString(),
				modified_at: new Date().toISOString(),
			};

			await archService.validateCard(card);
			validationMessage = '✓ Card validation passed';
			validationPass = true;
		} catch (error) {
			validationMessage = '✗ ' + (error instanceof Error ? error.message : 'Validation failed');
			validationPass = false;
		}
	}

	function applyCardTemplate(templateName: string) {
		const template = applyTemplate(cardType, templateName);
		cardName = template.name || '';
		cardDescription = template.description || '';
		if (template.status) cardStatus = template.status;
	}

	function handleReset() {
		cardType = CardTypeEnum.Requirement;
		cardId = '';
		cardName = '';
		cardDescription = '';
		cardStatus = CardStatusEnum.Proposed;
		validationMessage = '';
		validationPass = false;
		onReset?.();
	}

	function handleSubmit() {
		onSubmit?.({
			editingId: editingCard?.id || null,
			cardId,
			cardType,
			cardName,
			cardDescription,
			cardStatus,
		});
		if (!editingCard) {
			handleReset();
		}
	}
</script>

<section class="card-form">
	<h3>{editingCard ? 'Edit Card' : 'Create New Card'}</h3>
	<form on:submit|preventDefault={handleSubmit}>
		{#if !editingCard}
			<Select
				label="Card Type"
				name="cardType"
				bind:value={cardType}
				options={cardTypes}
				required
				disabled={editingCard !== null}
			/>

			{#if getTemplatesForType(cardType).length > 0}
				<div class="templates-section">
					<label for="template-select">Quick Templates</label>
					<div class="templates-grid">
						{#each getTemplatesForType(cardType) as template (template.name)}
							<button
								type="button"
								class="template-button"
								on:click={() => applyCardTemplate(template.name)}
								title={template.description}
							>
								<span class="template-icon">{template.icon}</span>
								<span class="template-name">{template.name}</span>
							</button>
						{/each}
					</div>
				</div>
			{/if}
		{/if}

		<TextField
			label={editingCard ? 'Card ID (read-only)' : 'Card ID'}
			name="cardId"
			placeholder="e.g., driver-automation"
			bind:value={cardId}
			required
			disabled={editingCard !== null}
		/>

		<TextField label="Name" name="cardName" placeholder="Enter card name" bind:value={cardName} required />

		<TextArea
			label="Description"
			name="cardDescription"
			placeholder="Enter detailed description..."
			bind:value={cardDescription}
			rows={6}
		/>

		{#if editingCard}
			<Select label="Status" name="cardStatus" bind:value={cardStatus} options={statuses} />
		{/if}

		<div class="form-buttons">
			<button type="button" class="md-button md-button--secondary" on:click={validateCard} disabled={isLoading}>
				Validate
			</button>
			<button type="submit" class="md-button md-button--primary" disabled={isLoading}>
				{isLoading ? '...'
				: editingCard ? 'Save'
				: 'Create'}
			</button>
			<button type="button" class="md-button md-button--outlined" on:click={handleReset} disabled={isLoading}>
				{editingCard ? 'Cancel' : 'Clear'}
			</button>
		</div>

		{#if validationMessage}
			<div class={`validation-message ${validationPass ? 'success' : 'error'}`}>
				{validationMessage}
			</div>
		{/if}
	</form>
</section>

<style>
	.card-form {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 12px;
		padding: 1.5rem;
		height: fit-content;
		position: sticky;
		top: 1rem;
		overflow-y: auto;
		max-height: calc(100vh - 200px);
	}

	h3 {
		font-size: 1.25rem;
		font-weight: 500;
		margin-bottom: 1.5rem;
		color: var(--md-sys-color-on-background);
	}

	form {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.templates-section {
		margin-top: 1.5rem;
		padding-top: 1.5rem;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}

	.templates-section label {
		display: block;
		font-weight: 500;
		margin-bottom: 0.75rem;
		color: var(--md-sys-color-on-surface);
	}

	.templates-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
		gap: 0.75rem;
	}

	.template-button {
		background-color: var(--md-sys-color-surface-container);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0.5rem;
		padding: 0.75rem;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		font-family: inherit;
		font-size: 0.875rem;
	}

	.template-button:hover {
		background-color: var(--md-sys-color-secondary-container);
		border-color: var(--md-sys-color-primary);
		transform: translateY(-2px);
	}

	.template-icon {
		font-size: 1.5rem;
	}

	.template-name {
		text-align: center;
		color: var(--md-sys-color-on-surface);
		font-weight: 500;
	}

	.form-buttons {
		display: flex;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.form-buttons button {
		flex: 1;
	}

	.validation-message {
		padding: 0.75rem 1rem;
		border-radius: 0.5rem;
		margin-top: 1rem;
		font-weight: 500;
	}

	.validation-message.success {
		background-color: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}

	.validation-message.error {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
</style>
