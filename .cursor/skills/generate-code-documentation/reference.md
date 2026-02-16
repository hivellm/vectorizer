# Documentation Templates and Reference

## Module Documentation Template

```rust
//! # [Module Name]
//!
//! [One-sentence description of module purpose]
//!
//! ## Overview
//!
//! [Detailed paragraph explaining what this module provides, its main
//! responsibilities, and key concepts. Explain the "why" not just the "what".]
//!
//! ## Key Components
//!
//! - **[Component1]**: [Brief description]
//! - **[Component2]**: [Brief description]
//! - **[Component3]**: [Brief description]
//!
//! ## Architecture
//!
//! [If applicable, explain how this module fits into the larger system]
//!
//! ## Usage
//!
//! ```rust
//! use crate::module_name;
//!
//! // Basic usage example
//! ```
//!
//! ## Examples
//!
//! ### Example 1: [Scenario]
//!
//! ```rust
//! use crate::module_name;
//!
//! // Example code
//! ```
//!
//! ### Example 2: [Another Scenario]
//!
//! ```rust
//! // Another example
//! ```
```

## Function Documentation Template

### Simple Function

```rust
/// [One-line summary: what the function does]
///
/// [Detailed description explaining behavior, side effects, and important
/// considerations. Use multiple paragraphs if needed.]
///
/// # Arguments
///
/// * `param1` - [Type] [Description of parameter and its purpose]
/// * `param2` - [Type] [Description of parameter]
///
/// # Returns
///
/// [Description of return value. For Result types, explain both Ok and Err cases]
///
/// # Errors
///
/// * `ErrorType::Variant1` - [When this error occurs and why]
/// * `ErrorType::Variant2` - [When this error occurs and why]
///
/// # Examples
///
/// ```rust
/// use crate::module::function_name;
///
/// let result = function_name(arg1, arg2)?;
/// ```
///
/// # Panics
///
/// [Document if function can panic, under what conditions]
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

### Async Function

```rust
/// [One-line summary]
///
/// [Detailed description. Mention that this is an async function and any
/// concurrency considerations.]
///
/// # Arguments
///
/// * `param` - [Type] [Description]
///
/// # Returns
///
/// Returns a `Future` that resolves to `Result<T, E>`:
/// - `Ok(T)`: [Success case]
/// - `Err(E)`: [Error cases]
///
/// # Errors
///
/// * `ErrorType::Variant` - [Description]
///
/// # Examples
///
/// ```rust
/// use crate::module::async_function;
///
/// let result = async_function(param).await?;
/// ```
///
/// # Cancellation
///
/// [If applicable, document cancellation behavior]
pub async fn async_function(param: Type) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

### Generic Function

```rust
/// [One-line summary]
///
/// [Detailed description. Explain what the generic type represents.]
///
/// # Type Parameters
///
/// * `T` - [Constraint] [Description of what T represents]
/// * `U` - [Constraint] [Description of what U represents]
///
/// # Arguments
///
/// * `param` - [Type] [Description]
///
/// # Returns
///
/// [Description]
///
/// # Examples
///
/// ```rust
/// use crate::module::generic_function;
///
/// let result = generic_function::<i32, String>(param)?;
/// ```
pub fn generic_function<T, U>(param: Type) -> Result<T, ErrorType>
where
    T: TraitBound,
    U: OtherBound,
{
    // Implementation
}
```

## Struct Documentation Template

```rust
/// [One-line summary: what the struct represents]
///
/// [Detailed description explaining:
/// - What the struct represents
/// - When to use it
/// - Important invariants or constraints
/// - Relationship to other types]
///
/// # Fields
///
/// * `field1` - [Type] [Description of field purpose and constraints]
/// * `field2` - [Type] [Description]
/// * `field3` - [Type] [Description]
///
/// # Invariants
///
/// [If applicable, document any invariants that must be maintained]
///
/// # Examples
///
/// ```rust
/// use crate::module::StructName;
///
/// let instance = StructName {
///     field1: value1,
///     field2: value2,
///     field3: value3,
/// };
/// ```
///
/// # Thread Safety
///
/// [If applicable, document thread safety guarantees]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructName {
    /// [Field description]
    pub field1: Type1,
    /// [Field description]
    pub field2: Type2,
    /// [Field description]
    pub field3: Type3,
}
```

## Enum Documentation Template

```rust
/// [One-line summary: what the enum represents]
///
/// [Detailed description explaining:
/// - What the enum represents
/// - When to use each variant
/// - Relationship to other types]
///
/// # Variants
///
/// * `Variant1` - [Description of when to use this variant]
/// * `Variant2(Type)` - [Description. Explain what the type parameter represents]
/// * `Variant3 { field: Type }` - [Description. Explain struct-like variant]
///
/// # Examples
///
/// ```rust
/// use crate::module::EnumName;
///
/// let value = EnumName::Variant1;
/// let value_with_data = EnumName::Variant2(data);
/// ```
///
/// # Serialization
///
/// [If applicable, document serialization format]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnumName {
    /// [Variant description]
    Variant1,
    /// [Variant description]
    Variant2(Type),
    /// [Variant description]
    Variant3 {
        /// [Field description]
        field: Type,
    },
}
```

## Trait Documentation Template

```rust
/// [One-line summary: what the trait represents]
///
/// [Detailed description explaining:
/// - What the trait represents
/// - When to implement it
/// - What implementors must provide
/// - Relationship to other traits]
///
/// # Required Methods
///
/// * `method1` - [Description of what implementors must provide]
/// * `method2` - [Description]
///
/// # Provided Methods
///
/// * `provided_method` - [Description of default implementation]
///
/// # Examples
///
/// ```rust
/// use crate::module::TraitName;
///
/// struct Implementor;
///
/// impl TraitName for Implementor {
///     fn method1(&self) -> Result<Type, Error> {
///         // Implementation
///     }
/// }
/// ```
///
/// # Supertraits
///
/// [If applicable, document required supertraits]
pub trait TraitName: SuperTrait {
    /// [Method description]
    fn method1(&self) -> Result<Type, Error>;
    
