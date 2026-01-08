/// TypeScript types for AURORA architecture models
export enum CardType {
	Mission = 'mission',
	Driver = 'driver',
	Requirement = 'requirement',
	Behavior = 'behavior',
	Interface = 'interface',
	Constraint = 'constraint',
	LogicalComponent = 'logical-component',
	DeployableNode = 'deployable-node',
	Actor = 'actor',
	Test = 'test',
	Artifact = 'artifact',
	View = 'view',
	Note = 'note',
}

/// Display names for card types used in UI rendering
export enum CardTypeDisplay {
	Mission = 'Mission',
	Driver = 'Driver',
	Requirement = 'Requirement',
	Behavior = 'Behavior',
	Interface = 'Interface',
	Constraint = 'Constraint',
	LogicalComponent = 'LogicalComponent',
	DeployableNode = 'DeployableNode',
	Actor = 'Actor',
	Test = 'Test',
	Artifact = 'Artifact',
	View = 'View',
	Note = 'Note',
}

/// Mapping from CardType enum to CardTypeDisplay enum
const cardTypeDisplayMap: Record<CardType, CardTypeDisplay> = {
	[CardType.Mission]: CardTypeDisplay.Mission,
	[CardType.Driver]: CardTypeDisplay.Driver,
	[CardType.Requirement]: CardTypeDisplay.Requirement,
	[CardType.Behavior]: CardTypeDisplay.Behavior,
	[CardType.Interface]: CardTypeDisplay.Interface,
	[CardType.Constraint]: CardTypeDisplay.Constraint,
	[CardType.LogicalComponent]: CardTypeDisplay.LogicalComponent,
	[CardType.DeployableNode]: CardTypeDisplay.DeployableNode,
	[CardType.Actor]: CardTypeDisplay.Actor,
	[CardType.Test]: CardTypeDisplay.Test,
	[CardType.Artifact]: CardTypeDisplay.Artifact,
	[CardType.View]: CardTypeDisplay.View,
	[CardType.Note]: CardTypeDisplay.Note,
};

export function toCardTypeDisplay(type: CardType): CardTypeDisplay {
	return cardTypeDisplayMap[type];
}

export function getCardTypesByDisplay(...displays: CardTypeDisplay[]): CardType[] {
	return Object.entries(cardTypeDisplayMap)
		.filter(([, display]) => displays.includes(display))
		.map(([type]) => type as CardType);
}

export enum CardStatus {
	Proposed = 'proposed',
	Approved = 'approved',
	Implemented = 'implemented',
	Verified = 'verified',
	Deprecated = 'deprecated',
	Retired = 'retired',
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
		[CardType.Mission]: 'Mission',
		[CardType.Driver]: 'Driver',
		[CardType.Requirement]: 'Requirement',
		[CardType.Behavior]: 'Behavior',
		[CardType.Interface]: 'Interface',
		[CardType.Constraint]: 'Constraint',
		[CardType.LogicalComponent]: 'Logical Component',
		[CardType.DeployableNode]: 'Deployable Node',
		[CardType.Actor]: 'Actor',
		[CardType.Test]: 'Test',
		[CardType.Artifact]: 'Artifact',
		[CardType.View]: 'View',
		[CardType.Note]: 'Note',
	};
	return labels[type] || type;
}

export function cardStatusLabel(status: CardStatus): string {
	const labels: Record<CardStatus, string> = {
		[CardStatus.Proposed]: 'Proposed',
		[CardStatus.Approved]: 'Approved',
		[CardStatus.Implemented]: 'Implemented',
		[CardStatus.Verified]: 'Verified',
		[CardStatus.Deprecated]: 'Deprecated',
		[CardStatus.Retired]: 'Retired',
	};
	return labels[status] || status;
}
