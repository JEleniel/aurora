use thiserror::Error;

/// Errors emitted during layout computation.
#[derive(Debug, Error)]
pub enum RenderError {
	#[error("Layout requires at least one root node")]
	MissingRoots,
	#[error("Layout has a duplicate node id: {0}")]
	DuplicateNodeId(String),
	#[error("Layout is missing node data for {0}")]
	MissingNode(String),
	#[error("Link target {1} referenced by {0} is not in the model")]
	UnknownTarget(String, String),
	#[error("Root node {0} has incoming edges")]
	RootHasIncoming(String),
	#[error("Node {0} has no incoming edges")]
	NodeHasNoIncoming(String),
	#[error("Nodes are unreachable from roots: {0:?}")]
	UnreachableNodes(Vec<String>),
	#[error("Backbone topological order failed")]
	BackboneOrderFailed,
	#[error("Layout is missing rank data for node {0}")]
	MissingRank(String),
	#[error("Layout is missing layer data for index {0}")]
	MissingLayer(usize),
	#[error("Layout is missing layer position data for node {0}")]
	MissingPosition(String),
	#[error("Layout is missing x position data for node {0}")]
	MissingX(String),
	#[error("Root node {0} has a non-zero rank")]
	RootRankNotZero(String),
	#[error("Backbone edge {0} -> {1} violates rank ordering")]
	BackboneRankOrder(String, String),
	#[error(
		"Edge classification mismatch (backbone {backbone_count}, loops {loop_count}, total {total_count})"
	)]
	EdgeClassificationMismatch {
		backbone_count: usize,
		loop_count: usize,
		total_count: usize,
	},
	#[error("Rendering views is not available in this build")]
	RenderUnavailable,
	#[error("SVG template is missing the {{viewbox}} placeholder")]
	SvgTemplateMissingViewbox,
	#[error("SVG template is missing the {{diagram}} placeholder")]
	SvgTemplateMissingDiagram,
	#[error("SVG rendering is missing node data for {0}")]
	SvgMissingNode(String),
	#[error("SVG rendering has a duplicate card id: {0}")]
	SvgDuplicateCardId(String),
	#[error("SVG edge routing failed")]
	SvgRouteFailed,
	#[error("Graphviz executable '{0}' is not available")]
	GraphvizUnavailable(String),
	#[error("Graphviz layout failed: {0}")]
	GraphvizFailed(String),
	#[error("Graphviz output could not be parsed: {0}")]
	GraphvizParse(String),
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),
	#[error("SVG output exceeds maximum size")]
	SvgTooLarge,
}
