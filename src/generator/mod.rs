use crate::cli::GenerateMode;
use crate::config::Framework;
use crate::ir::ResolvedSpec;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub mod backend;
pub mod php;

pub use backend::{CodegenBackend, CodegenContext, PlainPhpBackend, RenderedFile};
pub use php::laravel::LaravelPhpBackend;

fn file_in_mode(path: &Path, mode: &GenerateMode, framework: &Framework) -> bool {
    match framework {
        Framework::Laravel => match mode {
            GenerateMode::Models => {
                path.starts_with("Models") || path.starts_with("Http")
            }
            GenerateMode::Client => path.starts_with("routes"),
            GenerateMode::All => true,
        },
        _ => match mode {
            GenerateMode::Models => path.starts_with("Models"),
            GenerateMode::Client => path.starts_with("Client"),
            GenerateMode::All => true,
        },
    }
}

pub fn run(
    spec: &ResolvedSpec,
    output: &Path,
    namespace: &str,
    mode: GenerateMode,
    framework: Framework,
) -> Result<()> {
    std::fs::create_dir_all(output)?;

    let ctx = CodegenContext { spec, namespace };
    let files = match framework {
        Framework::Laravel => LaravelPhpBackend::new().render(&ctx)?,
        _ => PlainPhpBackend::new().render(&ctx)?,
    };

    for file in &files {
        if file_in_mode(&file.rel_path, &mode, &framework) {
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
) -> Result<BTreeMap<PathBuf, String>> {
    let ctx = CodegenContext { spec, namespace };
    let files = match framework {
        Framework::Laravel => LaravelPhpBackend::new().run_dry(&ctx)?,
        _ => PlainPhpBackend::new().run_dry(&ctx)?,
    };
    Ok(files
        .into_iter()
        .filter(|(p, _)| file_in_mode(p, mode, framework))
        .collect())
}

/// Print every would-be file to stdout with a separator header, then a summary.
/// No files are written.
pub fn run_dry_print(
    spec: &ResolvedSpec,
    namespace: &str,
    mode: GenerateMode,
    framework: Framework,
) -> Result<()> {
    let files = run_dry_filtered(spec, namespace, &mode, &framework)?;
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
pub fn run_diff(
    spec: &ResolvedSpec,
    output: &Path,
    namespace: &str,
    mode: GenerateMode,
    framework: Framework,
) -> Result<bool> {
    let files = run_dry_filtered(spec, namespace, &mode, &framework)?;
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
