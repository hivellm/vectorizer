# Collection Organization System Specification

**Status**: Specification  
**Priority**: Medium  
**Complexity**: Low  
**Created**: October 1, 2025

## Problem Statement

As the number of collections grows, organization becomes critical:
- Current: Flat list of collections
- Problem: Hard to find specific collections among hundreds
- Need: Hierarchical organization with tags and categories

## Proposed Solution

### 1. Collection Namespaces

```rust
pub struct CollectionNamespace {
    pub path: String,                    // "projects/gateway/code"
    pub name: String,                    // "code"
    pub full_name: String,               // "projects.gateway.code"
    pub parent: Option<String>,          // "projects.gateway"
    pub children: Vec<String>,
}

// Examples:
// projects.gateway.code
// projects.gateway.docs
// projects.gateway.tests
// projects.governance.bips
// projects.governance.proposals
```

### 2. Tags System

```rust
pub struct CollectionTags {
    pub collection_name: String,
    pub tags: HashSet<String>,
    pub auto_tags: HashSet<String>,      // Auto-generated
    pub custom_tags: HashSet<String>,    // User-defined
}

// Auto-tags based on analysis
fn generate_auto_tags(collection: &Collection) -> HashSet<String> {
    let mut tags = HashSet::new();
    
    // Language tags
    if collection.contains_extension(".rs") { tags.insert("rust".to_string()); }
    if collection.contains_extension(".ts") { tags.insert("typescript".to_string()); }
    if collection.contains_extension(".py") { tags.insert("python".to_string()); }
    
    // Content tags
    if collection.avg_chunk_size() > 1000 { tags.insert("long-form".to_string()); }
    if collection.has_code_patterns() { tags.insert("code".to_string()); }
    
    // Size tags
    match collection.vector_count() {
        0..=1000 => tags.insert("small".to_string()),
        1001..=10000 => tags.insert("medium".to_string()),
        _ => tags.insert("large".to_string()),
    };
    
    tags
}
```

