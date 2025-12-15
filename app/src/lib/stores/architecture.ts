/// Svelte store for architecture state
import { writable, derived, type Readable } from "svelte/store";
import type { Card, CardStatus, CardType, Link, ModelStatistics, ProjectMetadata } from "../types";
import * as archService from "../services/architecture";

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
			version: "1.0.0",
		},
		statistics: null,
		selectedCardId: null,
		loading: false,
		error: null,
	};

	const { subscribe, set, update } = writable<ArchitectureState>(initialState);

	return {
		subscribe,

		// Load architecture from ZIP
		async loadFromZip(path: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.loadArchitecture(path);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				throw error;
			}
		},

		// Save architecture to ZIP
		async saveToZip(path: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.saveArchitecture(path);
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				throw error;
			}
		},

		// Refresh all data from backend
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
				throw error;
			}
		},

		// Create card
		async createCard(
			id: string,
			cardType: CardType,
			name: string,
			description?: string,
		) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.createCard(id, cardType, name, description);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				throw error;
			}
		},

		// Delete card
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
				throw error;
			}
		},

		// Update card
		async updateCard(
			id: string,
			name?: string,
			description?: string,
			status?: CardStatus,
		) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.updateCard(id, name, description, status);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				throw error;
			}
		},

		// Create link
		async createLink(sourceId: string, targetId?: string, targetUrl?: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.createLink(sourceId, targetId, targetUrl);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				throw error;
			}
		},

		// Select card
		selectCard(id: string | null) {
			update((state) => ({ ...state, selectedCardId: id }));
		},

		// Update metadata
		async updateMetadata(name?: string, description?: string, rootDriverId?: string) {
			update((state) => ({ ...state, loading: true, error: null }));
			try {
				await archService.updateMetadata(name, description, rootDriverId);
				await this.refresh();
				update((state) => ({ ...state, loading: false }));
			} catch (error) {
				const errorMsg = error instanceof Error ? error.message : String(error);
				update((state) => ({ ...state, loading: false, error: errorMsg }));
				throw error;
			}
		},

		// Clear state
		clear() {
			set(initialState);
		},
	};
}

export const architecture = createArchitectureStore();

// Derived stores
export const allCards: Readable<Card[]> = derived(architecture, (state) =>
	Array.from(state.cards.values()),
);

export const allLinks: Readable<Link[]> = derived(
	architecture,
	(state) => state.links,
);

export const selectedCard: Readable<Card | null> = derived(
	architecture,
	(state) => (state.selectedCardId ? state.cards.get(state.selectedCardId) || null : null),
);

export const cardsByType = (type: CardType): Readable<Card[]> =>
	derived(allCards, (cards) => cards.filter((c) => c.type === type));

export const cardsByStatus = (status: CardStatus): Readable<Card[]> =>
	derived(allCards, (cards) => cards.filter((c) => c.status === status));

export const linksFrom = (cardId: string): Readable<Link[]> =>
	derived(allLinks, (links) => links.filter((l) => l.source_id === cardId));

export const linksTo = (cardId: string): Readable<Link[]> =>
	derived(allLinks, (links) =>
		links.filter((l) => l.target_id === cardId),
	);
