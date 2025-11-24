#[cfg(test)]
mod debug_tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;
    use tokio;
    use tracing::info;

    use super::*;

    #[tokio::test]
    async fn test_debug_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let data_dir = temp_dir.path().join("data");

        info!("Creating directory: {:?}", data_dir);

        // Try to create directory
        match fs::create_dir_all(&data_dir) {
            Ok(_) => info!("Directory created successfully"),
            Err(e) => {
                info!("Failed to create directory: {:?}", e);
                panic!("Directory creation failed: {:?}", e);
            }
        }

        // Verify directory exists
        if data_dir.exists() {
            info!("Directory exists: {:?}", data_dir);
        } else {
            panic!("Directory does not exist after creation");
        }

        // Try to create a file in the directory
        let test_file = data_dir.join("test.txt");
        match fs::write(&test_file, "test content") {
            Ok(_) => info!("File created successfully"),
            Err(e) => {
                info!("Failed to create file: {:?}", e);
                panic!("File creation failed: {:?}", e);
            }
        }

        info!("All tests passed!");
    }
}
