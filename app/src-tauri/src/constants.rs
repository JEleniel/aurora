/// Application constants and configuration values
///
/// This module centralizes all hardcoded values used throughout the application,
/// ensuring consistency and making it easy to update versions and other constants.

/// Application version
pub const APP_VERSION: &str = "1.0.0";

/// Gap types for traceability analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapType {
    /// Source card has no outgoing links
    OrphanedSource,
    /// Target card has no incoming links
    UnreferencedTarget,
}

impl GapType {
    pub fn as_str(&self) -> &str {
        match self {
            GapType::OrphanedSource => "orphaned_source",
            GapType::UnreferencedTarget => "unreferenced_target",
        }
    }
}

impl std::fmt::Display for GapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Severity levels for traceability gaps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Gap must be addressed immediately
    High,
    /// Gap should be addressed soon
    Medium,
    /// Gap is informational
    Low,
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_type_display() {
        assert_eq!(GapType::OrphanedSource.as_str(), "orphaned_source");
        assert_eq!(GapType::UnreferencedTarget.as_str(), "unreferenced_target");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::High.as_str(), "high");
        assert_eq!(Severity::Medium.as_str(), "medium");
        assert_eq!(Severity::Low.as_str(), "low");
    }
}
