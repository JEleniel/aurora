import { invoke } from '@tauri-apps/api/core';

export interface RelationshipEntry {
	id: string;
	name: string;
	card_type: string;
	depth: number;
}

export interface ImpactAssessment {
	direct_dependents: number;
	direct_dependencies: number;
	transitive_dependents: number;
	transitive_dependencies: number;
	circular_dependency_risk: boolean;
	total_impact_scope: number;
}

export interface RelationshipAnalysis {
	card_id: string;
	card_name: string;
	upstream: RelationshipEntry[];
	downstream: RelationshipEntry[];
	total_affected: number;
	circular_refs: string[];
	analysis_depth: number;
}

export async function analyzeRelationships(
	cardId: string,
	maxDepth: number = Number.MAX_SAFE_INTEGER
): Promise<RelationshipAnalysis> {
	return invoke<RelationshipAnalysis>('analyze_relationships', {
		card_id: cardId,
		max_depth: maxDepth,
	});
}

export function calculateImpact(analysis: RelationshipAnalysis): ImpactAssessment {
	return {
		direct_dependents: analysis.upstream.filter((r) => r.depth === 1).length,
		direct_dependencies: analysis.downstream.filter((r) => r.depth === 1).length,
		transitive_dependents: analysis.upstream.filter((r) => r.depth > 1).length,
		transitive_dependencies: analysis.downstream.filter((r) => r.depth > 1).length,
		circular_dependency_risk: analysis.circular_refs.length > 0,
		total_impact_scope: analysis.total_affected,
	};
}
