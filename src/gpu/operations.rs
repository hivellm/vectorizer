//! Operações GPU de alto nível

use super::context::GpuContext;
use super::shaders::ShaderType;
use crate::error::{Result, VectorizerError};
use crate::models::DistanceMetric;
use std::sync::Arc;

#[cfg(feature = "wgpu-gpu")]
use wgpu::{BindGroup, BindGroupLayout, Buffer, ComputePipeline, ShaderModule};

#[cfg(feature = "wgpu-gpu")]
use super::buffers::{BufferManager, ComputeParams};

/// Interface de operações GPU
pub trait GpuOperations {
    /// Calcular similaridade coseno entre query e múltiplos vetores
    async fn cosine_similarity(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<f32>>;

    /// Calcular distância euclidiana entre query e múltiplos vetores
    async fn euclidean_distance(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<f32>>;

    /// Calcular produto escalar entre query e múltiplos vetores
    async fn dot_product(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<f32>>;

    /// Busca em lote: múltiplas queries contra múltiplos vetores
    async fn batch_search(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
        metric: DistanceMetric,
    ) -> Result<Vec<Vec<f32>>>;
}

#[cfg(feature = "wgpu-gpu")]
impl GpuOperations for GpuContext {
    async fn cosine_similarity(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<f32>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let dimension = query.len();
        let vector_count = vectors.len();

        // Verificar se deve usar GPU
        let operations = vector_count * dimension;
        if !self.should_use_gpu(operations) {
            return self.cosine_similarity_cpu(query, vectors);
        }

        // Preparar dados para GPU
        let mut flat_vectors = Vec::with_capacity(vector_count * dimension);
        for v in vectors {
            if v.len() != dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: dimension,
                    got: v.len(),
                });
            }
            flat_vectors.extend_from_slice(v);
        }

        // Executar na GPU
        self.execute_compute(
            query,
            &flat_vectors,
            1,
            vector_count,
            dimension,
            ShaderType::CosineSimilarity,
        )
        .await
    }

    async fn euclidean_distance(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<f32>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let dimension = query.len();
        let vector_count = vectors.len();

        let operations = vector_count * dimension;
        if !self.should_use_gpu(operations) {
            return self.euclidean_distance_cpu(query, vectors);
        }

        let mut flat_vectors = Vec::with_capacity(vector_count * dimension);
        for v in vectors {
            if v.len() != dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: dimension,
                    got: v.len(),
                });
            }
            flat_vectors.extend_from_slice(v);
        }

        self.execute_compute(
            query,
            &flat_vectors,
            1,
            vector_count,
            dimension,
            ShaderType::EuclideanDistance,
        )
        .await
    }

    async fn dot_product(
        &self,
        query: &[f32],
        vectors: &[Vec<f32>],
    ) -> Result<Vec<f32>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        let dimension = query.len();
        let vector_count = vectors.len();

        let operations = vector_count * dimension;
        if !self.should_use_gpu(operations) {
            return self.dot_product_cpu(query, vectors);
        }

        let mut flat_vectors = Vec::with_capacity(vector_count * dimension);
        for v in vectors {
            if v.len() != dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: dimension,
                    got: v.len(),
                });
            }
            flat_vectors.extend_from_slice(v);
        }

        self.execute_compute(
            query,
            &flat_vectors,
            1,
            vector_count,
            dimension,
            ShaderType::DotProduct,
        )
        .await
    }

    async fn batch_search(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
        metric: DistanceMetric,
    ) -> Result<Vec<Vec<f32>>> {
        if queries.is_empty() || vectors.is_empty() {
            return Ok(vec![Vec::new(); queries.len()]);
        }

        let dimension = queries[0].len();
        let query_count = queries.len();
        let vector_count = vectors.len();

        let operations = query_count * vector_count * dimension;
        if !self.should_use_gpu(operations) {
            return self.batch_search_cpu(queries, vectors, metric);
        }

        // Achatar queries
        let mut flat_queries = Vec::with_capacity(query_count * dimension);
        for q in queries {
            if q.len() != dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: dimension,
                    got: q.len(),
                });
            }
            flat_queries.extend_from_slice(q);
        }

        // Achatar vectors
        let mut flat_vectors = Vec::with_capacity(vector_count * dimension);
        for v in vectors {
            if v.len() != dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: dimension,
                    got: v.len(),
                });
            }
            flat_vectors.extend_from_slice(v);
        }

        // Selecionar shader baseado na métrica
        let shader = match metric {
            DistanceMetric::Cosine => ShaderType::CosineSimilarity,
            DistanceMetric::Euclidean => ShaderType::EuclideanDistance,
            DistanceMetric::DotProduct => ShaderType::DotProduct,
            _ => return Err(VectorizerError::Other(format!("Métrica {:?} não suportada na GPU", metric))),
        };

        let shader = ShaderType::select_for_dimension(shader, dimension);

        // Executar na GPU
        let flat_results = self
            .execute_compute(
                &flat_queries,
                &flat_vectors,
                query_count,
                vector_count,
                dimension,
                shader,
            )
            .await?;

        // Reorganizar resultados
        let mut results = Vec::with_capacity(query_count);
        for i in 0..query_count {
            let start = i * vector_count;
            let end = start + vector_count;
            results.push(flat_results[start..end].to_vec());
        }

        Ok(results)
    }
}

