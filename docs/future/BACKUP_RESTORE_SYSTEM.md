# Backup & Restore System Specification

**Status**: Specification  
**Priority**: 🟢 **P2 - LOW** ⬇️  
**Complexity**: Medium  
**Created**: October 1, 2025  
**Updated**: October 1, 2025 - **PRIORITY DOWNGRADED BASED ON BENCHMARK ANALYSIS**

## 🎯 **WHY P2 PRIORITY - BENCHMARK INSIGHTS**

**Priority downgraded** based on benchmark analysis showing:
1. **System stability**: Benchmarks prove current system is reliable
2. **Performance excellence**: < 1ms search latency, no critical issues
3. **Manual backup sufficient**: Current backup methods work adequately
4. **Focus on higher ROI**: Quantization delivers 4x memory reduction + better quality
5. **Resource optimization**: Better to focus on features with immediate user value

## Requirements

- ✅ **Simple** - One command to backup/restore
- ✅ **Efficient** - Binary format with compression
- ✅ **Complete** - All application data in one file
- ❌ **NOT automatic** - Manual trigger only (avoid accumulation)
- ✅ **Incremental** - Optional differential backups
- ✅ **Verifiable** - Integrity checks

## Backup Format

### Archive Structure

```
vectorizer-backup-2025-10-01-153000.vzb  (compressed binary)
│
├── metadata.json           # Backup metadata
├── workspace-config.yml    # Workspace configuration
├── dynamic-config.yml      # Dynamic collections config
├── collections/
│   ├── workspace/
│   │   ├── {project}/
│   │   │   ├── {collection}/
│   │   │   │   ├── vectors.bin
│   │   │   │   ├── index.hnsw
│   │   │   │   ├── metadata.json
│   │   │   │   └── cache.bin
│   │   └── ...
│   └── dynamic/
│       ├── {collection-id}/
│       │   ├── vectors.bin
│       │   ├── index.hnsw
│       │   ├── metadata.json
│       │   └── wal.log
│       └── ...
├── auth/
│   ├── api_keys.bin        # Encrypted
│   ├── users.bin           # Encrypted
│   └── sessions.bin
├── metrics/
│   └── historical.bin
└── checksum.sha256         # Integrity verification
```

### Metadata

```json
{
  "version": "1.0",
  "vectorizer_version": "0.21.0",
  "created_at": "2025-10-01T15:30:00Z",
  "backup_type": "full",
  "compression": "lz4",
  "encryption": "aes-256-gcm",
  "collections": {
    "workspace": 15,
    "dynamic": 3
  },
  "total_vectors": 1245382,
  "total_size_mb": 3420.5,
  "compressed_size_mb": 856.2,
  "checksum": "sha256:..."
}
```

## Implementation

### 1. Backup Manager

```rust
use lz4_flex::frame::{FrameEncoder, FrameDecoder};
use tar::Builder as TarBuilder;

pub struct BackupManager {
    data_dir: PathBuf,
    backup_dir: PathBuf,
}

impl BackupManager {
    pub async fn create_backup(
        &self,
        options: BackupOptions,
    ) -> Result<BackupResult> {
        let backup_id = format!(
            "vectorizer-backup-{}.vzb",
            Utc::now().format("%Y-%m-%d-%H%M%S")
        );
        let backup_path = self.backup_dir.join(&backup_id);
        
        info!("Creating backup: {}", backup_id);
        
        // 1. Create tar archive
        let tar_buffer = Vec::new();
        let mut tar = TarBuilder::new(tar_buffer);
        
        // 2. Add metadata
        let metadata = self.generate_metadata().await?;
        tar.append_data(
            &mut metadata.as_bytes(),
            Path::new("metadata.json"),
            metadata.len() as u64,
        )?;
        
        // 3. Add workspace config
        self.add_file_to_tar(&mut tar, "workspace-config.yml").await?;
        
        // 4. Add all collections
        for collection_path in self.discover_collections().await? {
            self.add_collection_to_tar(&mut tar, &collection_path).await?;
        }
        
        // 5. Add auth data (encrypted)
        self.add_encrypted_auth_data(&mut tar).await?;
        
        // 6. Add metrics (optional)
        if options.include_metrics {
            self.add_metrics_to_tar(&mut tar).await?;
        }
        
        // 7. Finish tar and get buffer
        let tar_buffer = tar.into_inner()?;
        
        // 8. Compress with LZ4
        let compressed = if options.compress {
            let mut encoder = FrameEncoder::new(Vec::new());
            encoder.write_all(&tar_buffer)?;
            encoder.finish()?
        } else {
            tar_buffer
        };
        
        // 9. Calculate checksum
        let checksum = calculate_sha256(&compressed);
        
        // 10. Write final file
        let mut file = File::create(&backup_path)?;
        file.write_all(&compressed)?;
        file.write_all(&checksum)?;
        file.sync_all()?;
        
        info!("Backup created successfully: {} ({} MB)",
            backup_id,
            compressed.len() as f64 / 1024.0 / 1024.0
        );
        
        Ok(BackupResult {
            backup_id,
            path: backup_path,
            size_mb: compressed.len() as f64 / 1024.0 / 1024.0,
            checksum: hex::encode(checksum),
            duration: start.elapsed(),
        })
    }
}
```

