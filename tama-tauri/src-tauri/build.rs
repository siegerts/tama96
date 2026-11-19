fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=src/desktop_pet.m");
        cc::Build::new()
            .file("src/desktop_pet.m")
            .flag("-fobjc-arc")
            .compile("desktop_pet");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
    tauri_build::build();
}
