use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct ArtifactsConfig {
    artifact: Vec<Artifact>,
}

#[derive(Deserialize)]
struct Artifact {
    name: String,
    step: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
    from: String,
    to: String,
}

/// Assemble artifacts defined in `symposium-artifacts.toml`.
///
/// Reads the config from `manifest_dir/symposium-artifacts.toml`,
/// copies files into `out_dir/artifacts/<name>/`, rendering all
/// text files through minijinja as templates.
///
/// Returns the path to the artifacts directory and a list of paths
/// that should be watched for changes (for cargo:rerun-if-changed).
pub fn assemble(manifest_dir: &Path, out_dir: &Path) -> AssemblyResult {
    let artifacts_dir = out_dir.to_path_buf();

    let config_path = manifest_dir.join("symposium-artifacts.toml");

    let config_str =
        fs::read_to_string(&config_path).expect("failed to read symposium-artifacts.toml");
    let config: ArtifactsConfig =
        toml::from_str(&config_str).expect("failed to parse symposium-artifacts.toml");

    let mut env = minijinja::Environment::new();
    env.set_keep_trailing_newline(true);

    let mut watch_paths = vec![config_path];

    for artifact in &config.artifact {
        let artifact_dir = artifacts_dir.join(&artifact.name);

        fs::create_dir_all(&artifact_dir)
            .unwrap_or_else(|e| panic!("failed to create {}: {e}", artifact_dir.display()));

        for step in &artifact.step {
            let from = manifest_dir.join(&step.from);
            let to = artifact_dir.join(&step.to);

            watch_paths.push(from.clone());

            if from.is_file() {
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent)
                        .unwrap_or_else(|e| panic!("failed to create {}: {e}", parent.display()));
                }
                copy_file(&from, &to, &env);
            } else {
                copy_dir_recursive(&from, &to, &env);
            }
        }
    }

    AssemblyResult {
        artifacts_dir,
        watch_paths,
    }
}

pub struct AssemblyResult {
    /// Path to the assembled artifacts directory.
    pub artifacts_dir: PathBuf,
    /// Paths that should be watched for changes.
    pub watch_paths: Vec<PathBuf>,
}

fn copy_dir_recursive(from: &Path, to: &Path, env: &minijinja::Environment) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("failed to create {}: {e}", to.display()));

    for entry in
        fs::read_dir(from).unwrap_or_else(|e| panic!("failed to read {}: {e}", from.display()))
    {
        let entry = entry.unwrap();
        let src = entry.path();
        let file_name = entry.file_name();

        if src.is_dir() {
            let dst = to.join(&file_name);
            copy_dir_recursive(&src, &dst, env);
        } else {
            let dst = to.join(&file_name);
            copy_file(&src, &dst, env);
        }
    }
}

fn copy_file(src: &Path, dst: &Path, env: &minijinja::Environment) {
    // Try to read as text and render through minijinja.
    // Binary files (non-UTF-8) are copied as-is.
    match fs::read_to_string(src) {
        Ok(template_src) => {
            let rendered = env
                .render_str(&template_src, minijinja::context!())
                .unwrap_or_else(|e| panic!("failed to render {}: {e}", src.display()));
            fs::write(dst, rendered)
                .unwrap_or_else(|e| panic!("failed to write {}: {e}", dst.display()));
        }
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            // Binary file — copy as-is
            fs::copy(src, dst).unwrap_or_else(|e| {
                panic!("failed to copy {} -> {}: {e}", src.display(), dst.display())
            });
        }
        Err(e) => panic!("failed to read {}: {e}", src.display()),
    }
}
