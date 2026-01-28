import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

type TauriWindow = Window & {
	__TAURI_INTERNALS__?: unknown;
	__TAURI__?: unknown;
};

export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

export type DiagnosticSeverity = 'Error' | 'Warning' | 'Info';

export interface ValidationDiagnostic {
	severity: DiagnosticSeverity;
	code: string;
	message: string;
	card_id?: string | null;
	path?: string | null;
}

export interface ValidationReport {
	diagnostics: ValidationDiagnostic[];
}

export interface Link {
	target: string;
	relationship: string;
}

export interface AuditEvent {
	editor: string;
	timestamp: string;
	event: string;
	hash?: string | null;
}

export interface AuditTrail {
	version: string;
	hash?: string | null;
	history: AuditEvent[];
}

export interface Card {
	$schema?: string | null;
	id: string;
	card_type: string;
	card_subtype?: string | null;
	name: string;
	description: string;
	status?: string | null;
	links: Link[];
	audit_trail: AuditTrail;
	attributes: JsonValue;
	[extra: string]: JsonValue | undefined;
}

export interface WorkspaceInfo {
	root: string;
	trusted: boolean;
}

export interface ModelHomeInfo {
	root: string;
	has_schema: boolean;
}

export interface CardRecord {
	card: Card;
	source_path: string;
}

export interface ModelSnapshot {
	home: string;
	cards: CardRecord[];
}

export interface RenderSummaryDto {
	cards_written: number;
	views_written: number;
	output_dir: string;
}

export interface RenderRequest {
	model_home: string;
	output_dir: string;
}

export interface CompactRequest {
	model_home: string;
	output_path?: string | null;
}

export type AuditBump = 'patch' | 'minor' | 'major';

export interface CardDraft {
	id: string;
	card_type: string;
	card_subtype?: string | null;
	name: string;
	description: string;
	status?: string | null;
	links: Link[];
	attributes: JsonValue;
	[extra: string]: JsonValue | undefined;
}

export interface CreateCardRequest {
	model_home: string;
	relative_path: string;
	editor: string;
	card: CardDraft;
}

export interface UpdateCardRequest {
	model_home: string;
	relative_path: string;
	editor: string;
	bump?: AuditBump | null;
	card: CardDraft;
}

export interface DeleteCardRequest {
	model_home: string;
	relative_path: string;
	editor: string;
	bump?: AuditBump | null;
}

export type CommandErrorPayload = {
	message?: string;
	error?: string;
	details?: string;
};

export type CommandError = string | CommandErrorPayload;

export const COMMANDS = {
	healthCheck: 'health_check',
	setWorkspace: 'set_workspace',
	workspaceStatus: 'workspace_status',
	discoverModels: 'discover_models',
	loadModelSnapshot: 'load_model_snapshot',
	validateModelSnapshot: 'validate_model_snapshot',
	renderCardMarkdown: 'render_card_markdown',
	renderViewsBundle: 'render_views_bundle',
	renderAllAssets: 'render_all_assets',
	writeCompactExport: 'write_compact_export',
	createCard: 'create_card',
	updateCard: 'update_card',
	deleteCard: 'delete_card'
} as const;

type CommandName = (typeof COMMANDS)[keyof typeof COMMANDS];

type CommandPayloads = {
	[COMMANDS.healthCheck]: { request?: undefined; response: string };
	[COMMANDS.setWorkspace]: {
		request: { root: string; trusted: boolean };
		response: WorkspaceInfo;
	};
	[COMMANDS.workspaceStatus]: { request?: undefined; response: WorkspaceInfo };
	[COMMANDS.discoverModels]: { request?: undefined; response: ModelHomeInfo[] };
	[COMMANDS.loadModelSnapshot]: {
		request: { model_home: string };
		response: ModelSnapshot;
	};
	[COMMANDS.validateModelSnapshot]: {
		request: { model_home: string };
		response: ValidationReport;
	};
	[COMMANDS.renderCardMarkdown]: {
		request: { request: RenderRequest };
		response: RenderSummaryDto;
	};
	[COMMANDS.renderViewsBundle]: {
		request: { request: RenderRequest };
		response: RenderSummaryDto;
	};
	[COMMANDS.renderAllAssets]: {
		request: { request: RenderRequest };
		response: RenderSummaryDto;
	};
	[COMMANDS.writeCompactExport]: {
		request: { request: CompactRequest };
		response: string;
	};
	[COMMANDS.createCard]: {
		request: CreateCardRequest;
		response: CardRecord;
	};
	[COMMANDS.updateCard]: {
		request: UpdateCardRequest;
		response: CardRecord;
	};
	[COMMANDS.deleteCard]: {
		request: DeleteCardRequest;
		response: CardRecord;
	};
};

