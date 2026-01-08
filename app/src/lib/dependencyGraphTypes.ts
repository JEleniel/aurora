/// Dependency Graph types
export interface GraphNode {
	id: string;
	name: string;
	card_type: string;
	group: number;
}

export interface GraphLink {
	source: string;
	target: string;
	distance: number;
}

export interface DependencyGraph {
	nodes: GraphNode[];
	links: GraphLink[];
	node_count: number;
	link_count: number;
	density: number;
}

export interface GraphMetrics {
	node_count: number;
	link_count: number;
	density: number;
	isolated_node_count: number;
	root_node_count: number;
	leaf_node_count: number;
	avg_degree: number;
	max_degree: number;
}

/// Helper to convert backend response to TypeScript format
export function parseDependencyGraph(data: unknown): DependencyGraph {
	const json = data as Record<string, unknown>;
	return {
		nodes: (json['nodes'] as Array<Record<string, unknown>>).map((node) => ({
			id: String(node['id']),
			name: String(node['name']),
			card_type: String(node['card_type']),
			group: Number(node['group']),
		})),
		links: (json['links'] as Array<Record<string, unknown>>).map((link) => ({
			source: String(link['source']),
			target: String(link['target']),
			distance: Number(link['distance']),
		})),
		node_count: Number(json['node_count']),
		link_count: Number(json['link_count']),
		density: Number(json['density']),
	};
}
