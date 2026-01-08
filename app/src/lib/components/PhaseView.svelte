<script lang="ts">
	import { allCards, cardToCreate } from '$lib/stores/architecture';
	import { CardType } from '$lib/types';
	import { goto } from '$app/navigation';
	import { type WorkflowPhase, WORKFLOW_PHASES } from '$lib/utils/workflow';

	interface Props {
		phase: WorkflowPhase;
	}

	let { phase }: Props = $props();

	let phaseInfo = $derived(WORKFLOW_PHASES[phase]);

	function getCardTypeLabel(type: CardType): string {
		const labels: Record<CardType, string> = {
			[CardType.Mission]: '🎯 Mission',
			[CardType.Driver]: '🚀 Driver',
			[CardType.Requirement]: '✓ Requirement',
			[CardType.Behavior]: '⚙️ Behavior',
			[CardType.Interface]: '🔌 Interface',
			[CardType.Constraint]: '🛑 Constraint',
			[CardType.LogicalComponent]: '📦 Logical Component',
			[CardType.DeployableNode]: '🖥️ Deployable Node',
			[CardType.Actor]: '👤 Actor',
			[CardType.Test]: '🧪 Test',
			[CardType.View]: '👁️ View',
			[CardType.Note]: '📝 Note',
			[CardType.Artifact]: '📄 Artifact',
		};
		return labels[type] || type;
	}

	function getCardCount(type: CardType): number {
		return $allCards.filter((c) => c.type === type).length;
	}

	function createCard(type: CardType) {
		cardToCreate.set({ type });
		goto('/cards');
	}
</script>

<div class="phase-view">
	<div class="phase-header">
		<div>
			<h1>{phaseInfo.name}</h1>
			<p class="phase-description">{phaseInfo.description}</p>
		</div>
		<div class="phase-number">Phase {phaseInfo.step}/5</div>
	</div>

	<div class="phase-content">
		<section class="card-types-section">
			<h2>Required Card Types</h2>
			<div class="card-types-grid">
				{#each phaseInfo.requiredCardTypes as cardType (cardType)}
					<div class="card-type-card required">
						<div class="card-type-header">
							<span class="card-type-label">{getCardTypeLabel(cardType)}</span>
							<span class="card-count">{getCardCount(cardType)}</span>
						</div>
						<p class="card-type-description">
							{#if getCardCount(cardType) > 0}
								<strong>{getCardCount(cardType)} created</strong>
							{:else}
								<em>Not yet created</em>
							{/if}
						</p>
						<button class="md-button md-button--secondary" onclick={() => createCard(cardType)}>
							+ Create {getCardTypeLabel(cardType)}
						</button>
					</div>
				{/each}
			</div>
		</section>

		{#if phaseInfo.recommendedCardTypes.length > 0}
			<section class="card-types-section">
				<h2>Recommended Card Types</h2>
				<div class="card-types-grid">
					{#each phaseInfo.recommendedCardTypes as cardType (cardType)}
						<div class="card-type-card recommended">
							<div class="card-type-header">
								<span class="card-type-label">{getCardTypeLabel(cardType)}</span>
								<span class="card-count">{getCardCount(cardType)}</span>
							</div>
							<p class="card-type-description">
								{#if getCardCount(cardType) > 0}
									<strong>{getCardCount(cardType)} created</strong>
								{:else}
									<em>Optional</em>
								{/if}
							</p>
							<button class="md-button md-button--tertiary" onclick={() => createCard(cardType)}>
								+ Create {getCardTypeLabel(cardType)}
							</button>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	</div>
</div>

<style>
	.phase-view {
		display: flex;
		flex-direction: column;
		gap: 2rem;
		padding: 0;
	}

	.phase-header {
		background: linear-gradient(
			135deg,
			var(--md-sys-color-primary-container),
			var(--md-sys-color-secondary-container)
		);
		border-radius: 12px;
		padding: 2rem;
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
	}

	.phase-header h1 {
		margin: 0 0 0.5rem 0;
		font-size: 2rem;
		font-weight: 600;
		color: var(--md-sys-color-on-primary-container);
	}

	.phase-description {
		margin: 0;
		font-size: 1.025rem;
		color: var(--md-sys-color-on-primary-container);
		opacity: 0.9;
		max-width: 70%;
	}

	.phase-number {
		background-color: rgba(255, 255, 255, 0.2);
		padding: 0.75rem 1.25rem;
		border-radius: 8px;
		font-weight: 600;
		color: var(--md-sys-color-on-primary-container);
		white-space: nowrap;
	}

	.phase-content {
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.card-types-section h2 {
		font-size: 1.25rem;
		font-weight: 600;
		margin: 0 0 1rem 0;
		color: var(--md-sys-color-on-surface);
	}

	.card-types-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 1.5rem;
	}

	.card-type-card {
		background-color: var(--md-sys-color-surface);
		border: 2px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 1.5rem;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.card-type-card.required {
		border-color: var(--md-sys-color-error);
		background: linear-gradient(
			135deg,
			color-mix(in srgb, var(--md-sys-color-error) 5%, transparent),
			var(--md-sys-color-surface)
		);
	}

	.card-type-card.required:hover {
		border-color: var(--md-sys-color-error);
		box-shadow: 0 4px 16px color-mix(in srgb, var(--md-sys-color-error) 15%, transparent);
	}

	.card-type-card.recommended {
		border-color: var(--md-sys-color-primary);
		background: linear-gradient(
			135deg,
			color-mix(in srgb, var(--md-sys-color-primary) 5%, transparent),
			var(--md-sys-color-surface)
		);
	}

	.card-type-card.recommended:hover {
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 4px 16px color-mix(in srgb, var(--md-sys-color-primary) 15%, transparent);
	}

	.card-type-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.card-type-label {
		font-size: 1.125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	.card-count {
		background-color: var(--md-sys-color-surface-container);
		padding: 0.25rem 0.75rem;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
		min-width: 32px;
		text-align: center;
	}

	.card-type-description {
		margin: 0;
		font-size: 0.9rem;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.4;
	}

	.card-type-card button {
		align-self: flex-start;
		margin-top: 0.5rem;
	}

	@media (max-width: 768px) {
		.phase-header {
			flex-direction: column;
			gap: 1rem;
		}

		.phase-description {
			max-width: 100%;
		}

		.card-types-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
