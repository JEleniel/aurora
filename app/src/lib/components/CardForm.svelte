<script lang="ts">
	import { onMount } from 'svelte';
	import { v4 as uuidv4 } from 'uuid';
	import TextField from '$lib/components/TextField.svelte';
	import TextArea from '$lib/components/TextArea.svelte';
	import Select from '$lib/components/Select.svelte';
	import LinkManager from '$lib/components/LinkManager.svelte';
	import WorkflowPhaseContext from '$lib/components/WorkflowPhaseContext.svelte';
	import * as archService from '$lib/services/architecture';
	import { cardToCreate } from '$lib/stores/architecture';
	import { getSuggestedLinkTargets } from '$lib/utils/linkSuggestions';
	import type { Card, CardStatus, CardType, Link } from '$lib/types';
	import { CardStatus as CardStatusEnum, CardType as CardTypeEnum } from '$lib/types';

	interface Props {
		editingCard?: Card | null;
		isLoading?: boolean;
		availableCards?: Card[];
		allLinks?: Link[];
		onReset?: () => void;
		onSubmit?: (detail: {
			editingId: string | null;
			cardId: string;
			cardType: CardType;
			cardName: string;
			cardDescription: string;
			cardStatus: CardStatus;
			linkToCardId?: string;
		}) => void;
	}

	const {
		editingCard = null,
		isLoading = false,
		availableCards = [],
		allLinks = [],
		onReset,
		onSubmit,
	}: Props = $props();

	let cardType: CardType = $state(CardTypeEnum.Requirement);
	let cardId: string = $state('');
	let cardName: string = $state('');
	let cardDescription: string = $state('');
	let cardStatus: CardStatus = $state(CardStatusEnum.Proposed);
	let linkToCardId: string = $state('');

	let validationMessage: string = $state('');
	let validationPass: boolean = $state(false);
	let formElement: HTMLElement;

	$effect(() => {
		if (editingCard) {
			cardType = editingCard.type;
			cardId = editingCard.id;
			cardName = editingCard.name;
			cardDescription = editingCard.description || '';
			cardStatus = editingCard.status || CardStatusEnum.Proposed;

			// Scroll form into view when editing
			if (formElement) {
				setTimeout(() => {
					formElement?.scrollIntoView({ behavior: 'smooth', block: 'start' });
				}, 0);
			}
		}
	});

	// Real-time validation
	$effect(() => {
		if (!editingCard && cardName && cardDescription) {
			validateCardRealtime();
		}
	});

	// Compute suggested link targets based on card type
	let suggestedLinkTargets = $derived(
		!editingCard && cardType && availableCards.length > 0 ?
			getSuggestedLinkTargets(
				{
					id: cardId,
					type: cardType,
					name: cardName,
					description: cardDescription,
					version: '1.0.0',
					status: cardStatus,
					created_at: new Date().toISOString(),
					modified_at: new Date().toISOString(),
				},
				availableCards.filter((c) => c.id !== cardId),
			)
		:	[],
	);

	const cardTypes = [
		{ value: CardTypeEnum.Mission, label: 'Mission' },
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

	onMount(() => {
		// Generate full v4 UUID for new cards
		if (!editingCard && !cardId) {
			cardId = uuidv4();
		}

		// If cardToCreate has initial values, apply them
		const unsubscribe = cardToCreate.subscribe((createData) => {
			if (createData && !editingCard) {
				cardType = createData.type;
				if (createData.name) {
					cardName = createData.name;
				}
			}
		});

		return unsubscribe;
	});

	async function validateCardRealtime() {
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
			validationMessage = '✓ Valid';
			validationPass = true;
		} catch (error) {
			validationMessage = '✗ ' + (error instanceof Error ? error.message : 'Invalid');
			validationPass = false;
		}
	}

	function handleReset() {
		cardType = CardTypeEnum.Requirement;
		cardId = uuidv4().substring(0, 8);
		cardName = '';
		cardDescription = '';
		cardStatus = CardStatusEnum.Proposed;
		linkToCardId = '';
		validationMessage = '';
		validationPass = false;
		onReset?.();
	}

	function handleSubmit(event: Event) {
		event.preventDefault();
		const submitDetail: {
			editingId: string | null;
			cardId: string;
			cardType: CardType;
			cardName: string;
			cardDescription: string;
			cardStatus: CardStatus;
			linkToCardId?: string;
		} = {
			editingId: editingCard?.id || null,
			cardId,
			cardType,
			cardName,
			cardDescription,
			cardStatus,
		};

		if (!editingCard && linkToCardId.trim()) {
			submitDetail.linkToCardId = linkToCardId;
		}

		onSubmit?.(submitDetail);
		if (!editingCard) {
			handleReset();
		}
	}