    /// [Method description with default implementation]
    fn provided_method(&self) -> Result<Type, Error> {
        // Default implementation
    }
}
```

## Constant Documentation Template

```rust
/// [One-line summary: what the constant represents]
///
/// [Detailed description explaining:
/// - What the constant represents
/// - When to use it
/// - Any constraints or invariants]
///
/// # Examples
///
/// ```rust
/// use crate::module::CONSTANT_NAME;
///
/// let value = CONSTANT_NAME;
/// ```
pub const CONSTANT_NAME: Type = value;
```

## Type Alias Documentation Template

```rust
/// [One-line summary: what the type alias represents]
///
/// [Detailed description explaining:
/// - What the alias represents
/// - Why it exists (convenience, clarity, etc.)
/// - When to use it vs the underlying type]
///
/// # Examples
///
/// ```rust
/// use crate::module::TypeAlias;
///
/// let value: TypeAlias = underlying_type_value;
/// ```
pub type TypeAlias = UnderlyingType;
```

## Error Documentation Template

```rust
/// [One-line summary: what this error represents]
///
/// [Detailed description explaining:
/// - When this error occurs
/// - What caused it
/// - How to handle it]
///
/// # Examples
///
/// ```rust
/// use crate::module::ErrorType;
///
/// match result {
///     Err(ErrorType::Variant) => {
///         // Handle error
///     }
///     _ => {}
/// }
/// ```
#[derive(thiserror::Error, Debug)]
pub enum ErrorType {
    /// [Error variant description]
    #[error("Error message format: {0}")]
    Variant(String),
}
```

## API Endpoint Documentation Template

For REST API endpoints, use this format in `docs/api/`:

```markdown
## Endpoint

**POST** `/api/v1/endpoint`

## Description

[Description of what the endpoint does]

## Request Format

**Content-Type:** `application/json`

## Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field1` | String | Yes | [Description] |
| `field2` | Integer | No | [Description] |

## Response Format

**Content-Type:** `application/json`

## Success Response (200 OK)

```json
{
  "field1": "value",
  "field2": 123
}
```

## Error Responses

| Status Code | Error Type | Description |
|-------------|------------|-------------|
| 400 | `BadRequest` | [When this error occurs] |
| 404 | `NotFound` | [When this error occurs] |
| 500 | `InternalError` | [When this error occurs] |

## Examples

### Example Request

```bash
curl -X POST http://localhost:15002/api/v1/endpoint \
  -H "Content-Type: application/json" \
  -d '{"field1": "value", "field2": 123}'
```

### Example Response

```json
{
  "field1": "value",
  "field2": 123
}
```
```

## Documentation Quality Checklist

When generating documentation, verify:

- [ ] One-line summary is clear and concise
- [ ] Detailed description explains "why" not just "what"
- [ ] All parameters are documented with types
- [ ] Return values are fully explained
- [ ] All error conditions are documented
- [ ] At least one example is provided
- [ ] Examples compile and are correct
- [ ] Panic conditions are documented (if applicable)
- [ ] Documentation is in English
- [ ] Formatting follows project standards
- [ ] No obvious information is repeated
- [ ] Edge cases are mentioned
- [ ] Constraints and invariants are documented

## Markdown Documentation Templates

### Module Documentation Template

```markdown
# [Module Name]

