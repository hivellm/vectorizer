<template>
  <div class="p-8">
    <div class="flex gap-6 h-full">
      <!-- Sidebar Navigation -->
      <div class="w-48 flex-shrink-0">
        <nav class="space-y-1">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="activeTab = tab.id"
            class="w-full flex items-center gap-3 px-3 py-2 text-sm rounded transition-colors"
            :class="activeTab === tab.id ? 'bg-bg-elevated text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-bg-hover'"
      >
            <i :class="[tab.icon, 'w-4 text-center text-xs']"></i>
            <span>{{ tab.label }}</span>
      </button>
        </nav>
    </div>

      <!-- Main Content -->
      <div class="flex-1 bg-bg-secondary border border-border rounded-xl p-6 overflow-y-auto">
        <!-- General Tab -->
        <div v-if="activeTab === 'general'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">General Settings</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Server</h3>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Host</label>
                  <input v-model="config.server.host" type="text" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Port</label>
                  <input v-model.number="config.server.port" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
              </div>
              <div class="mt-4">
                <label class="block text-xs font-medium text-text-secondary mb-2">Data Directory</label>
                <input v-model="config.server.data_dir" type="text" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
              </div>
            </div>
          </div>
          </div>

        <!-- Embedding Tab -->
        <div v-if="activeTab === 'embedding'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Embedding Settings</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Model Configuration</h3>
              <div>
                <label class="block text-xs font-medium text-text-secondary mb-2">Default Model</label>
                <select v-model="config.collections.defaults.embedding.model" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                  <option value="bm25">BM25</option>
                  <option value="bow">Bag of Words</option>
                  <option value="hash">Hash</option>
                  <option value="ngram">N-gram</option>
                  <option value="hnsw">HNSW</option>
                  <option value="optimized_hnsw">Optimized HNSW</option>
                </select>
              </div>
            </div>
          </div>
          </div>

        <!-- Collections Tab -->
        <div v-if="activeTab === 'collections'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Collection Defaults</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Vector Settings</h3>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Dimension</label>
                  <input v-model.number="config.collections.defaults.dimension" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Metric</label>
                  <select v-model="config.collections.defaults.metric" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                    <option value="cosine">Cosine</option>
                    <option value="euclidean">Euclidean</option>
                    <option value="dot_product">Dot Product</option>
                  </select>
                </div>
              </div>
          </div>

            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Index Settings</h3>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Index Type</label>
                  <select v-model="config.collections.defaults.index.type" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                    <option value="hnsw">HNSW</option>
                    <option value="optimized_hnsw">Optimized HNSW</option>
                  </select>
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Quantization Type</label>
                  <select v-model="config.collections.defaults.quantization.type" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                    <option value="sq">Scalar Quantization</option>
                  </select>
                </div>
          </div>
        </div>

            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Sharding</h3>
              <div class="space-y-4">
                <div>
                  <CustomCheckbox v-model="config.collections.defaults.sharding.enabled" label="Enable Sharding" @change="markDirty" />
                </div>
                <div v-if="config.collections.defaults.sharding.enabled" class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Target Max Size</label>
                    <input v-model.number="config.collections.defaults.sharding.target_max_size" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Soft Limit Size</label>
                    <input v-model.number="config.collections.defaults.sharding.soft_limit_size" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                </div>
              </div>
            </div>
            </div>
          </div>

        <!-- Performance Tab -->
        <div v-if="activeTab === 'performance'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Performance Settings</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">CPU</h3>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Max Threads</label>
                  <input v-model.number="config.performance.cpu.max_threads" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Memory Pool Size (MB)</label>
                  <input v-model.number="config.performance.cpu.memory_pool_size_mb" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
              </div>
              <div class="mt-4">
                <CustomCheckbox v-model="config.performance.cpu.enable_simd" label="Enable SIMD" @change="markDirty" />
          </div>
        </div>

            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Batch Processing</h3>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Default Batch Size</label>
                  <input v-model.number="config.performance.batch.default_size" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Max Batch Size</label>
                  <input v-model.number="config.performance.batch.max_size" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
              </div>
              <div class="mt-4">
                <CustomCheckbox v-model="config.performance.batch.parallel_processing" label="Parallel Processing" @change="markDirty" />
              </div>
            </div>
          </div>
          </div>

        <!-- File Watcher Tab -->
        <div v-if="activeTab === 'file_watcher'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">File Watcher Settings</h2>
          
          <div class="space-y-6">
            <CustomCheckbox v-model="config.file_watcher.enabled" label="Enable File Watcher" @change="markDirty" />
            
            <div v-if="config.file_watcher.enabled" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Debounce Delay (ms)</label>
                  <input v-model.number="config.file_watcher.debounce_delay_ms" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Collection Name</label>
                  <input v-model="config.file_watcher.collection_name" type="text" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
              </div>
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Min File Size (bytes)</label>
                  <input v-model.number="config.file_watcher.min_file_size_bytes" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Max File Size (bytes)</label>
                  <input v-model.number="config.file_watcher.max_file_size_bytes" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
              </div>
              <CustomCheckbox v-model="config.file_watcher.hash_validation_enabled" label="Hash Validation Enabled" @change="markDirty" />
            </div>
          </div>
          </div>

        <!-- Logging Tab -->
        <div v-if="activeTab === 'logging'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Logging Settings</h2>
          
          <div class="space-y-6">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-medium text-text-secondary mb-2">Log Level</label>
                <select v-model="config.logging.level" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                  <option value="error">Error</option>
                  <option value="warn">Warning</option>
                  <option value="info">Info</option>
                  <option value="debug">Debug</option>
                  <option value="trace">Trace</option>
                </select>
          </div>
              <div>
                <label class="block text-xs font-medium text-text-secondary mb-2">Log Format</label>
                <select v-model="config.logging.format" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                  <option value="json">JSON</option>
                  <option value="text">Text</option>
            </select>
          </div>
        </div>

            <div class="space-y-4">
              <div>
                <CustomCheckbox v-model="config.logging.log_requests" label="Log Requests" @change="markDirty" />
              </div>
              <div>
                <CustomCheckbox v-model="config.logging.log_responses" label="Log Responses" @change="markDirty" />
              </div>
              <div>
                <CustomCheckbox v-model="config.logging.log_errors" label="Log Errors" @change="markDirty" />
              </div>
            </div>
          </div>
        </div>

        <!-- Transmutation Tab -->
        <div v-if="activeTab === 'transmutation'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Transmutation Settings</h2>
          
          <div class="space-y-6">
            <CustomCheckbox v-model="config.transmutation.enabled" label="Enable Transmutation" @change="markDirty" />
            
            <div v-if="config.transmutation.enabled" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Max File Size (MB)</label>
                  <input v-model.number="config.transmutation.max_file_size_mb" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Conversion Timeout (seconds)</label>
                  <input v-model.number="config.transmutation.conversion_timeout_secs" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
              </div>
              <CustomCheckbox v-model="config.transmutation.preserve_images" label="Preserve Images" @change="markDirty" />
            </div>
          </div>
        </div>

        <!-- Normalization Tab -->
        <div v-if="activeTab === 'normalization'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Normalization Settings</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">General</h3>
              <div class="space-y-4">
                <CustomCheckbox v-model="config.normalization.enabled" label="Enable Normalization" @change="markDirty" />
                <div v-if="config.normalization.enabled">
                  <label class="block text-xs font-medium text-text-secondary mb-2">Normalization Level</label>
                  <select v-model="config.normalization.level" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                    <option value="conservative">Conservative</option>
                    <option value="moderate">Moderate</option>
                    <option value="aggressive">Aggressive</option>
                  </select>
                </div>
              </div>
            </div>

            <div v-if="config.normalization.enabled">
              <h3 class="text-sm font-semibold text-text-primary mb-3">Line Endings</h3>
              <div class="space-y-4">
                <div>
                  <CustomCheckbox v-model="config.normalization.line_endings.normalize_crlf" label="Normalize CRLF to LF" @change="markDirty" />
                </div>
                <div>
                  <CustomCheckbox v-model="config.normalization.line_endings.normalize_cr" label="Normalize CR to LF" @change="markDirty" />
                </div>
                <div>
                  <CustomCheckbox v-model="config.normalization.line_endings.collapse_multiple_newlines" label="Collapse Multiple Newlines" @change="markDirty" />
                </div>
                <div>
                  <CustomCheckbox v-model="config.normalization.line_endings.trim_trailing_whitespace" label="Trim Trailing Whitespace" @change="markDirty" />
                </div>
              </div>
            </div>

            <div v-if="config.normalization.enabled">
              <h3 class="text-sm font-semibold text-text-primary mb-3">Content Detection</h3>
              <div class="space-y-4">
                <div>
                  <CustomCheckbox v-model="config.normalization.content_detection.enabled" label="Enable Content Detection" @change="markDirty" />
                </div>
                <div>
                  <CustomCheckbox v-model="config.normalization.content_detection.preserve_code_structure" label="Preserve Code Structure" @change="markDirty" />
                </div>
                <div>
                  <CustomCheckbox v-model="config.normalization.content_detection.preserve_markdown_format" label="Preserve Markdown Format" @change="markDirty" />
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Workspace Tab -->
        <div v-if="activeTab === 'workspace'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Workspace Settings</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">General</h3>
              <div class="space-y-4">
                <div>
                  <CustomCheckbox v-model="config.workspace.enabled" label="Enable Workspace" @change="markDirty" />
                </div>
                <div>
                  <CustomCheckbox v-model="config.workspace.auto_load_collections" label="Auto Load Collections" @change="markDirty" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-text-secondary mb-2">Default Workspace File</label>
                  <input v-model="config.workspace.default_workspace_file" type="text" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Storage Tab -->
        <div v-if="activeTab === 'storage'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">Storage Settings</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Compression</h3>
              <div class="space-y-4">
                <CustomCheckbox v-model="config.storage.compression.enabled" label="Enable Compression" @change="markDirty" />
                <div v-if="config.storage.compression.enabled" class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Format</label>
                    <select v-model="config.storage.compression.format" @change="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-border-light transition-colors text-sm">
                      <option value="zstd">Zstandard</option>
                    </select>
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Level (1-22)</label>
                    <input v-model.number="config.storage.compression.level" type="number" min="1" max="22" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                </div>
              </div>
            </div>

            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Snapshots</h3>
              <div class="space-y-4">
                <CustomCheckbox v-model="config.storage.snapshots.enabled" label="Enable Snapshots" @change="markDirty" />
                <div v-if="config.storage.snapshots.enabled" class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Interval (hours)</label>
                    <input v-model.number="config.storage.snapshots.interval_hours" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Retention (days)</label>
                    <input v-model.number="config.storage.snapshots.retention_days" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Max Snapshots</label>
                    <input v-model.number="config.storage.snapshots.max_snapshots" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Snapshots Path</label>
                    <input v-model="config.storage.snapshots.path" type="text" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- API Tab -->
        <div v-if="activeTab === 'api'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">API Settings</h2>
          
          <div class="space-y-6">
            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">REST API</h3>
              <div class="space-y-4">
                <CustomCheckbox v-model="config.api.rest.enabled" label="Enable REST API" @change="markDirty" />
                <div v-if="config.api.rest.enabled" class="space-y-4">
                  <CustomCheckbox v-model="config.api.rest.cors_enabled" label="Enable CORS" @change="markDirty" />
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <label class="block text-xs font-medium text-text-secondary mb-2">Max Request Size (MB)</label>
                      <input v-model.number="config.api.rest.max_request_size_mb" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                    </div>
                    <div>
                      <label class="block text-xs font-medium text-text-secondary mb-2">Timeout (seconds)</label>
                      <input v-model.number="config.api.rest.timeout_seconds" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                    </div>
                  </div>
                </div>
              </div>
          </div>

            <div>
              <h3 class="text-sm font-semibold text-text-primary mb-3">MCP (Model Context Protocol)</h3>
              <div class="space-y-4">
                <CustomCheckbox v-model="config.api.mcp.enabled" label="Enable MCP" @change="markDirty" />
                <div v-if="config.api.mcp.enabled" class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Port</label>
                    <input v-model.number="config.api.mcp.port" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-text-secondary mb-2">Max Connections</label>
                    <input v-model.number="config.api.mcp.max_connections" type="number" @input="markDirty" class="w-full px-3 py-2 bg-bg-tertiary border border-border rounded text-text-primary placeholder-text-muted focus:outline-none focus:border-border-light transition-colors text-sm" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- YAML Editor Tab -->
        <div v-if="activeTab === 'yaml'">
          <h2 class="text-xl font-semibold text-text-primary mb-6">YAML Editor</h2>
          <MonacoEditor
            v-model:value="yamlContent"
            language="yaml"
            :read-only="false"
            height="500px"
            @change="markDirty"
          />
        </div>

        <!-- Help Text -->
        <div class="mt-6 p-4 bg-bg-tertiary border border-border rounded">
          <p class="text-xs text-text-secondary">
            Configure your Vectorizer settings using the form above or edit the YAML directly. Changes are automatically saved when you click 'Save & Restart'.
          </p>
        </div>
      </div>
    </div>

    <!-- Unsaved Changes Indicator -->
    <div v-if="isDirty" class="fixed bottom-6 right-6 px-4 py-3 bg-bg-elevated border border-border rounded-lg shadow-lg flex items-center gap-3 z-tooltip">
      <i class="fas fa-exclamation-circle text-warning"></i>
      <span class="text-sm text-text-secondary">Unsaved changes</span>
    </div>

    <!-- Success Toast -->
    <div v-if="showSaveSuccess" class="fixed top-4 right-4 z-modal">
      <div class="bg-success text-white px-6 py-3 rounded-lg shadow-lg flex items-center gap-3 animate-slide-in">
        <i class="fas fa-check-circle"></i>
        <span>{{ saveMessage }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, watch } from 'vue';
