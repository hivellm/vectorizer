fn main() {
    // Embed Windows icon resource
    #[cfg(all(target_os = "windows", not(target_env = "msvc")))]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Vectorizer");
        res.set("FileDescription", "High-performance vector database");
        res.set("CompanyName", "HiveLLM Contributors");
        res.set("LegalCopyright", "Copyright © 2025 HiveLLM Contributors");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }

    // Skip resource generation on MSVC to avoid CVT1100 duplicate resource error
    // The winres crate can conflict with MSVC toolchain's default resource handling
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        println!(
            "cargo:warning=Skipping winres resource generation on MSVC to avoid duplicate resource errors"
        );
    }
}
