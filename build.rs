use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=assets");

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be available"),
    );
    let assets_src = manifest_dir.join("assets");
    if !assets_src.is_dir() {
        panic!("assets directory is missing: {}", assets_src.display());
    }

    for profile in ["debug", "release"] {
        let assets_dst = manifest_dir.join("target").join(profile).join("assets");
        copy_dir_recursive(&assets_src, &assets_dst).unwrap_or_else(|err| {
            panic!(
                "Failed to sync assets from {} to {}: {}",
                assets_src.display(),
                assets_dst.display(),
                err
            )
        });
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
