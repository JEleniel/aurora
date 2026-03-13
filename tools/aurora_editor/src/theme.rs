//! Theme resolution and accessible palette helpers for the editor shell.

use crate::config::ThemePreference;

/// The current OS-level theme reported by the desktop window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemTheme {
	Light,
	#[default]
	Dark,
}

/// The effective theme after applying user preference to the current system theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedTheme {
	Light,
	Dark,
}

impl ResolvedTheme {
	/// Resolve the theme that should be rendered for the active preference.
	pub fn from_preference(preference: &ThemePreference, system_theme: SystemTheme) -> Self {
		match preference {
			ThemePreference::System => system_theme.into(),
			ThemePreference::Light => Self::Light,
			ThemePreference::Dark => Self::Dark,
		}
	}

	/// CSS `color-scheme` value for the current theme.
	pub fn as_color_scheme(self) -> &'static str {
		match self {
			Self::Light => "light",
			Self::Dark => "dark",
		}
	}
}

impl From<SystemTheme> for ResolvedTheme {
	fn from(value: SystemTheme) -> Self {
		match value {
			SystemTheme::Light => Self::Light,
			SystemTheme::Dark => Self::Dark,
		}
	}
}

/// Accessible palette shared across the editor shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePalette {
	pub resolved_theme: ResolvedTheme,
	pub background: &'static str,
	pub panel_background: &'static str,
	pub foreground: &'static str,
	pub muted_foreground: &'static str,
	pub border: &'static str,
	pub rail_background: &'static str,
	pub button_background: &'static str,
	pub button_hover_background: &'static str,
	pub button_foreground: &'static str,
	pub input_background: &'static str,
	pub input_foreground: &'static str,
	pub accent: &'static str,
	pub success_background: &'static str,
	pub success_foreground: &'static str,
	pub error_background: &'static str,
	pub error_foreground: &'static str,
}

impl ThemePalette {
	/// Build the active palette for the current preference and system theme.
	pub fn resolve(preference: &ThemePreference, system_theme: SystemTheme) -> Self {
		Self::for_resolved(ResolvedTheme::from_preference(preference, system_theme))
	}

	/// Build the palette for a fixed light or dark mode.
	pub fn for_resolved(resolved_theme: ResolvedTheme) -> Self {
		match resolved_theme {
			ResolvedTheme::Light => Self {
				resolved_theme,
				background: "#f4f7fb",
				panel_background: "#ffffff",
				foreground: "#122033",
				muted_foreground: "#334155",
				border: "#94a3b8",
				rail_background: "#dbe4ee",
				button_background: "#e2e8f0",
				button_hover_background: "#cbd5e1",
				button_foreground: "#122033",
				input_background: "#ffffff",
				input_foreground: "#122033",
				accent: "#1d4ed8",
				success_background: "#166534",
				success_foreground: "#ecfdf5",
				error_background: "#7f1d1d",
				error_foreground: "#fef2f2",
			},
			ResolvedTheme::Dark => Self {
				resolved_theme,
				background: "#0f172a",
				panel_background: "#162033",
				foreground: "#f8fafc",
				muted_foreground: "#cbd5e1",
				border: "#64748b",
				rail_background: "#1e293b",
				button_background: "#334155",
				button_hover_background: "#475569",
				button_foreground: "#f8fafc",
				input_background: "#162033",
				input_foreground: "#f8fafc",
				accent: "#60a5fa",
				success_background: "#14532d",
				success_foreground: "#dcfce7",
				error_background: "#7f1d1d",
				error_foreground: "#fee2e2",
			},
		}
	}
}

/// Global CSS shared by the shell.
pub fn global_theme_css(palette: &ThemePalette) -> String {
	format!(
		r#"
:root {{
	color-scheme: {};
}}

* {{
	box-sizing: border-box;
}}

body {{
	margin: 0;
	background: {};
	color: {};
}}

button,
input,
select,
textarea {{
	font: inherit;
	color: {};
	background: {};
	border: 1px solid {};
	border-radius: 10px;
	padding: 10px 12px;
}}

button {{
	cursor: pointer;
	background: {};
	color: {};
}}

button:hover {{
	background: {};
}}

button:focus-visible,
input:focus-visible,
select:focus-visible,
textarea:focus-visible {{
	outline: 3px solid {};
	outline-offset: 2px;
}}

.aurora-resize-rail:focus-visible {{
	outline: 3px solid {};
	outline-offset: 2px;
}}

ul {{
	margin: 0;
}}
"#,
		palette.resolved_theme.as_color_scheme(),
		palette.background,
		palette.foreground,
		palette.input_foreground,
		palette.input_background,
		palette.border,
		palette.button_background,
		palette.button_foreground,
		palette.button_hover_background,
		palette.accent,
		palette.accent,
	)
}

