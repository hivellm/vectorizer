fn main() {
    // Embed Windows icon resource
    #[cfg(target_os = "windows")]
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
}

