//! Hierarchical layout for Aurora view graphs.

mod api;
mod graph;
mod ordering;
mod types;

#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod graph_tests;
#[cfg(test)]
mod ordering_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod types_tests;

pub use api::layout_model;
pub use types::{Layout, LayoutEdge, LayoutNode};