#[cfg(not(test))]
use dioxus_desktop::{DesktopContext, tao::window::Theme as TaoTheme};

#[cfg(not(test))]
impl SystemTheme {
	/// Read the theme reported by Tao for the current desktop window.
	pub fn from_tao(theme: TaoTheme) -> Self {
		match theme {
			TaoTheme::Light => Self::Light,
			TaoTheme::Dark => Self::Dark,
			_ => Self::Dark,
		}
	}

	/// Read the current theme directly from the active desktop window.
	pub fn from_window(window: &DesktopContext) -> Self {
		Self::from_tao(window.theme())
	}
}

#[cfg(not(test))]
/// Convert editor configuration into a desktop window theme override.
pub fn preferred_window_theme(preference: ThemePreference) -> Option<TaoTheme> {
	match preference {
		ThemePreference::System => None,
		ThemePreference::Light => Some(TaoTheme::Light),
		ThemePreference::Dark => Some(TaoTheme::Dark),
	}
}

#[cfg(test)]
mod tests {
	use super::{ResolvedTheme, SystemTheme, ThemePalette};
	use crate::config::ThemePreference;

	#[test]
	fn system_preference_tracks_current_os_theme() {
		assert_eq!(
			ResolvedTheme::from_preference(&ThemePreference::System, SystemTheme::Light),
			ResolvedTheme::Light
		);
		assert_eq!(
			ResolvedTheme::from_preference(&ThemePreference::System, SystemTheme::Dark),
			ResolvedTheme::Dark
		);
	}

	#[test]
	fn explicit_preferences_override_system_theme() {
		assert_eq!(
			ResolvedTheme::from_preference(&ThemePreference::Light, SystemTheme::Dark),
			ResolvedTheme::Light
		);
		assert_eq!(
			ResolvedTheme::from_preference(&ThemePreference::Dark, SystemTheme::Light),
			ResolvedTheme::Dark
		);
	}

	#[test]
	fn palette_foregrounds_meet_wcag_aa_contrast() {
		for palette in [
			ThemePalette::for_resolved(ResolvedTheme::Light),
			ThemePalette::for_resolved(ResolvedTheme::Dark),
		] {
			assert!(contrast_ratio(palette.background, palette.foreground) >= 4.5);
			assert!(contrast_ratio(palette.panel_background, palette.foreground) >= 4.5);
			assert!(contrast_ratio(palette.button_background, palette.button_foreground) >= 4.5);
			assert!(contrast_ratio(palette.background, palette.muted_foreground) >= 4.5);
			assert!(contrast_ratio(palette.success_background, palette.success_foreground) >= 4.5);
			assert!(contrast_ratio(palette.error_background, palette.error_foreground) >= 4.5);
		}
	}

	#[test]
	fn global_css_uses_resolved_color_scheme() {
		let css = super::global_theme_css(&ThemePalette::resolve(
			&ThemePreference::System,
			SystemTheme::Light,
		));

		assert!(css.contains("color-scheme: light;"));
		assert!(css.contains(".aurora-resize-rail:focus-visible"));
	}

	fn contrast_ratio(background: &str, foreground: &str) -> f64 {
		let lighter = relative_luminance(parse_rgb(background));
		let darker = relative_luminance(parse_rgb(foreground));
		let (max, min) = if lighter >= darker {
			(lighter, darker)
		} else {
			(darker, lighter)
		};
		(max + 0.05) / (min + 0.05)
	}

	fn relative_luminance((red, green, blue): (u8, u8, u8)) -> f64 {
		let red = linearize_channel(red);
		let green = linearize_channel(green);
		let blue = linearize_channel(blue);
		(0.2126 * red) + (0.7152 * green) + (0.0722 * blue)
	}

	fn linearize_channel(channel: u8) -> f64 {
		let normalized = f64::from(channel) / 255.0;
		if normalized <= 0.03928 {
			normalized / 12.92
		} else {
			((normalized + 0.055) / 1.055).powf(2.4)
		}
	}

	fn parse_rgb(hex: &str) -> (u8, u8, u8) {
		assert_eq!(hex.len(), 7);
		assert!(hex.starts_with('#'));
		let red = u8::from_str_radix(&hex[1..3], 16).expect("valid hex red channel");
		let green = u8::from_str_radix(&hex[3..5], 16).expect("valid hex green channel");
		let blue = u8::from_str_radix(&hex[5..7], 16).expect("valid hex blue channel");
		(red, green, blue)
	}
}
