<script lang="ts">
	import { CardType as CardTypeEnum } from '$lib/types';
	import { getAllPhasesInOrder, type PhaseDefinition } from '$lib/utils/workflow';
	import type { Card, CardType } from '$lib/types';

	let { allCards = [], onCreateCard }: { allCards: Card[]; onCreateCard?: (cardType: CardType) => void } = $props();

	let existingCardTypes = $derived(new Set(allCards.map((c) => c.type)));
	let phases = $derived(getAllPhasesInOrder());

	function hasRequiredCards(phaseDef: PhaseDefinition): boolean {
		return phaseDef.requiredCardTypes.some((type) => existingCardTypes.has(type));
	}

	function getMissingCardTypes(phaseDef: PhaseDefinition): CardTypeEnum[] {
		return phaseDef.requiredCardTypes.filter((type) => !existingCardTypes.has(type));
	}

	function getCompletedCardCount(phaseDef: PhaseDefinition): number {
		return phaseDef.requiredCardTypes.filter((type) => existingCardTypes.has(type)).length;
	}

	function getPhaseStatus(phaseDef: PhaseDefinition): 'completed' | 'in-progress' | 'pending' | 'blocked' {
		const missingRequired = getMissingCardTypes(phaseDef);

		if (missingRequired.length === 0) {
			return 'completed';
		} else if (getCompletedCardCount(phaseDef) > 0) {
			return 'in-progress';
		} else if (phaseDef.step === 1) {
			return 'pending';
		} else {
			// Check if previous phase has required cards
			const previousPhase = phases.find((p) => p.step === phaseDef.step - 1);
			if (previousPhase && !hasRequiredCards(previousPhase)) {
				return 'blocked';
			}
			return 'pending';
		}
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

	function handleCreateCard(cardType: CardTypeEnum) {
		onCreateCard?.(cardType);
	}
</script>

<div class="workflow-checklist">
	<div class="checklist-header">
		<h3>Architecture Workflow Progress</h3>
		<div class="progress-bar-container">
			<div
				class="progress-bar"
				style="width: {(phases.filter((p) => hasRequiredCards(p)).length / phases.length) * 100}%"
			></div>
		</div>
		<p class="progress-text">
			{phases.filter((p) => hasRequiredCards(p)).length} of {phases.length} phases progressed
		</p>
	</div>

	<div class="phases-list">
		{#each phases as phase (phase.phase)}
			{@const status = getPhaseStatus(phase)}
			{@const missingTypes = getMissingCardTypes(phase)}
			{@const completedCount = getCompletedCardCount(phase)}

			<div class="phase-item" class:status>
				<div class="phase-header">
					<div class="phase-number">{phase.step}</div>
					<div class="phase-info">
						<h4>{phase.name}</h4>
						<p class="phase-description">{phase.description}</p>
					</div>
					<div class="phase-status">
						{#if status === 'completed'}
							<span class="badge completed">✓ Complete</span>
						{:else if status === 'in-progress'}
							<span class="badge in-progress">⟳ In Progress</span>
						{:else if status === 'blocked'}
							<span class="badge blocked">⊘ Blocked</span>
						{:else}
							<span class="badge pending">○ Pending</span>
						{/if}
					</div>
				</div>

				{#if status !== 'completed' || missingTypes.length > 0}
					<div class="phase-cards">
						<div class="required-cards">
							<h5>Required Cards ({completedCount}/{phase.requiredCardTypes.length})</h5>
							<div class="card-types-list">
								{#each phase.requiredCardTypes as cardType}
									{@const hasCard = existingCardTypes.has(cardType)}
									<div class="card-type-item" class:present={hasCard}>
										<div class="card-type-label">{getCardTypeLabel(cardType)}</div>
										{#if hasCard}
											<div class="checkmark">✓</div>
										{:else}
											<button
												class="create-btn"
												onclick={() => handleCreateCard(cardType)}
												title="Create {getCardTypeLabel(cardType)}"
											>
												+
											</button>
										{/if}
									</div>
								{/each}
							</div>
						</div>

						{#if phase.recommendedCardTypes.length > 0}
							<div class="recommended-cards">
								<h5>Recommended Cards</h5>
								<div class="card-types-list">
									{#each phase.recommendedCardTypes as cardType}
										{@const hasCard = existingCardTypes.has(cardType)}
										<div class="card-type-item recommended" class:present={hasCard}>
											<div class="card-type-label">{getCardTypeLabel(cardType)}</div>
											{#if hasCard}
												<div class="checkmark">✓</div>
											{:else}
												<button
													class="create-btn"
													onclick={() => handleCreateCard(cardType)}
													title="Create {getCardTypeLabel(cardType)}"
												>
													+
												</button>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.workflow-checklist {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.checklist-header {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.checklist-header h3 {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 600;
	}

	.progress-bar-container {
		width: 100%;
		height: 8px;
		background-color: var(--md-color-surface-variant);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-bar {
		height: 100%;
		background: linear-gradient(90deg, var(--md-color-primary), var(--md-color-secondary));
		transition: width 0.3s ease;
	}

	.progress-text {
		margin: 0;
		font-size: 0.875rem;
		color: var(--md-color-on-surface-variant);
	}

	.phases-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.phase-item {
		border: 1px solid var(--md-color-outline);
		border-radius: 8px;
		padding: 1rem;
		background-color: var(--md-color-surface);
		transition: all 0.3s ease;
	}

	/* Unused state-based styling - kept for future use
	.phase-item.completed {
		border-color: var(--md-color-tertiary);
		background-color: rgba(105, 155, 96, 0.08);
	}

	.phase-item.in-progress {
		border-color: var(--md-color-primary);
		background-color: rgba(63, 81, 181, 0.08);
	}

	.phase-item.blocked {
		opacity: 0.6;
		border-color: var(--md-color-error);
	}
	*/

	.phase-header {
		display: flex;
		gap: 1rem;
		align-items: flex-start;
	}

	.phase-number {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		background-color: var(--md-color-primary-container);
		color: var(--md-color-on-primary-container);
		font-weight: 700;
		font-size: 0.875rem;
		flex-shrink: 0;
	}

	/* Unused state-specific number styling - kept for future use
	.phase-item.completed .phase-number {
		background-color: var(--md-color-tertiary-container);
		color: var(--md-color-on-tertiary-container);
	}
	*/

	.phase-info {
		flex: 1;
	}

	.phase-info h4 {
		margin: 0 0 0.25rem 0;
		font-size: 1rem;
		font-weight: 600;
		line-height: 1.3;
	}

	.phase-description {
		margin: 0;
		font-size: 0.875rem;
		color: var(--md-color-on-surface-variant);
		line-height: 1.4;
	}

	.phase-status {
		display: flex;
		align-items: center;
		flex-shrink: 0;
	}

	.badge {
		display: inline-block;
		padding: 0.375rem 0.75rem;
		border-radius: 12px;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}

	.badge.completed {
		background-color: rgba(105, 155, 96, 0.12);
		color: #4caf50;
	}

	.badge.in-progress {
		background-color: rgba(63, 81, 181, 0.12);
		color: #3f51b5;
	}

	.badge.blocked {
		background-color: rgba(244, 67, 54, 0.12);
		color: #f44336;
	}

	.badge.pending {
		background-color: rgba(117, 117, 117, 0.12);
		color: #666;
	}

	.phase-cards {
		margin-top: 1rem;
		padding-top: 1rem;
		border-top: 1px solid var(--md-color-outline-variant);
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.required-cards h5,
	.recommended-cards h5 {
		margin: 0 0 0.75rem 0;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-color-on-surface-variant);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.card-types-list {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
	}

	.card-type-item {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--md-color-outline-variant);
		border-radius: 6px;
		background-color: var(--md-color-surface-variant);
		font-size: 0.875rem;
		transition: all 0.2s ease;
	}

	.card-type-item.recommended {
		opacity: 0.7;
	}

	.card-type-item.present {
		border-color: var(--md-color-tertiary);
		background-color: rgba(105, 155, 96, 0.12);
		color: var(--md-color-tertiary);
		font-weight: 600;
	}

	.card-type-label {
		flex: 1;
	}

	.checkmark {
		color: var(--md-color-tertiary);
		font-weight: 700;
		font-size: 0.875rem;
	}

	.create-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border: none;
		border-radius: 50%;
		background-color: var(--md-color-primary);
		color: white;
		font-size: 0.75rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
		padding: 0;
	}

	.create-btn:hover {
		background-color: var(--md-color-primary-container);
		color: var(--md-color-on-primary-container);
	}

	.create-btn:active {
		transform: scale(0.95);
	}

	@media (max-width: 768px) {
		.phase-header {
			flex-direction: column;
		}

		.phase-status {
			width: 100%;
		}

		.badge {
			width: 100%;
			text-align: center;
		}

		.card-types-list {
			gap: 0.5rem;
		}

		.card-type-item {
			font-size: 0.8rem;
			padding: 0.375rem 0.625rem;
		}
	}
</style>
