use super::Link;
use crate::registry::CardDefinition;

#[test]
fn get_markdown_formats_link() {
	let link = Link {
		target: "MIS-001".to_string(),
		relationship: "rel".to_string(),
	};

	let card_type = CardDefinition::get_by_acronym("MIS").card_type;
	let expected = format!("- rel [MIS-001](../{}/MIS-001.md)\n", card_type);
	let actual = link.get_markdown();
	assert_eq!(actual, expected);
}