## Overview

[Brief description of module purpose]

## Purpose

[Detailed explanation of what this module provides, its main responsibilities, and key concepts]

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
- `param1` - `Type1` - [Description of parameter]
- `param2` - `Type2` - [Description of parameter]

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
- `field1` - `Type` - [Description]
- `field2` - `Type` - [Description]

## Usage Examples

[Complete usage examples]

## Thread Safety

[If applicable, document thread safety guarantees]

## Performance

[If applicable, document performance characteristics]

## See Also

- [Related documentation links]
```

### Function Documentation in Markdown

```markdown
### `function_name`

[One-line summary of what the function does]

[Detailed description explaining behavior, side effects, and important considerations]

**Signature:**
```rust
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType>
```

**Parameters:**
- `param1` - `Type1` - [Description of parameter and its purpose]
- `param2` - `Type2` - [Description]

**Returns:**
[Description of return value. For Result types, explain both Ok and Err cases]

**Errors:**
- `ErrorType::Variant1` - [When this error occurs and why]
- `ErrorType::Variant2` - [When this error occurs and why]

**Example:**
```rust
use crate::module::function_name;

let result = function_name(arg1, arg2)?;
```

**Panics:**
[Document if function can panic, under what conditions]

**Performance:**
[If applicable, document performance characteristics]
```

### Struct Documentation in Markdown

```markdown
### `StructName`

[One-line summary of what the struct represents]

[Detailed description explaining what the struct represents, when to use it, important invariants or constraints]

**Fields:**
- `field1` - `Type1` - [Description of field purpose and constraints]
- `field2` - `Type2` - [Description]

**Example:**
```rust
use crate::module::StructName;

let instance = StructName {
    field1: value1,
    field2: value2,
};
```

**Invariants:**
[If applicable, document any invariants that must be maintained]

**Thread Safety:**
[If applicable, document thread safety guarantees]
```

### Enum Documentation in Markdown

```markdown
### `EnumName`

[One-line summary of what the enum represents]

[Detailed description explaining what the enum represents, when to use each variant]

**Variants:**
- `Variant1` - [Description of when to use this variant]
- `Variant2(Type)` - [Description. Explain what the type parameter represents]
- `Variant3 { field: Type }` - [Description. Explain struct-like variant]

**Example:**
```rust
use crate::module::EnumName;

let value = EnumName::Variant1;
let value_with_data = EnumName::Variant2(data);
```

**Serialization:**
[If applicable, document serialization format]
```

### Trait Documentation in Markdown

```markdown
### `TraitName`

[One-line summary of what the trait represents]

[Detailed description explaining what the trait represents, when to implement it, what implementors must provide]

**Required Methods:**
- `method1` - [Description of what implementors must provide]
- `method2` - [Description]

**Provided Methods:**
- `provided_method` - [Description of default implementation]

**Example:**
```rust
use crate::module::TraitName;

struct Implementor;

impl TraitName for Implementor {
    fn method1(&self) -> Result<Type, Error> {
        // Implementation
    }
}
```

**Supertraits:**
[If applicable, document required supertraits]
```

### API Endpoint Documentation in Markdown

```markdown
## Endpoint

**POST** `/api/v1/endpoint`

## Description

[Description of what the endpoint does]

## Request Format

**Content-Type:** `application/json`

## Request Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field1` | String | Yes | [Description] |
| `field2` | Integer | No | [Description] |

## Response Format

**Content-Type:** `application/json`

## Success Response (200 OK)

```json
{
  "field1": "value",
  "field2": 123
}
```

## Error Responses

| Status Code | Error Type | Description |
|-------------|------------|-------------|
| 400 | `BadRequest` | [When this error occurs] |
| 404 | `NotFound` | [When this error occurs] |
| 500 | `InternalError` | [When this error occurs] |

## Examples

### Example Request

```bash
curl -X POST http://localhost:15002/api/v1/endpoint \
  -H "Content-Type: application/json" \
  -d '{"field1": "value", "field2": 123}'
```

### Example Response

```json
{
  "field1": "value",
  "field2": 123
}
```
```

### Module Index Template

```markdown
# Module Documentation

## Core Modules

- [Module Name](module_name.md) - [Brief description]
- [Another Module](another_module.md) - [Brief description]

## Reference

- [Type Reference](../reference/types.md)
- [Error Reference](../reference/errors.md)

## API Documentation

- [API Overview](../api/README.md)
- [Endpoints](../api/endpoints.md)
```