import { useVectorizerStore } from '../stores/vectorizer';
import CustomCheckbox from '../components/CustomCheckbox.vue';
import MonacoEditor from '../components/MonacoEditor.vue';

const vectorizerStore = useVectorizerStore();

const activeTab = ref('general');
const isDirty = ref(false);
const yamlContent = ref('');
const showSaveSuccess = ref(false);
const saveMessage = ref('');

const tabs = [
  { id: 'general', label: 'General', icon: 'fas fa-server' },
  { id: 'embedding', label: 'Embedding', icon: 'fas fa-brain' },
  { id: 'collections', label: 'Collections', icon: 'fas fa-layer-group' },
  { id: 'performance', label: 'Performance', icon: 'fas fa-tachometer-alt' },
  { id: 'file_watcher', label: 'File Watcher', icon: 'fas fa-eye' },
  { id: 'logging', label: 'Logging', icon: 'fas fa-file-alt' },
  { id: 'transmutation', label: 'Transmutation', icon: 'fas fa-exchange-alt' },
  { id: 'normalization', label: 'Normalization', icon: 'fas fa-align-left' },
  { id: 'workspace', label: 'Workspace', icon: 'fas fa-folder' },
  { id: 'storage', label: 'Storage', icon: 'fas fa-database' },
  { id: 'api', label: 'API', icon: 'fas fa-plug' },
  { id: 'yaml', label: 'YAML', icon: 'fas fa-code' }
];

