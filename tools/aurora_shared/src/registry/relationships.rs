pub struct RelationshipDefinition {
	pub relationship: &'static str,
	pub description: &'static str,
	pub source_card_types: &'static [&'static str],
	pub target_card_types: &'static [&'static str],
}

impl RelationshipDefinition {
	pub fn try_get_by_relationship(relationship: &str) -> Option<&'static RelationshipDefinition> {
		Self::DEFINITIONS
			.iter()
			.find(|def| def.relationship == relationship)
	}

	pub fn try_get_by_source_card_type(
		source_card_type: &str,
	) -> Vec<&'static RelationshipDefinition> {
		Self::DEFINITIONS
			.iter()
			.filter(|def| {
				def.source_card_types.contains(&source_card_type)
					|| def.source_card_types.contains(&"*")
			})
			.collect()
	}

	pub fn try_get_by_target_card_type(
		target_card_type: &str,
	) -> Vec<&'static RelationshipDefinition> {
		Self::DEFINITIONS
			.iter()
			.filter(|def| {
				def.target_card_types.contains(&target_card_type)
					|| def.target_card_types.contains(&"*")
			})
			.collect()
	}

	pub fn get_all() -> &'static [RelationshipDefinition] {
		Self::DEFINITIONS
	}
}

impl RelationshipDefinition {
	const DEFINITIONS: &'static [RelationshipDefinition] = &[
		RelationshipDefinition {
			relationship: "invokes",
			description: "Invokes an interface by contract.",
			source_card_types: &["Component"],
			target_card_types: &["Interface"],
		},
		RelationshipDefinition {
			relationship: "composes",
			description: "Defines a composition relationship.",
			source_card_types: &["Application"],
			target_card_types: &["Component"],
		},
		RelationshipDefinition {
			relationship: "contains",
			description: "Defines boundary containment.",
			source_card_types: &["Boundary"],
			target_card_types: &["!Mission"],
		},
		RelationshipDefinition {
			relationship: "provisions",
			description: "Provisions runtime infrastructure.",
			source_card_types: &["Deployment"],
			target_card_types: &["Node"],
		},
		RelationshipDefinition {
			relationship: "desires",
			description: "Expresses a goal or story.",
			source_card_types: &["Actor"],
			target_card_types: &["Story"],
		},
		RelationshipDefinition {
			relationship: "documents",
			description: "Records a decision tied to a requirement.",
			source_card_types: &["ADR"],
			target_card_types: &["Requirement"],
		},
		RelationshipDefinition {
			relationship: "drives",
			description: "Influences a downstream requirement.",
			source_card_types: &["Driver"],
			target_card_types: &["Requirement"],
		},
		RelationshipDefinition {
			relationship: "enables",
			description: "Makes a capability feasible.",
			source_card_types: &["Feature"],
			target_card_types: &["Capability"],
		},
		RelationshipDefinition {
			relationship: "establishes",
			description: "Introduces a downstream driver.",
			source_card_types: &["Mission"],
			target_card_types: &["Driver"],
		},
		RelationshipDefinition {
			relationship: "exposes",
			description: "Publishes an interface.",
			source_card_types: &["Data Store"],
			target_card_types: &["Interface"],
		},
		RelationshipDefinition {
			relationship: "generates",
			description: "Produces an artifact.",
			source_card_types: &["Component"],
			target_card_types: &["Artifact"],
		},
		RelationshipDefinition {
			relationship: "instantiates",
			description: "Creates a runtime instance.",
			source_card_types: &["Node"],
			target_card_types: &["Node Instance"],
		},
		RelationshipDefinition {
			relationship: "hosts",
			description: "Hosts a runtime component.",
			source_card_types: &["Node"],
			target_card_types: &["Component"],
		},
		RelationshipDefinition {
			relationship: "hosts",
			description: "Hosts a runtime data store.",
			source_card_types: &["Node"],
			target_card_types: &["Data Store"],
		},
		RelationshipDefinition {
			relationship: "implements",
			description: "Implements a feature.",
			source_card_types: &["Component"],
			target_card_types: &["Feature"],
		},
		RelationshipDefinition {
			relationship: "realizes",
			description: "Realizes a class definition.",
			source_card_types: &["Component"],
			target_card_types: &["Class"],
		},
		RelationshipDefinition {
			relationship: "fulfills",
			description: "Fulfills a test.",
			source_card_types: &["Component"],
			target_card_types: &["Test"],
		},
		RelationshipDefinition {
			relationship: "enforces",
			description: "Applies a control.",
			source_card_types: &["Component", "System"],
			target_card_types: &["Control"],
		},
		RelationshipDefinition {
			relationship: "implies",
			description: "Implies a constraint.",
			source_card_types: &["Story"],
			target_card_types: &["Constraint"],
		},
		RelationshipDefinition {
			relationship: "includes",
			description: "Scopes an element within a boundary.",
			source_card_types: &["*"],
			target_card_types: &["Boundary"],
		},
		RelationshipDefinition {
			relationship: "integrates",
			description: "Integrates applications into a system.",
			source_card_types: &["System"],
			target_card_types: &["Application"],
		},
		RelationshipDefinition {
			relationship: "involves",
			description: "Includes an actor in a mission.",
			source_card_types: &["Mission"],
			target_card_types: &["Actor"],
		},
		RelationshipDefinition {
			relationship: "is",
			description: "Classifies an artifact as an asset.",
			source_card_types: &["Artifact"],
			target_card_types: &["Asset"],
		},
		RelationshipDefinition {
			relationship: "mitigates",
			description: "Reduces a risk.",
			source_card_types: &["Control"],
			target_card_types: &["Risk"],
		},
		RelationshipDefinition {
			relationship: "requires",
			description: "Requires a system or application.",
			source_card_types: &["Mission"],
			target_card_types: &["System", "Application"],
		},
		RelationshipDefinition {
			relationship: "owns",
			description: "Defines ownership responsibility.",
			source_card_types: &["Actor"],
			target_card_types: &["Asset"],
		},
		RelationshipDefinition {
			relationship: "performs",
			description: "Executes an activity.",
			source_card_types: &["Actor"],
			target_card_types: &["Activity"],
		},
		RelationshipDefinition {
			relationship: "persists",
			description: "Persists an artifact to storage.",
			source_card_types: &["Artifact"],
			target_card_types: &["Data Store"],
		},
		RelationshipDefinition {
			relationship: "presents",
			description: "Introduces a threat.",
			source_card_types: &["Actor"],
			target_card_types: &["Threat"],
		},
		RelationshipDefinition {
			relationship: "raises",
			description: "Raises a risk.",
			source_card_types: &["Threat"],
			target_card_types: &["Risk"],
		},
		RelationshipDefinition {
			relationship: "safeguards",
			description: "Protects an asset.",
			source_card_types: &["Control"],
			target_card_types: &["Asset"],
		},
		RelationshipDefinition {
			relationship: "necessitates",
			description: "Necessitates a process.",
			source_card_types: &["Capability"],
			target_card_types: &["Process"],
		},
		RelationshipDefinition {
			relationship: "executes",
			description: "Executes and owns a state machine.",
			source_card_types: &["Component"],
			target_card_types: &["State Machine"],
		},
		RelationshipDefinition {
			relationship: "satisfies",
			description: "Satisfies a requirement.",
			source_card_types: &["Capability"],
			target_card_types: &["Requirement"],
		},
		RelationshipDefinition {
			relationship: "starts in",
			description: "Defines the initial state.",
			source_card_types: &["State Machine"],
			target_card_types: &["State"],
		},
		RelationshipDefinition {
			relationship: "transitions to",
			description: "Transitions to a next state.",
			source_card_types: &["State", "Predicate", "Event"],
			target_card_types: &["State"],
		},
		RelationshipDefinition {
			relationship: "evaluates",
			description: "Evaluates a condition.",
			source_card_types: &["Activity"],
			target_card_types: &["Condition"],
		},
		RelationshipDefinition {
			relationship: "emits",
			description: "Emits an event.",
			source_card_types: &["Activity"],
			target_card_types: &["Event"],
		},
		RelationshipDefinition {
			relationship: "leads",
			description: "Leads to another activity.",
			source_card_types: &["Activity"],
			target_card_types: &["Activity"],
		},
	];
}
