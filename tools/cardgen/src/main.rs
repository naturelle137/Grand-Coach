// SPDX-License-Identifier: EUPL-1.2

//! Writes the generated card faces to `assets/cards/` (or the directory
//! given as the first argument). Run with `cargo run -p cardgen`.

use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let out = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/cards"),
        PathBuf::from,
    );
    let files = cardgen::generate();
    for (path, svg) in &files {
        let path = out.join(path);
        std::fs::create_dir_all(path.parent().expect("file has a directory"))?;
        std::fs::write(&path, svg)?;
    }
    println!("wrote {} files to {}", files.len(), out.display());
    Ok(())
}
