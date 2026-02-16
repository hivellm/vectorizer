---
name: generate-code-documentation
description: Analyzes Rust code and generates comprehensive documentation following project standards. Traverses functions, structs, enums, traits, and modules to create complete documentation with examples, parameters, return values, and error conditions. Use when documenting code, generating API docs, or when the user asks to document functions, modules, or create documentation.
---

# Generate Code Documentation

## Overview

This skill generates comprehensive documentation for Rust code by analyzing functions, structs, enums, traits, and modules. It follows Vectorizer project standards and creates documentation in English with proper formatting.

## Documentation Standards

### Module Documentation (`//!`)

Use `//!` for crate/module-level documentation at the top of files:

```rust
//! # Module Name
//!
//! Brief description of the module's purpose and responsibility.
//!
//! ## Overview
//!
//! Detailed explanation of what this module provides.
//!
//! ## Examples
//!
//! ```rust
//! use crate::module_name;
//!
//! // Example usage
//! ```
```

### Item Documentation (`///`)

Use `///` for all public functions, structs, enums, traits, and their fields:

```rust
/// Brief one-line description.
///
/// Detailed multi-line description explaining:
/// - What the function/struct/enum does
/// - When to use it
/// - Important behavior or constraints
///
/// # Arguments
///
/// * `param_name` - Description of parameter
/// * `other_param` - Description of other parameter
///
/// # Returns
///
/// Description of return value. Include:
/// - Success case: What is returned
/// - Error case: What errors can occur
///
/// # Errors
///
/// * `ErrorType::Variant` - When this error occurs
/// * `ErrorType::Other` - When this error occurs
///
/// # Examples
///
/// ```rust
/// use crate::module::function;
///
/// let result = function(param1, param2)?;
/// assert_eq!(result, expected_value);
/// ```
///
/// # Panics
///
/// Document if function can panic and under what conditions.
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

## Documentation Generation Workflow

### Step 1: Analyze Code Structure

1. **Read the target file(s)** using `read_file` or MCP Hive-Vectorizer `get_file_content`
2. **Identify all public items**:
   - Functions (`pub fn`)
   - Structs (`pub struct`)
   - Enums (`pub enum`)
   - Traits (`pub trait`)
   - Modules (`pub mod`)
   - Constants (`pub const`)
   - Type aliases (`pub type`)

3. **Check existing documentation**:
   - Does module have `//!` docs?
   - Do public items have `///` docs?
   - Are examples present?
   - Are error conditions documented?

### Step 2: Generate Module Documentation

For each module file:

```rust
//! # [Module Name]
//!
//! [Brief description of module purpose]
//!
//! ## Overview
//!
//! [Detailed explanation of module responsibilities and key concepts]
//!
//! ## Key Components
//!
//! - **[Component1]**: [Description]
//! - **[Component2]**: [Description]
//!
//! ## Usage
//!
//! ```rust
//! use crate::module_name;
//!
//! // Example usage
//! ```
```

### Step 3: Document Functions

For each public function:

1. **Analyze function signature**:
   - Function name and purpose
   - Parameters (types, names, purposes)
   - Return type and meaning
   - Error types possible

2. **Generate documentation**:
   ```rust
   /// [One-line summary of what function does]
   ///
   /// [Detailed description explaining behavior, side effects, constraints]
   ///
   /// # Arguments
   ///
   /// * `param_name` - [Type] [Description of parameter and its purpose]
   /// * `other_param` - [Type] [Description]
   ///
   /// # Returns
   ///
   /// Returns `Result<T, E>` where:
   /// - `Ok(T)`: [Description of success case]
   /// - `Err(E)`: [Description of error cases]
   ///
   /// # Errors
   ///
   /// * `ErrorType::Variant` - [When this error occurs and why]
   ///
   /// # Examples
   ///
   /// ```rust
   /// use crate::module::function;
   ///
   /// let result = function(arg1, arg2)?;
   /// ```
   ///
   /// # Panics
   ///
   /// [Document if function can panic]
   pub fn function_name(...) -> Result<...> {
   ```

### Step 4: Document Structs

For each public struct:

```rust
/// [One-line summary of struct purpose]
///
/// [Detailed description of what the struct represents and when to use it]
///
/// # Fields
///
/// * `field_name` - [Type] [Description of field purpose]
/// * `other_field` - [Type] [Description]
///
/// # Examples
///
/// ```rust
/// use crate::module::StructName;
///
/// let instance = StructName {
///     field_name: value,
///     other_field: value,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StructName {
    /// [Field description]
    pub field_name: Type,
    /// [Other field description]
    pub other_field: Type,
}
```

### Step 5: Document Enums

For each public enum:

```rust
/// [One-line summary of enum purpose]
///
/// [Detailed description of what the enum represents]
///
/// # Variants
///
/// * `Variant1` - [Description of when to use this variant]
/// * `Variant2(Type)` - [Description with parameter explanation]
///
/// # Examples
///
/// ```rust
/// use crate::module::EnumName;
///
/// let value = EnumName::Variant1;
/// ```
#[derive(Debug, Clone)]
pub enum EnumName {
    /// [Variant description]
    Variant1,
    /// [Variant description]
    Variant2(Type),
}
```

### Step 6: Document Traits

For each public trait:

```rust
/// [One-line summary of trait purpose]
///
/// [Detailed description of trait contract and when to implement it]
///
/// # Required Methods
///
/// * `method_name` - [Description of what implementors must provide]
///
/// # Examples
///
/// ```rust
/// use crate::module::TraitName;
///
/// struct Implementor;
///
/// impl TraitName for Implementor {
///     // Implementation
/// }
/// ```
pub trait TraitName {
    /// [Method description]
    fn method_name(&self) -> Result<Type, Error>;
}
```

## Code Analysis Checklist

When analyzing code for documentation:

- [ ] Identify all public items (functions, structs, enums, traits)
- [ ] Check if module has `//!` documentation
- [ ] Check if each public item has `///` documentation
- [ ] Verify examples are present and correct
- [ ] Verify error conditions are documented
- [ ] Check parameter descriptions are complete
- [ ] Verify return value descriptions
- [ ] Check for panic conditions
- [ ] Verify documentation is in English
- [ ] Check formatting follows project standards

