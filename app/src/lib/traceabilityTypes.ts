/// Traceability Matrix types and utilities
export interface MatrixCell {
	sourceId: string;
	targetId: string;
	linkCount: number;
	hasLink: boolean;
}

export interface MatrixRow {
	id: string;
	name: string;
	cardType: string;
	cells: MatrixCell[];
	totalLinks: number;
}

export interface TraceabilityMatrix {
	targetType: string;
	sourceType: string;
	rows: MatrixRow[];
	columns: string[];
	coverage: number;
	orphanedSources: string[];
	unreferencedTargets: string[];
}

export interface GapAnalysis {
	gapType: string;
	sourceId: string;
	targetId: string | null;
	severity: 'high' | 'medium' | 'low';
	recommendation: string;
}

/// Helper to convert backend response to TypeScript format
export function parseTraceabilityMatrix(data: unknown): TraceabilityMatrix {
	const json = data as Record<string, unknown>;
	return {
		targetType: String(json['target_type']),
		sourceType: String(json['source_type']),
		rows: (json['rows'] as Array<Record<string, unknown>>).map((row) => ({
			id: String(row['id']),
			name: String(row['name']),
			cardType: String(row['card_type']),
			cells: (row['cells'] as Array<Record<string, unknown>>).map((cell) => ({
				sourceId: String(cell['source_id']),
				targetId: String(cell['target_id']),
				linkCount: Number(cell['link_count']),
				hasLink: Boolean(cell['has_link']),
			})),
			totalLinks: Number(row['total_links']),
		})),
		columns: json['columns'] as string[],
		coverage: Number(json['coverage']),
		orphanedSources: json['orphaned_sources'] as string[],
		unreferencedTargets: json['unreferenced_targets'] as string[],
	};
}
