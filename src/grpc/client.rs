use crate::grpc::vectorizer::{
    vectorizer_service_client::VectorizerServiceClient,
    SearchRequest, SearchResponse,
    Empty, ListCollectionsResponse,
    EmbedRequest, EmbedResponse,
    IndexingProgressResponse,
    UpdateIndexingProgressRequest,
    HealthResponse,
};
use crate::config::GrpcClientConfig;
use tonic::transport::Channel;
use std::time::Duration;

#[derive(Clone)]
pub struct VectorizerGrpcClient {
    client: VectorizerServiceClient<Channel>,
}

impl VectorizerGrpcClient {
    pub async fn new(config: GrpcClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(config.server_url)?
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(5))
            .keep_alive_timeout(Duration::from_secs(config.keep_alive_interval))
            .keep_alive_while_idle(true)
            .tcp_keepalive(Some(Duration::from_secs(config.keep_alive_interval)))
            .tcp_nodelay(true)
            .connect()
            .await?;

        let client = VectorizerServiceClient::new(channel);
        Ok(Self { client })
    }

    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let config = crate::config::GrpcConfig::from_env();
        Self::new(config.client).await
    }

    pub async fn search(
        &mut self,
        collection: String,
        query: String,
        limit: i32,
    ) -> Result<SearchResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(SearchRequest {
            collection,
            query,
            limit,
            threshold: None,
        });

        let response = self.client.search(request).await?;
        Ok(response.into_inner())
    }

    pub async fn list_collections(&mut self) -> Result<ListCollectionsResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.list_collections(request).await?;
        Ok(response.into_inner())
    }

    pub async fn embed_text(
        &mut self,
        text: String,
        provider: String,
    ) -> Result<EmbedResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(EmbedRequest { text, provider });
        let response = self.client.embed_text(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_indexing_progress(&mut self) -> Result<IndexingProgressResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.get_indexing_progress(request).await?;
        Ok(response.into_inner())
    }

    pub async fn update_indexing_progress(
        &mut self,
        collection_name: String,
        status: String,
        progress: f32,
        vector_count: i32,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(UpdateIndexingProgressRequest {
            collection_name,
            status,
            progress,
            vector_count,
            error_message,
        });

        self.client.update_indexing_progress(request).await?;
        Ok(())
    }

    pub async fn health_check(&mut self) -> Result<HealthResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(Empty {});
        let response = self.client.health_check(request).await?;
        Ok(response.into_inner())
    }
}
