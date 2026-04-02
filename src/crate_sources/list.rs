//! List workspace crates with available guidance

use std::path::Path;

use anyhow::Result;
use cargo_metadata::{CargoOpt, MetadataCommand};

/// A crate in the workspace's dependency graph
pub struct WorkspaceCrate {
    pub name: String,
    pub version: String,
}

/// Load workspace crates and return as `(name, semver::Version)` pairs
/// for predicate evaluation. Returns an empty list on failure.
pub fn workspace_semver_pairs(cwd: &Path) -> Vec<(String, semver::Version)> {
    list_all_workspace_crates(cwd)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| semver::Version::parse(&c.version).ok().map(|v| (c.name, v)))
        .collect()
}

/// List all crates in the workspace's resolved dependency graph.
///
/// Eventually this will also indicate which crates have specialized
/// guidance available; for now it just returns the dependency list.
fn list_all_workspace_crates(cwd: &Path) -> Result<Vec<WorkspaceCrate>> {
    let metadata = MetadataCommand::new()
        .features(CargoOpt::AllFeatures)
        .current_dir(cwd)
        .exec()?;

    let mut crates: Vec<_> = metadata
        .packages
        .iter()
        .map(|p| WorkspaceCrate {
            name: p.name.to_string(),
            version: p.version.to_string(),
        })
        .collect();

    crates.sort_by(|a, b| a.name.cmp(&b.name));
    crates.dedup_by(|a, b| a.name == b.name);

    Ok(crates)
}
