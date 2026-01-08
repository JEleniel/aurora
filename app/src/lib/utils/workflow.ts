/**
 * AURORA Workflow System - Simplified to 5 core architectural phases.
 *
 * Philosophy: Minimal, composable, evidence-driven architecture design.
 * Each phase represents a coherent set of architectural decisions and their traceability.
 */

import { CardType } from '../types';

export enum WorkflowPhase {
	Drivers = '1_drivers',
	Landscape = '2_landscape',
	Structure = '3_structure',
	Dynamics = '4_dynamics',
	Operations = '5_operations',
}

export interface PhaseDefinition {
	phase: WorkflowPhase;
	step: number;
	name: string;
	description: string;
	requiredCardTypes: CardType[];
	recommendedCardTypes: CardType[];
}

/**
 * AURORA Workflow Phases - Five Core Architectural Activities
 *
 * The five phases represent the essential activities in architecture design,
 * organized by the concerns being addressed. Each phase builds on the previous,
 * establishing traceability from decisions to implementation.
 */
export const WORKFLOW_PHASES: Record<WorkflowPhase, PhaseDefinition> = {
	[WorkflowPhase.Drivers]: {
		phase: WorkflowPhase.Drivers,
		step: 1,
		name: 'Establish Drivers',
		description: 'Define mission, stakeholders, requirements, constraints, and architectural drivers.',
		requiredCardTypes: [CardType.Mission, CardType.Driver],
		recommendedCardTypes: [CardType.Requirement, CardType.Constraint, CardType.Actor],
	},

	[WorkflowPhase.Landscape]: {
		phase: WorkflowPhase.Landscape,
		step: 2,
		name: 'Define Landscape',
		description: 'Establish system boundaries, external interfaces, and integration points.',
		requiredCardTypes: [CardType.Interface, CardType.Actor],
		recommendedCardTypes: [CardType.View, CardType.Artifact],
	},

	[WorkflowPhase.Structure]: {
		phase: WorkflowPhase.Structure,
		step: 3,
		name: 'Design Structure',
		description: 'Decompose into logical components with clear responsibilities and contracts.',
		requiredCardTypes: [CardType.LogicalComponent, CardType.Interface],
		recommendedCardTypes: [CardType.Artifact, CardType.View],
	},

	[WorkflowPhase.Dynamics]: {
		phase: WorkflowPhase.Dynamics,
		step: 4,
		name: 'Model Dynamics',
		description: 'Define runtime behavior, interactions, data architecture, and performance characteristics.',
		requiredCardTypes: [CardType.Behavior, CardType.Artifact],
		recommendedCardTypes: [CardType.Test, CardType.View],
	},

	[WorkflowPhase.Operations]: {
		phase: WorkflowPhase.Operations,
		step: 5,
		name: 'Plan Operations',
		description: 'Map to deployment infrastructure, establish security and observability, document decisions.',
		requiredCardTypes: [CardType.DeployableNode, CardType.Constraint],
		recommendedCardTypes: [CardType.Note, CardType.View],
	},
};

/**
 * Get the workflow phase for a given card type.
 * Maps each card type to its primary phase.
 */
export function getPhaseForCardType(cardType: CardType): WorkflowPhase | null {
	const phaseMapping: Record<CardType, WorkflowPhase> = {
		[CardType.Mission]: WorkflowPhase.Drivers,
		[CardType.Driver]: WorkflowPhase.Drivers,
		[CardType.Requirement]: WorkflowPhase.Drivers,
		[CardType.Constraint]: WorkflowPhase.Drivers,
		[CardType.Actor]: WorkflowPhase.Landscape,
		[CardType.Interface]: WorkflowPhase.Landscape,
		[CardType.LogicalComponent]: WorkflowPhase.Structure,
		[CardType.Artifact]: WorkflowPhase.Dynamics,
		[CardType.Behavior]: WorkflowPhase.Dynamics,
		[CardType.Test]: WorkflowPhase.Dynamics,
		[CardType.DeployableNode]: WorkflowPhase.Operations,
		[CardType.Note]: WorkflowPhase.Operations,
		[CardType.View]: WorkflowPhase.Operations,
	};
	return phaseMapping[cardType] || null;
}

/**
 * Calculate overall workflow progress.
 * Returns count of phases that have at least one required card.
 */
export function calculateWorkflowProgress(cards: any[]): {
	totalPhases: number;
	completedPhases: number;
	percentComplete: number;
	nextPhase: WorkflowPhase | null;
} {
	const phases = getAllPhasesInOrder();
	let completedCount = 0;
	let nextPhaseToComplete: WorkflowPhase | null = null;

	for (const phaseDef of phases) {
		const hasRequiredCards = phaseDef.requiredCardTypes.some((type) => cards.some((c) => c.type === type));

		if (hasRequiredCards) {
			completedCount++;
		} else if (!nextPhaseToComplete) {
			nextPhaseToComplete = phaseDef.phase;
		}
	}

	return {
		totalPhases: phases.length,
		completedPhases: completedCount,
		percentComplete: Math.round((completedCount / phases.length) * 100),
		nextPhase: nextPhaseToComplete,
	};
}

/**
 * Get all phases ordered by step number.
 */
export function getAllPhasesInOrder(): PhaseDefinition[] {
	return Object.values(WORKFLOW_PHASES).sort((a, b) => a.step - b.step);
}

/**
 * Get recommended next card type to create based on current phase.
 */
export function getRecommendedNextCardType(
	currentPhase: WorkflowPhase,
	existingCardTypes: Set<CardType>,
): CardType | null {
	const phaseDef = WORKFLOW_PHASES[currentPhase];
	if (!phaseDef) return null;

	// First, check for missing required types
	for (const requiredType of phaseDef.requiredCardTypes) {
		if (!existingCardTypes.has(requiredType)) {
			return requiredType;
		}
	}

	// Then suggest recommended types
	for (const recommendedType of phaseDef.recommendedCardTypes) {
		if (!existingCardTypes.has(recommendedType)) {
			return recommendedType;
		}
	}

	return null;
}
