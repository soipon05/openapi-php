//! Top-level generator API.
//!
//! Entry points: `run()` (write to disk), `run_dry_print()` (preview to stdout),
//! `run_diff()` (compare against existing files).  Framework dispatch is
//! centralised in `make_backend()`.

use crate::cli::GenerateMode;
use crate::config::{Framework, PhpVersion};
use crate::ir::ResolvedSpec;
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub mod backend;
pub mod php;

pub use backend::{CodegenBackend, CodegenContext, RenderedFile};
pub use php::laravel::LaravelPhpBackend;
pub use php::plain::PlainPhpBackend;

// ─── Framework dispatch ───────────────────────────────────────────────────

fn make_backend(
    framework: &Framework,
    templates_dir: Option<&Path>,
) -> Result<Box<dyn CodegenBackend>> {
    match framework {
        Framework::Plain => Ok(Box::new(PlainPhpBackend::new(templates_dir)?)),
        Framework::Laravel => Ok(Box::new(LaravelPhpBackend::new(templates_dir)?)),
        Framework::Symfony => {
            eprintln!("warning: Symfony backend is not yet implemented; falling back to plain PHP");
            Ok(Box::new(PlainPhpBackend::new(templates_dir)?))
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────

/// Write generated PHP files to `output`.
///
/// # Errors
///
/// Returns an error if:
/// - Any generated file's relative path contains `..` (path traversal attempt).
/// - Any filesystem operation (directory creation, file write) fails.
///
/// Note: `run_dry_filtered` does not write files and therefore does not apply
/// the path-traversal guard.
// 8 args: inheriting pre-existing 7-arg signature plus `split_by_tag`;
// grouping into a struct would be a larger refactor left for a follow-up.
#[allow(clippy::too_many_arguments)]
pub fn run(
    spec: &ResolvedSpec,
    output: &Path,
    namespace: &str,
    mode: GenerateMode,
    framework: Framework,
    templates_dir: Option<&Path>,
    php_version: &PhpVersion,
    split_by_tag: bool,
) -> Result<()> {
    std::fs::create_dir_all(output)?;

    let backend = make_backend(&framework, templates_dir)?;
    let ctx = CodegenContext {
        spec,
        namespace,
        php_version,
        split_by_tag,
    };
    let files = backend.render(&ctx)?;

    for file in &files {
        if backend.filter_by_mode(&file.rel_path, &mode) {
            if path_escapes_base(&file.rel_path) {
                bail!(
                    "Generated file path escapes output directory: {}",
                    file.rel_path.display()
                );
            }
            let full_path = output.join(&file.rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, &file.content)?;
            println!("  📄 {}", file.rel_path.display());
        }
    }

    println!("✅ Done → {}", output.display());
    Ok(())
}

/// Return mode-filtered rendered files without touching the filesystem.
/// Useful for testing and as the shared core of `run_dry_print` / `run_diff`.
pub fn run_dry_filtered(
    spec: &ResolvedSpec,
    namespace: &str,
    mode: &GenerateMode,
    framework: &Framework,
    templates_dir: Option<&Path>,
    php_version: &PhpVersion,
    split_by_tag: bool,
) -> Result<BTreeMap<PathBuf, String>> {
    let backend = make_backend(framework, templates_dir)?;
    let ctx = CodegenContext {
        spec,
        namespace,
        php_version,
        split_by_tag,
    };
    let files = backend.render(&ctx)?;
    Ok(files
        .into_iter()
        .map(|f| (f.rel_path, f.content))
        .filter(|(p, _)| backend.filter_by_mode(p, mode))
        .collect())
}

/// Print every would-be file to stdout with a separator header, then a summary.
/// No files are written.
pub fn run_dry_print(
    spec: &ResolvedSpec,
    namespace: &str,
    mode: GenerateMode,
    framework: Framework,
    templates_dir: Option<&Path>,
    php_version: &PhpVersion,
    split_by_tag: bool,
) -> Result<()> {
    let files = run_dry_filtered(
        spec,
        namespace,
        &mode,
        &framework,
        templates_dir,
        php_version,
        split_by_tag,
    )?;
    let count = files.len();
    for (path, content) in &files {
        println!("=== {} ===", path.display());
        println!("{}", content);
    }
    println!("🔍 Dry run: {} file(s) would be generated", count);
    Ok(())
}

/// Compare generated files against existing files on disk and print a colored
/// unified diff.  Returns `true` when at least one file differs (caller should
/// exit with code 1 for CI use).  No files are written.
// 8 args: same reason as `run` above.
#[allow(clippy::too_many_arguments)]
pub fn run_diff(
    spec: &ResolvedSpec,
    output: &Path,
    namespace: &str,
    mode: GenerateMode,
    framework: Framework,
    templates_dir: Option<&Path>,
    php_version: &PhpVersion,
    split_by_tag: bool,
) -> Result<bool> {
    let files = run_dry_filtered(
        spec,
        namespace,
        &mode,
        &framework,
        templates_dir,
        php_version,
        split_by_tag,
    )?;
    let total = files.len();
    let mut changed = 0usize;

    for (rel_path, new_content) in &files {
        let full_path = output.join(rel_path);
        let (old_content, old_label) = if full_path.exists() {
            (
                std::fs::read_to_string(&full_path)?,
                format!("{} (existing)", rel_path.display()),
            )
        } else {
            (String::new(), format!("{} (new file)", rel_path.display()))
        };

        if &old_content == new_content {
            continue;
        }

        changed += 1;
        let new_label = format!("{} (generated)", rel_path.display());
        print_diff(&old_label, &new_label, &old_content, new_content);
    }

    if changed > 0 {
        println!("❌ {} file(s) changed (exit code 1)", changed);
        Ok(true)
    } else {
        println!("✅ No changes ({} file(s) up to date)", total);
        Ok(false)
    }
}

/// Returns `true` if `rel_path` contains a `..` (ParentDir) component that
/// would allow a generated file to escape the intended output directory.
fn path_escapes_base(rel_path: &Path) -> bool {
    rel_path
        .components()
        .any(|c| c == std::path::Component::ParentDir)
}

fn print_diff(old_label: &str, new_label: &str, old: &str, new: &str) {
    use similar::TextDiff;

    let diff = TextDiff::from_lines(old, new);
    let mut ud = diff.unified_diff();
    ud.header(old_label, new_label);
    let text = ud.to_string();

    for line in text.lines() {
        if line.starts_with("---") || line.starts_with("+++") || line.starts_with("@@") {
            println!("{}", line);
        } else if line.starts_with('-') {
            println!("\x1b[31m{}\x1b[0m", line);
        } else if line.starts_with('+') {
            println!("\x1b[32m{}\x1b[0m", line);
        } else {
            println!("{}", line);
        }
    }
}
