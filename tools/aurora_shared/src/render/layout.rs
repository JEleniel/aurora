//! Graphviz-backed layout for Aurora view graphs.

mod graph;
mod graphviz;
mod graphviz_api;
mod types;

#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod graph_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod types_tests;

#[cfg(test)]
use graphviz_api as api;

pub use graphviz_api::{
	build_layout_dot_with_family, layout_model, layout_model_best_family, layout_model_with_family,
};
pub use types::{Layout, LayoutCoordinateSpace, LayoutEdge, LayoutFamily, LayoutNode, LayoutPoint};
