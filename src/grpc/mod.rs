pub mod server;
pub mod client;

// Re-export generated types
pub use vectorizer::vectorizer_service_server::VectorizerServiceServer;
pub use vectorizer::vectorizer_service_client::VectorizerServiceClient;

pub mod vectorizer {
    tonic::include_proto!("vectorizer");
}
