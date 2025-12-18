<script lang="ts">
	import { allCards, cardToCreate } from '$lib/stores/architecture';
	import { CardType as CardTypeEnum } from '$lib/types';
	import { goto } from '$app/navigation';
	import { getAllPhasesInOrder, calculateWorkflowProgress } from '$lib/utils/workflow';

	let phases = $derived(getAllPhasesInOrder());
	let progress = $derived(calculateWorkflowProgress($allCards));

	function handleCreateCardOfType(cardType: CardTypeEnum) {
		cardToCreate.set({ type: cardType });
		goto('/cards');
	}

	function getCardTypeLabel(type: CardTypeEnum): string {
		const labels: Record<CardTypeEnum, string> = {
			[CardTypeEnum.Mission]: '🎯 Mission',
			[CardTypeEnum.Driver]: '🚀 Driver',
			[CardTypeEnum.Requirement]: '✓ Requirement',
			[CardTypeEnum.Behavior]: '⚙️ Behavior',
			[CardTypeEnum.Interface]: '🔌 Interface',
			[CardTypeEnum.Constraint]: '🛑 Constraint',
			[CardTypeEnum.LogicalComponent]: '📦 Component',
			[CardTypeEnum.DeployableNode]: '🖥️ Node',
			[CardTypeEnum.Actor]: '👤 Actor',
			[CardTypeEnum.Test]: '🧪 Test',
			[CardTypeEnum.View]: '👁️ View',
			[CardTypeEnum.Note]: '📝 Note',
			[CardTypeEnum.Artifact]: '📄 Artifact',
		};
		return labels[type] || type;
	}
</script>

<div class="workflow-page">
	<div class="workflow-header">
		<h1>Architecture Workflow</h1>
		<p class="subtitle">Five essential phases for systematic design</p>
	</div>

	<div class="progress-bar-container">
		<div class="progress-bar" style="width: {progress.percentComplete}%"></div>
		<p class="progress-text">{progress.completedPhases} of {progress.totalPhases} phases</p>
	</div>

	<div class="phases-grid">
		{#each phases as phase (phase.phase)}
			{@const hasRequiredCards = phase.requiredCardTypes.some((type) => $allCards.some((c) => c.type === type))}

			<div class="phase-card" class:completed={hasRequiredCards}>
				<div class="phase-card-header">
					<div class="phase-number">{phase.step}</div>
					<h3>{phase.name}</h3>
					{#if hasRequiredCards}
						<span class="phase-badge">✓ Complete</span>
					{/if}
				</div>

				<p class="phase-description">{phase.description}</p>

				<div class="cards-section">
					<h4>Required</h4>
					<div class="card-types">
						{#each phase.requiredCardTypes as cardType}
							{@const hasCard = $allCards.some((c) => c.type === cardType)}
							<button
								class="card-type-btn"
								class:present={hasCard}
								onclick={() => handleCreateCardOfType(cardType)}
								title={hasCard ? 'Already created' : 'Click to create'}
							>
								{getCardTypeLabel(cardType)}
								{#if hasCard}
									<span class="check">✓</span>
								{/if}
							</button>
						{/each}
					</div>

					{#if phase.recommendedCardTypes.length > 0}
						<h4>Recommended</h4>
						<div class="card-types">
							{#each phase.recommendedCardTypes as cardType}
								{@const hasCard = $allCards.some((c) => c.type === cardType)}
								<button
									class="card-type-btn recommended"
									class:present={hasCard}
									onclick={() => handleCreateCardOfType(cardType)}
									title={hasCard ? 'Already created' : 'Click to create'}
								>
									{getCardTypeLabel(cardType)}
									{#if hasCard}
										<span class="check">✓</span>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{/each}
	</div>
</div>

<style>
	.workflow-page {
		padding: 0;
	}

	.workflow-header {
		text-align: center;
		margin-bottom: 2rem;
		padding-bottom: 1.5rem;
		border-bottom: 1px solid var(--md-color-outline-variant);
	}

	.workflow-header h1 {
		margin: 0 0 0.5rem 0;
		font-size: 1.75rem;
		font-weight: 700;
	}

	.subtitle {
		margin: 0;
		font-size: 0.95rem;
		color: var(--md-color-on-surface-variant);
	}

	.progress-bar-container {
		width: 100%;
		height: 8px;
		background-color: var(--md-color-surface-variant);
		border-radius: 4px;
		overflow: hidden;
		margin-bottom: 0.5rem;
	}

	.progress-bar {
		height: 100%;
		background: linear-gradient(90deg, var(--md-color-primary), var(--md-color-tertiary));
		transition: width 0.3s ease;
	}

	.progress-text {
		margin: 0;
		font-size: 0.8rem;
		color: var(--md-color-on-surface-variant);
		text-align: right;
		margin-bottom: 2rem;
	}

	.phases-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
		gap: 1.5rem;
	}

	.phase-card {
		padding: 1.5rem;
		border: 2px solid var(--md-color-outline-variant);
		border-radius: 12px;
		background: var(--md-color-surface);
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.phase-card:hover {
		border-color: var(--md-color-primary);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
	}

	.phase-card.completed {
		border-color: var(--md-color-tertiary);
		background: linear-gradient(135deg, rgba(105, 155, 96, 0.06), transparent);
	}

	.phase-card-header {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 0.5rem;
	}

	.phase-number {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border-radius: 8px;
		background-color: var(--md-color-primary-container);
		color: var(--md-color-on-primary-container);
		font-weight: 700;
		font-size: 1.25rem;
		flex-shrink: 0;
	}

	.phase-card-header h3 {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 600;
		flex: 1;
	}

	.phase-badge {
		padding: 0.25rem 0.75rem;
		background-color: var(--md-color-tertiary-container);
		color: var(--md-color-on-tertiary-container);
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}

	.phase-description {
		margin: 0;
		font-size: 0.9rem;
		line-height: 1.5;
		color: var(--md-color-on-surface-variant);
	}

	.cards-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.cards-section h4 {
		margin: 0.5rem 0 0.5rem 0;
		font-size: 0.8rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--md-color-on-surface-variant);
	}

	.cards-section h4:first-child {
		margin-top: 0;
	}

	.card-types {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}

	.card-type-btn {
		display: inline-flex;
		align-items: center;
		gap: 0.375rem;
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--md-color-outline);
		border-radius: 6px;
		background: var(--md-color-surface);
		color: var(--md-color-on-surface);
		font-size: 0.85rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.card-type-btn:hover {
		border-color: var(--md-color-primary);
		background: var(--md-color-primary-container);
		color: var(--md-color-on-primary-container);
	}

	.card-type-btn.recommended {
		opacity: 0.75;
	}

	.card-type-btn.present {
		border-color: var(--md-color-tertiary);
		background: var(--md-color-tertiary-container);
		color: var(--md-color-on-tertiary-container);
	}

	.check {
		font-weight: 700;
	}

	@media (max-width: 768px) {
		.workflow-header h1 {
			font-size: 1.5rem;
		}

		.phases-grid {
			grid-template-columns: 1fr;
			gap: 1rem;
		}

		.phase-card {
			padding: 1rem;
			gap: 0.75rem;
		}

		.phase-number {
			width: 40px;
			height: 40px;
			font-size: 1.1rem;
		}

		.phase-card-header h3 {
			font-size: 1rem;
		}
	}
</style>