#[cfg(feature = "wgpu-gpu")]
impl GpuContext {
    /// Executar compute shader
    async fn execute_compute(
        &self,
        queries: &[f32],
        vectors: &[f32],
        query_count: usize,
        vector_count: usize,
        dimension: usize,
        shader_type: ShaderType,
    ) -> Result<Vec<f32>> {
        use wgpu::util::DeviceExt;

        let buffer_manager = BufferManager::new(
            Arc::new(self.device().clone()),
            Arc::new(self.queue().clone()),
        );

        // Criar parâmetros
        let params = ComputeParams::new(query_count, vector_count, dimension);

        // Criar buffers
        let params_buffer = buffer_manager.create_uniform_buffer("params", params.as_bytes())?;
        let query_buffer = buffer_manager.create_storage_buffer("queries", queries)?;
        let vector_buffer = buffer_manager.create_storage_buffer("vectors", vectors)?;

        let result_size = params.total_comparisons() * std::mem::size_of::<f32>();
        let result_buffer = buffer_manager.create_storage_buffer_rw(
            "results",
            result_size as u64,
        )?;

        // Criar shader module
        let shader = self.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_type.source().into()),
        });

        // Criar bind group layout
        let bind_group_layout =
            self.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Criar bind group
        let bind_group = self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: query_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: vector_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: result_buffer.as_entire_binding(),
                },
            ],
        });

        // Criar pipeline
        let pipeline_layout =
            self.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = self.device().create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some(shader_type.entry_point()),
            compilation_options: Default::default(),
            cache: None,
        });

        // Criar staging buffer para ler resultados
        let staging_buffer = buffer_manager.create_staging_buffer("staging", result_size as u64)?;

        // Executar compute shader
        let workgroup_size = self.optimal_workgroup_size(params.total_comparisons());
        let workgroup_count = self.workgroups_needed(params.total_comparisons(), workgroup_size);

        let mut encoder = self.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute_encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // Copiar resultados para staging buffer
        encoder.copy_buffer_to_buffer(&result_buffer, 0, &staging_buffer, 0, result_size as u64);

        self.queue().submit(std::iter::once(encoder.finish()));

        // Ler resultados (de forma síncrona após submit)
        buffer_manager.read_buffer_sync(&staging_buffer)
    }

    // Implementações CPU fallback
    fn cosine_similarity_cpu(&self, query: &[f32], vectors: &[Vec<f32>]) -> Result<Vec<f32>> {
        use rayon::prelude::*;

        let results: Vec<f32> = vectors
            .par_iter()
            .map(|v| {
                let dot: f32 = query.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                let norm_q: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_v: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();

                if norm_q > 0.0 && norm_v > 0.0 {
                    dot / (norm_q * norm_v)
                } else {
                    0.0
                }
            })
            .collect();

        Ok(results)
    }

    fn euclidean_distance_cpu(&self, query: &[f32], vectors: &[Vec<f32>]) -> Result<Vec<f32>> {
        use rayon::prelude::*;

        let results: Vec<f32> = vectors
            .par_iter()
            .map(|v| {
                query
                    .iter()
                    .zip(v.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    .sqrt()
            })
            .collect();

        Ok(results)
    }

    fn dot_product_cpu(&self, query: &[f32], vectors: &[Vec<f32>]) -> Result<Vec<f32>> {
        use rayon::prelude::*;

        let results: Vec<f32> = vectors
            .par_iter()
            .map(|v| query.iter().zip(v.iter()).map(|(a, b)| a * b).sum())
            .collect();

        Ok(results)
    }

    fn batch_search_cpu(
        &self,
        queries: &[Vec<f32>],
        vectors: &[Vec<f32>],
        metric: DistanceMetric,
    ) -> Result<Vec<Vec<f32>>> {
        use rayon::prelude::*;

        let results: Vec<Vec<f32>> = queries
            .par_iter()
            .map(|query| {
                match metric {
                    DistanceMetric::Cosine => self.cosine_similarity_cpu(query, vectors),
                    DistanceMetric::Euclidean => self.euclidean_distance_cpu(query, vectors),
                    DistanceMetric::DotProduct => self.dot_product_cpu(query, vectors),
                    _ => Err(VectorizerError::Other(format!("Métrica {:?} não suportada", metric))),
                }
                .unwrap_or_default()
            })
            .collect();

        Ok(results)
    }
}
