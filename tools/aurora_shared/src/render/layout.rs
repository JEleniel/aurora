//! Hierarchical layout for Aurora view graphs.

mod api;
mod graph;
mod types;

#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod graph_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod types_tests;

pub use api::{layout_model, layout_model_best_family, layout_model_with_family};
pub use types::{Layout, LayoutEdge, LayoutFamily, LayoutNode};
