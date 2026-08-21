use std::env;
use std::path::PathBuf;

fn main() {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let linker_script = PathBuf::from(manifest_dir).join("linker.ld");
        println!("cargo:rustc-link-arg=-T{}", linker_script.display());
        println!("cargo:rerun-if-changed=linker.ld");
    }
}
