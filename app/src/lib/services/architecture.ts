/// Service for interacting with architecture backend
import { invoke } from '@tauri-apps/api/core';
import type { Card, CardStatus, CardType, Link, ModelStatistics, ProjectMetadata } from '../types';

export async function loadArchitecture(path: string): Promise<string> {
	return await invoke('load_architecture', { path });
}

export async function saveArchitecture(path: string): Promise<string> {
	return await invoke('save_architecture', { path });
}

export async function createCard(id: string, cardType: CardType, name: string, description?: string): Promise<string> {
	return await invoke('create_card', {
		id,
		card_type: cardType,
		name,
		description,
	});
}

export async function deleteCard(id: string): Promise<string> {
	return await invoke('delete_card', { id });
}

export async function getCards(): Promise<Card[]> {
	const result = await invoke<string>('get_cards', {});
	return JSON.parse(result);
}

export async function getCardsByType(cardType: CardType): Promise<Card[]> {
	const result = await invoke<string>('get_cards_by_type', {
		card_type: cardType,
	});
	return JSON.parse(result);
}

export async function updateCard(
	id: string,
	name?: string,
	description?: string,
	status?: CardStatus,
): Promise<string> {
	return await invoke('update_card', {
		id,
		name,
		description,
		status,
	});
}

export async function createLink(sourceId: string, targetId?: string, targetUrl?: string): Promise<string> {
	if (!targetId && !targetUrl) {
		throw new Error('Either targetId or targetUrl must be provided');
	}
	return await invoke('create_link', {
		source_id: sourceId,
		target_id: targetId,
		target_url: targetUrl,
	});
}

export async function getLinks(): Promise<Link[]> {
	const result = await invoke<string>('get_links', {});
	return JSON.parse(result);
}

export async function getStatistics(): Promise<ModelStatistics> {
	const result = await invoke<string>('get_statistics', {});
	return JSON.parse(result);
}

export async function getMetadata(): Promise<ProjectMetadata> {
	const result = await invoke<string>('get_metadata', {});
	return JSON.parse(result);
}

export async function updateMetadata(name?: string, description?: string, rootDriverId?: string): Promise<string> {
	return await invoke('update_metadata', {
		name,
		description,
		root_driver_id: rootDriverId,
	});
}
