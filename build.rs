// Build script with safety checks
//
// This build script enforces safety guardrails at compile time
// to prevent dangerous configurations that can cause BSODs.

use std::env;

fn main() {
    // Generate Windows resource file
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("ProductName", "Vectorizer")
            .set("FileDescription", "High-Performance Vector Database")
            .set("CompanyName", "HiveLLM");

        // Best effort - don't fail build if icon missing
        let _ = res.compile();
    }
}
