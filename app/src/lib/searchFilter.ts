import type { Card, CardStatus, CardType } from '$lib/types';

export interface SearchFilters {
	searchText?: string;
	cardTypes?: CardType[];
	statuses?: CardStatus[];
	tags?: string[];
}

export interface FilterStats {
	totalCards: number;
	filteredCards: number;
	byType: Record<string, number>;
	byStatus: Record<string, number>;
}

/**
 * Search and filter cards based on multiple criteria
 */
export function searchCards(cards: Card[], filters: SearchFilters): Card[] {
	let results = [...cards];

	// Full-text search
	if (filters.searchText && filters.searchText.trim()) {
		const searchLower = filters.searchText.toLowerCase();
		results = results.filter(
			(card) =>
				card.name.toLowerCase().includes(searchLower) ||
				card.description?.toLowerCase().includes(searchLower) ||
				card.id.toLowerCase().includes(searchLower),
		);
	}

	// Filter by card types
	if (filters.cardTypes && filters.cardTypes.length > 0) {
		results = results.filter((card) => filters.cardTypes!.includes(card.type));
	}

	// Filter by statuses
	if (filters.statuses && filters.statuses.length > 0) {
		results = results.filter((card) => card.status && filters.statuses!.includes(card.status));
	}

	// Filter by tags (if attributes contain tags array)
	if (filters.tags && filters.tags.length > 0) {
		results = results.filter((card) => {
			const cardTags = card.attributes?.['tags'];
			if (!Array.isArray(cardTags)) return false;
			return filters.tags!.some((tag) => cardTags.includes(tag));
		});
	}

	return results;
}

/**
 * Calculate filter statistics
 */
export function calculateFilterStats(cards: Card[], filteredCards: Card[]): FilterStats {
	const byType: Record<string, number> = {};
	const byStatus: Record<string, number> = {};

	filteredCards.forEach((card) => {
		const typeKey = card.type;
		byType[typeKey] = (byType[typeKey] || 0) + 1;

		if (card.status) {
			const statusKey = card.status;
			byStatus[statusKey] = (byStatus[statusKey] || 0) + 1;
		}
	});

	return {
		totalCards: cards.length,
		filteredCards: filteredCards.length,
		byType,
		byStatus,
	};
}

/**
 * Extract all unique tags from cards
 */
export function extractAllTags(cards: Card[]): string[] {
	const tags = new Set<string>();

	cards.forEach((card) => {
		const cardTags = card.attributes?.['tags'];
		if (Array.isArray(cardTags)) {
			cardTags.forEach((tag: string) => tags.add(tag));
		}
	});

	return Array.from(tags).sort();
}
