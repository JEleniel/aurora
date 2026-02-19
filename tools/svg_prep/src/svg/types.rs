use xmltree::Element;

#[derive(Debug, Clone)]
pub(super) struct IconGroup {
	pub(super) id: String,
	pub(super) group: Element,
	pub(super) defs: Vec<Element>,
	pub(super) styles: Vec<String>,
	pub(super) width: f64,
	pub(super) height: f64,
}