### 3. Categories & Metadata

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct CollectionMetadata {
    pub namespace: String,
    pub category: CollectionCategory,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CollectionCategory {
    Code { language: String },
    Documentation { format: String },
    Configuration,
    Data { schema: Option<String> },
    Mixed,
}
```

### 4. Workspace Configuration

```yaml
# Hierarchical namespace in workspace config
workspace:
  name: "HiveLLM"
  
  # Namespace structure
  namespaces:
    projects:
      gateway:
        description: "MCP Integration Gateway"
        tags: [typescript, mcp, backend]
        
      governance:
        description: "Governance System"
        tags: [bip, proposals, governance]
      
      vectorizer:
        description: "Vector Database"
        tags: [rust, database, search]
  
  # Collections reference namespaces
  collections:
    - namespace: "projects.gateway.code"
      type: code
      include_patterns: ["src/**/*.ts"]
    
    - namespace: "projects.gateway.docs"
      type: documentation
      include_patterns: ["docs/**/*.md"]
```

## Query & Filter System

### Search Collections

```rust
pub struct CollectionQuery {
    pub namespace: Option<String>,       // "projects.gateway.*"
    pub tags: Vec<String>,               // ["typescript", "code"]
    pub category: Option<CollectionCategory>,
    pub min_vectors: Option<usize>,
    pub max_vectors: Option<usize>,
    pub text_search: Option<String>,     // Search in names/descriptions
}

impl VectorStore {
    pub async fn find_collections(&self, query: CollectionQuery) -> Vec<Collection> {
        self.collections
            .iter()
            .filter(|c| {
                // Namespace filter
                if let Some(ns) = &query.namespace {
                    if !c.namespace.starts_with(ns) {
                        return false;
                    }
                }
                
                // Tag filter (AND logic)
                if !query.tags.is_empty() {
                    if !query.tags.iter().all(|tag| c.tags.contains(tag)) {
                        return false;
                    }
                }
                
                // Category filter
                if let Some(cat) = &query.category {
                    if &c.category != cat {
                        return false;
                    }
                }
                
                // Size filters
                if let Some(min) = query.min_vectors {
                    if c.vector_count() < min {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }
}
```

## Dashboard UI

```html
<div class="collection-browser">
    <!-- Namespace tree view -->
    <div class="namespace-tree">
        <h3>Collections</h3>
        
        <div class="tree-node">
            <span class="node-icon">📁</span>
            <span class="node-name">projects</span>
            <span class="node-count">(47)</span>
            
            <div class="tree-children">
                <div class="tree-node">
                    <span class="node-icon">🚪</span>
                    <span class="node-name">gateway</span>
                    <span class="node-count">(4)</span>
                    
                    <div class="tree-children">
                        <div class="tree-leaf">
                            <span class="leaf-icon">📄</span>
                            <span class="leaf-name">code</span>
                            <span class="leaf-stats">1.2K vectors</span>
                        </div>
                        <div class="tree-leaf">
                            <span class="leaf-icon">📄</span>
                            <span class="leaf-name">docs</span>
                            <span class="leaf-stats">456 vectors</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    
    <!-- Tag cloud -->
    <div class="tag-cloud">
        <h4>Filter by Tags</h4>
        <button class="tag" data-tag="typescript">typescript (8)</button>
        <button class="tag" data-tag="rust">rust (12)</button>
        <button class="tag" data-tag="documentation">documentation (15)</button>
        <button class="tag" data-tag="code">code (25)</button>
    </div>
    
    <!-- Search and filters -->
    <div class="collection-search">
        <input type="search" placeholder="Search collections..." id="collection-search">
        
        <select id="category-filter">
            <option value="">All Categories</option>
            <option value="code">Code</option>
            <option value="documentation">Documentation</option>
            <option value="configuration">Configuration</option>
        </select>
        
        <select id="size-filter">
            <option value="">Any Size</option>
            <option value="small">Small (< 1K vectors)</option>
            <option value="medium">Medium (1K-10K)</option>
            <option value="large">Large (> 10K)</option>
        </select>
    </div>
</div>
```

## API Endpoints

```http
# Get namespace tree
GET /api/collections/tree
Response: {
  "root": {
    "name": "projects",
    "children": [
      {
        "name": "gateway",
        "collections": [
          {"name": "code", "vector_count": 1234},
          {"name": "docs", "vector_count": 456}
        ]
      }
    ]
  }
}

# Search collections
GET /api/collections/search?q=gateway&tags=typescript,code&category=code
Response: {
  "results": [...],
  "total": 4
}

# Get tags
GET /api/collections/tags
Response: {
  "tags": [
    {"name": "typescript", "count": 8},
    {"name": "rust", "count": 12},
    ...
  ]
}
```

## Implementation

```rust
// src/models/collection_organization.rs
pub struct CollectionOrganizer {
    namespace_tree: NamespaceTree,
    tag_index: TagIndex,
    category_index: CategoryIndex,
}

impl CollectionOrganizer {
    pub fn organize_collection(&mut self, collection: &Collection) {
        // 1. Extract namespace
        let namespace = self.extract_namespace(&collection.name);
        self.namespace_tree.add(namespace.clone(), collection);
        
        // 2. Generate and index tags
        let auto_tags = generate_auto_tags(collection);
        let all_tags = auto_tags.union(&collection.custom_tags);
        for tag in all_tags {
            self.tag_index.add(tag, &collection.name);
        }
        
        // 3. Index by category
        self.category_index.add(&collection.category, &collection.name);
    }
    
    pub fn search(&self, query: CollectionQuery) -> Vec<&Collection> {
        // Multi-index query optimization
        let candidates = if let Some(ns) = &query.namespace {
            self.namespace_tree.get_by_prefix(ns)
        } else if !query.tags.is_empty() {
            self.tag_index.get_by_tags(&query.tags)
        } else if let Some(cat) = &query.category {
            self.category_index.get_by_category(cat)
        } else {
            self.get_all_collections()
        };
        
        // Apply remaining filters
        self.apply_filters(candidates, &query)
    }
}
```

## Success Criteria

- ✅ Collections organized in logical namespaces
- ✅ Tag-based filtering working
- ✅ Search finds collections quickly
- ✅ Tree view intuitive and performant
- ✅ Auto-tagging accurate (>90%)
- ✅ Handles 1000+ collections efficiently

---

**Estimated Effort**: 2 weeks  
**Dependencies**: Dashboard improvements  
**Risk**: Low

