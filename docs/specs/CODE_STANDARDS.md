# 📝 Code Standards - Vectorizer Project

## 🌍 Language Requirements

### **MANDATORY: All implementation code MUST be in ENGLISH** 🇺🇸

This includes:
- ✅ **File names**: `detector.rs`, `vulkan_backend.rs` (NOT `detector_gpu.rs` in Portuguese)
- ✅ **Function names**: `detect_gpu()`, `initialize_backend()` (NOT `detectar_gpu()`)
- ✅ **Variable names**: `gpu_context`, `backend_type` (NOT `contexto_gpu`)
- ✅ **Comments**: `// Initialize Metal backend` (NOT `// Inicializar backend Metal`)
- ✅ **Log messages**: `"GPU detected"` (NOT `"GPU detectada"`)
- ✅ **Error messages**: `"Failed to initialize"` (NOT `"Falha ao inicializar"`)
- ✅ **Struct/Enum names**: `GpuBackendType`, `VulkanConfig` (NOT `TipoBackendGpu`)
- ✅ **Documentation**: Doc comments in English

### Exceptions (Portuguese allowed)
- ❌ **User-facing CLI messages**: Can be in Portuguese for Brazilian users
- ❌ **README files**: Can have Portuguese versions (README_PT.md)
- ❌ **Project documentation**: Can have Portuguese for planning
- ❌ **Task descriptions**: Can be in Portuguese in task queue
- ❌ **Commit messages**: Can be in Portuguese

---

## ✅ CORRECT Examples

### File Structure
```
src/gpu/
├── mod.rs
├── config.rs
├── context.rs
├── operations.rs
├── metal_collection.rs
├── backends/
│   ├── mod.rs
│   ├── detector.rs          ✅ ENGLISH
│   ├── vulkan.rs            ✅ ENGLISH
│   ├── dx12.rs              ✅ ENGLISH
│   └── metal.rs             ✅ ENGLISH
└── shaders/
    ├── similarity.wgsl
    └── distance.wgsl
```

### Code Example (CORRECT)
```rust
/// Detect available GPU backends on the system
pub fn detect_available_backends() -> Vec<GpuBackendType> {
    let mut available = Vec::new();
    
    // Try Metal first on macOS
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    if metal_available() {
        available.push(GpuBackendType::Metal);
    }
    
    // Try Vulkan (universal)
    if vulkan_available() {
        available.push(GpuBackendType::Vulkan);
    }
    
    // Try DirectX 12 on Windows
    #[cfg(target_os = "windows")]
    if dx12_available() {
        available.push(GpuBackendType::DirectX12);
    }
    
    available
}
```

### Struct Example (CORRECT)
```rust
/// GPU backend configuration
#[derive(Debug, Clone)]
pub struct GpuBackendConfig {
    /// Preferred backend type
    pub preferred_backend: Option<GpuBackendType>,
    /// Backend priority order
    pub backend_priority: Vec<GpuBackendType>,
    /// Enable automatic detection
    pub auto_detect: bool,
}
```

### Log Messages (CORRECT)
```rust
info!("Metal GPU detected and enabled");
warn!("Vulkan detection failed, falling back to CPU");
error!("Failed to initialize GPU context: {}", err);
eprintln!("✅ Using Metal backend (Apple Silicon)");
```

---

## ❌ INCORRECT Examples (DO NOT DO THIS)

### File Structure (WRONG)
```
src/gpu/
├── backends/
│   ├── detector.rs
│   ├── vulkan.rs
│   ├── dx12.rs
│   └── metal.rs            ❌ OK (metal is English)
```

### Code Example (WRONG - Portuguese)
```rust
/// Detectar backends GPU disponíveis no sistema   ❌ WRONG
pub fn detectar_backends_disponiveis() -> Vec<TipoBackendGpu> {  ❌ WRONG
    let mut disponiveis = Vec::new();  ❌ WRONG
    
    // Tentar Metal primeiro no macOS  ❌ WRONG
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    if metal_disponivel() {  ❌ WRONG
        disponiveis.push(TipoBackendGpu::Metal);  ❌ WRONG
    }
    
    disponiveis  ❌ WRONG
}
```

### Struct Example (WRONG - Portuguese)
```rust
/// Configuração do backend GPU  ❌ WRONG
#[derive(Debug, Clone)]
pub struct ConfigBackendGpu {  ❌ WRONG
    /// Backend preferido  ❌ WRONG
    pub backend_preferido: Option<TipoBackendGpu>,  ❌ WRONG
    /// Ordem de prioridade  ❌ WRONG
    pub prioridade_backend: Vec<TipoBackendGpu>,  ❌ WRONG
}
```

### Log Messages (WRONG - Portuguese)
```rust
info!("Metal GPU detectada e habilitada");  ❌ WRONG
warn!("Detecção Vulkan falhou, usando CPU");  ❌ WRONG
eprintln!("✅ Usando backend Metal");  ❌ WRONG
```

