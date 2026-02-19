//! SVG transformation routines for `svg_prep`.

mod build;
mod defs;
mod icons;
mod io;
mod optimize;
mod proof;
mod refs;
mod shapes;
mod style;
mod types;
mod util;

#[cfg(test)]
mod tests;

use anyhow::Result;
use tracing::info;

use crate::cli::{BuildArgs, Cli, Command, OptimizeIconsArgs, OptimizeShapesArgs};

pub fn run(args: &Cli) -> Result<()> {
	match &args.command {
		Some(Command::OptimizeIcons(opt)) => run_optimize_icons(opt),
		Some(Command::OptimizeShapes(opt)) => run_optimize_shapes(opt),
		None => run_build(&args.build),
	}
}

fn run_build(args: &BuildArgs) -> Result<()> {
	info!("Building SVG template, proof sheets, and available_cards");
	build::run_build(args)
}

fn run_optimize_icons(args: &OptimizeIconsArgs) -> Result<()> {
	info!("Optimizing icons from {}", args.input.display());
	optimize::optimize_icons_dir(&args.input, &args.output)?;

	info!("Building icon proof sheet from optimized icons");
	let icons = icons::load_icon_groups(&args.output)?;
	let mut icons_svg = proof::build_icons_svg(&icons, "Icons");
	util::strip_non_root_namespaces_in_place(&mut icons_svg);
	util::remove_attribute_by_local_name_in_place(&mut icons_svg, "nodetypes");
	info!("Writing icon proof sheet to {}", args.proof.display());
	io::write_svg_file(&args.proof, &icons_svg)
}

fn run_optimize_shapes(args: &OptimizeShapesArgs) -> Result<()> {
	info!("Optimizing shapes from {}", args.input.display());
	optimize::optimize_shapes_dir(&args.input, &args.output)?;

	info!("Building shapes proof sheet from optimized shapes");
	let shapes = shapes::load_shape_groups(&args.output)?;
	let mut shapes_svg = proof::build_icons_svg(&shapes, "Shapes");
	util::strip_non_root_namespaces_in_place(&mut shapes_svg);
	util::remove_attribute_by_local_name_in_place(&mut shapes_svg, "nodetypes");
	info!("Writing shapes proof sheet to {}", args.proof.display());
	io::write_svg_file(&args.proof, &shapes_svg)
}
