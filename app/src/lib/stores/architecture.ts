/// Svelte store for architecture state
import { writable, derived, type Readable } from 'svelte/store';
import type { Card, CardStatus, CardType, Link, ModelStatistics, ProjectMetadata } from '../types';
import * as archService from '../services/architecture';
import { errorLogging } from '../services/errorLogging';

interface ArchitectureState {
	cards: Map<string, Card>;
	links: Link[];
	metadata: ProjectMetadata;
	statistics: ModelStatistics | null;
	selectedCardId: string | null;
	loading: boolean;
	error: string | null;
}

function createArchitectureStore() {
	const initialState: ArchitectureState = {
		cards: new Map(),
		links: [],
		metadata: {
			version: '1.0.0',
		},
		statistics: null,
		selectedCardId: null,
		loading: false,
		error: null,
	};

	const { subscribe, set, update } = writable<ArchitectureState>(initialState);

	return {
		subscribe,

		async loadFromZip(path: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.loadArchitecture(path);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
			}
		},

		async saveToZip(path: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.saveArchitecture(path);
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
			}
		},

		async refresh() {
			try {
				const [cards, links, metadata, statistics] = await Promise.all([
					archService.getCards(),
					archService.getLinks(),
					archService.getMetadata(),
					archService.getStatistics(),
				]);

				const cardMap = new Map(cards.map((card) => [card.id, card]));

				update((state) => ({
					...state,
					cards: cardMap,
					links,
					metadata,
					statistics,
				}));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, error: errorMsg }));
				await errorLogging.warn(`Failed to refresh data: ${errorMsg}`, 'architectureStore');
			}
		},

		async createCard(id: string, cardType: CardType, name: string, description?: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.createCard(id, cardType, name, description);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				await errorLogging.error(`Failed to create card: ${errorMsg}`, 'architectureStore');
			}
		},

		async deleteCard(id: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.deleteCard(id);
				await this.refresh();
				update((state) => ({
					...state,
					loading: false,
					selectedCardId: state.selectedCardId === id ? null : state.selectedCardId,
				}));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				await errorLogging.error(`Failed to delete card: ${errorMsg}`, 'architectureStore');
			}
		},

		async updateCard(id: string, name?: string, description?: string, status?: CardStatus) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.updateCard(id, name, description, status);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				await errorLogging.error(`Failed to update card: ${errorMsg}`, 'architectureStore');
			}
		},

		async createLink(sourceId: string, targetId?: string, targetUrl?: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.createLink(sourceId, targetId, targetUrl);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				await errorLogging.error(`Failed to create link: ${errorMsg}`, 'architectureStore');
			}
		},

		async deleteLink(sourceId: string, targetId?: string, targetUrl?: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.deleteLink(sourceId, targetId, targetUrl);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				await errorLogging.error(`Failed to delete link: ${errorMsg}`, 'architectureStore');
			}
		},

		async updateLink(sourceId: string, targetId?: string, targetUrl?: string, metadata?: Record<string, unknown>) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.updateLink(sourceId, targetId, targetUrl, metadata);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				await errorLogging.error(`Failed to update link: ${errorMsg}`, 'architectureStore');
			}
		},

		selectCard(id: string | null) {
			update((state) => ({ ...state, selectedCardId: id }));
		},

		async updateMetadata(name?: string, description?: string, rootDriverId?: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.updateMetadata(name, description, rootDriverId);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				await errorLogging.error(`Failed to update metadata: ${errorMsg}`, 'architectureStore');
			}
		},

		clear() {
			set(initialState);
		},
	};
}

export const architecture = createArchitectureStore();

export const allCards: Readable<Card[]> = derived(architecture, (state) => Array.from(state.cards.values()));

export const allLinks: Readable<Link[]> = derived(architecture, (state) => state.links);

export const selectedCard: Readable<Card | null> = derived(architecture, (state) =>
	state.selectedCardId ? state.cards.get(state.selectedCardId) || null : null,
);

export const cardsByType = (type: CardType): Readable<Card[]> =>
	derived(allCards, (cards) => cards.filter((c) => c.type === type));
export const cardsByStatus = (status: CardStatus): Readable<Card[]> =>
	derived(allCards, (cards) => cards.filter((c) => c.status === status));

export const linksFrom = (cardId: string): Readable<Link[]> =>
	derived(allLinks, (links) => links.filter((l) => l.source_id === cardId));

export const linksTo = (cardId: string): Readable<Link[]> =>
	derived(allLinks, (links) => links.filter((l) => l.target_id === cardId));

// Store for initializing a new card with a specific type
export const cardToCreate = writable<{ type: CardType; name?: string } | null>(null);
