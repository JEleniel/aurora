pub const CARD_PREFIXES: [(&str, &str); 30] = [
	("ACT", "Actor"),
	("APP", "Application"),
	("ART", "Artifact"),
	("AST", "Asset"),
	("ATV", "Activity"),
	("BND", "Boundary"),
<<<<<<< HEAD
=======
	("DRI", "Driver"),
>>>>>>> bd934f2a22ba6c59dcf4fd3c5b2629e9f3d72b35
	("CAP", "Capability"),
	("CNS", "Constraint"),
	("COM", "Component"),
	("CON", "Condition"),
	("CTL", "Control"),
	("DEP", "Deployment"),
	("DTS", "Data Store"),
	("EVT", "Event"),
	("FEA", "Feature"),
	("INT", "Interface"),
	("MIS", "Mission"),
	("NIN", "Node Instance"),
	("NOD", "Node"),
	("NOT", "Note"),
	("PRO", "Process"),
	("REQ", "Requirement"),
	("RIS", "Risk"),
	("STA", "State"),
	("STM", "State Machine"),
	("STR", "Story"),
	("SYS", "System"),
	("TES", "Test"),
	("THR", "Threat"),
];

pub fn get_card_type(key: &str) -> Option<String> {
	CARD_PREFIXES
		.iter()
		.find(|(k, _)| *k == key)
		.map(|(_, v)| v.to_string())
}

pub fn get_prefix(card_type: &str) -> Option<String> {
	CARD_PREFIXES
		.iter()
		.find(|(_, v)| *v == card_type)
		.map(|(k, _)| k.to_string())
}
