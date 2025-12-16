import type { Card, CardStatus } from '$lib/types';
import type { Link } from '$lib/types';

/**
 * Get status CSS class for visual styling
 */
export function getStatusClass(status: CardStatus): string {
	return `status-${status}`;
}

/**
 * Calculate related cards from link graph (cards with outgoing or incoming links)
 */
export function getRelatedCards(selectedCardId: string | null | undefined, allCards: Card[], allLinks: Link[]): Card[] {
	if (!selectedCardId) return [];

	const outgoing = allLinks
		.filter((l) => l.source_id === selectedCardId && l.target_id)
		.map((l) => allCards.find((c) => c.id === l.target_id));

	const incoming = allLinks
		.filter((l) => l.target_id === selectedCardId && l.source_id)
		.map((l) => allCards.find((c) => c.id === l.source_id));

	const related = [...new Set([...outgoing, ...incoming])].filter(Boolean) as Card[];
	return related;
}

/**
 * Get other cards (non-related and not selected)
 */
export function getOtherCards(
	selectedCardId: string | null | undefined,
	allCards: Card[],
	relatedCards: Card[],
): Card[] {
	if (!selectedCardId) return [];
	return allCards.filter((c) => c.id !== selectedCardId && !relatedCards.some((rc) => rc.id === c.id));
}