### 2. Restore Manager

```rust
impl BackupManager {
    pub async fn restore_backup(
        &self,
        backup_path: &Path,
        options: RestoreOptions,
    ) -> Result<RestoreResult> {
        info!("Restoring backup from: {}", backup_path.display());
        
        // 1. Read and verify checksum
        let (compressed_data, stored_checksum) = self.read_and_split_checksum(backup_path)?;
        let calculated_checksum = calculate_sha256(&compressed_data);
        
        if stored_checksum != calculated_checksum {
            return Err(Error::ChecksumMismatch {
                expected: hex::encode(stored_checksum),
                actual: hex::encode(calculated_checksum),
            });
        }
        
        // 2. Decompress
        let tar_data = if options.is_compressed() {
            let mut decoder = FrameDecoder::new(&compressed_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        } else {
            compressed_data
        };
        
        // 3. Extract tar archive
        let mut archive = tar::Archive::new(&tar_data[..]);
        
        // 4. Read metadata
        let metadata = self.extract_metadata(&mut archive)?;
        
        // 5. Validate compatibility
        self.validate_compatibility(&metadata)?;
        
        // 6. Stop current operations
        if options.stop_server {
            self.stop_server().await?;
        }
        
        // 7. Restore workspace config
        self.restore_workspace_config(&mut archive).await?;
        
        // 8. Restore collections
        let mut restored_collections = 0;
        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?;
            
            if path.starts_with("collections/") {
                self.restore_collection_entry(entry).await?;
                restored_collections += 1;
            }
        }
        
        // 9. Restore auth data
        if options.restore_auth {
            self.restore_auth_data(&mut archive).await?;
        }
        
        // 10. Restore metrics
        if options.restore_metrics {
            self.restore_metrics(&mut archive).await?;
        }
        
        // 11. Rebuild indexes if needed
        if options.rebuild_indexes {
            self.rebuild_all_indexes().await?;
        }
        
        // 12. Start server
        if options.start_server {
            self.start_server().await?;
        }
        
        Ok(RestoreResult {
            collections_restored: restored_collections,
            duration: start.elapsed(),
            warnings: vec![],
        })
    }
}
```

### 3. Incremental Backups

```rust
pub struct IncrementalBackup {
    base_backup: PathBuf,
    last_backup_time: DateTime<Utc>,
}

impl IncrementalBackup {
    pub async fn create_incremental(
        &self,
        since: DateTime<Utc>,
    ) -> Result<BackupResult> {
        // 1. Find changed collections
        let changed = self.find_changed_since(since).await?;
        
        if changed.is_empty() {
            return Ok(BackupResult::no_changes());
        }
        
        // 2. Create differential backup
        let backup_id = format!(
            "vectorizer-incremental-{}.vzb",
            Utc::now().format("%Y-%m-%d-%H%M%S")
        );
        
        let mut tar = TarBuilder::new(Vec::new());
        
        // 3. Add metadata with base reference
        let metadata = IncrementalMetadata {
            backup_type: BackupType::Incremental,
            base_backup: self.base_backup.file_name().unwrap().to_string_lossy().to_string(),
            since,
            changes: changed.len(),
        };
        
        tar.append_data(..., "metadata.json", ...)?;
        
        // 4. Add only changed collections
        for collection in changed {
            self.add_collection_to_tar(&mut tar, &collection).await?;
        }
        
        // 5. Compress and save
        // ... (same as full backup)
        
        Ok(BackupResult { ... })
    }
}
```

