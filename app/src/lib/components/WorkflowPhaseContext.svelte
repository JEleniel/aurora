<script lang="ts">
	import { getPhaseForCardType, WORKFLOW_PHASES } from '$lib/utils/workflow';
	import type { CardType as CardTypeEnum } from '$lib/types';

	let { cardType = null }: { cardType: CardTypeEnum | null } = $props();

	let phase = $derived(cardType ? getPhaseForCardType(cardType) : null);
	let phaseDef = $derived(phase ? WORKFLOW_PHASES[phase] : null);
</script>

{#if phaseDef}
	<div class="workflow-context">
		<div class="context-header">
			<div class="step-indicator">Step {phaseDef.step}/5</div>
			<h4>{phaseDef.name}</h4>
		</div>
		<p class="context-description">{phaseDef.description}</p>
	</div>
{/if}

<style>
	.workflow-context {
		padding: 1rem;
		background: linear-gradient(135deg, rgba(63, 81, 181, 0.08), rgba(103, 58, 183, 0.08));
		border: 1px solid var(--md-color-primary-container);
		border-radius: 8px;
		margin-bottom: 1rem;
		border-left: 4px solid var(--md-color-primary);
	}

	.context-header {
		display: flex;
		gap: 0.75rem;
		align-items: flex-start;
		margin-bottom: 0.75rem;
	}

	.step-indicator {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background-color: var(--md-color-primary);
		color: white;
		padding: 0.375rem 0.75rem;
		border-radius: 6px;
		font-size: 0.75rem;
		font-weight: 700;
		white-space: nowrap;
	}

	.context-header h4 {
		margin: 0;
		font-size: 0.95rem;
		font-weight: 600;
		line-height: 1.3;
		flex: 1;
	}

	.context-description {
		margin: 0.75rem 0;
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--md-color-on-surface-variant);
	}

	/* .context-tips {
		padding: 0.75rem;
		background-color: rgba(255, 255, 255, 0.5);
		border-radius: 6px;
		margin-top: 0.75rem;
	}

	.tip-label {
		margin: 0 0 0.25rem 0;
		font-size: 0.75rem;
		font-weight: 700;
		color: var(--md-color-on-surface);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.tip-text {
		margin: 0;
		font-size: 0.85rem;
		line-height: 1.4;
		color: var(--md-color-on-surface-variant);
	}
	*/
</style>
