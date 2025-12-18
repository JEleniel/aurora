/// Card type colors for visual distinction
/// Uses colorblind-friendly palette (Paul Tol)
import { CardType } from '$lib/types';

const cardColorMap: Record<CardType, string> = {
	[CardType.Mission]: '#0072B2', // Strong blue
	[CardType.Driver]: '#D55E00', // Vermillion/orange
	[CardType.Requirement]: '#009E73', // Green
	[CardType.Behavior]: '#56B4E9', // Sky blue
	[CardType.Interface]: '#0072B2', // Strong blue (variant)
	[CardType.Constraint]: '#E69F00', // Orange
	[CardType.LogicalComponent]: '#009E73', // Green (variant)
	[CardType.DeployableNode]: '#CC79A7', // Reddish purple
	[CardType.Actor]: '#F0E442', // Yellow
	[CardType.Test]: '#D55E00', // Vermillion (variant)
	[CardType.Artifact]: '#999999', // Gray
	[CardType.View]: '#56B4E9', // Sky blue (variant)
	[CardType.Note]: '#CCCCCC', // Light gray
};

export function getCardColor(type: CardType): string {
	return cardColorMap[type] || '#999999';
}
