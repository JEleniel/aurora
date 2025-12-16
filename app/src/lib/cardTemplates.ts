import type { Card, CardType, CardStatus } from '$lib/types';
import { CardType as CardTypeEnum, CardStatus as CardStatusEnum } from '$lib/types';

export interface CardTemplate {
	name: string;
	description: string;
	tags: string[];
	icon: string;
}

export const cardTemplates: Record<CardType, CardTemplate[]> = {
	[CardTypeEnum.Driver]: [
		{
			name: 'Business Driver',
			description: 'Capture a business need or goal',
			tags: ['business', 'strategic'],
			icon: '🎯',
		},
		{
			name: 'Technical Driver',
			description: 'Technical constraint or requirement',
			tags: ['technical', 'architecture'],
			icon: '⚙️',
		},
		{
			name: 'Compliance Driver',
			description: 'Regulatory or policy requirement',
			tags: ['compliance', 'governance'],
			icon: '📋',
		},
	],
	[CardTypeEnum.Requirement]: [
		{
			name: 'Functional Requirement',
			description: 'What the system must do',
			tags: ['functional', 'capability'],
			icon: '✨',
		},
		{
			name: 'Non-Functional Requirement',
			description: 'Quality attributes (performance, security, etc.)',
			tags: ['quality', 'performance'],
			icon: '⚡',
		},
		{
			name: 'User Requirement',
			description: 'User-facing capability or workflow',
			tags: ['user', 'ux'],
			icon: '👤',
		},
	],
	[CardTypeEnum.Behavior]: [
		{
			name: 'System Behavior',
			description: 'Observable behavior of the system',
			tags: ['behavior', 'action'],
			icon: '🔄',
		},
		{
			name: 'Business Process',
			description: 'Workflow or business process',
			tags: ['process', 'workflow'],
			icon: '📊',
		},
		{
			name: 'Integration Flow',
			description: 'Data or message flow between systems',
			tags: ['integration', 'api'],
			icon: '🔗',
		},
	],
	[CardTypeEnum.Interface]: [
		{
			name: 'API Interface',
			description: 'REST, GraphQL, or service interface',
			tags: ['api', 'integration'],
			icon: '🔌',
		},
		{
			name: 'User Interface',
			description: 'UI component or screen',
			tags: ['ui', 'frontend'],
			icon: '🖥️',
		},
		{
			name: 'Message Queue',
			description: 'Asynchronous messaging interface',
			tags: ['messaging', 'async'],
			icon: '📮',
		},
	],
	[CardTypeEnum.Constraint]: [
		{
			name: 'Performance Constraint',
			description: 'Latency, throughput, or resource limits',
			tags: ['performance', 'quality'],
			icon: '⚡',
		},
		{
			name: 'Security Constraint',
			description: 'Security or compliance requirement',
			tags: ['security', 'compliance'],
			icon: '🔒',
		},
		{
			name: 'Deployment Constraint',
			description: 'Infrastructure or deployment requirement',
			tags: ['deployment', 'infrastructure'],
			icon: '📦',
		},
	],
	[CardTypeEnum.LogicalComponent]: [
		{
			name: 'Service Component',
			description: 'Microservice or logical service',
			tags: ['service', 'component'],
			icon: '📦',
		},
		{
			name: 'Data Component',
			description: 'Data store or database',
			tags: ['data', 'storage'],
			icon: '💾',
		},
		{
			name: 'Integration Component',
			description: 'Adapter or integration layer',
			tags: ['integration', 'adapter'],
			icon: '🔄',
		},
	],
	[CardTypeEnum.DeployableNode]: [
		{
			name: 'Container',
			description: 'Docker container or artifact',
			tags: ['container', 'deployment'],
			icon: '🐳',
		},
		{
			name: 'Virtual Machine',
			description: 'VM or compute instance',
			tags: ['infrastructure', 'compute'],
			icon: '💻',
		},
		{
			name: 'Server Node',
			description: 'Physical or logical server',
			tags: ['server', 'infrastructure'],
			icon: '🖥️',
		},
	],
	[CardTypeEnum.Actor]: [
		{
			name: 'End User',
			description: 'System user or user role',
			tags: ['user', 'stakeholder'],
			icon: '👤',
		},
		{
			name: 'System Administrator',
			description: 'Operator or system maintainer',
			tags: ['operator', 'admin'],
			icon: '👨‍💼',
		},
		{
			name: 'External System',
			description: 'Third-party or external system',
			tags: ['external', 'integration'],
			icon: '🔗',
		},
	],
	[CardTypeEnum.Test]: [
		{
			name: 'Unit Test',
			description: 'Component-level test',
			tags: ['unit', 'testing'],
			icon: '🧪',
		},
		{
			name: 'Integration Test',
			description: 'Cross-component test',
			tags: ['integration', 'testing'],
			icon: '🔗',
		},
		{
			name: 'End-to-End Test',
			description: 'Full workflow test',
			tags: ['e2e', 'testing'],
			icon: '✅',
		},
	],
	[CardTypeEnum.Artifact]: [
		{
			name: 'Document',
			description: 'Design or specification document',
			tags: ['documentation', 'design'],
			icon: '📄',
		},
		{
			name: 'Code Repository',
			description: 'Source code or artifact repository',
			tags: ['code', 'repository'],
			icon: '📦',
		},
		{
			name: 'Data File',
			description: 'Configuration, schema, or data file',
			tags: ['data', 'config'],
			icon: '📋',
		},
	],
	[CardTypeEnum.View]: [
		{
			name: 'Architecture Diagram',
			description: 'System architecture or component diagram',
			tags: ['diagram', 'visualization'],
			icon: '📐',
		},
		{
			name: 'Process Flow',
			description: 'Process or sequence diagram',
			tags: ['process', 'flow'],
			icon: '📊',
		},
		{
			name: 'Data Model',
			description: 'Entity-relationship or data diagram',
			tags: ['data', 'model'],
			icon: '📊',
		},
	],
	[CardTypeEnum.Note]: [
		{
			name: 'Technical Note',
			description: 'Technical discussion or decision',
			tags: ['technical', 'note'],
			icon: '📝',
		},
		{
			name: 'Meeting Minutes',
			description: 'Meeting notes or action items',
			tags: ['meeting', 'note'],
			icon: '📋',
		},
		{
			name: 'Rationale',
			description: 'Design rationale or justification',
			tags: ['rationale', 'decision'],
			icon: '💡',
		},
	],
};

/**
 * Get template suggestions for a card type
 */
export function getTemplatesForType(cardType: CardType): CardTemplate[] {
	return cardTemplates[cardType] || [];
}

/**
 * Apply template to populate card fields
 */
export function applyTemplate(cardType: CardType, templateName: string): Partial<Card> {
	const template = getTemplatesForType(cardType).find((t) => t.name === templateName);

	if (!template) {
		return {};
	}

	return {
		name: templateName,
		description: template.description,
		status: CardStatusEnum.Proposed as CardStatus,
		attributes: {
			tags: template.tags,
			icon: template.icon,
		},
	};
}
