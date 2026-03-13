//! Four-region editor shell sizing helpers.

const DEFAULT_LEFT_PERCENT: u8 = 20;
const DEFAULT_RIGHT_PERCENT: u8 = 20;
const DEFAULT_BOTTOM_PERCENT: u8 = 20;
const MIN_SIDE_PERCENT: u8 = 15;
const MAX_SIDE_PERCENT: u8 = 35;
const MAX_TOTAL_SIDE_PERCENT: u8 = 70;
const MIN_BOTTOM_PERCENT: u8 = 15;
const MAX_BOTTOM_PERCENT: u8 = 40;
const RESIZE_STEP_PERCENT: u8 = 1;

/// A resizable region in the editor shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellRegion {
	LeftSidebar,
	RightSidebar,
	BottomPanel,
}

/// A resize command applied to a shell region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeCommand {
	Decrease,
	Increase,
	Min,
	Max,
}

/// Layout state for the editor shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellLayout {
	left_sidebar_percent: u8,
	right_sidebar_percent: u8,
	bottom_panel_percent: u8,
}

impl Default for ShellLayout {
	fn default() -> Self {
		Self {
			left_sidebar_percent: DEFAULT_LEFT_PERCENT,
			right_sidebar_percent: DEFAULT_RIGHT_PERCENT,
			bottom_panel_percent: DEFAULT_BOTTOM_PERCENT,
		}
	}
}

impl ShellLayout {
	/// Current share for the requested shell region.
	pub fn percent_for(self, region: ShellRegion) -> u8 {
		match region {
			ShellRegion::LeftSidebar => self.left_sidebar_percent(),
			ShellRegion::RightSidebar => self.right_sidebar_percent(),
			ShellRegion::BottomPanel => self.bottom_panel_percent(),
		}
	}

	/// Minimum supported share for the requested shell region.
	pub fn min_percent_for(self, region: ShellRegion) -> u8 {
		match region {
			ShellRegion::LeftSidebar | ShellRegion::RightSidebar => MIN_SIDE_PERCENT,
			ShellRegion::BottomPanel => MIN_BOTTOM_PERCENT,
		}
	}

	/// Maximum supported share for the requested shell region.
	pub fn max_percent_for(self, region: ShellRegion) -> u8 {
		match region {
			ShellRegion::LeftSidebar => max_allowed_side_percent(self.right_sidebar_percent),
			ShellRegion::RightSidebar => max_allowed_side_percent(self.left_sidebar_percent),
			ShellRegion::BottomPanel => MAX_BOTTOM_PERCENT,
		}
	}

	/// Current left sidebar width share.
	pub fn left_sidebar_percent(self) -> u8 {
		self.left_sidebar_percent
	}

	/// Current right sidebar width share.
	pub fn right_sidebar_percent(self) -> u8 {
		self.right_sidebar_percent
	}

	/// Current bottom panel height share.
	pub fn bottom_panel_percent(self) -> u8 {
		self.bottom_panel_percent
	}

	/// Remaining share for the main center column.
	pub fn center_column_percent(self) -> u8 {
		100 - self.left_sidebar_percent - self.right_sidebar_percent
	}

	/// Remaining share for the main top row.
	pub fn top_panel_percent(self) -> u8 {
		100 - self.bottom_panel_percent
	}

	/// Grid template for the shell columns.
	pub fn column_template(self) -> String {
		format!(
			"minmax(220px, {}fr) 42px minmax(360px, {}fr) 42px minmax(220px, {}fr)",
			self.left_sidebar_percent,
			self.center_column_percent(),
			self.right_sidebar_percent,
		)
	}

	/// Grid template for the shell rows.
	pub fn row_template(self) -> String {
		format!(
			"minmax(260px, {}fr) 42px minmax(160px, {}fr)",
			self.top_panel_percent(),
			self.bottom_panel_percent,
		)
	}

	/// Increase the left sidebar width by one step.
	pub fn grow_left(self) -> Self {
		self.adjust_region(ShellRegion::LeftSidebar, ResizeCommand::Increase)
	}

	/// Decrease the left sidebar width by one step.
	pub fn shrink_left(self) -> Self {
		self.adjust_region(ShellRegion::LeftSidebar, ResizeCommand::Decrease)
	}

	/// Increase the right sidebar width by one step.
	pub fn grow_right(self) -> Self {
		self.adjust_region(ShellRegion::RightSidebar, ResizeCommand::Increase)
	}

