use std::{fs::File, path::Path};

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};

fn main() {
    if let Ok(os) = std::env::var("CARGO_CFG_TARGET_OS") {
        if os.as_str() == "horizon" {
            let dkp = std::env::var("DEVKITPRO").unwrap();
            println!("cargo:rustc-link-search={dkp}/libctru/lib");
            println!("cargo:rustc-link-lib=citro2d");
            println!("cargo:rustc-link-lib=citro3d");
        } else if let Ok(arch) = std::env::var("CARGO_CFG_TARGET_ARCH") {
            if arch != "wasm32" {
                let dest = std::env::var("OUT_DIR").unwrap();
                let mut file = File::create(Path::new(&dest).join("gl_bindings.rs")).unwrap();

                Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, [])
                    .write_bindings(GlobalGenerator, &mut file)
                    .unwrap();
            }
        }
    }
}