const config = reactive({
  file_watcher: {
    enabled: false,
    debounce_delay_ms: 1000,
    min_file_size_bytes: 1,
    max_file_size_bytes: 10485760,
    hash_validation_enabled: true,
    collection_name: 'workspace-files'
  },
  server: {
    host: '127.0.0.1',
    port: 15002,
    mcp_port: 15002,
    data_dir: './data'
  },
  logging: {
    level: 'info',
    format: 'json',
    log_requests: true,
    log_responses: false,
    log_errors: true
  },
  collections: {
    defaults: {
      dimension: 512,
      metric: 'cosine',
      embedding: {
        model: 'bm25',
        bow: {
          vocab_size: 50000,
          max_sequence_length: 512
        },
        hash: {
          hash_size: 1000000
        },
        ngram: {
          ngram_range: [1, 3],
          vocab_size: 100000
        },
        bm25: {
          k1: 1.5,
          b: 0.75
        }
      },
      sharding: {
        enabled: true,
        target_max_size: 10000,
        soft_limit_size: 8000,
        split_strategy: 'kmeans2',
        routing_strategy: 'min_size',
        search: {
          parallel_enabled: true,
          dedup_enabled: true,
          rerank_enabled: false,
          rerank_top_k: 100
        },
        background: {
          auto_merge_enabled: false,
          merge_threshold: 0.3,
          cleanup_interval_secs: 300
        }
      },
      quantization: {
        type: 'sq',
        sq: {
          bits: 8
        }
      },
      index: {
        type: 'hnsw',
        hnsw: {
          m: 16,
          ef_construction: 200,
          ef_search: 64
        },
        optimized_hnsw: {
          batch_size: 1000,
          parallel: true,
          initial_capacity: 100000,
          max_connections: 16,
          max_connections_0: 32
        }
      }
    }
  },
  transmutation: {
    enabled: true,
    max_file_size_mb: 50,
    conversion_timeout_secs: 300,
    preserve_images: false
  },
  normalization: {
    enabled: true,
    level: 'conservative',
    line_endings: {
      normalize_crlf: true,
      normalize_cr: true,
      collapse_multiple_newlines: true,
      trim_trailing_whitespace: true
    },
    content_detection: {
      enabled: true,
      preserve_code_structure: true,
      preserve_markdown_format: true
    },
    cache: {
      enabled: true,
      max_entries: 10000,
      ttl_seconds: 3600
    },
    stages: {
      on_file_read: true,
      on_chunk_creation: true,
      on_payload_return: true,
      on_cache_load: true
    }
  },
  performance: {
    cpu: {
      max_threads: 8,
      enable_simd: true,
      memory_pool_size_mb: 1024
    },
    batch: {
      default_size: 100,
      max_size: 1000,
      parallel_processing: true
    }
  },
  workspace: {
    enabled: true,
    default_workspace_file: './vectorize-workspace.yml',
    auto_load_collections: true
  },
  storage: {
    compression: {
      enabled: true,
      format: 'zstd',
      level: 3
    },
    snapshots: {
      enabled: true,
      interval_hours: 1,
      retention_days: 2,
      max_snapshots: 48,
      path: './data/snapshots'
    },
    compaction: {
      batch_size: 1000,
      auto_compact: true
    }
  },
  api: {
    rest: {
      enabled: true,
      cors_enabled: true,
      max_request_size_mb: 10,
      timeout_seconds: 30
    },
    mcp: {
      enabled: true,
      port: 15002,
      max_connections: 100
    }
  }
});