</script>

<section class="card-form" bind:this={formElement}>
	<div class="form-header">
		<h3>{editingCard ? 'Edit Card' : 'Create New Card'}</h3>
		<code class="card-id" title="Auto-generated UUID v4">{cardId}</code>
	</div>

	{#if !editingCard}
		<WorkflowPhaseContext {cardType} />
	{/if}

	<form onsubmit={handleSubmit}>
		{#if !editingCard}
			<Select
				label="Card Type"
				name="cardType"
				bind:value={cardType}
				options={cardTypes}
				required
				disabled={editingCard !== null}
			/>
		{/if}

		<div class="input-with-flag">
			<TextField label="Name" name="cardName" placeholder="Enter card name" bind:value={cardName} required />
			{#if validationMessage}
				<div
					class={`validation-flag ${validationPass ? 'success' : 'error'}`}
					title={validationMessage}
					role="status"
					aria-live="polite"
					aria-atomic="true"
				>
					{validationPass ? '✓ Valid' : '✗ Invalid'}
				</div>
			{/if}
		</div>

		<TextArea
			label="Description"
			name="cardDescription"
			placeholder="Enter detailed description..."
			bind:value={cardDescription}
			rows={6}
		/>

		{#if editingCard}
			<Select label="Status" name="cardStatus" bind:value={cardStatus} options={statuses} />
		{:else}
			<Select
				label="Link to Card (Optional)"
				name="linkToCardId"
				bind:value={linkToCardId}
				options={[
					{ value: '', label: 'None' },
					...suggestedLinkTargets.map((card) => ({
						value: card.id,
						label: `✨ ${card.name} (${card.type})`,
					})),
					...(suggestedLinkTargets.length < availableCards.length ?
						[
							{ value: '', label: '─────────────' },
							...availableCards
								.filter((c) => !suggestedLinkTargets.some((s) => s.id === c.id))
								.map((card) => ({
									value: card.id,
									label: `${card.name} (${card.type})`,
								})),
						]
					:	[]),
				]}
			/>
		{/if}

		<div class="form-buttons">
			<button type="submit" class="md-button md-button--primary" disabled={isLoading}>
				{isLoading ? '...'
				: editingCard ? 'Save'
				: 'Create'}
			</button>
			<button type="button" class="md-button md-button--outlined" onclick={handleReset} disabled={isLoading}>
				{editingCard ? 'Cancel' : 'Clear'}
			</button>
		</div>
	</form>

	{#if editingCard}
		<LinkManager cardId={editingCard.id} allCards={availableCards} {allLinks} {isLoading} />
	{/if}
</section>

<style>
	.card-form {
		background-color: var(--md-sys-color-surface-container);
		border-radius: 12px;
		padding: 1rem;
		height: fit-content;
		position: sticky;
		top: 0;
		overflow-y: auto;
		max-height: calc(100vh - 100px);
	}

	.form-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	h3 {
		font-size: 1.25rem;
		font-weight: 500;
		margin: 0;
		color: var(--md-sys-color-on-background);
		flex: 1;
	}

	.card-id {
		font-size: 0.5rem;
		font-family: 'Courier New', monospace;
		background-color: var(--md-sys-color-surface-variant);
		color: var(--md-sys-color-on-surface-variant);
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		word-break: break-all;
		max-width: 140px;
		line-height: 1.2;
		text-align: right;
		flex-shrink: 0;
	}

	form {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.form-buttons {
		display: flex;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.form-buttons button {
		flex: 1;
	}

	.input-with-flag {
		position: relative;
		display: flex;
		align-items: flex-start;
		gap: 0.5rem;
	}

	.validation-flag {
		position: absolute;
		top: 2px;
		right: 8px;
		font-size: 0.7rem;
		font-weight: bold;
		padding: 0.2rem 0.4rem;
		border-radius: 3px;
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.2rem;
		height: 1.2rem;
	}

	.validation-flag.success {
		background-color: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}

	.validation-flag.error {
		background-color: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
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

	@media (max-width: 1200px) {
		.card-form {
			position: relative;
			top: 0;
			max-height: none;
			padding: 1rem;
		}

		.form-buttons {
			flex-wrap: wrap;
		}

		.form-buttons button {
			min-width: 0;
		}

		.templates-grid {
			flex-wrap: wrap;
		}

		.template-button {
			flex-shrink: 1;
		}
	}

	@media (max-width: 768px) {
		.card-form {
			padding: 0.75rem;
		}

		h3 {
			margin-bottom: 1rem;
			font-size: 1.1rem;
		}

		form {
			gap: 0.75rem;
		}

		.form-buttons {
			margin-top: 0.75rem;
		}
	}
</style>