## Documentation Quality Standards

### Required Elements

1. **One-line summary**: Brief description in first line
2. **Detailed description**: Multi-line explanation
3. **Parameters**: All parameters documented with types and purposes
4. **Return values**: Success and error cases explained
5. **Error conditions**: All possible errors documented
6. **Examples**: At least one usage example
7. **Panics**: Document if function can panic

### Optional Elements

- **Performance notes**: For performance-critical functions
- **Thread safety**: For concurrent code
- **Lifetime parameters**: For complex lifetime scenarios
- **Generic constraints**: For generic functions/types

## Examples

### Example 1: Function Documentation

**Before:**
```rust
pub fn create_collection(name: &str, config: CollectionConfig) -> Result<(), VectorizerError> {
    // Implementation
}
```

**After:**
```rust
/// Creates a new vector collection with the specified name and configuration.
///
/// This function registers a new collection in the vector store. The collection
/// must have a unique name and valid configuration parameters.
///
/// # Arguments
///
/// * `name` - Unique identifier for the collection (must not already exist)
/// * `config` - Configuration specifying dimension, metric, and index settings
///
/// # Returns
///
/// Returns `Ok(())` if the collection was created successfully.
///
/// # Errors
///
/// * `VectorizerError::CollectionAlreadyExists` - If a collection with this name already exists
/// * `VectorizerError::InvalidConfig` - If the configuration is invalid
/// * `VectorizerError::IoError` - If there was an I/O error during creation
///
/// # Examples
///
/// ```rust
/// use vectorizer::db::vector_store::VectorStore;
/// use vectorizer::models::CollectionConfig;
///
/// let store = VectorStore::new();
/// let config = CollectionConfig::default();
///
/// store.create_collection("my_collection", config)?;
/// ```
pub fn create_collection(name: &str, config: CollectionConfig) -> Result<(), VectorizerError> {
    // Implementation
}
```

### Example 2: Struct Documentation

**Before:**
```rust
pub struct CollectionConfig {
    pub dimension: usize,
    pub metric: DistanceMetric,
}
```

**After:**
```rust
/// Configuration for a vector collection.
///
/// Specifies the properties and behavior of a vector collection, including
/// vector dimensions, distance metrics, and indexing parameters.
///
/// # Fields
///
/// * `dimension` - The dimensionality of vectors in this collection (must be > 0)
/// * `metric` - The distance metric used for similarity calculations
///
/// # Examples
///
/// ```rust
/// use vectorizer::models::{CollectionConfig, DistanceMetric};
///
/// let config = CollectionConfig {
///     dimension: 512,
///     metric: DistanceMetric::Cosine,
/// };
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionConfig {
    /// Vector dimension (must be greater than 0)
    pub dimension: usize,
    /// Distance metric for similarity calculations
    pub metric: DistanceMetric,
}
```

## Markdown Documentation Generation

In addition to code comments (`//!` and `///`), generate standalone Markdown documentation files in `docs/` directory.