## CLI Commands

```bash
# Create full backup
vzr backup create --output ./backups/

# Create incremental backup
vzr backup create --incremental --since-backup backup-001.vzb

# List backups
vzr backup list
# Output:
# ID  Type          Date                Size    Collections
# 1   Full          2025-10-01 10:00   856 MB  18
# 2   Incremental   2025-10-01 14:00   45 MB   3
# 3   Full          2025-10-01 18:00   892 MB  18

# Restore from backup
vzr backup restore backup-001.vzb

# Restore with options
vzr backup restore backup-001.vzb \
    --skip-auth \
    --rebuild-indexes \
    --no-stop-server

# Verify backup integrity
vzr backup verify backup-001.vzb

# Show backup info
vzr backup info backup-001.vzb
```

## Dashboard Integration

```html
<div class="backup-manager">
    <h2>Backup & Restore</h2>
    
    <!-- Create Backup -->
    <div class="backup-create">
        <h3>Create Backup</h3>
        
        <form id="backup-form">
            <label>
                <input type="radio" name="type" value="full" checked> Full Backup
            </label>
            <label>
                <input type="radio" name="type" value="incremental"> Incremental Backup
            </label>
            
            <label>
                <input type="checkbox" name="include-metrics"> Include Metrics
            </label>
            
            <label>
                <input type="checkbox" name="compress" checked> Compress (LZ4)
            </label>
            
            <button type="submit" class="btn-primary">Create Backup</button>
        </form>
        
        <div id="backup-progress" style="display:none">
            <progress id="backup-progress-bar" max="100" value="0"></progress>
            <span id="backup-status">Creating backup...</span>
        </div>
    </div>
    
    <!-- Backup List -->
    <div class="backup-list">
        <h3>Available Backups</h3>
        
        <table class="backups-table">
            <thead>
                <tr>
                    <th>Date</th>
                    <th>Type</th>
                    <th>Size</th>
                    <th>Collections</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody id="backups-tbody">
                <!-- Dynamic rows -->
            </tbody>
        </table>
    </div>
</div>

<script>
async function createBackup(formData) {
    const progressDiv = document.getElementById('backup-progress');
    progressDiv.style.display = 'block';
    
    const response = await fetch('/api/backup/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData)
    });
    
    // Stream progress
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    
    while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        
        const text = decoder.decode(value);
        const progress = JSON.parse(text);
        
        document.getElementById('backup-progress-bar').value = progress.percent;
        document.getElementById('backup-status').textContent = progress.message;
    }
    
    progressDiv.style.display = 'none';
    alert('Backup created successfully!');
    loadBackups();
}
</script>
```

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Full Backup (1M vectors) | < 60s | Including compression |
| Incremental Backup | < 10s | Only changed data |
| Restore | < 120s | Including decompression |
| Verify | < 30s | Checksum validation |
| Compression Ratio | 4:1 | LZ4 typical |

## Configuration

```yaml
# config.yml
backup:
  # Storage
  backup_dir: ./backups
  max_backups: 10          # Keep last 10
  auto_cleanup: true
  
  # Compression
  compression:
    enabled: true
    algorithm: lz4
    level: 1               # 1-12 (higher = slower but smaller)
  
  # Encryption (optional)
  encryption:
    enabled: false
    algorithm: aes-256-gcm
    key_file: ./backup.key
  
  # What to include
  include:
    workspace_collections: true
    dynamic_collections: true
    auth_data: true
    metrics: false         # Usually skip metrics
    logs: false            # Skip logs
  
  # Verification
  verify_after_backup: true
  verify_before_restore: true
```

