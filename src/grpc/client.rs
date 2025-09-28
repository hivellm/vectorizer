use crate::grpc::vectorizer::{
    vectorizer_service_client::VectorizerServiceClient,
    SearchRequest, SearchResponse,
    Empty, ListCollectionsResponse,
    EmbedRequest, EmbedResponse,
    IndexingProgressResponse,
    UpdateIndexingProgressRequest,
    HealthResponse,
    CreateCollectionRequest, CreateCollectionResponse,
    DeleteCollectionRequest, DeleteCollectionResponse,
    InsertTextsRequest, InsertTextsResponse,
    DeleteVectorsRequest, DeleteVectorsResponse,
    GetVectorRequest, GetVectorResponse,
    GetCollectionInfoRequest, CollectionInfo,
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

    // Collection operations
    pub async fn create_collection(
        &mut self,
        name: String,
        dimension: i32,
        similarity_metric: String,
    ) -> Result<CreateCollectionResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(CreateCollectionRequest {
            name,
            dimension,
            similarity_metric,
            hnsw_config: None,
            compression_config: None,
        });

        let response = self.client.create_collection(request).await?;
        Ok(response.into_inner())
    }

    pub async fn delete_collection(
        &mut self,
        collection_name: String,
    ) -> Result<DeleteCollectionResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(DeleteCollectionRequest { collection_name });
        let response = self.client.delete_collection(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_collection_info(
        &mut self,
        collection_name: String,
    ) -> Result<CollectionInfo, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetCollectionInfoRequest { collection_name });
        let response = self.client.get_collection_info(request).await?;
        Ok(response.into_inner())
    }

    // Vector operations
    pub async fn insert_texts(
        &mut self,
        collection: String,
        texts: Vec<(String, String, Option<std::collections::HashMap<String, String>>)>,
        provider: String,
    ) -> Result<InsertTextsResponse, Box<dyn std::error::Error>> {
        let text_data: Vec<crate::grpc::vectorizer::TextData> = texts
            .into_iter()
            .map(|(id, text, metadata)| {
                let metadata_map = metadata.unwrap_or_default();
                crate::grpc::vectorizer::TextData {
                    id,
                    text,
                    metadata: metadata_map,
                }
            })
            .collect();

        let request = tonic::Request::new(InsertTextsRequest {
            collection,
            texts: text_data,
            provider,
        });

        let response = self.client.insert_texts(request).await?;
        Ok(response.into_inner())
    }

    pub async fn delete_vectors(
        &mut self,
        collection: String,
        vector_ids: Vec<String>,
    ) -> Result<DeleteVectorsResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(DeleteVectorsRequest {
            collection,
            vector_ids,
        });

        let response = self.client.delete_vectors(request).await?;
        Ok(response.into_inner())
    }

    pub async fn get_vector(
        &mut self,
        collection: String,
        vector_id: String,
    ) -> Result<GetVectorResponse, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetVectorRequest {
            collection,
            vector_id,
        });

        let response = self.client.get_vector(request).await?;
        Ok(response.into_inner())
    }

    /// Summarize text using GRPC
    pub async fn summarize_text(
        &mut self,
        request: crate::grpc::vectorizer::SummarizeTextRequest,
    ) -> Result<crate::grpc::vectorizer::SummarizeTextResponse, tonic::Status> {
        let request = tonic::Request::new(request);
        let response = self.client.summarize_text(request).await?;
        Ok(response.into_inner())
    }

    /// Summarize context using GRPC
    pub async fn summarize_context(
        &mut self,
        request: crate::grpc::vectorizer::SummarizeContextRequest,
    ) -> Result<crate::grpc::vectorizer::SummarizeContextResponse, tonic::Status> {
        let request = tonic::Request::new(request);
        let response = self.client.summarize_context(request).await?;
        Ok(response.into_inner())
    }

    /// Get summary by ID using GRPC
    pub async fn get_summary(
        &mut self,
        request: crate::grpc::vectorizer::GetSummaryRequest,
    ) -> Result<crate::grpc::vectorizer::GetSummaryResponse, tonic::Status> {
        let request = tonic::Request::new(request);
        let response = self.client.get_summary(request).await?;
        Ok(response.into_inner())
    }

    /// List summaries using GRPC
    pub async fn list_summaries(
        &mut self,
        request: crate::grpc::vectorizer::ListSummariesRequest,
    ) -> Result<crate::grpc::vectorizer::ListSummariesResponse, tonic::Status> {
        let request = tonic::Request::new(request);
        let response = self.client.list_summaries(request).await?;
        Ok(response.into_inner())
    }
}
