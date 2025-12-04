use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // During installation, copy source templates to user config
    // This happens when PROFILE=release and we're installing
    if env::var("PROFILE").unwrap_or_default() == "release" {
        if let Ok(out_dir) = env::var("OUT_DIR") {
            // Check if this is an install (not just a build)
            if out_dir.contains(".cargo/registry") || out_dir.contains("release/build") {
                install_templates_to_user_config();
            }
        }
    }

    // Tell Cargo to rerun if source templates change
    println!("cargo:rerun-if-changed=source-templates/");
}

fn install_templates_to_user_config() {
    use std::path::PathBuf;

    // Get user config directory
    let home = match env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        Ok(h) => PathBuf::from(h),
        Err(_) => return,
    };

    let user_templates_dir = home.join(".config/mucm/templates");
    let source_templates = Path::new("source-templates");

    if !source_templates.exists() {
        return;
    }

    // Remove old templates if they exist
    if user_templates_dir.exists() {
        let _ = fs::remove_dir_all(&user_templates_dir);
    }

    // Create parent directory
    if let Some(parent) = user_templates_dir.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // Copy templates
    if copy_dir_recursive(source_templates, &user_templates_dir).is_ok() {
        println!("cargo:warning=✓ Updated templates in ~/.config/mucm/templates/");
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