## Implementation

```rust
// src/backup/mod.rs
pub mod manager;
pub mod format;
pub mod compression;
pub mod encryption;
pub mod verification;

// src/backup/manager.rs
pub struct BackupManager {
    config: BackupConfig,
    data_dir: PathBuf,
    backup_dir: PathBuf,
}

impl BackupManager {
    pub async fn create_full_backup(&self) -> Result<BackupResult> {
        let start = Instant::now();
        let backup_id = self.generate_backup_id();
        
        // Create backup builder
        let mut builder = BackupBuilder::new(&backup_id);
        
        // Add all data
        builder.add_workspace_config()?;
        builder.add_workspace_collections().await?;
        builder.add_dynamic_collections().await?;
        builder.add_auth_data().await?;
        
        if self.config.include.metrics {
            builder.add_metrics().await?;
        }
        
        // Build and compress
        let archive = builder.build()?;
        let compressed = self.compress(archive)?;
        
        // Calculate checksum
        let checksum = self.calculate_checksum(&compressed)?;
        
        // Write to disk
        let backup_path = self.backup_dir.join(&backup_id);
        self.write_backup(&backup_path, &compressed, &checksum).await?;
        
        // Verify if configured
        if self.config.verify_after_backup {
            self.verify_backup(&backup_path).await?;
        }
        
        // Cleanup old backups
        if self.config.auto_cleanup {
            self.cleanup_old_backups().await?;
        }
        
        Ok(BackupResult {
            backup_id,
            path: backup_path,
            size_mb: compressed.len() as f64 / 1024.0 / 1024.0,
            duration: start.elapsed(),
            collections_backed_up: builder.collection_count(),
            checksum: hex::encode(checksum),
        })
    }
    
    pub async fn restore_backup(
        &self,
        backup_path: &Path,
        options: RestoreOptions,
    ) -> Result<RestoreResult> {
        // 1. Verify backup
        if self.config.verify_before_restore {
            self.verify_backup(backup_path).await?;
        }
        
        // 2. Read backup
        let (compressed, checksum) = self.read_backup(backup_path).await?;
        
        // 3. Decompress
        let archive = self.decompress(&compressed)?;
        
        // 4. Extract metadata
        let metadata = self.extract_metadata(&archive)?;
        
        // 5. Validate compatibility
        self.validate_compatibility(&metadata)?;
        
        // 6. Confirm with user (if interactive)
        if options.interactive {
            self.confirm_restore(&metadata)?;
        }
        
        // 7. Stop server
        if options.stop_server {
            self.stop_server().await?;
        }
        
        // 8. Clear existing data (if requested)
        if options.clear_existing {
            self.clear_data_dir().await?;
        }
        
        // 9. Extract all files
        let extractor = ArchiveExtractor::new(archive);
        let restored = extractor.extract_all(&self.data_dir).await?;
        
        // 10. Rebuild indexes if needed
        if options.rebuild_indexes {
            self.rebuild_all_indexes().await?;
        }
        
        // 11. Start server
        if options.start_server {
            self.start_server().await?;
        }
        
        Ok(RestoreResult {
            collections_restored: restored.collections,
            vectors_restored: restored.vectors,
            duration: start.elapsed(),
        })
    }
}
```

### 3. Incremental Backup Strategy

```rust
pub struct IncrementalBackupStrategy {
    base_backup: PathBuf,
    change_tracker: ChangeTracker,
}

pub struct ChangeTracker {
    // Track modifications since last backup
    modified_collections: HashSet<String>,
    last_backup_time: DateTime<Utc>,
}

impl ChangeTracker {
    pub fn record_change(&mut self, collection: &str) {
        self.modified_collections.insert(collection.to_string());
    }
    
    pub fn get_changes_since(&self, since: DateTime<Utc>) -> Vec<String> {
        self.modified_collections
            .iter()
            .filter(|c| self.was_modified_since(c, since))
            .cloned()
            .collect()
    }
}
```

## API Endpoints