	/// Decrease the right sidebar width by one step.
	pub fn shrink_right(self) -> Self {
		self.adjust_region(ShellRegion::RightSidebar, ResizeCommand::Decrease)
	}

	/// Increase the bottom panel height by one step.
	pub fn grow_bottom(self) -> Self {
		self.adjust_region(ShellRegion::BottomPanel, ResizeCommand::Increase)
	}

	/// Decrease the bottom panel height by one step.
	pub fn shrink_bottom(self) -> Self {
		self.adjust_region(ShellRegion::BottomPanel, ResizeCommand::Decrease)
	}

	/// Apply a resize command to the requested shell region.
	pub fn adjust_region(self, region: ShellRegion, command: ResizeCommand) -> Self {
		match region {
			ShellRegion::LeftSidebar => self.adjust_left(command),
			ShellRegion::RightSidebar => self.adjust_right(command),
			ShellRegion::BottomPanel => self.adjust_bottom(command),
		}
	}

	fn adjust_left(self, command: ResizeCommand) -> Self {
		let requested_percent = match command {
			ResizeCommand::Decrease => self
				.left_sidebar_percent
				.saturating_sub(RESIZE_STEP_PERCENT),
			ResizeCommand::Increase => self
				.left_sidebar_percent
				.saturating_add(RESIZE_STEP_PERCENT),
			ResizeCommand::Min => self.min_percent_for(ShellRegion::LeftSidebar),
			ResizeCommand::Max => self.max_percent_for(ShellRegion::LeftSidebar),
		};
		self.with_left_sidebar_percent(requested_percent)
	}

	fn adjust_right(self, command: ResizeCommand) -> Self {
		let requested_percent = match command {
			ResizeCommand::Decrease => self
				.right_sidebar_percent
				.saturating_sub(RESIZE_STEP_PERCENT),
			ResizeCommand::Increase => self
				.right_sidebar_percent
				.saturating_add(RESIZE_STEP_PERCENT),
			ResizeCommand::Min => self.min_percent_for(ShellRegion::RightSidebar),
			ResizeCommand::Max => self.max_percent_for(ShellRegion::RightSidebar),
		};
		self.with_right_sidebar_percent(requested_percent)
	}

	fn adjust_bottom(self, command: ResizeCommand) -> Self {
		let requested_percent = match command {
			ResizeCommand::Decrease => self
				.bottom_panel_percent
				.saturating_sub(RESIZE_STEP_PERCENT),
			ResizeCommand::Increase => self
				.bottom_panel_percent
				.saturating_add(RESIZE_STEP_PERCENT),
			ResizeCommand::Min => self.min_percent_for(ShellRegion::BottomPanel),
			ResizeCommand::Max => self.max_percent_for(ShellRegion::BottomPanel),
		};
		self.with_bottom_panel_percent(requested_percent)
	}

	fn with_left_sidebar_percent(mut self, requested_percent: u8) -> Self {
		let max_percent = max_allowed_side_percent(self.right_sidebar_percent);
		self.left_sidebar_percent = requested_percent.clamp(MIN_SIDE_PERCENT, max_percent);
		self
	}

	fn with_right_sidebar_percent(mut self, requested_percent: u8) -> Self {
		let max_percent = max_allowed_side_percent(self.left_sidebar_percent);
		self.right_sidebar_percent = requested_percent.clamp(MIN_SIDE_PERCENT, max_percent);
		self
	}

	fn with_bottom_panel_percent(mut self, requested_percent: u8) -> Self {
		self.bottom_panel_percent = requested_percent.clamp(MIN_BOTTOM_PERCENT, MAX_BOTTOM_PERCENT);
		self
	}
}

fn max_allowed_side_percent(other_side_percent: u8) -> u8 {
	MAX_SIDE_PERCENT.min(MAX_TOTAL_SIDE_PERCENT.saturating_sub(other_side_percent))
}

