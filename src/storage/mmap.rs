use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use memmap2::{MmapMut, MmapOptions};

use crate::error::{Result, VectorizerError};

// Header size: 8 bytes for count (usize)
const HEADER_SIZE: usize = 8;

/// Memory-mapped vector storage for handling large datasets
/// Note: This struct is NOT thread-safe on its own. It should be wrapped
/// in an external RwLock (as done in VectorStorageBackend) for concurrent access.
#[derive(Debug)]
pub struct MmapVectorStorage {
    file: File,
    mmap: MmapMut,
    dimension: usize,
    count: usize,
    capacity: usize,
}

impl MmapVectorStorage {
    /// Open or create a memory-mapped vector storage
    pub fn open(path: &Path, dimension: usize) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let file_len = file.metadata()?.len() as usize;
        let vector_size = dimension * std::mem::size_of::<f32>();

        // Ensure file is at least large enough for header or initial capacity
        let min_file_size = HEADER_SIZE + (vector_size * 1000).max(1024 * 1024);
        if file_len < HEADER_SIZE {
            file.set_len(min_file_size as u64)?;
        }

        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        // Read count from header
        let count = if file_len >= HEADER_SIZE {
            let count_bytes = &mmap[0..HEADER_SIZE];
            usize::from_le_bytes([
                count_bytes[0],
                count_bytes[1],
                count_bytes[2],
                count_bytes[3],
                count_bytes[4],
                count_bytes[5],
                count_bytes[6],
                count_bytes[7],
            ])
        } else {
            0
        };

        // Calculate capacity (excluding header)
        let data_size = mmap.len().saturating_sub(HEADER_SIZE);
        let capacity = data_size / vector_size;

        Ok(Self {
            file,
            mmap,
            dimension,
            count,
            capacity,
        })
    }

    /// Append a vector to storage
    pub fn append(&mut self, vector: &[f32]) -> Result<usize> {
        if vector.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        let vector_size = self.dimension * std::mem::size_of::<f32>();

        // Resize if needed
        if self.count >= self.capacity {
            let current_len = self.mmap.len();
            let new_len = (current_len * 2).max(HEADER_SIZE + vector_size * 1000);
            self.file.set_len(new_len as u64)?;

            self.mmap = unsafe { MmapOptions::new().map_mut(&self.file)? };
            let data_size = self.mmap.len().saturating_sub(HEADER_SIZE);
            self.capacity = data_size / vector_size;
        }

        // Calculate offset (after header)
        let offset = HEADER_SIZE + (self.count * vector_size);
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(vector.as_ptr() as *const u8, vector_size) };

        self.mmap[offset..offset + vector_size].copy_from_slice(bytes);

        // Update count in header
        let id = self.count;
        self.count += 1;

        // Write new count to header
        let new_count_bytes = self.count.to_le_bytes();
        self.mmap[0..HEADER_SIZE].copy_from_slice(&new_count_bytes);

        Ok(id)
    }

    /// Update a vector at a specific index
    pub fn update(&mut self, index: usize, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        if index >= self.count {
            return Err(VectorizerError::Storage(format!(
                "Index {} out of bounds (count: {})",
                index, self.count
            )));
        }

        let vector_size = self.dimension * std::mem::size_of::<f32>();
        let offset = HEADER_SIZE + (index * vector_size);
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(vector.as_ptr() as *const u8, vector_size) };

        self.mmap[offset..offset + vector_size].copy_from_slice(bytes);

        Ok(())
    }

    /// Read a vector by index
    pub fn get(&self, index: usize) -> Result<Vec<f32>> {
        if index >= self.count {
            return Err(VectorizerError::NotFound(format!("Vector index {}", index)));
        }

        let vector_size = self.dimension * std::mem::size_of::<f32>();
        // Calculate offset (after header)
        let offset = HEADER_SIZE + (index * vector_size);

        let bytes = &self.mmap[offset..offset + vector_size];

        let mut vector = vec![0.0f32; self.dimension];
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                vector.as_mut_ptr() as *mut u8,
                vector_size,
            );
        }

        Ok(vector)
    }

    /// Get number of vectors stored
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Flush changes to disk
    pub fn flush(&self) -> Result<()> {
        self.mmap.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_mmap_storage() {
        let tmp_file = NamedTempFile::new().unwrap();
        let path = tmp_file.path();
        let dim = 4;

        let mut storage = MmapVectorStorage::open(path, dim).unwrap();

        let v1 = vec![1.0, 2.0, 3.0, 4.0];
        let v2 = vec![5.0, 6.0, 7.0, 8.0];

        let id1 = storage.append(&v1).unwrap();
        let id2 = storage.append(&v2).unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(storage.len(), 2);

        let read_v1 = storage.get(0).unwrap();
        let read_v2 = storage.get(1).unwrap();

        assert_eq!(read_v1, v1);
        assert_eq!(read_v2, v2);
    }

    #[test]
    fn test_mmap_persistence_and_recovery() {
        use std::fs;
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test_mmap.vec");
        let dim = 128;

        // Create storage and insert vectors
        {
            let mut storage = MmapVectorStorage::open(&path, dim).unwrap();

            for i in 0..10 {
                let vector: Vec<f32> = (0..dim).map(|j| (i as f32 + j as f32) * 0.1).collect();
                storage.append(&vector).unwrap();
            }

            storage.flush().unwrap();
        }

        // Verify file exists
        assert!(path.exists());

        // Reopen storage and verify data persistence
        {
            let storage = MmapVectorStorage::open(&path, dim).unwrap();
            assert_eq!(storage.len(), 10);

            // Verify we can read the vectors
            for i in 0..10 {
                let vector = storage.get(i).unwrap();
                assert_eq!(vector.len(), dim);
                // Verify first element matches expected pattern
                assert!((vector[0] - (i as f32 * 0.1)).abs() < 0.001);
            }
        }
    }
}
