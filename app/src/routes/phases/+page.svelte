<script lang="ts">
	import { allCards } from '$lib/stores/architecture';
	import { goto } from '$app/navigation';
	import {
		WorkflowPhase,
		WORKFLOW_PHASES,
		calculateWorkflowProgress,
		getAllPhasesInOrder,
	} from '$lib/utils/workflow';

	const phases = getAllPhasesInOrder();
	const progress = calculateWorkflowProgress($allCards);

	function getPhaseIcon(phase: WorkflowPhase): string {
		const icons: Record<WorkflowPhase, string> = {
			[WorkflowPhase.Drivers]: '🎯',
			[WorkflowPhase.Landscape]: '🗺️',
			[WorkflowPhase.Structure]: '🏗️',
			[WorkflowPhase.Dynamics]: '⚙️',
			[WorkflowPhase.Operations]: '🚀',
		};
		return icons[phase] || '•';
	}

	function getPhaseRoute(phase: WorkflowPhase): string {
		const routes: Record<WorkflowPhase, string> = {
			[WorkflowPhase.Drivers]: '/phases/drivers',
			[WorkflowPhase.Landscape]: '/phases/landscape',
			[WorkflowPhase.Structure]: '/phases/structure',
			[WorkflowPhase.Dynamics]: '/phases/dynamics',
			[WorkflowPhase.Operations]: '/phases/operations',
		};
		return routes[phase];
	}

	function getCardCount(phase: WorkflowPhase): number {
		const allRequiredAndRecommended = [
			...WORKFLOW_PHASES[phase].requiredCardTypes,
			...WORKFLOW_PHASES[phase].recommendedCardTypes,
		];
		return $allCards.filter((c) => allRequiredAndRecommended.includes(c.type)).length;
	}

	function getRequiredCount(phase: WorkflowPhase): number {
		const required = WORKFLOW_PHASES[phase].requiredCardTypes;
		return $allCards.filter((c) => required.includes(c.type)).length;
	}

	async function navigateToPhase(phase: WorkflowPhase) {
		goto(getPhaseRoute(phase));
	}
</script>

