// Vectorizer Dashboard - Qdrant-style Implementation
class VectorizerDashboard {
    constructor() {
        this.apiBaseUrl = '/api/v1';
        this.currentPage = 'overview';
        this.refreshInterval = null;
        this.autoRefresh = false;
        this.collections = [];
        this.clusterInfo = null;
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.loadPage('overview');
        this.startAutoRefresh();
    }

    setupEventListeners() {
        // Navigation
        document.querySelectorAll('.nav-item').forEach(item => {
            item.addEventListener('click', (e) => {
                e.preventDefault();
                const page = e.currentTarget.dataset.page;
                this.loadPage(page);
            });
        });

        // Refresh button
        document.getElementById('refresh-btn').addEventListener('click', () => {
            this.refreshCurrentPage();
        });

        // Modal close
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('modal-overlay') || e.target.classList.contains('modal-close')) {
                this.closeModal();
            }
        });

        // Escape key to close modal
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                this.closeModal();
            }
        });
    }

    async loadPage(page) {
        // Stop any existing refresh intervals
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
        
        // Update navigation
        document.querySelectorAll('.nav-item').forEach(item => {
            item.classList.remove('active');
        });
        document.querySelector(`[data-page="${page}"]`).classList.add('active');

        // Update page title
        const titles = {
            overview: 'Overview',
            collections: 'Collections',
            search: 'Search',
            vectors: 'Vectors',
            cluster: 'Cluster Info',
            console: 'Console'
        };
        document.getElementById('current-page-title').textContent = titles[page] || page;

        // Update content
        const content = document.getElementById('main-content');
        this.currentPage = page;

        this.showLoading();

        try {
            switch (page) {
                case 'overview':
                    await this.loadOverview(content);
                    this.startOverviewAutoRefresh();
                    break;
                case 'collections':
                    await this.loadCollections(content);
                    break;
                case 'search':
                    await this.loadSearch(content);
                    break;
                case 'vectors':
                    await this.loadVectors(content);
                    break;
                case 'cluster':
                    await this.loadClusterInfo(content);
                    break;
                case 'console':
                    await this.loadConsole(content);
                    break;
                default:
                    content.innerHTML = '<div class="error">Página não encontrada</div>';
            }
        } catch (error) {
            console.error('Erro ao carregar página:', error);
            content.innerHTML = `<div class="error">Erro ao carregar conteúdo: ${error.message}</div>`;
        } finally {
            this.hideLoading();
        }
    }

    async loadOverview(content) {
        const collections = await this.fetchAPI('/collections');
        this.collections = collections.collections;

        const totalVectors = this.collections.reduce((sum, col) => sum + col.vector_count, 0);
        const avgDimension = this.collections.length > 0 
            ? Math.round(this.collections.reduce((sum, col) => sum + col.dimension, 0) / this.collections.length)
            : 0;

        content.innerHTML = `
            <div class="overview-grid">
                <!-- Stats Cards -->
                <div class="stats-section">
                    <div class="stats-grid">
                        <div class="stat-card">
                            <div class="stat-icon">
                                <i class="fas fa-layer-group"></i>
                            </div>
                            <div class="stat-content">
                                <div class="stat-value">${this.collections.length}</div>
                                <div class="stat-label">Collections</div>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon">
                                <i class="fas fa-vector-square"></i>
                            </div>
                            <div class="stat-content">
                                <div class="stat-value">${this.formatNumber(totalVectors)}</div>
                                <div class="stat-label">Total Vectors</div>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon">
                                <i class="fas fa-cube"></i>
                            </div>
                            <div class="stat-content">
                                <div class="stat-value">${avgDimension}</div>
                                <div class="stat-label">Avg Dimension</div>
                            </div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-icon">
                                <i class="fas fa-server"></i>
                            </div>
                            <div class="stat-content">
                                <div class="stat-value status-online">Online</div>
                                <div class="stat-label">Server Status</div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Indexing Progress -->
                <div class="section" id="indexing-progress-section">
                    ${await this.renderIndexingProgressSection()}
                </div>

                <!-- Collections Overview -->
                <div class="section">
                    <div class="section-header">
                        <h2><i class="fas fa-layer-group"></i> Collections Overview</h2>
                        <button class="btn btn-primary" onclick="dashboard.loadPage('collections')">
                            <i class="fas fa-eye"></i> View All
                        </button>
                    </div>
                    <div class="collections-table-container">
                        ${this.renderCollectionsTable(this.collections.slice(0, 5))}
                    </div>
                </div>

                <!-- Quick Actions -->
                <div class="section">
                    <div class="section-header">
                        <h2><i class="fas fa-bolt"></i> Quick Actions</h2>
                    </div>
                    <div class="quick-actions">
                        <button class="action-card" onclick="dashboard.loadPage('search')">
                            <i class="fas fa-search"></i>
                            <span>Search Vectors</span>
                        </button>
                        <button class="action-card" onclick="dashboard.loadPage('vectors')">
                            <i class="fas fa-vector-square"></i>
                            <span>Browse Vectors</span>
                        </button>
                        <button class="action-card" onclick="dashboard.loadPage('console')">
                            <i class="fas fa-terminal"></i>
                            <span>API Console</span>
                        </button>
                    </div>
                </div>

                <!-- System Info -->
                <div class="section">
                    <div class="section-header">
                        <h2><i class="fas fa-info-circle"></i> System Information</h2>
                    </div>
                    <div class="system-info">
                        <div class="info-item">
                            <span class="info-label">Version:</span>
                            <span class="info-value">1.0.0</span>
                        </div>
                        <div class="info-item">
                            <span class="info-label">Uptime:</span>
                            <span class="info-value">${this.getUptime()}</span>
                        </div>
                        <div class="info-item">
                            <span class="info-label">Memory Usage:</span>
                            <span class="info-value">${this.getMemoryUsage()}</span>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    async loadCollections(content) {
        const collections = await this.fetchAPI('/collections');
        this.collections = collections.collections;

        content.innerHTML = `
            <div class="collections-page">
                <div class="page-header">
                    <div class="header-content">
                        <h1><i class="fas fa-layer-group"></i> Collections</h1>
                        <p>Manage your vector collections</p>
                    </div>
                    <div class="header-actions">
                        <button class="btn btn-primary" onclick="dashboard.showCreateCollectionModal()">
                            <i class="fas fa-plus"></i> Create Collection
                        </button>
                    </div>
                </div>

                <div class="collections-grid">
                    ${this.collections.map(col => `
                        <div class="collection-card" data-collection="${col.name}">
                            <div class="collection-header">
                                <div class="collection-name">
                                    <i class="fas fa-database"></i>
                                    <span>${col.name}</span>
                                </div>
                                <div class="collection-menu">
                                    <button class="btn btn-icon" onclick="dashboard.showCollectionMenu('${col.name}', event)">
                                        <i class="fas fa-ellipsis-v"></i>
                                    </button>
                                </div>
                            </div>
                            <div class="collection-stats">
                                <div class="stat-row">
                                    <span class="stat-label">Source:</span>
                                    <span class="stat-value status-${col.indexing_status.status === 'cached' ? 'cached' : 'completed'}">${col.indexing_status.status === 'cached' ? 'Cache' : 'Indexed'}</span>
                                </div>
                                <div class="stat-row">
                                    <span class="stat-label">Vectors:</span>
                                    <span class="stat-value">${this.formatNumber(col.vector_count)}</span>
                                </div>
                                <div class="stat-row">
                                    <span class="stat-label">Dimension:</span>
                                    <span class="stat-value">${col.dimension}</span>
                                </div>
                                <div class="stat-row">
                                    <span class="stat-label">Metric:</span>
                                    <span class="stat-value">${col.metric}</span>
                                </div>
                                <div class="stat-row">
                                    <span class="stat-label">Status:</span>
                                    <span class="stat-value status-${col.indexing_status.status}">${this.formatStatus(col.indexing_status.status)}</span>
                                </div>
                                ${this.renderIndexingProgress(col.indexing_status)}
                            </div>
                            <div class="collection-actions">
                                <button class="btn btn-secondary btn-sm" onclick="dashboard.viewCollectionDetails('${col.name}')">
                                    <i class="fas fa-eye"></i> View
                                </button>
                                <button class="btn btn-primary btn-sm" onclick="dashboard.browseCollectionVectors('${col.name}')">
                                    <i class="fas fa-vector-square"></i> Browse
                                </button>
                            </div>
                        </div>
                    `).join('')}
                </div>

                ${this.collections.length === 0 ? `
                    <div class="empty-state">
                        <i class="fas fa-database"></i>
                        <h3>No Collections Found</h3>
                        <p>Create your first collection to start storing vectors</p>
                        <button class="btn btn-primary" onclick="dashboard.showCreateCollectionModal()">
                            <i class="fas fa-plus"></i> Create Collection
                        </button>
                    </div>
                ` : ''}
            </div>
        `;
    }

    async loadSearch(content) {
        const collections = await this.fetchAPI('/collections');

        content.innerHTML = `
            <div class="search-page">
                <div class="page-header">
                    <h1><i class="fas fa-search"></i> Vector Search</h1>
                    <p>Search for similar vectors across your collections</p>
                </div>

                <div class="search-form-container">
                    <div class="search-form">
                        <div class="form-row">
                            <div class="form-group">
                                <label for="search-collection">Collection</label>
                                <select id="search-collection" class="form-control">
                                    <option value="">Select a collection...</option>
                                    ${collections.collections.map(col => 
                                        `<option value="${col.name}">${col.name} (${col.vector_count} vectors)</option>`
                                    ).join('')}
                                </select>
                            </div>
                            <div class="form-group">
                                <label for="search-limit">Limit</label>
                                <input type="number" id="search-limit" class="form-control" value="10" min="1" max="100">
                            </div>
                        </div>

                        <div class="search-tabs">
                            <button class="tab-btn active" data-tab="text">
                                <i class="fas fa-font"></i> Text Search
                            </button>
                            <button class="tab-btn" data-tab="vector">
                                <i class="fas fa-vector-square"></i> Vector Search
                            </button>
                        </div>

                        <div class="tab-content">
                            <div class="tab-pane active" id="text-search">
                                <div class="form-group">
                                    <label for="search-query">Search Query</label>
                                    <textarea id="search-query" class="form-control" rows="3" placeholder="Enter your search text..."></textarea>
                                </div>
                                <button class="btn btn-primary btn-lg" onclick="dashboard.performTextSearch()">
                                    <i class="fas fa-search"></i> Search
                                </button>
                            </div>
                            <div class="tab-pane" id="vector-search">
                                <div class="form-group">
                                    <label for="search-vector">Vector (JSON Array)</label>
                                    <textarea id="search-vector" class="form-control" rows="5" placeholder="[0.1, 0.2, 0.3, ...]"></textarea>
                                </div>
                                <button class="btn btn-primary btn-lg" onclick="dashboard.performVectorSearch()">
                                    <i class="fas fa-search"></i> Search
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <div id="search-results" class="search-results-container"></div>
            </div>
        `;

        // Setup tab switching
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const tab = e.target.dataset.tab;
                document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
                document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
                e.target.classList.add('active');
                document.getElementById(`${tab}-search`).classList.add('active');
            });
        });
    }

    async loadVectors(content) {
        const collections = await this.fetchAPI('/collections');

        content.innerHTML = `
            <div class="vectors-page">
                <div class="page-header">
                    <h1><i class="fas fa-vector-square"></i> Vector Browser</h1>
                    <p>Browse and manage vectors in your collections</p>
                </div>

                <div class="vectors-controls">
                    <div class="form-group">
                        <label for="vector-collection">Collection</label>
                        <select id="vector-collection" class="form-control" onchange="dashboard.loadVectorsList(1, 20)">
                            <option value="">Select a collection...</option>
                            ${collections.collections.map(col => 
                                `<option value="${col.name}">${col.name}</option>`
                            ).join('')}
                        </select>
                    </div>
                    <div class="vectors-actions">
                        <button class="btn btn-secondary" onclick="dashboard.loadVectorsList(1, 20)" disabled id="refresh-vectors">
                            <i class="fas fa-sync-alt"></i> Refresh
                        </button>
                    </div>
                </div>

                <div id="vectors-list" class="vectors-list-container">
                    <div class="empty-state">
                        <i class="fas fa-vector-square"></i>
                        <h3>Select a Collection</h3>
                        <p>Choose a collection to browse its vectors</p>
                    </div>
                </div>
            </div>
        `;
    }

    async loadClusterInfo(content) {
        // Mock cluster info since we don't have a real cluster
        const clusterInfo = {
            status: 'green',
            nodes: 1,
            version: '1.0.0',
            collections_count: this.collections.length,
            vectors_count: this.collections.reduce((sum, col) => sum + col.vector_count, 0)
        };

        content.innerHTML = `
            <div class="cluster-page">
                <div class="page-header">
                    <h1><i class="fas fa-project-diagram"></i> Cluster Information</h1>
                    <p>Overview of your Vectorizer cluster</p>
                </div>

                <div class="cluster-grid">
                    <div class="cluster-status-card">
                        <div class="status-header">
                            <h3>Cluster Status</h3>
                            <span class="status-badge status-${clusterInfo.status}">${clusterInfo.status.toUpperCase()}</span>
                        </div>
                        <div class="status-details">
                            <div class="detail-item">
                                <span class="label">Nodes:</span>
                                <span class="value">${clusterInfo.nodes}</span>
                            </div>
                            <div class="detail-item">
                                <span class="label">Version:</span>
                                <span class="value">${clusterInfo.version}</span>
                            </div>
                            <div class="detail-item">
                                <span class="label">Collections:</span>
                                <span class="value">${clusterInfo.collections_count}</span>
                            </div>
                            <div class="detail-item">
                                <span class="label">Total Vectors:</span>
                                <span class="value">${this.formatNumber(clusterInfo.vectors_count)}</span>
                            </div>
                        </div>
                    </div>

                    <div class="node-info-card">
                        <h3>Node Information</h3>
                        <div class="node-details">
                            <div class="detail-item">
                                <span class="label">Node ID:</span>
                                <span class="value">vectorizer-node-1</span>
                            </div>
                            <div class="detail-item">
                                <span class="label">Host:</span>
                                <span class="value">127.0.0.1:15001</span>
                            </div>
                            <div class="detail-item">
                                <span class="label">Role:</span>
                                <span class="value">Master</span>
                            </div>
                            <div class="detail-item">
                                <span class="label">Uptime:</span>
                                <span class="value">${this.getUptime()}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        `;
    }

    async loadConsole(content) {
        content.innerHTML = `
            <div class="console-page">
                <div class="page-header">
                    <h1><i class="fas fa-terminal"></i> API Console</h1>
                    <p>Test API endpoints directly from the dashboard</p>
                </div>

                <div class="console-container">
                    <div class="console-form">
                        <div class="form-row">
                            <div class="form-group">
                                <label for="console-method">Method</label>
                                <select id="console-method" class="form-control">
                                    <option value="GET">GET</option>
                                    <option value="POST">POST</option>
                                    <option value="DELETE">DELETE</option>
                                </select>
                            </div>
                            <div class="form-group flex-1">
                                <label for="console-endpoint">Endpoint</label>
                                <input type="text" id="console-endpoint" class="form-control" placeholder="/collections" value="/collections">
                            </div>
                        </div>
                        
                        <div class="form-group">
                            <label for="console-body">Request Body (JSON)</label>
                            <textarea id="console-body" class="form-control code-editor" rows="8" placeholder='{\n  "key": "value"\n}'></textarea>
                        </div>

                        <div class="console-actions">
                            <button class="btn btn-primary" onclick="dashboard.executeConsoleRequest()">
                                <i class="fas fa-play"></i> Execute
                            </button>
                            <button class="btn btn-secondary" onclick="dashboard.clearConsole()">
                                <i class="fas fa-trash"></i> Clear
                            </button>
                        </div>
                    </div>

                    <div class="console-output">
                        <div class="output-header">
                            <h3>Response</h3>
                            <span id="response-status" class="response-status"></span>
                        </div>
                        <pre id="console-response" class="response-content">Execute a request to see the response...</pre>
                    </div>
                </div>
            </div>
        `;
    }

    // Search Functions
    async performTextSearch() {
        const collection = document.getElementById('search-collection').value;
        const query = document.getElementById('search-query').value.trim();
        const limit = parseInt(document.getElementById('search-limit').value);

        if (!collection) {
            this.showToast('Please select a collection', 'warning');
            return;
        }

        if (!query) {
            this.showToast('Please enter a search query', 'warning');
            return;
        }

        try {
            this.showToast('Searching...', 'info');
            
            const results = await this.fetchAPI(`/collections/${collection}/search/text`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query, limit })
            });

            this.displaySearchResults(results, 'text', query);
        } catch (error) {
            this.showToast(`Search failed: ${error.message}`, 'error');
        }
    }

    async performVectorSearch() {
        const collection = document.getElementById('search-collection').value;
        const vectorText = document.getElementById('search-vector').value.trim();
        const limit = parseInt(document.getElementById('search-limit').value);

        if (!collection) {
            this.showToast('Please select a collection', 'warning');
            return;
        }

        if (!vectorText) {
            this.showToast('Please enter a vector', 'warning');
            return;
        }

        try {
            const vector = JSON.parse(vectorText);
            if (!Array.isArray(vector)) {
                throw new Error('Vector must be an array');
            }

            this.showToast('Searching...', 'info');
            
            const results = await this.fetchAPI(`/collections/${collection}/search`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ vector, limit })
            });

            this.displaySearchResults(results, 'vector', vector);
        } catch (error) {
            this.showToast(`Search failed: ${error.message}`, 'error');
        }
    }

    displaySearchResults(results, type, query) {
        const container = document.getElementById('search-results');
        
        if (!results.results || results.results.length === 0) {
            container.innerHTML = `
                <div class="no-results">
                    <i class="fas fa-search"></i>
                    <h3>No Results Found</h3>
                    <p>No vectors match your search criteria</p>
                </div>
            `;
            return;
        }

        container.innerHTML = `
            <div class="results-header">
                <h3>Search Results</h3>
                <span class="results-count">${results.results.length} results found</span>
            </div>
            <div class="results-list">
                ${results.results.map((result, index) => `
                    <div class="result-item">
                        <div class="result-header">
                            <span class="result-rank">#${index + 1}</span>
                            <span class="result-score">Score: ${result.score ? result.score.toFixed(4) : '0.0000'}</span>
                            <span class="result-id">ID: ${result.id}</span>
                        </div>
                        <div class="result-content">
                            ${result.payload ? `
                                <div class="payload-content">
                                    <h4>Payload:</h4>
                                    <pre>${this.formatJSON(result.payload)}</pre>
                                </div>
                            ` : ''}
                        </div>
                    </div>
                `).join('')}
            </div>
        `;

        this.showToast(`Found ${results.results.length} results`, 'success');
    }

    // Vector Management
    async loadVectorsList(page = 1, limit = 20) {
        const collection = document.getElementById('vector-collection').value;
        const container = document.getElementById('vectors-list');
        const refreshBtn = document.getElementById('refresh-vectors');

        if (!collection) {
            container.innerHTML = `
                <div class="empty-state">
                    <i class="fas fa-vector-square"></i>
                    <h3>Select a Collection</h3>
                    <p>Choose a collection to browse its vectors</p>
                </div>
            `;
            refreshBtn.disabled = true;
            return;
        }

        refreshBtn.disabled = false;

        try {
            this.showLoading();
            // Ensure page and limit are valid numbers to avoid NaN
            const validPage = Math.max(1, parseInt(page) || 1);
            const validLimit = Math.max(1, Math.min(100, parseInt(limit) || 20));
            
            // Calculate offset properly to avoid NaN
            const offset = (validPage - 1) * validLimit;
            
            // Debug logging
            console.log('Vector Browser Debug:', {
                originalPage: page,
                originalLimit: limit,
                validPage,
                validLimit,
                offset,
                collection
            });
            
            const data = await this.fetchAPI(`/collections/${collection}/vectors?limit=${validLimit}&offset=${offset}`);
            
            container.innerHTML = `
                <div class="vectors-header">
                    <h3>Vectors in ${collection}</h3>
                    <span class="vectors-count">${data.total || 0} total vectors</span>
                </div>
                
                <!-- Pagination Controls -->
                <div class="pagination-controls">
                    <div class="pagination-info">
                        Showing ${offset + 1}-${Math.min(offset + validLimit, data.total || 0)} of ${data.total || 0} vectors
                    </div>
                    <div class="pagination-buttons">
                        <button class="btn btn-secondary btn-sm" 
                                onclick="dashboard.loadVectorsList(${validPage - 1}, ${validLimit})" 
                                ${validPage <= 1 ? 'disabled' : ''}>
                            <i class="fas fa-chevron-left"></i> Previous
                        </button>
                        <span class="pagination-page">Page ${validPage}</span>
                        <button class="btn btn-secondary btn-sm" 
                                onclick="dashboard.loadVectorsList(${validPage + 1}, ${validLimit})" 
                                ${(offset + validLimit) >= (data.total || 0) ? 'disabled' : ''}>
                            Next <i class="fas fa-chevron-right"></i>
                        </button>
                    </div>
                </div>

                <div class="vectors-grid">
                    ${(data.vectors || []).map((vector, index) => `
                        <div class="vector-card">
                            <div class="vector-header">
                                <span class="vector-id">${vector.id}</span>
                                <button class="btn btn-icon" onclick="dashboard.viewVectorDetails('${collection}', '${vector.id}')">
                                    <i class="fas fa-eye"></i>
                                </button>
                            </div>
                            
                            <div class="vector-content">
                                <div class="vector-preview">
                                    ${vector.payload ? `
                                        <div class="payload-preview">
                                            <strong>Content:</strong>
                                            <div class="content-text">${this.truncateText(this.formatJSON(vector.payload), 120)}</div>
                                        </div>
                                    ` : '<span class="no-payload">No payload</span>'}
                                </div>
                                
                                <div class="vector-metadata">
                                    <div class="metadata-row">
                                        <span class="metadata-label">Source:</span>
                                        <span class="metadata-value">${vector.metadata?.source || 'Unknown'}</span>
                                    </div>
                                    <div class="metadata-row">
                                        <span class="metadata-label">File Type:</span>
                                        <span class="metadata-value">${vector.metadata?.file_type || 'Unknown'}</span>
                                    </div>
                                    <div class="metadata-row">
                                        <span class="metadata-label">Chunk:</span>
                                        <span class="metadata-value">${vector.metadata?.chunk_index || 0}</span>
                                    </div>
                                    <div class="metadata-row">
                                        <span class="metadata-label">Model:</span>
                                        <span class="metadata-value">${vector.metadata?.embedding_model || 'Unknown'}</span>
                                    </div>
                                    <div class="metadata-row">
                                        <span class="metadata-label">Dimension:</span>
                                        <span class="metadata-value">${vector.metadata?.dimension || 'Unknown'}</span>
                                    </div>
                                    <div class="metadata-row">
                                        <span class="metadata-label">Score:</span>
                                        <span class="metadata-value">${vector.metadata?.similarity_score ? vector.metadata.similarity_score.toFixed(3) : 'N/A'}</span>
                                    </div>
                                    <div class="metadata-row">
                                        <span class="metadata-label">Created:</span>
                                        <span class="metadata-value">${vector.metadata?.created_at ? this.formatDateTime(vector.metadata.created_at) : 'Unknown'}</span>
                                    </div>
                                </div>
                            </div>
                            
                            <div class="vector-actions">
                                <button class="btn btn-primary btn-sm" onclick="dashboard.viewVectorDetails('${collection}', '${vector.id}')">
                                    <i class="fas fa-eye"></i> View Details
                                </button>
                                <button class="btn btn-secondary btn-sm" onclick="dashboard.copyVectorId('${vector.id}')">
                                    <i class="fas fa-copy"></i> Copy ID
                                </button>
                            </div>
                        </div>
                    `).join('')}
                </div>
                
                ${(!data.vectors || data.vectors.length === 0) ? `
                    <div class="empty-state">
                        <i class="fas fa-vector-square"></i>
                        <h3>No Vectors Found</h3>
                        <p>This collection appears to be empty</p>
                    </div>
                ` : ''}
            `;
        } catch (error) {
            container.innerHTML = `<div class="error">Error loading vectors: ${error.message}</div>`;
        } finally {
            this.hideLoading();
        }
    }

    // Collection Management
    async viewCollectionDetails(collectionName) {
        try {
            const collection = await this.fetchAPI(`/collections/${collectionName}`);
            
            const modalContent = `
                <div class="modal-header">
                    <h2><i class="fas fa-database"></i> ${collectionName}</h2>
                    <button class="modal-close">&times;</button>
                </div>
                <div class="modal-body">
                    <div class="collection-details-grid">
                        <div class="detail-section">
                            <h3>Basic Information</h3>
                            <div class="detail-list">
                                <div class="detail-item">
                                    <span class="label">Name:</span>
                                    <span class="value">${collection.name}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Vector Count:</span>
                                    <span class="value">${this.formatNumber(collection.vector_count)}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Dimension:</span>
                                    <span class="value">${collection.dimension}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Distance Metric:</span>
                                    <span class="value">${collection.metric}</span>
                                </div>
                            </div>
                        </div>
                        <div class="detail-section">
                            <h3>Timestamps</h3>
                            <div class="detail-list">
                                <div class="detail-item">
                                    <span class="label">Created:</span>
                                    <span class="value">${this.formatDateTime(collection.created_at)}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Updated:</span>
                                    <span class="value">${this.formatDateTime(collection.updated_at)}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-primary" onclick="dashboard.browseCollectionVectors('${collectionName}'); dashboard.closeModal();">
                        <i class="fas fa-vector-square"></i> Browse Vectors
                    </button>
                    <button class="btn btn-secondary" onclick="dashboard.closeModal()">Close</button>
                </div>
            `;
            
            this.showModal(modalContent);
        } catch (error) {
            this.showToast(`Error loading collection details: ${error.message}`, 'error');
        }
    }

    async browseCollectionVectors(collectionName) {
        // Switch to vectors page and select the collection
        await this.loadPage('vectors');
        document.getElementById('vector-collection').value = collectionName;
        await this.loadVectorsList(1, 20);
    }

    // Vector Details Modal
    async viewVectorDetails(collectionName, vectorId) {
        try {
            this.showLoading();
            const vector = await this.fetchAPI(`/collections/${collectionName}/vectors/${vectorId}`);
            
            const modalContent = `
                <div class="modal-header">
                    <h2><i class="fas fa-vector-square"></i> Vector Details</h2>
                    <button class="modal-close">&times;</button>
                </div>
                <div class="modal-body">
                    <div class="vector-details-grid">
                        <div class="detail-section">
                            <h3>Basic Information</h3>
                            <div class="detail-list">
                                <div class="detail-item">
                                    <span class="label">Vector ID:</span>
                                    <span class="value">${vector.id}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Collection:</span>
                                    <span class="value">${collectionName}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Dimension:</span>
                                    <span class="value">${vector.metadata?.dimension || 'Unknown'}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Embedding Model:</span>
                                    <span class="value">${vector.metadata?.embedding_model || 'Unknown'}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Similarity Score:</span>
                                    <span class="value">${vector.metadata?.similarity_score ? vector.metadata.similarity_score.toFixed(4) : 'N/A'}</span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="detail-section">
                            <h3>Source Information</h3>
                            <div class="detail-list">
                                <div class="detail-item">
                                    <span class="label">Source File:</span>
                                    <span class="value">${vector.metadata?.source || 'Unknown'}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">File Type:</span>
                                    <span class="value">${vector.metadata?.file_type || 'Unknown'}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Chunk Index:</span>
                                    <span class="value">${vector.metadata?.chunk_index || 0}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="label">Created:</span>
                                    <span class="value">${vector.metadata?.created_at ? this.formatDateTime(vector.metadata.created_at) : 'Unknown'}</span>
                                </div>
                            </div>
                        </div>
                        
                        <div class="detail-section full-width">
                            <h3>Vector Content</h3>
                            <div class="vector-content-display">
                                ${vector.payload ? `
                                    <div class="content-section">
                                        <h4>Payload Data:</h4>
                                        <pre class="json-display">${this.formatJSON(vector.payload)}</pre>
                                    </div>
                                ` : '<p class="no-content">No payload data available</p>'}
                                
                                <div class="content-section">
                                    <h4>Vector Embedding (first 10 values):</h4>
                                    <pre class="vector-display">${vector.embedding ? 
                                        JSON.stringify(vector.embedding.slice(0, 10), null, 2) + '\n...' : 
                                        'Embedding data not available'
                                    }</pre>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" onclick="dashboard.copyVectorId('${vectorId}'); dashboard.closeModal();">
                        <i class="fas fa-copy"></i> Copy ID
                    </button>
                    <button class="btn btn-primary" onclick="dashboard.copyVectorData('${collectionName}', '${vectorId}'); dashboard.closeModal();">
                        <i class="fas fa-download"></i> Copy Data
                    </button>
                    <button class="btn btn-secondary" onclick="dashboard.closeModal()">Close</button>
                </div>
            `;
            
            this.showModal(modalContent);
        } catch (error) {
            this.showToast(`Error loading vector details: ${error.message}`, 'error');
        } finally {
            this.hideLoading();
        }
    }

    // Utility function to copy vector ID
    copyVectorId(vectorId) {
        navigator.clipboard.writeText(vectorId).then(() => {
            this.showToast('Vector ID copied to clipboard', 'success');
        }).catch(() => {
            this.showToast('Failed to copy vector ID', 'error');
        });
    }

    // Utility function to copy vector data
    async copyVectorData(collectionName, vectorId) {
        try {
            const vector = await this.fetchAPI(`/collections/${collectionName}/vectors/${vectorId}`);
            const dataToCopy = {
                id: vector.id,
                metadata: vector.metadata,
                payload: vector.payload,
                embedding: vector.embedding ? vector.embedding.slice(0, 10) : null // First 10 values only
            };
            
            navigator.clipboard.writeText(JSON.stringify(dataToCopy, null, 2)).then(() => {
                this.showToast('Vector data copied to clipboard', 'success');
            }).catch(() => {
                this.showToast('Failed to copy vector data', 'error');
            });
        } catch (error) {
            this.showToast(`Error copying vector data: ${error.message}`, 'error');
        }
    }

    // Console Functions
    async executeConsoleRequest() {
        const method = document.getElementById('console-method').value;
        const endpoint = document.getElementById('console-endpoint').value;
        const bodyText = document.getElementById('console-body').value.trim();
        
        const statusEl = document.getElementById('response-status');
        const responseEl = document.getElementById('console-response');

        try {
            const options = { method };
            
            if (bodyText && (method === 'POST' || method === 'PUT')) {
                options.headers = { 'Content-Type': 'application/json' };
                options.body = bodyText;
            }

            const response = await fetch(`${this.apiBaseUrl}${endpoint}`, options);
            const responseData = await response.json();
            
            statusEl.textContent = `${response.status} ${response.statusText}`;
            statusEl.className = `response-status ${response.ok ? 'success' : 'error'}`;
            
            responseEl.textContent = JSON.stringify(responseData, null, 2);
        } catch (error) {
            statusEl.textContent = 'Error';
            statusEl.className = 'response-status error';
            responseEl.textContent = error.message;
        }
    }

    clearConsole() {
        document.getElementById('console-endpoint').value = '/collections';
        document.getElementById('console-body').value = '';
        document.getElementById('response-status').textContent = '';
        document.getElementById('console-response').textContent = 'Execute a request to see the response...';
    }

    // Utility Functions
    renderCollectionsTable(collections) {
        if (collections.length === 0) {
            return '<div class="empty-table">No collections found</div>';
        }

        return `
            <table class="data-table">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Source</th>
                        <th>Vectors</th>
                        <th>Dimension</th>
                        <th>Metric</th>
                        <th>Created</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    ${collections.map(col => `
                        <tr>
                            <td><strong>${col.name}</strong></td>
                            <td><span class="status-badge status-${col.indexing_status.status === 'cached' ? 'cached' : 'completed'}">${col.indexing_status.status === 'cached' ? 'Cache' : 'Indexed'}</span></td>
                            <td>${this.formatNumber(col.vector_count)}</td>
                            <td>${col.dimension}</td>
                            <td>${col.metric}</td>
                            <td>${this.formatDate(col.created_at)}</td>
                            <td>
                                <button class="btn btn-sm btn-secondary" onclick="dashboard.viewCollectionDetails('${col.name}')">
                                    <i class="fas fa-eye"></i>
                                </button>
                                <button class="btn btn-sm btn-primary" onclick="dashboard.browseCollectionVectors('${col.name}')">
                                    <i class="fas fa-vector-square"></i>
                                </button>
                            </td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;
    }

    async fetchAPI(endpoint, options = {}) {
        const url = `${this.apiBaseUrl}${endpoint}`;
        const response = await fetch(url, options);
        
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        return await response.json();
    }

    showModal(content) {
        const overlay = document.getElementById('modal-overlay');
        const modal = document.getElementById('modal-content');
        
        modal.innerHTML = content;
        overlay.style.display = 'flex';
    }

    closeModal() {
        const overlay = document.getElementById('modal-overlay');
        overlay.style.display = 'none';
    }

    showLoading() {
        document.getElementById('loading-overlay').style.display = 'flex';
    }

    hideLoading() {
        document.getElementById('loading-overlay').style.display = 'none';
    }

    showToast(message, type = 'info') {
        const container = document.getElementById('toast-container');
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        toast.innerHTML = `
            <i class="fas fa-${this.getToastIcon(type)}"></i>
            <span>${message}</span>
        `;
        
        container.appendChild(toast);
        
        setTimeout(() => toast.classList.add('show'), 100);
        setTimeout(() => {
            toast.classList.remove('show');
            setTimeout(() => container.removeChild(toast), 300);
        }, 3000);
    }

    getToastIcon(type) {
        const icons = {
            success: 'check-circle',
            error: 'exclamation-circle',
            warning: 'exclamation-triangle',
            info: 'info-circle'
        };
        return icons[type] || 'info-circle';
    }

    formatNumber(num) {
        return new Intl.NumberFormat().format(num);
    }

    formatDate(dateString) {
        if (!dateString) return 'N/A';
        return new Date(dateString).toLocaleDateString('pt-BR');
    }

    formatDateTime(dateString) {
        if (!dateString) return 'N/A';
        return new Date(dateString).toLocaleString('pt-BR');
    }

    formatStatus(status) {
        switch (status) {
            case 'completed': return 'Concluído';
            case 'processing': return 'Processando';
            case 'indexing': return 'Indexando';
            case 'pending': return 'Pendente';
            case 'failed': return 'Falhou';
            case 'cached': return 'Do Cache';
            default: return status;
        }
    }

    async renderIndexingProgressSection() {
        try {
            const progressData = await this.fetchAPI('/indexing/progress');
            
            // Calculate overall stats
            const totalCollections = progressData.total_collections || 0;
            const completedCollections = progressData.completed_collections || 0;
            const processingCollections = progressData.processing_collections || 0;
            const overallProgress = progressData.overall_progress || 0;
            
            // Check if any indexing is happening
            const isIndexing = processingCollections > 0 || completedCollections < totalCollections;
            
            if (!isIndexing && completedCollections === totalCollections && activeCollections.every(c => c.status === 'cached' || c.status === 'completed')) {
                return ''; // Don't show section if all indexing is complete
            }
            
            // Get collections being processed
            const activeCollections = progressData.collections
                ? progressData.collections.filter(c => c.status === 'processing' || c.status === 'pending')
                : [];
            
            return `
                <div class="section-header">
                    <h2><i class="fas fa-sync fa-spin"></i> Indexação em Progresso</h2>
                    <span class="indexing-stats">
                        ${completedCollections}/${totalCollections} collections completas
                    </span>
                </div>
                <div class="indexing-overview">
                    <div class="overall-progress">
                        <div class="progress-info">
                            <span>Progresso Geral</span>
                            <span class="progress-percent">${Math.round(overallProgress)}%</span>
                        </div>
                        <div class="progress-bar large">
                            <div class="progress-fill" style="width: ${overallProgress}%"></div>
                        </div>
                    </div>
                    
                    ${activeCollections.length > 0 ? `
                        <div class="active-collections">
                            <h4>Collections Ativas:</h4>
                            <div class="collection-progress-list">
                                ${activeCollections.map(col => `
                                    <div class="collection-progress-item">
                                        <div class="collection-info">
                                            <span class="collection-name">${col.name}</span>
                                            <span class="collection-status status-${col.status}">
                                                ${this.formatStatus(col.status)}
                                            </span>
                                        </div>
                                        ${col.status === 'processing' ? `
                                            <div class="progress-bar small">
                                                <div class="progress-fill" style="width: ${col.progress}%"></div>
                                            </div>
                                            <span class="progress-text">${Math.round(col.progress)}%</span>
                                        ` : ''}
                                    </div>
                                `).join('')}
                            </div>
                        </div>
                    ` : ''}
                </div>
            `;
        } catch (error) {
            console.error('Error loading indexing progress:', error);
            return '';
        }
    }

    renderIndexingProgress(indexingStatus) {
        if (indexingStatus.status === 'completed' || indexingStatus.status === 'failed') {
            return '';
        }

        const progressPercent = Math.round(indexingStatus.progress);
        return `
            <div class="indexing-progress">
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${progressPercent}%"></div>
                </div>
                <div class="progress-text">${progressPercent}%</div>
            </div>
        `;
    }

    truncateText(text, maxLength) {
        if (text.length <= maxLength) return text;
        return text.substring(0, maxLength) + '...';
    }
    
    startOverviewAutoRefresh() {
        // Only refresh indexing progress section
        this.refreshInterval = setInterval(async () => {
            if (this.currentPage !== 'overview') return;
            
            try {
                const progressSection = document.getElementById('indexing-progress-section');
                if (progressSection) {
                    const progressHTML = await this.renderIndexingProgressSection();
                    progressSection.innerHTML = progressHTML;
                    
                    // If no indexing is happening, stop refreshing
                    if (!progressHTML) {
                        clearInterval(this.refreshInterval);
                        this.refreshInterval = null;
                    }
                }
            } catch (error) {
                console.error('Error refreshing indexing progress:', error);
            }
        }, 2000); // Refresh every 2 seconds
    }

    getUptime() {
        // Mock uptime
        return '2h 34m';
    }

    getMemoryUsage() {
        // Mock memory usage
        return '256 MB';
    }

    startAutoRefresh() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }
        
        this.refreshInterval = setInterval(() => {
            if (this.autoRefresh) {
                this.refreshCurrentPage();
            }
        }, this.currentPage === 'collections' ? 5000 : 30000); // Refresh every 5 seconds on collections page, 30 seconds otherwise
    }

    refreshCurrentPage() {
        if (this.currentPage === 'collections') {
            // Refresh only collections data without reloading the entire page
            this.refreshCollections();
        } else {
            this.loadPage(this.currentPage);
        }
    }

    async refreshCollections() {
        try {
            const response = await fetch('/api/collections');
            const data = await response.json();

            if (data.collections) {
                this.collections = data.collections;
                this.updateCollectionsDisplay();
            }
        } catch (error) {
            console.error('Failed to refresh collections:', error);
        }
    }

    updateCollectionsDisplay() {
        const collectionsGrid = document.querySelector('.collections-grid');
        if (collectionsGrid) {
            // Find the collections grid and update its content
            const newContent = this.collections.map(col => `
                <div class="collection-card" data-collection="${col.name}">
                    <div class="collection-header">
                        <div class="collection-name">
                            <i class="fas fa-database"></i>
                            <span>${col.name}</span>
                        </div>
                        <div class="collection-menu">
                            <button class="btn btn-icon" onclick="dashboard.showCollectionMenu('${col.name}', event)">
                                <i class="fas fa-ellipsis-v"></i>
                            </button>
                        </div>
                    </div>
                    <div class="collection-stats">
                        <div class="stat-row">
                            <span class="stat-label">Source:</span>
                            <span class="stat-value status-${col.indexing_status.status === 'cached' ? 'cached' : 'completed'}">${col.indexing_status.status === 'cached' ? 'Cache' : 'Indexed'}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">Vectors:</span>
                            <span class="stat-value">${this.formatNumber(col.vector_count)}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">Dimension:</span>
                            <span class="stat-value">${col.dimension}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">Metric:</span>
                            <span class="stat-value">${col.metric}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">Status:</span>
                            <span class="stat-value status-${col.indexing_status.status}">${this.formatStatus(col.indexing_status.status)}</span>
                        </div>
                        ${this.renderIndexingProgress(col.indexing_status)}
                    </div>
                    <div class="collection-actions">
                        <button class="btn btn-secondary btn-sm" onclick="dashboard.viewCollectionDetails('${col.name}')">
                            <i class="fas fa-eye"></i> View
                        </button>
                        <button class="btn btn-primary btn-sm" onclick="dashboard.browseCollectionVectors('${col.name}')">
                            <i class="fas fa-vector-square"></i> Browse
                        </button>
                    </div>
                </div>
            `).join('');

            collectionsGrid.innerHTML = newContent;
        }
    }

    formatJSON(obj) {
        if (!obj) return 'null';
        
        const jsonStr = JSON.stringify(obj, null, 2);
        
        // Simple syntax highlighting
        return jsonStr
            .replace(/"([^"]+)":/g, '<span class="json-key">"$1":</span>')
            .replace(/: "([^"]*)"/g, ': <span class="json-string">"$1"</span>')
            .replace(/: (\d+\.?\d*)/g, ': <span class="json-number">$1</span>')
            .replace(/: (true|false)/g, ': <span class="json-boolean">$1</span>')
            .replace(/: null/g, ': <span class="json-null">null</span>');
    }
}

    // Test function to verify offset calculation
    testOffsetCalculation() {
        console.log('Testing offset calculations:');
        
        const testCases = [
            { page: 1, limit: 20, expected: 0 },
            { page: 2, limit: 20, expected: 20 },
            { page: 3, limit: 10, expected: 20 },
            { page: undefined, limit: undefined, expected: 0 },
            { page: null, limit: null, expected: 0 },
            { page: '2', limit: '20', expected: 20 }
        ];
        
        testCases.forEach((testCase, index) => {
            const validPage = Math.max(1, parseInt(testCase.page) || 1);
            const validLimit = Math.max(1, Math.min(100, parseInt(testCase.limit) || 20));
            const offset = (validPage - 1) * validLimit;
            
            console.log(`Test ${index + 1}:`, {
                input: testCase,
                validPage,
                validLimit,
                calculatedOffset: offset,
                expected: testCase.expected,
                passed: offset === testCase.expected
            });
        });
    }

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new VectorizerDashboard();
    
    // Run offset calculation test
    setTimeout(() => {
        window.dashboard.testOffsetCalculation();
    }, 1000);
});