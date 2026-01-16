fn main() {
    // Rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    // Windows-specific: embed application manifest/icon if desired
    #[cfg(windows)]
    {
        // Uncomment if you want to embed a Windows icon
        // use embed_resource;
        // embed_resource::compile("assets/app.rc", embed_resource::NONE);
    }
}
