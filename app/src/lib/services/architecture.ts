/// Service for interacting with architecture backend
import { invoke } from '@tauri-apps/api/core';
import type { Card, CardStatus, CardType, Link, ModelStatistics, ProjectMetadata } from '../types';
import { errorLogging } from './errorLogging';

/**
 * Parse error response from backend (either string or JSON AppError)
 */
function parseBackendError(error: unknown): { userMessage: string; technicalMessage: string } {
	if (typeof error === 'string') {
		try {
			const parsed = JSON.parse(error);
			if (parsed.user_message && parsed.technical_message) {
				return {
					userMessage: parsed.user_message,
					technicalMessage: parsed.technical_message,
				};
			}
		} catch {
			// Not JSON, treat as plain message
		}
		return {
			userMessage: error,
			technicalMessage: error,
		};
	}

	if (error instanceof Error) {
		return {
			userMessage: error.message,
			technicalMessage: error.stack || error.message,
		};
	}

	return {
		userMessage: 'An unexpected error occurred',
		technicalMessage: String(error),
	};
}

export async function loadArchitecture(path: string): Promise<string> {
	try {
		return await invoke('load_architecture', { path });
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'loadArchitecture', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function saveArchitecture(path: string): Promise<string> {
	try {
		const result = await invoke<string>('save_architecture', { path });
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'saveArchitecture', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function createCard(id: string, cardType: CardType, name: string, description?: string): Promise<string> {
	try {
		const result = await invoke<string>('create_card', {
			id,
			card_type: cardType,
			name,
			description,
		});
		await errorLogging.info(`Card '${id}' created successfully`, 'createCard');
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'createCard', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function deleteCard(id: string): Promise<string> {
	try {
		const result = await invoke<string>('delete_card', { id });
		await errorLogging.info(`Card '${id}' deleted successfully`, 'deleteCard');
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'deleteCard', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function getCards(): Promise<Card[]> {
	try {
		const result = await invoke<string>('get_cards', {});
		return JSON.parse(result);
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error('Failed to retrieve cards', 'getCards', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function getCardsByType(cardType: CardType): Promise<Card[]> {
	try {
		const result = await invoke<string>('get_cards_by_type', {
			card_type: cardType,
		});
		return JSON.parse(result);
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(`Failed to retrieve ${cardType} cards`, 'getCardsByType', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function updateCard(
	id: string,
	name?: string,
	description?: string,
	status?: CardStatus,
): Promise<string> {
	try {
		const result = await invoke<string>('update_card', {
			id,
			name,
			description,
			status,
		});
		await errorLogging.info(`Card '${id}' updated successfully`, 'updateCard');
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'updateCard', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function createLink(sourceId: string, targetId?: string, targetUrl?: string): Promise<string> {
	if (!targetId && !targetUrl) {
		const err = new Error('Either targetId or targetUrl must be provided');
		await errorLogging.error('Invalid link parameters', 'createLink', err);
		throw err;
	}

	try {
		const result = await invoke<string>('create_link', {
			source_id: sourceId,
			target_id: targetId,
			target_url: targetUrl,
		});
		await errorLogging.info(`Link created from ${sourceId} to ${targetId || targetUrl}`, 'createLink');
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'createLink', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function deleteLink(sourceId: string, targetId?: string, targetUrl?: string): Promise<string> {
	if (!targetId && !targetUrl) {
		const err = new Error('Either targetId or targetUrl must be provided');
		await errorLogging.error('Invalid link parameters', 'deleteLink', err);
		throw err;
	}

	try {
		const result = await invoke<string>('delete_link', {
			source_id: sourceId,
			target_id: targetId,
			target_url: targetUrl,
		});
		await errorLogging.info(`Link deleted from ${sourceId} to ${targetId || targetUrl}`, 'deleteLink');
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'deleteLink', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function updateLink(
	sourceId: string,
	targetId: string | undefined,
	targetUrl: string | undefined,
	metadata?: Record<string, unknown>,
): Promise<string> {
	if (!targetId && !targetUrl) {
		const err = new Error('Either targetId or targetUrl must be provided');
		await errorLogging.error('Invalid link parameters', 'updateLink', err);
		throw err;
	}

	try {
		const result = await invoke<string>('update_link', {
			source_id: sourceId,
			target_id: targetId,
			target_url: targetUrl,
			metadata,
		});
		await errorLogging.info(`Link updated from ${sourceId} to ${targetId || targetUrl}`, 'updateLink');
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'updateLink', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function getLinks(): Promise<Link[]> {
	try {
		const result = await invoke<string>('get_links', {});
		return JSON.parse(result);
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error('Failed to retrieve links', 'getLinks', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function getStatistics(): Promise<ModelStatistics> {
	try {
		const result = await invoke<string>('get_statistics', {});
		return JSON.parse(result);
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error('Failed to retrieve statistics', 'getStatistics', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function getMetadata(): Promise<ProjectMetadata> {
	try {
		const result = await invoke<string>('get_metadata', {});
		return JSON.parse(result);
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error('Failed to retrieve metadata', 'getMetadata', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function updateMetadata(name?: string, description?: string, rootDriverId?: string): Promise<string> {
	try {
		const result = await invoke<string>('update_metadata', {
			name,
			description,
			root_driver_id: rootDriverId,
		});
		await errorLogging.info('Model metadata updated successfully', 'updateMetadata');
		return result;
	} catch (error) {
		const { userMessage, technicalMessage } = parseBackendError(error);
		await errorLogging.error(userMessage, 'updateMetadata', new Error(technicalMessage));
		throw new Error(userMessage);
	}
}

export async function validateCard(card: Card): Promise<string> {
	try {
		const result = await invoke<string>('validate_card', { card });
		await errorLogging.debug(`Card "${card.name}" validated successfully`, 'validateCard');
		return result;
	} catch (error) {
		const { userMessage } = parseBackendError(error);
		await errorLogging.debug(`Card "${card.name}" validation failed: ${userMessage}`, 'validateCard');
		throw new Error(userMessage);
	}
}

export async function generateTraceabilityMatrix(sourceType: string, targetType: string): Promise<unknown> {
	try {
		const result = await invoke<string>('generate_traceability_matrix', {
			source_type: sourceType,
			target_type: targetType,
		});
		await errorLogging.info(
			`Traceability matrix generated: ${sourceType} -> ${targetType}`,
			'generateTraceabilityMatrix',
		);
		return JSON.parse(result);
	} catch (error) {
		const { userMessage } = parseBackendError(error);
		await errorLogging.error(
			`Failed to generate traceability matrix: ${userMessage}`,
			'generateTraceabilityMatrix',
			error instanceof Error ? error : new Error(String(error)),
		);
		throw new Error(userMessage);
	}
}

export async function generateDependencyGraph(): Promise<unknown> {
	try {
		const result = await invoke<string>('generate_dependency_graph');
		await errorLogging.info('Dependency graph generated successfully', 'generateDependencyGraph');
		return JSON.parse(result);
	} catch (error) {
		const { userMessage } = parseBackendError(error);
		await errorLogging.error(
			`Failed to generate dependency graph: ${userMessage}`,
			'generateDependencyGraph',
			error instanceof Error ? error : new Error(String(error)),
		);
		throw new Error(userMessage);
	}
}
