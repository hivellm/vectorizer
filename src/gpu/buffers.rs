//! Gerenciamento de buffers GPU

#[cfg(feature = "wgpu-gpu")]
use bytemuck;
use tracing::warn;
#[cfg(feature = "wgpu-gpu")]
use wgpu::util::DeviceExt;
#[cfg(feature = "wgpu-gpu")]
use wgpu::{Buffer, BufferUsages, Device, Queue};

use crate::error::{Result, VectorizerError};

/// Gerenciador de buffers GPU
#[derive(Debug, Clone)]
pub struct BufferManager {
    #[cfg(feature = "wgpu-gpu")]
    device: std::sync::Arc<Device>,

    #[cfg(feature = "wgpu-gpu")]
    queue: std::sync::Arc<Queue>,
}

#[cfg(feature = "wgpu-gpu")]
impl BufferManager {
    pub fn new(device: std::sync::Arc<Device>, queue: std::sync::Arc<Queue>) -> Self {
        Self { device, queue }
    }

    /// Criar buffer de storage para leitura na GPU
    pub fn create_storage_buffer(&self, label: &str, data: &[f32]) -> Result<Buffer> {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(data),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
        Ok(buffer)
    }

    /// Criar buffer de storage para escrita/leitura na GPU
    pub fn create_storage_buffer_rw(&self, label: &str, size: u64) -> Result<Buffer> {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(buffer)
    }

    /// Criar buffer que pode ser mapeado para leitura (sem STORAGE)
    pub fn create_read_buffer(&self, label: &str, size: u64) -> Result<Buffer> {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(buffer)
    }

    /// Criar buffer uniforme
    pub fn create_uniform_buffer(&self, label: &str, data: &[u8]) -> Result<Buffer> {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: data,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });
        Ok(buffer)
    }

    /// Criar buffer para staging (copiar resultados de volta para CPU)
    pub fn create_staging_buffer(&self, label: &str, size: u64) -> Result<Buffer> {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(buffer)
    }

    /// Ler dados de volta do buffer (síncrono com polling ativo)
    pub fn read_buffer_sync(&self, buffer: &Buffer) -> Result<Vec<f32>> {
        use std::sync::{Arc, Mutex};

        // Verificar se o buffer tem dados antes de tentar lê-lo
        if buffer.size() == 0 {
            warn!("Attempting to read from empty buffer (size=0), returning empty vector");
            return Ok(Vec::new());
        }

        let buffer_slice = buffer.slice(..);

        // Usar um flag compartilhado para saber quando o mapeamento completou
        let mapped = Arc::new(Mutex::new(None));
        let mapped_clone = mapped.clone();

        // Iniciar mapeamento assíncrono
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            *mapped_clone.lock().unwrap() = Some(result);
        });

        // Fazer polling ativo do device até o mapeamento completar
        // CRÍTICO: Este é o motivo do travamento anterior - precisamos fazer polling!
        let max_attempts = 10000; // ~100ms timeout
        for _ in 0..max_attempts {
            // Poll do device para processar operações pendentes (wgpu 27.0)
            // PollType::Poll é non-blocking
            let _ = self.device.poll(wgpu::PollType::Poll);

            // Verificar se o mapeamento completou
            if let Some(result) = mapped.lock().unwrap().as_ref() {
                result.as_ref().map_err(|e| {
                    VectorizerError::Other(format!("Erro ao mapear buffer: {:?}", e))
                })?;
                break;
            }

            // Pequeno sleep para não consumir 100% CPU
            std::thread::sleep(std::time::Duration::from_micros(10));
        }

        // Verificar timeout
        if mapped.lock().unwrap().is_none() {
            return Err(VectorizerError::Other(
                "Timeout ao aguardar mapeamento de buffer GPU".to_string(),
            ));
        }

        // Ler dados mapeados
        let data = buffer_slice.get_mapped_range();
        let vec_result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        buffer.unmap();

        Ok(vec_result)
    }

    /// Atualizar dados de um buffer
    pub fn update_buffer(&self, buffer: &Buffer, data: &[f32]) {
        self.queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(data));
    }

    /// Calcular tamanho do buffer alinhado
    pub fn aligned_buffer_size(size: u64) -> u64 {
        // wgpu requer alinhamento de 256 bytes para buffers uniformes
        const ALIGNMENT: u64 = 256;
        ((size + ALIGNMENT - 1) / ALIGNMENT) * ALIGNMENT
    }
}

/// Parâmetros para computação
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg(feature = "wgpu-gpu")]
pub struct ComputeParams {
    pub query_count: u32,
    pub vector_count: u32,
    pub dimension: u32,
    pub _padding: u32,
}

#[cfg(feature = "wgpu-gpu")]
unsafe impl bytemuck::Pod for ComputeParams {}
#[cfg(feature = "wgpu-gpu")]
unsafe impl bytemuck::Zeroable for ComputeParams {}

#[cfg(feature = "wgpu-gpu")]
impl ComputeParams {
    pub fn new(query_count: usize, vector_count: usize, dimension: usize) -> Self {
        Self {
            query_count: query_count as u32,
            vector_count: vector_count as u32,
            dimension: dimension as u32,
            _padding: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    /// Calcular número total de comparações
    pub fn total_comparisons(&self) -> usize {
        (self.query_count as usize) * (self.vector_count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_buffer_size() {
        #[cfg(feature = "wgpu-gpu")]
        {
            assert_eq!(BufferManager::aligned_buffer_size(100), 256);
            assert_eq!(BufferManager::aligned_buffer_size(256), 256);
            assert_eq!(BufferManager::aligned_buffer_size(257), 512);
        }
    }

    #[test]
    fn test_compute_params() {
        #[cfg(feature = "wgpu-gpu")]
        {
            let params = ComputeParams::new(10, 100, 512);
            assert_eq!(params.query_count, 10);
            assert_eq!(params.vector_count, 100);
            assert_eq!(params.dimension, 512);
            assert_eq!(params.total_comparisons(), 1000);
        }
    }
}