---

## 📚 Naming Conventions

### Functions
```rust
// ✅ CORRECT
pub fn detect_backend() -> Result<GpuBackendType>
pub fn initialize_vulkan() -> Result<VulkanBackend>
pub fn get_gpu_info() -> GpuInfo

// ❌ WRONG
pub fn detectar_backend() -> Result<TipoBackendGpu>
pub fn inicializar_vulkan() -> Result<BackendVulkan>
pub fn obter_info_gpu() -> InfoGpu
```

### Variables
```rust
// ✅ CORRECT
let gpu_context = GpuContext::new()?;
let backend_type = GpuBackendType::Vulkan;
let available_backends = detect_available_backends();

// ❌ WRONG
let contexto_gpu = GpuContext::new()?;
let tipo_backend = TipoBackendGpu::Vulkan;
let backends_disponiveis = detectar_backends_disponiveis();
```

### Structs/Enums
```rust
// ✅ CORRECT
pub enum GpuBackendType {
    Metal,
    Vulkan,
    DirectX12,
}

pub struct VulkanBackend {
    instance: Instance,
    adapter: Adapter,
}

// ❌ WRONG
pub enum TipoBackendGpu {
    Metal,
    Vulkan,
    DirectX12,
}

pub struct BackendVulkan {
    instancia: Instance,
    adaptador: Adapter,
}
```

---

## 🎯 User-Facing Messages (Portuguese OK)

### CLI Output (Can be Portuguese)
```rust
// ✅ ACCEPTABLE for Brazilian users
println!("🚀 Iniciando servidor Vectorizer...");
println!("✅ GPU Metal detectada: Apple M3 Pro");
println!("⚠️ Nenhuma GPU encontrada, usando CPU");
```

### Error Messages to Users (Can be Portuguese)
```rust
// ✅ ACCEPTABLE for Brazilian users
eprintln!("❌ Erro: Não foi possível inicializar a GPU");
eprintln!("💡 Dica: Compile com --features wgpu-gpu para habilitar Metal");
```

---

## 📖 Documentation Standards

### Doc Comments (MUST be English)
```rust
// ✅ CORRECT
/// Creates a new GPU context with automatic backend detection.
///
/// This function tries backends in order of priority:
/// 1. Metal (macOS Apple Silicon)
/// 2. Vulkan (AMD/Linux/Universal)
/// 3. DirectX 12 (Windows)
/// 4. GPU (NVIDIA)
/// 5. CPU (fallback)
///
/// # Examples
/// ```
/// let context = GpuContext::new_auto()?;
/// ```
pub fn new_auto() -> Result<Self>

// ❌ WRONG
/// Cria um novo contexto GPU com detecção automática.  ❌ WRONG
pub fn new_auto() -> Result<Self>
```

### README Files
```
✅ README.md (English - primary)
✅ README_PT.md (Portuguese - optional)
✅ docs/ARCHITECTURE.md (English)
✅ docs/ARQUITETURA_PT.md (Portuguese - optional)
```

---

## 🚀 Migration Guide (Portuguese → English)

### Step 1: Rename Functions
```bash
# Find Portuguese function names
rg "pub fn [a-z_]*ar\(" src/

# Rename them
# detectar_backend() → detect_backend()
# inicializar_gpu() → initialize_gpu()
# obter_info() → get_info()
```

### Step 2: Rename Variables
```bash
# Find Portuguese variables
rg "let [a-z_]*ado " src/

# Rename them
# backend_detectado → detected_backend
# contexto_gpu → gpu_context
# configuracao → configuration
```

### Step 3: Translate Comments
```bash
# Find Portuguese comments
rg "^[[:space:]]*//.*[áàâãéêíóôõúç]" src/

# Translate to English
```

### Step 4: Update Log Messages
```bash
# Find Portuguese log messages
rg 'info!\(".*[áàâãéêíóôõúç]' src/
rg 'warn!\(".*[áàâãéêíóôõúç]' src/
rg 'error!\(".*[áàâãéêíóôõúç]' src/

# Translate internal logs to English
# Keep user-facing logs in Portuguese if desired
```

---

## ✅ Checklist Before Commit

- [ ] All function names are in English
- [ ] All variable names are in English
- [ ] All struct/enum names are in English
- [ ] All comments are in English
- [ ] Doc comments (///) are in English
- [ ] Internal log messages are in English
- [ ] File names are in English
- [ ] No Portuguese words in implementation code
- [ ] User-facing messages can remain in Portuguese (optional)

---

## 📞 Questions?

If unsure about naming:
1. Check Rust standard library for similar patterns
2. Use American English spelling (e.g., `initialize` not `initialise`)
3. Be descriptive but concise
4. Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

**When in doubt: USE ENGLISH!** 🇺🇸

