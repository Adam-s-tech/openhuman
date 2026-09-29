//! OpenHuman-compatible names for the portable `tinyskills` model.

pub use tinyskills::{
    Skill as Workflow, SkillFrontmatter as WorkflowFrontmatter, SkillScope as WorkflowScope,
};

pub const MAX_DESCRIPTION_LEN: usize = 1024;
pub const MAX_NAME_LEN: usize = 64;
pub const RESOURCE_DIRS: &[&str] = &[
    "scripts",
    "references",
    "assets",
    "templates",
    "examples",
    "prompts",
];
pub const SKILL_JSON: &str = "skill.json";
pub const SKILL_MD: &str = "SKILL.md";
pub const SKILL_TOML: &str = "skill.toml";
pub const WORKFLOW_MD: &str = "WORKFLOW.md";
pub const WORKFLOW_TOML: &str = "workflow.toml";

pub(crate) const TRUST_MARKER: &str = "trust";
pub const MAX_WORKFLOW_RESOURCE_BYTES: u64 = 128 * 1024;

#[cfg(test)]
#[path = "ops_types_tests.rs"]
mod ops_types_tests;
