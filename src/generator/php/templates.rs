use anyhow::{Context, Result};
use minijinja::Environment;
use std::path::Path;

pub fn add_template_with_override(
    env: &mut Environment<'static>,
    templates_dir: Option<&Path>,
    template_name: &'static str,
    relative_path: &str,
    builtin_source: &'static str,
) -> Result<()> {
    if let Some(templates_dir) = templates_dir {
        let override_path = templates_dir.join(relative_path);
        if override_path.exists() {
            let source = std::fs::read_to_string(&override_path).with_context(|| {
                format!(
                    "Failed to read template override {}",
                    override_path.display()
                )
            })?;
            env.add_template_owned(template_name.to_string(), source)
                .with_context(|| {
                    format!(
                        "Failed to parse template override {}",
                        override_path.display()
                    )
                })?;
            return Ok(());
        }
    }

    env.add_template(template_name, builtin_source)
        .expect("built-in template is valid");
    Ok(())
}
