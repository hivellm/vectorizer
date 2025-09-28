use vectorizer::logging;

fn main() {
    println!("Testing centralized logging system...");
    
    // Initialize logging
    if let Err(e) = logging::init_logging("test-service") {
        eprintln!("Failed to initialize logging: {}", e);
        return;
    }
    
    // Generate some logs
    tracing::info!("This is a test log message");
    tracing::warn!("This is a test warning");
    tracing::error!("This is a test error");
    
    println!("Logging test completed. Check .logs directory for log files.");
}
