use super::Link;

#[test]
fn markdown_with_href_formats_link() {
	let link = Link {
		target: "MIS-001".to_string(),
		relationship: "rel".to_string(),
	};
	let expected = "- rel [MIS-001](MIS-001/Mission/MIS-001-Mission.md)\n".to_string();
	let actual = link.markdown_with_href("MIS-001/Mission/MIS-001-Mission.md");
	assert_eq!(actual, expected);
}
