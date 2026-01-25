import { invoke } from '@tauri-apps/api/core';

type TauriWindow = Window & {
	__TAURI_INTERNALS__?: unknown;
	__TAURI__?: unknown;
};

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

export interface WorkspaceInfo {
	root: string;
	trusted: boolean;
}

export interface ModelHomeInfo {
	root: string;
	has_schema: boolean;
}

export interface RenderSummaryDto {
	cards_written: number;
	views_written: number;
	output_dir: string;
}

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

async function invokeTauri<T>(command: string, payload?: Record<string, unknown>): Promise<T> {
	assertTauriAvailable();
	return invoke<T>(command, payload);
}

export async function healthCheck(): Promise<string> {
	return invokeTauri('health_check');
}

export async function setWorkspace(root: string, trusted: boolean): Promise<WorkspaceInfo> {
	return invokeTauri('set_workspace', { root, trusted });
}

export async function workspaceStatus(): Promise<WorkspaceInfo> {
	return invokeTauri('workspace_status');
}

export async function discoverModels(): Promise<ModelHomeInfo[]> {
	const result = await invokeTauri<unknown>('discover_models');
	const payload =
		Array.isArray(result) ? result
		: result && typeof result === 'object' && Array.isArray((result as Record<string, unknown>)['models']) ?
			((result as Record<string, unknown>)['models'] as unknown[])
		:	[];
	if (payload.length === 0) {
		return [];
	}
	return payload
		.map((entry) => {
			const record = entry as Record<string, unknown>;
			const root =
				typeof record['root'] === 'string' ? (record['root'] as string)
				: typeof record['rootPath'] === 'string' ? (record['rootPath'] as string)
				: typeof record['root_path'] === 'string' ? (record['root_path'] as string)
				: '';
			const hasSchema =
				typeof record['has_schema'] === 'boolean' ? (record['has_schema'] as boolean)
				: typeof record['hasSchema'] === 'boolean' ? (record['hasSchema'] as boolean)
				: false;
			return {
				root,
				has_schema: hasSchema,
			};
		})
		.filter((entry) => entry.root.length > 0);
}

export async function validateModelSnapshot(modelHome: string): Promise<ValidationReport> {
	return invokeTauri('validate_model_snapshot', { modelHome });
}

export async function renderAllAssets(modelHome: string, outputDir: string): Promise<RenderSummaryDto> {
	return invokeTauri('render_all_assets', {
		request: {
			modelHome,
			outputDir,
		},
	});
}

export async function writeCompactExport(modelHome: string, outputPath: string | null): Promise<string> {
	return invokeTauri('write_compact_export', {
		request: {
			modelHome,
			outputPath,
		},
	});
}
