/// TypeScript types for AURORA architecture models
export enum CardType {
	Driver = "driver",
	Requirement = "requirement",
	Behavior = "behavior",
	Interface = "interface",
	Constraint = "constraint",
	LogicalComponent = "logical-component",
	DeployableNode = "deployable-node",
	Actor = "actor",
	Test = "test",
	Artifact = "artifact",
	View = "view",
	Note = "note",
}

export enum CardStatus {
	Proposed = "proposed",
	Approved = "approved",
	Implemented = "implemented",
	Verified = "verified",
	Deprecated = "deprecated",
	Retired = "retired",
}

export interface Card {
	id: string;
	type: CardType;
	name: string;
	description?: string;
	version?: string;
	status?: CardStatus;
	attributes?: Record<string, unknown>;
	created_at: string;
	modified_at: string;
}

export interface LinkMetadata {
	weight?: number;
	confidence?: number;
	view_context?: string;
	link_title?: string;
}

export interface Link {
	source_id: string;
	target_id?: string | null;
	target_url?: string | null;
	metadata?: LinkMetadata;
	created_at: string;
	modified_at: string;
}

export interface ProjectMetadata {
	name?: string;
	description?: string;
	version?: string;
	root_driver_id?: string;
}

export interface ArchitectureModel {
	cards: Record<string, Card>;
	links: Link[];
	metadata: ProjectMetadata;
}

export interface ModelStatistics {
	total_cards: number;
	total_links: number;
	cards_by_type: Record<CardType, number>;
	cards_by_status: Record<CardStatus, number>;
}

export function cardTypeFolder(type: CardType): string {
	return type as string;
}

export function cardTypeLabel(type: CardType): string {
	const labels: Record<CardType, string> = {
		[CardType.Driver]: "Driver",
		[CardType.Requirement]: "Requirement",
		[CardType.Behavior]: "Behavior",
		[CardType.Interface]: "Interface",
		[CardType.Constraint]: "Constraint",
		[CardType.LogicalComponent]: "Logical Component",
		[CardType.DeployableNode]: "Deployable Node",
		[CardType.Actor]: "Actor",
		[CardType.Test]: "Test",
		[CardType.Artifact]: "Artifact",
		[CardType.View]: "View",
		[CardType.Note]: "Note",
	};
	return labels[type] || type;
}

export function cardStatusLabel(status: CardStatus): string {
	const labels: Record<CardStatus, string> = {
		[CardStatus.Proposed]: "Proposed",
		[CardStatus.Approved]: "Approved",
		[CardStatus.Implemented]: "Implemented",
		[CardStatus.Verified]: "Verified",
		[CardStatus.Deprecated]: "Deprecated",
		[CardStatus.Retired]: "Retired",
	};
	return labels[status] || status;
}