```http
# Create backup
POST /api/backup/create
Content-Type: application/json
{
  "type": "full",
  "compress": true,
  "include_metrics": false
}
Response: 202 Accepted
{
  "job_id": "backup-123",
  "status": "in_progress"
}

# Get backup status
GET /api/backup/status/backup-123
Response: {
  "status": "in_progress",
  "progress": 45,
  "message": "Backing up collection gateway-code..."
}

# List backups
GET /api/backup/list
Response: [
  {
    "id": "vectorizer-backup-2025-10-01-153000.vzb",
    "created_at": "2025-10-01T15:30:00Z",
    "size_mb": 856.2,
    "type": "full",
    "collections": 18,
    "vectors": 1245382
  }
]

# Download backup
GET /api/backup/download/{backup_id}
Response: Binary download

# Restore backup
POST /api/backup/restore/{backup_id}
Content-Type: application/json
{
  "stop_server": true,
  "rebuild_indexes": false,
  "restore_auth": true
}

# Delete backup
DELETE /api/backup/{backup_id}
```

## Backup Rotation Strategy

```rust
pub struct BackupRotation {
    keep_daily: usize,    // 7
    keep_weekly: usize,   // 4
    keep_monthly: usize,  // 6
}

impl BackupRotation {
    pub fn should_keep(&self, backup: &BackupMetadata, now: DateTime<Utc>) -> bool {
        let age = now.signed_duration_since(backup.created_at);
        
        match age.num_days() {
            0 => true,                    // Keep today's
            1..=7 => true,                // Keep last 7 days
            8..=30 => {
                // Keep one per week
                backup.created_at.weekday() == Weekday::Sun
            },
            31..=180 => {
                // Keep one per month
                backup.created_at.day() == 1
            },
            _ => false,                   // Older than 6 months
        }
    }
    
    pub async fn cleanup_old_backups(&self, backups: Vec<BackupMetadata>) -> Result<usize> {
        let now = Utc::now();
        let mut removed = 0;
        
        for backup in backups {
            if !self.should_keep(&backup, now) {
                self.delete_backup(&backup.path).await?;
                removed += 1;
            }
        }
        
        Ok(removed)
    }
}
```

## Security

### Encryption

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct BackupEncryption {
    cipher: Aes256Gcm,
}

impl BackupEncryption {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(b"unique nonce"); // Generate random
        let ciphertext = self.cipher.encrypt(nonce, data)
            .map_err(|e| Error::EncryptionFailed(e.to_string()))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Extract nonce
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))
    }
}
```

## Testing

```rust
#[tokio::test]
async fn test_full_backup_restore_cycle() {
    // 1. Setup test data
    let store = create_test_store_with_data(10_000).await;
    let manager = BackupManager::new("./test-backups");
    
    // 2. Create backup
    let backup = manager.create_full_backup(BackupOptions::default()).await?;
    assert!(backup.path.exists());
    
    // 3. Modify data
    store.insert_vectors("test-collection", generate_vectors(1000)).await?;
    
    // 4. Restore backup
    manager.restore_backup(&backup.path, RestoreOptions::default()).await?;
    
    // 5. Verify data matches original
    let restored_store = load_vector_store().await?;
    assert_eq!(store.vector_count(), restored_store.vector_count());
}

#[tokio::test]
async fn test_incremental_backup() {
    let manager = BackupManager::new("./test-backups");
    
    // Create base backup
    let base = manager.create_full_backup(Default::default()).await?;
    
    // Modify one collection
    modify_collection("test-collection").await?;
    
    // Create incremental
    let incremental = manager.create_incremental_backup(&base.path).await?;
    
    // Should be much smaller
    assert!(incremental.size_mb < base.size_mb * 0.2);
}
```

## Success Criteria

- ✅ Backup completes in < 60s for 1M vectors
- ✅ Compression ratio ≥ 4:1
- ✅ Restore completes in < 120s
- ✅ Zero data loss on restore
- ✅ Checksum verification 100% reliable
- ✅ Incremental backups < 20% of full size
- ✅ CLI and Dashboard UIs both work
- ✅ Encrypted backups secure

---

**Estimated Effort**: 3 weeks  
**Dependencies**: None  
**Risk**: Low (well-defined problem)