export function isTauriAvailable(): boolean {
	if (typeof window === 'undefined') {
		return false;
	}
	const tauriWindow = window as TauriWindow;
	return Boolean(tauriWindow.__TAURI_INTERNALS__ ?? tauriWindow.__TAURI__);
}

function assertTauriAvailable(): void {
	if (!isTauriAvailable()) {
		throw new Error('Tauri backend is not available in this session.');
	}
}

async function invokeTauri<Command extends CommandName>(
	command: Command,
	payload?: CommandPayloads[Command]['request']
): Promise<CommandPayloads[Command]['response']> {
	assertTauriAvailable();
	return invoke(command, payload);
}

export async function healthCheck(): Promise<string> {
	return invokeTauri(COMMANDS.healthCheck);
}

export async function setWorkspace(root: string, trusted: boolean): Promise<WorkspaceInfo> {
	return invokeTauri(COMMANDS.setWorkspace, { root, trusted });
}

export async function workspaceStatus(): Promise<WorkspaceInfo> {
	return invokeTauri(COMMANDS.workspaceStatus);
}

export async function discoverModels(): Promise<ModelHomeInfo[]> {
	return invokeTauri(COMMANDS.discoverModels);
}

export async function chooseWorkspaceFolder(): Promise<string | null> {
	assertTauriAvailable();
	const selection = await open({
		title: 'Select workspace folder',
		directory: true,
		recursive: true,
		multiple: false
	});
	if (Array.isArray(selection)) {
		return selection[0] ?? null;
	}
	return typeof selection === 'string' ? selection : null;
}

export async function validateModelSnapshot(modelHome: string): Promise<ValidationReport> {
	return invokeTauri(COMMANDS.validateModelSnapshot, { model_home: modelHome });
}

export async function loadModelSnapshot(modelHome: string): Promise<ModelSnapshot> {
	return invokeTauri(COMMANDS.loadModelSnapshot, { model_home: modelHome });
}

export async function renderCardMarkdown(modelHome: string, outputDir: string): Promise<RenderSummaryDto> {
	return invokeTauri(COMMANDS.renderCardMarkdown, {
		request: {
			model_home: modelHome,
			output_dir: outputDir
		}
	});
}

export async function renderViewsBundle(modelHome: string, outputDir: string): Promise<RenderSummaryDto> {
	return invokeTauri(COMMANDS.renderViewsBundle, {
		request: {
			model_home: modelHome,
			output_dir: outputDir
		}
	});
}

export async function renderAllAssets(modelHome: string, outputDir: string): Promise<RenderSummaryDto> {
	return invokeTauri(COMMANDS.renderAllAssets, {
		request: {
			model_home: modelHome,
			output_dir: outputDir
		}
	});
}

export async function writeCompactExport(modelHome: string, outputPath: string | null): Promise<string> {
	return invokeTauri(COMMANDS.writeCompactExport, {
		request: {
			model_home: modelHome,
			output_path: outputPath
		}
	});
}

export async function createCard(request: CreateCardRequest): Promise<CardRecord> {
	return invokeTauri(COMMANDS.createCard, request);
}

export async function updateCard(request: UpdateCardRequest): Promise<CardRecord> {
	return invokeTauri(COMMANDS.updateCard, request);
}

export async function deleteCard(request: DeleteCardRequest): Promise<CardRecord> {
	return invokeTauri(COMMANDS.deleteCard, request);
}