<div class="phases-container">
	<div class="phases-header">
		<h1>Workflow Phases</h1>
		<p class="subtitle">Five core architectural activities for systematic design</p>
		<div class="progress-container">
			<div class="progress-bar-wrapper">
				<div class="progress-bar" style={`--progress-width: ${progress.percentComplete}%`}></div>
			</div>
			<p class="progress-text">{progress.completedPhases} of {progress.totalPhases} phases completed</p>
		</div>
	</div>

	<div class="phases-grid">
		{#each phases as phase (phase.phase)}
			<button class="phase-card" onclick={() => navigateToPhase(phase.phase)}>
				<div class="phase-card-icon">{getPhaseIcon(phase.phase)}</div>
				<div class="phase-card-content">
					<h3>Phase {phase.step}: {phase.name}</h3>
					<p class="phase-card-description">{phase.description}</p>
					<div class="phase-card-stats">
						<span class="stat">
							<strong>{getCardCount(phase.phase)}</strong> cards
						</span>
						<span class="separator">•</span>
						<span class="stat">
							<strong>{getRequiredCount(phase.phase)}</strong> required
						</span>
					</div>
				</div>
				<div class="phase-card-arrow">→</div>
			</button>
		{/each}
	</div>

	<div class="phases-info">
		<div class="info-box">
			<h4>📋 About These Phases</h4>
			<p>AURORA organizes architecture design into five essential phases, each addressing specific concerns:</p>
			<ul>
				<li><strong>Drivers:</strong> Establish mission, stakeholders, requirements, constraints</li>
				<li><strong>Landscape:</strong> Define system boundaries and external interfaces</li>
				<li><strong>Structure:</strong> Decompose into logical components with clear contracts</li>
				<li><strong>Dynamics:</strong> Model runtime behavior and interactions</li>
				<li><strong>Operations:</strong> Map to deployment infrastructure and governance</li>
			</ul>
		</div>
	</div>
</div>

<style>
	.phases-container {
		display: flex;
		flex-direction: column;
		gap: 2rem;
		padding: 0;
	}

	.phases-header {
		background: linear-gradient(135deg, var(--md-sys-color-primary), var(--md-sys-color-tertiary));
		border-radius: 12px;
		padding: 2.5rem;
		color: var(--md-sys-color-on-primary);
	}

	.phases-header h1 {
		margin: 0 0 0.5rem 0;
		font-size: 2.25rem;
		font-weight: 600;
	}

	.subtitle {
		margin: 0 0 1.5rem 0;
		font-size: 1.1rem;
		opacity: 0.95;
	}

	.progress-container {
		margin-top: 1.5rem;
	}

	.progress-bar-wrapper {
		background-color: rgba(255, 255, 255, 0.2);
		border-radius: 8px;
		height: 8px;
		overflow: hidden;
		margin-bottom: 0.75rem;
	}

	.progress-bar {
		background-color: rgba(255, 255, 255, 0.9);
		height: 100%;
		width: var(--progress-width, 0%);
		transition: width 0.3s ease;
	}

	.progress-text {
		margin: 0;
		font-size: 0.9rem;
		opacity: 0.9;
	}

	.phases-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
		gap: 1.5rem;
	}

	.phase-card {
		background: var(--md-sys-color-surface);
		border: 2px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 1.75rem;
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.2, 0, 0, 1);
		text-align: left;
		display: flex;
		gap: 1.5rem;
		align-items: flex-start;
		font-family: inherit;
		color: inherit;
		font-size: inherit;
	}

	.phase-card:hover {
		border-color: var(--md-sys-color-primary);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
		transform: translateY(-4px);
		background: var(--md-sys-color-surface-container);
	}

	.phase-card-icon {
		font-size: 2.5rem;
		flex-shrink: 0;
	}

	.phase-card-content {
		flex: 1;
		min-width: 0;
	}

	.phase-card-content h3 {
		margin: 0 0 0.5rem 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}

	.phase-card-description {
		margin: 0 0 0.75rem 0;
		font-size: 0.95rem;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.4;
	}

	.phase-card-stats {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}

	.separator {
		opacity: 0.5;
	}

	.phase-card-arrow {
		font-size: 1.5rem;
		color: var(--md-sys-color-primary);
		flex-shrink: 0;
		opacity: 0;
		transition: opacity 0.3s;
	}

	.phase-card:hover .phase-card-arrow {
		opacity: 1;
	}

	.phases-info {
		display: flex;
		gap: 1.5rem;
	}

	.info-box {
		background: linear-gradient(
			135deg,
			color-mix(in srgb, var(--md-sys-color-secondary) 10%, transparent),
			color-mix(in srgb, var(--md-sys-color-tertiary) 10%, transparent)
		);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 12px;
		padding: 1.5rem;
		flex: 1;
	}

	.info-box h4 {
		margin: 0 0 0.75rem 0;
		font-size: 1rem;
		color: var(--md-sys-color-on-surface);
	}

	.info-box p {
		margin: 0 0 1rem 0;
		font-size: 0.95rem;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.5;
	}

	.info-box ul {
		margin: 0;
		padding-left: 1.5rem;
		list-style: none;
	}

	.info-box li {
		margin-bottom: 0.5rem;
		font-size: 0.9rem;
		color: var(--md-sys-color-on-surface-variant);
		line-height: 1.4;
	}

	.info-box strong {
		color: var(--md-sys-color-on-surface);
	}

	@media (max-width: 768px) {
		.phases-header {
			padding: 1.5rem;
		}

		.phases-header h1 {
			font-size: 1.75rem;
		}

		.phases-grid {
			grid-template-columns: 1fr;
		}

		.phase-card {
			flex-direction: column;
			align-items: center;
			text-align: center;
		}

		.phase-card-content {
			text-align: center;
		}

		.phase-card-arrow {
			opacity: 0;
		}
	}
</style>
