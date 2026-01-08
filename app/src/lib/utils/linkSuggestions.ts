import type { Card, CardType } from '$lib/types';
import { CardType as CardTypeEnum } from '$lib/types';

/**
 * Common link patterns in AURORA architecture
 * Based on architectural best practices and documentation conventions
 */
interface LinkPattern {
	sourceType: CardType;
	targetTypes: CardType[];
	reason: string;
}

/**
 * Define common link patterns between AURORA card types
 * These patterns follow AURORA architecture conventions
 */
const linkPatterns: LinkPattern[] = [
	// Drivers pattern: Drivers are satisfied/refined by requirements
	{
		sourceType: CardTypeEnum.Driver,
		targetTypes: [CardTypeEnum.Requirement, CardTypeEnum.Constraint],
		reason: 'Driver is satisfied by requirements and constraints',
	},
	// Requirements pattern: Requirements are realized through behaviors, interfaces, or components
	{
		sourceType: CardTypeEnum.Requirement,
		targetTypes: [CardTypeEnum.Behavior, CardTypeEnum.Interface, CardTypeEnum.LogicalComponent],
		reason: 'Requirement is realized through behaviors, interfaces, or components',
	},
	// Behaviors pattern: Behaviors interact through interfaces
	{
		sourceType: CardTypeEnum.Behavior,
		targetTypes: [CardTypeEnum.Interface, CardTypeEnum.Actor, CardTypeEnum.Behavior],
		reason: 'Behavior uses interfaces and actors',
	},
	// Interfaces pattern: Interfaces connect to logical components and deployable nodes
	{
		sourceType: CardTypeEnum.Interface,
		targetTypes: [CardTypeEnum.LogicalComponent, CardTypeEnum.DeployableNode],
		reason: 'Interface is exposed by components and deployed to nodes',
	},
	// Logical Components pattern: Components implement requirements and deploy to nodes
	{
		sourceType: CardTypeEnum.LogicalComponent,
		targetTypes: [CardTypeEnum.Requirement, CardTypeEnum.DeployableNode, CardTypeEnum.Interface],
		reason: 'Logical component realizes requirements and deploys to nodes',
	},
	// Deployable Nodes pattern: Nodes host logical components
	{
		sourceType: CardTypeEnum.DeployableNode,
		targetTypes: [CardTypeEnum.LogicalComponent, CardTypeEnum.Constraint],
		reason: 'Deployable node hosts components and must satisfy constraints',
	},
	// Constraints pattern: Constraints apply to many element types
	{
		sourceType: CardTypeEnum.Constraint,
		targetTypes: [
			CardTypeEnum.Requirement,
			CardTypeEnum.Behavior,
			CardTypeEnum.LogicalComponent,
			CardTypeEnum.DeployableNode,
		],
		reason: 'Constraint applies to requirements, behaviors, components, and nodes',
	},
	// Actors pattern: Actors interact with behaviors and interfaces
	{
		sourceType: CardTypeEnum.Actor,
		targetTypes: [CardTypeEnum.Behavior, CardTypeEnum.Interface],
		reason: 'Actor interacts with behaviors and interfaces',
	},
	// Tests pattern: Tests verify behaviors, interfaces, and components
	{
		sourceType: CardTypeEnum.Test,
		targetTypes: [CardTypeEnum.Behavior, CardTypeEnum.Interface, CardTypeEnum.LogicalComponent],
		reason: 'Test verifies behaviors, interfaces, and components',
	},
];

/**
 * Get suggested link targets for a source card
 * @param sourceCard The source card
 * @param availableTargets All available cards to link to (excluding the source)
 * @returns Array of suggested targets, sorted by relevance
 */
export function getSuggestedLinkTargets(sourceCard: Card, availableTargets: Card[]): Card[] {
	// Find patterns that apply to this source type
	const applicablePatterns = linkPatterns.filter((p) => p.sourceType === sourceCard.type);

	// Collect all compatible target types
	const compatibleTargetTypes = new Set<CardType>();
	applicablePatterns.forEach((p) => {
		p.targetTypes.forEach((t) => compatibleTargetTypes.add(t));
	});

	// Filter available targets by compatible types
	const suggestedTargets = availableTargets.filter((card) => compatibleTargetTypes.has(card.type));

	// Sort by type order and name
	suggestedTargets.sort((a, b) => {
		// Prioritize by type order (most common first)
		const typeOrder = (type: CardType) => {
			const typeIndex = Array.from(compatibleTargetTypes).indexOf(type);
			return typeIndex >= 0 ? typeIndex : 999;
		};

		const typeOrderDiff = typeOrder(a.type) - typeOrder(b.type);
		if (typeOrderDiff !== 0) return typeOrderDiff;

		// Then sort by name alphabetically
		return a.name.localeCompare(b.name);
	});

	return suggestedTargets;
}

/**
 * Get a description of why a target card is suggested
 * @param sourceType Source card type
 * @param targetType Target card type
 * @returns A human-readable reason for the suggestion
 */
export function getLinkReasonDescription(sourceType: CardType, targetType: CardType): string {
	const pattern = linkPatterns.find((p) => p.sourceType === sourceType && p.targetTypes.includes(targetType));
	return pattern ? pattern.reason : 'Related card';
}

/**
 * Check if two card types can be linked
 * Returns true if there's a documented pattern or if one is a Note/View/Artifact
 * @param sourceType Source card type
 * @param targetType Target card type
 * @returns True if the link is valid according to patterns
 */
export function canLink(sourceType: CardType, targetType: CardType): boolean {
	// Notes, Views, and Artifacts can link to anything
	const flexibleTypes = [CardTypeEnum.Note, CardTypeEnum.View, CardTypeEnum.Artifact];
	if (flexibleTypes.includes(sourceType) || flexibleTypes.includes(targetType)) {
		return true;
	}

	// Check if there's a documented pattern
	return linkPatterns.some((p) => p.sourceType === sourceType && p.targetTypes.includes(targetType));
}