### Markdown Documentation Structure

Generate Markdown files following this structure:

```
docs/
├── api/                    # API documentation
│   ├── README.md          # API overview
│   └── [endpoint].md      # Individual endpoint docs
├── modules/                # Module documentation
│   ├── [module-name].md   # Module documentation
└── reference/             # Reference documentation
    └── [component].md     # Component reference
```

### Markdown File Template

For each module or component, create a Markdown file:

```markdown
# [Module/Component Name]

## Overview

[Brief description of what this module/component does]

## Purpose

[Detailed explanation of purpose and responsibilities]

## Key Components

### [Component 1]

[Description of component 1]

### [Component 2]

[Description of component 2]

## Functions

### `function_name`

[Function description]

**Signature:**
```rust
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType>
```

**Parameters:**
- `param1` - [Type1] [Description]
- `param2` - [Type2] [Description]

**Returns:**
[Description of return value]

**Errors:**
- `ErrorType::Variant` - [When this error occurs]

**Example:**
```rust
// Example usage
```

## Types

### `StructName`

[Struct description]

**Fields:**
- `field1` - [Type] [Description]
- `field2` - [Type] [Description]

## Usage Examples

[Complete usage examples]

## See Also

- [Related documentation links]
```

### Generating Markdown from Code Documentation

1. **Extract code documentation** from `//!` and `///` comments
2. **Convert to Markdown format**:
   - Module docs (`//!`) → Overview section
   - Function docs (`///`) → Functions section
   - Struct/Enum docs → Types section
   - Examples → Usage Examples section

3. **Organize by module**:
   - One Markdown file per module
   - Group related modules in subdirectories
   - Create index files for navigation

4. **Add cross-references**:
   - Link to related modules
   - Link to API documentation
   - Link to examples

### Markdown Documentation Workflow

1. **Analyze code structure** - Identify modules and components
2. **Extract documentation** - Read `//!` and `///` comments
3. **Generate Markdown** - Convert to Markdown format
4. **Organize files** - Create appropriate directory structure
5. **Add navigation** - Create index and cross-references
6. **Verify links** - Ensure all links are valid

## API Documentation Generation

For REST API endpoints, also generate documentation in `docs/api/`:

1. **Identify API handlers** in `src/server/` or `src/api/`
2. **Extract endpoint information**:
   - HTTP method (GET, POST, etc.)
   - Path pattern
   - Request parameters
   - Request body structure
   - Response structure
   - Error responses

3. **Generate API documentation** following format in `docs/api/README.md`

## Workflow for Complete Documentation

1. **Identify target**: File, module, or entire crate
2. **Analyze structure**: Read code and identify public items
3. **Check existing docs**: See what's already documented
4. **Generate code documentation**:
   - Add `//!` module documentation if missing
   - Add `///` item documentation for all public items
   - Include examples, parameters, return values, errors
5. **Generate Markdown documentation**:
   - Extract documentation from code comments
   - Convert to Markdown format
   - Organize in `docs/modules/` or appropriate directory
   - Create cross-references and navigation
6. **Enhance existing docs**: Improve incomplete documentation
7. **Add examples**: Include usage examples in both code and Markdown
8. **Verify standards**: Ensure all docs follow project standards
9. **Update API docs**: If applicable, update `docs/api/` files
10. **Verify Markdown**: Check links and formatting

## Best Practices

### DO's ✅

- ✅ Always document public items
- ✅ Include examples for complex functions
- ✅ Document all error conditions
- ✅ Use clear, concise language
- ✅ Follow project naming conventions
- ✅ Keep examples up-to-date with code
- ✅ Document edge cases and constraints

### DON'Ts ❌

- ❌ Don't document private items (unless internal module docs)
- ❌ Don't repeat obvious information
- ❌ Don't use vague descriptions
- ❌ Don't skip error documentation
- ❌ Don't use Portuguese in code documentation
- ❌ Don't create documentation that will quickly become outdated

## Integration with MCP Hive-Vectorizer

When analyzing code, prefer using MCP Hive-Vectorizer tools:

- Use `get_file_content` instead of `read_file` for indexed files
- Use `intelligent_search` to find related code
- Use `get_related_files` to discover dependencies
- Use `get_project_outline` to understand module structure

## Verification

After generating documentation:

1. Run `cargo doc --no-deps` to verify documentation compiles
2. Check for warnings about missing documentation
3. Verify examples compile and run
4. Ensure all public items are documented
5. Verify documentation follows English-only rule
