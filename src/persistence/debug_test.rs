#[cfg(test)]
mod debug_tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;
    use std::fs;
    use tokio;

    #[tokio::test]
    async fn test_debug_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let data_dir = temp_dir.path().join("data");
        
        println!("Creating directory: {:?}", data_dir);
        
        // Try to create directory
        match fs::create_dir_all(&data_dir) {
            Ok(_) => println!("Directory created successfully"),
            Err(e) => {
                println!("Failed to create directory: {:?}", e);
                panic!("Directory creation failed: {:?}", e);
            }
        }
        
        // Verify directory exists
        if data_dir.exists() {
            println!("Directory exists: {:?}", data_dir);
        } else {
            panic!("Directory does not exist after creation");
        }
        
        // Try to create a file in the directory
        let test_file = data_dir.join("test.txt");
        match fs::write(&test_file, "test content") {
            Ok(_) => println!("File created successfully"),
            Err(e) => {
                println!("Failed to create file: {:?}", e);
                panic!("File creation failed: {:?}", e);
            }
        }
        
        println!("All tests passed!");
    }
}
