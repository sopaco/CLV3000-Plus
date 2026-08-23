use crate::category::CleanupCategory;
use crate::models::{RiskLevel, TechStack};

/// Known cleanup targets relative to a project root or global cache.
#[derive(Debug, Clone)]
pub struct CleanupRule {
    pub relative: &'static str,
    pub stack: TechStack,
    pub risk: RiskLevel,
    pub category: CleanupCategory,
    pub description: &'static str,
    pub global: bool,
    /// Require a marker file under the detected project root (supports `*.ext` globs).
    pub requires_marker: Option<&'static str>,
    /// Name prefix (`cmake-build-`) or suffix pattern (`*.egg-info` → ends with `.egg-info`).
    pub relative_prefix: Option<&'static str>,
    /// Parent directory name must match (e.g. `vendor` for Ruby `vendor/bundle`).
    pub requires_parent: Option<&'static str>,
}

impl CleanupRule {
    pub const fn project(
        relative: &'static str,
        stack: TechStack,
        risk: RiskLevel,
        category: CleanupCategory,
        description: &'static str,
    ) -> Self {
        Self {
            relative,
            stack,
            risk,
            category,
            description,
            global: false,
            requires_marker: None,
            relative_prefix: None,
            requires_parent: None,
        }
    }

    pub const fn global(
        relative: &'static str,
        stack: TechStack,
        risk: RiskLevel,
        category: CleanupCategory,
        description: &'static str,
    ) -> Self {
        Self {
            relative,
            stack,
            risk,
            category,
            description,
            global: true,
            requires_marker: None,
            relative_prefix: None,
            requires_parent: None,
        }
    }

    pub const fn marker(self, marker: &'static str) -> Self {
        Self {
            requires_marker: Some(marker),
            ..self
        }
    }

    pub const fn prefix(self, prefix: &'static str) -> Self {
        Self {
            relative_prefix: Some(prefix),
            ..self
        }
    }

    pub const fn parent(self, name: &'static str) -> Self {
        Self {
            requires_parent: Some(name),
            ..self
        }
    }
}