/// Map keyboard input to a resize command for a shell region.
pub fn resize_command_for_key(region: ShellRegion, key: &str) -> Option<ResizeCommand> {
	match (region, key) {
		(_, "Home") => Some(ResizeCommand::Min),
		(_, "End") => Some(ResizeCommand::Max),
		(ShellRegion::LeftSidebar, "ArrowLeft") => Some(ResizeCommand::Decrease),
		(ShellRegion::LeftSidebar, "ArrowRight") => Some(ResizeCommand::Increase),
		(ShellRegion::RightSidebar, "ArrowLeft") => Some(ResizeCommand::Increase),
		(ShellRegion::RightSidebar, "ArrowRight") => Some(ResizeCommand::Decrease),
		(ShellRegion::BottomPanel, "ArrowUp") => Some(ResizeCommand::Increase),
		(ShellRegion::BottomPanel, "ArrowDown") => Some(ResizeCommand::Decrease),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::{ResizeCommand, ShellLayout, ShellRegion, resize_command_for_key};

	#[test]
	fn default_shell_matches_project_plan_defaults() {
		let layout = ShellLayout::default();
		assert_eq!(layout.percent_for(ShellRegion::LeftSidebar), 20);
		assert_eq!(layout.percent_for(ShellRegion::RightSidebar), 20);
		assert_eq!(layout.percent_for(ShellRegion::BottomPanel), 20);
		assert_eq!(layout.center_column_percent(), 60);
	}

	#[test]
	fn sidebars_do_not_consume_the_center_column() {
		let mut layout = ShellLayout::default();
		for _ in 0..15 {
			layout = layout.grow_left();
		}
		assert_eq!(layout.left_sidebar_percent(), 35);
		assert_eq!(layout.center_column_percent(), 45);

		for _ in 0..15 {
			layout = layout.grow_right();
		}
		assert_eq!(layout.right_sidebar_percent(), 35);
		assert_eq!(layout.center_column_percent(), 30);
	}

	#[test]
	fn sidebar_shrink_operations_stop_at_minimum_bounds() {
		let mut layout = ShellLayout::default();
		for _ in 0..20 {
			layout = layout.shrink_left().shrink_right();
		}
		assert_eq!(layout.left_sidebar_percent(), 15);
		assert_eq!(layout.right_sidebar_percent(), 15);
		assert_eq!(layout.center_column_percent(), 70);
	}

	#[test]
	fn bottom_panel_stays_within_usable_bounds() {
		let mut layout = ShellLayout::default();
		for _ in 0..50 {
			layout = layout.grow_bottom();
		}
		assert_eq!(layout.bottom_panel_percent(), 40);
		assert_eq!(layout.top_panel_percent(), 60);

		for _ in 0..50 {
			layout = layout.shrink_bottom();
		}
		assert_eq!(layout.bottom_panel_percent(), 15);
		assert_eq!(layout.top_panel_percent(), 85);
	}

	#[test]
	fn grid_templates_reflect_current_layout() {
		let layout = ShellLayout::default()
			.grow_left()
			.shrink_right()
			.grow_bottom();
		assert_eq!(
			layout.column_template(),
			"minmax(220px, 21fr) 42px minmax(360px, 60fr) 42px minmax(220px, 19fr)"
		);
		assert_eq!(
			layout.row_template(),
			"minmax(260px, 79fr) 42px minmax(160px, 21fr)"
		);
	}

	#[test]
	fn resize_commands_can_jump_to_region_bounds() {
		let layout = ShellLayout::default()
			.adjust_region(ShellRegion::LeftSidebar, ResizeCommand::Min)
			.adjust_region(ShellRegion::RightSidebar, ResizeCommand::Max)
			.adjust_region(ShellRegion::BottomPanel, ResizeCommand::Max);

		assert_eq!(layout.left_sidebar_percent(), 15);
		assert_eq!(layout.right_sidebar_percent(), 35);
		assert_eq!(layout.bottom_panel_percent(), 40);
	}

	#[test]
	fn resize_keyboard_shortcuts_match_shell_geometry() {
		assert_eq!(
			resize_command_for_key(ShellRegion::LeftSidebar, "ArrowLeft"),
			Some(ResizeCommand::Decrease)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::LeftSidebar, "ArrowRight"),
			Some(ResizeCommand::Increase)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::RightSidebar, "ArrowLeft"),
			Some(ResizeCommand::Increase)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::RightSidebar, "ArrowRight"),
			Some(ResizeCommand::Decrease)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::BottomPanel, "ArrowUp"),
			Some(ResizeCommand::Increase)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::BottomPanel, "ArrowDown"),
			Some(ResizeCommand::Decrease)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::BottomPanel, "Home"),
			Some(ResizeCommand::Min)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::BottomPanel, "End"),
			Some(ResizeCommand::Max)
		);
		assert_eq!(
			resize_command_for_key(ShellRegion::LeftSidebar, "ArrowUp"),
			None
		);
	}
}
