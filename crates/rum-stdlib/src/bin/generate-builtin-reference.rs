use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reference =
        rum_stdlib::docgen::generate_builtin_reference().map_err(std::io::Error::other)?;
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output_path = repo_root.join("docs/reference/builtins.md");

    fs::create_dir_all(
        output_path
            .parent()
            .expect("reference output path should have a parent"),
    )?;
    fs::write(&output_path, reference.markdown)?;

    println!(
        "wrote {} builtin entries to {}",
        reference.entry_count,
        output_path.display()
    );
    Ok(())
}