function markDirty(): void {
  isDirty.value = true;
}

function showSuccessToast(message: string): void {
  saveMessage.value = message;
  showSaveSuccess.value = true;
  setTimeout(() => {
    showSaveSuccess.value = false;
  }, 3000);
}

async function loadConfig(): Promise<void> {
  try {
    const client = vectorizerStore.client;
    if (!client) {
      console.error('Vectorizer client not initialized');
      return;
    }
    
    const response = await fetch(`${client.config.baseURL}/api/config`, {
      headers: client.config.apiKey ? {
        'Authorization': `Bearer ${client.config.apiKey}`
      } : {}
    });
    
    if (response.ok) {
      const configData = await response.json();
      Object.assign(config, configData);
      
      // Convert to YAML for the YAML editor
      try {
        const yaml = await import('js-yaml');
        yamlContent.value = yaml.dump(configData, { indent: 2, lineWidth: 120 });
      } catch (e) {
        // Fallback to JSON if js-yaml is not available
        yamlContent.value = JSON.stringify(configData, null, 2);
      }
      
      isDirty.value = false;
    }
  } catch (error) {
    console.error('Failed to load config:', error);
  }
}

async function saveAndRestart(): Promise<void> {
  try {
    const client = vectorizerStore.client;
    if (!client) {
      console.error('Vectorizer client not initialized');
      showSuccessToast('Vectorizer client not initialized');
      return;
    }
    
    let payload;

    if (activeTab.value === 'yaml') {
      // Convert YAML to JSON
      try {
        const yaml = await import('js-yaml');
        payload = yaml.load(yamlContent.value);
      } catch (e) {
        console.error('Failed to parse YAML:', e);
        showSuccessToast('Failed to parse YAML');
        return;
      }
    } else {
      payload = config;
    }

    const response = await fetch(`${client.config.baseURL}/api/config`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(client.config.apiKey ? { 'Authorization': `Bearer ${client.config.apiKey}` } : {})
      },
      body: JSON.stringify(payload)
    });

    if (response.ok) {
      isDirty.value = false;
      showSuccessToast('Configuration saved successfully!');
      // Reload to sync all tabs
      await loadConfig();
    } else {
      showSuccessToast('Failed to save configuration');
    }
  } catch (error) {
    console.error('Failed to save config:', error);
    showSuccessToast('Failed to save configuration');
  }
}

onMounted(() => {
  loadConfig();
  
  // Listen for events from App.vue top bar
  window.addEventListener('reload-config', loadConfig);
  window.addEventListener('save-config', saveAndRestart);
});

onUnmounted(() => {
  window.removeEventListener('reload-config', loadConfig);
  window.removeEventListener('save-config', saveAndRestart);
});
</script>
