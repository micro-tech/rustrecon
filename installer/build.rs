use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        // Embed the Windows manifest
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let manifest_path = Path::new(&manifest_dir).join("app.manifest");

        if manifest_path.exists() {
            println!("cargo:rerun-if-changed=app.manifest");

            // Create a resource file that references the manifest
            let out_dir = env::var("OUT_DIR").unwrap();
            let resource_file = Path::new(&out_dir).join("resources.rc");

            let rc_content = format!(
                "1 24 \"{}\"\n",
                manifest_path.to_string_lossy().replace("\\", "\\\\")
            );

            fs::write(&resource_file, rc_content).expect("Failed to write resource file");

            // Try to compile the resource file
            let mut res = winres::WindowsResource::new();
            res.set_manifest_file(&manifest_path.to_string_lossy());
            res.set_icon("installer-icon.ico"); // Optional: add icon if available

            if let Err(e) = res.compile() {
                println!("cargo:warning=Failed to compile Windows resources: {}", e);
                println!("cargo:warning=Continuing without manifest embedding");
            }
        }

        // Link additional Windows libraries that might be needed
        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=user32");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
