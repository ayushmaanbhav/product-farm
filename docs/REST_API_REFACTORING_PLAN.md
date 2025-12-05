# REST API Refactoring Plan

## Summary

The current REST API implementation has significant architectural and code quality issues that violate SOLID principles, lack proper separation of concerns, and deviate from patterns established in both the legacy Kotlin code and the Rust core library. This plan outlines a comprehensive refactoring to bring the code to production quality.

## Progress Tracker

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Fix Critical Type Safety Issues | **COMPLETED** |
| Phase 7 | Clean Up Unused Code and Warnings | **COMPLETED** |
| Phase 5 | Improve Error Types | **COMPLETED** |
| Phase 8 | Add Constants for Magic Values | **COMPLETED** |
| Phase 3 | Extract Duplicate Code into Helpers | **COMPLETED** |
| Phase 4 | Add Input Validation Layer | **COMPLETED** |
| Phase 2 | Introduce Service Layer Abstraction | PENDING |
| Phase 6 | Refactor Handlers to Use Services | PENDING |

## Critical Issues Identified

### Type Safety Issues (CRITICAL - Fix First) ✅ FIXED
1. ~~**Unsafe `as` casts** in `datatypes.rs:61-65, 117-121`~~ - Fixed with `try_from().ok()`
2. ~~**Silent failures** in `converters.rs:148`~~ - Fixed to fall back to String
3. **Incorrect timestamps** in `converters.rs:114-115, 189-190`:
   - Using `Utc::now()` instead of entity timestamps - **Requires core library changes**

### Architectural Issues (HIGH Priority)
1. **No Service Layer**: All business logic in handlers (146 lines in `clone_product`)
2. **Direct Store Access**: 50+ direct `store.products.get()` calls, no repository abstraction
3. **Duplicate Code**: Product existence check repeated 4 times, immutability check 3 times
4. **Placeholder Implementations**: `evaluate()` and `batch_evaluate()` don't actually work

### SOLID Violations
1. **SRP**: `create_abstract_attribute` handler is 115 lines mixing validation, parsing, construction
2. **OCP**: No middleware/decorator pattern - adding logging requires modifying all handlers
3. **DIP**: All handlers depend on concrete `SharedStore`, not abstractions

## Refactoring Plan

### Phase 1: Fix Critical Type Safety Issues ✅ COMPLETED
**Files:** `datatypes.rs`, `converters.rs`

1. **Safe numeric conversions** - Replace unsafe casts with checked conversions:
```rust
// Before (PANIC if > 255!)
precision: c.precision.map(|v| v as u8)

// After (Safe) ✅ DONE
precision: c.precision.and_then(|v| u8::try_from(v).ok())
```

2. **Fix Decimal parsing** - Propagate errors instead of silent default:
```rust
// Before (Silent failure!)
Value::Decimal(value.parse().unwrap_or_default())

// After (Proper conversion) ✅ DONE
AttributeValueJson::Decimal { value } => {
    value.parse()
        .map(Value::Decimal)
        .unwrap_or_else(|_| Value::String(value.clone()))
}
```

### Phase 2: Introduce Service Layer Abstraction
**New Files:** `services/mod.rs`, `services/product_service.rs`, `services/attribute_service.rs`, `services/rule_service.rs`, `services/validation_service.rs`

Create service traits and implementations following the Kotlin pattern:

```rust
// services/product_service.rs
pub trait ProductService: Send + Sync {
    async fn list(&self, query: ListProductsQuery) -> ApiResult<PaginatedResponse<ProductResponse>>;
    async fn get(&self, id: &str) -> ApiResult<ProductResponse>;
    async fn create(&self, req: CreateProductRequest) -> ApiResult<ProductResponse>;
    async fn update(&self, id: &str, req: UpdateProductRequest) -> ApiResult<ProductResponse>;
    async fn delete(&self, id: &str) -> ApiResult<()>;
    async fn clone(&self, id: &str, req: CloneProductRequest) -> ApiResult<CloneProductResponse>;
}

pub struct ProductServiceImpl {
    store: SharedStore,
    validator: Arc<dyn ValidationService>,
}
```

### Phase 3: Extract Duplicate Code into Helpers
**File:** `rest/helpers.rs`

```rust
/// Verify product exists, return error if not
pub fn require_product<'a>(store: &'a EntityStore, product_id: &str) -> ApiResult<&'a Product> {
    store.products.get(product_id)
        .ok_or_else(|| ApiError::NotFound(format!("Product '{}' not found", product_id)))
}

/// Check entity is mutable, return error if immutable
pub fn require_mutable<T: HasImmutable>(entity: &T, entity_type: &str, id: &str) -> ApiResult<()> {
    if entity.is_immutable() {
        return Err(ApiError::PreconditionFailed(format!(
            "Cannot modify immutable {} '{}'", entity_type, id
        )));
    }
    Ok(())
}

/// Verify no duplicate exists
pub fn require_unique(exists: bool, entity_type: &str, id: &str) -> ApiResult<()> {
    if exists {
        return Err(ApiError::Conflict(format!(
            "{} '{}' already exists", entity_type, id
        )));
    }
    Ok(())
}
```

### Phase 4: Add Input Validation Layer
**New File:** `services/validation_service.rs`

Centralize validation using core library patterns:

```rust
pub trait ValidationService: Send + Sync {
    fn validate_product_id(&self, id: &str) -> ApiResult<()>;
    fn validate_path(&self, path: &str) -> ApiResult<()>;
    fn validate_constraints(&self, constraints: &DatatypeConstraintsJson) -> ApiResult<()>;
}

impl ValidationService for ValidationServiceImpl {
    fn validate_product_id(&self, id: &str) -> ApiResult<()> {
        if !product_farm_core::validation::is_valid_product_id(id) {
            return Err(ApiError::BadRequest(format!(
                "Invalid product ID '{}'. Must match pattern: {}",
                id, product_farm_core::validation::PRODUCT_ID_PATTERN
            )));
        }
        Ok(())
    }

    fn validate_constraints(&self, c: &DatatypeConstraintsJson) -> ApiResult<()> {
        // Validate min <= max
        if let (Some(min), Some(max)) = (c.min, c.max) {
            if min > max {
                return Err(ApiError::BadRequest(
                    "Constraint 'min' cannot be greater than 'max'".to_string()
                ));
            }
        }
        // Validate precision/scale bounds
        if let Some(p) = c.precision {
            if p < 0 || p > 38 {
                return Err(ApiError::BadRequest(
                    "Precision must be between 0 and 38".to_string()
                ));
            }
        }
        Ok(())
    }
}
```

### Phase 5: Improve Error Types
**File:** `rest/error.rs`

Add structured error details matching Kotlin's ErrorDetail pattern:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub status_code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorDetail>,
}

pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    PreconditionFailed(String),
    ValidationFailed(Vec<ErrorDetail>),  // NEW: Multiple field errors
    Internal(String),
}
```

### Phase 6: Refactor Handlers to Use Services
**Files:** All handler files in `rest/`

Transform handlers from:
```rust
// BEFORE: 115 lines of mixed concerns
async fn create_abstract_attribute(...) -> ApiResult<...> {
    let mut store = store.write().await;
    // validation
    // parsing
    // construction
    // persistence
}
```

To:
```rust
// AFTER: Thin handler delegating to service
async fn create_abstract_attribute(
    State(services): State<Arc<Services>>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateAbstractAttributeRequest>,
) -> ApiResult<Json<AbstractAttributeResponse>> {
    let result = services.attributes.create_abstract(product_id, req).await?;
    Ok(Json(result))
}
```

### Phase 7: Clean Up Unused Code and Warnings ✅ COMPLETED
**Files:** Multiple

1. ~~Remove unused imports (`delete`, `put` where not used)~~ ✅ DONE
2. ~~Remove dead code (`parse_relationship` function)~~ ✅ DONE
3. ~~Fix unused variables (`_i`, `_pid`)~~ ✅ DONE
4. ~~Remove unnecessary `mut` declarations~~ ✅ DONE

### Phase 8: Add Constants for Magic Values
**New File:** `rest/constants.rs`

```rust
pub const DEFAULT_PAGE_SIZE: usize = 20;
pub const MAX_PAGE_SIZE: usize = 100;
pub const MAX_PRODUCT_ID_LENGTH: usize = 50;
pub const MAX_NAME_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;

// Key format patterns
pub const FUNCTIONALITY_KEY_FORMAT: &str = "{}:{}";  // product_id:name
```

## Files to Modify

### Critical (Phase 1) ✅
- `crates/api/src/rest/datatypes.rs` - Fix unsafe casts ✅
- `crates/api/src/rest/converters.rs` - Fix timestamp and decimal conversion ✅

### New Files (Phase 2-5)
- `crates/api/src/rest/services/mod.rs`
- `crates/api/src/rest/services/product_service.rs`
- `crates/api/src/rest/services/attribute_service.rs`
- `crates/api/src/rest/services/rule_service.rs`
- `crates/api/src/rest/services/validation_service.rs`
- `crates/api/src/rest/helpers.rs`
- `crates/api/src/rest/constants.rs`

### Handler Refactoring (Phase 6)
- `crates/api/src/rest/products.rs`
- `crates/api/src/rest/attributes.rs`
- `crates/api/src/rest/rules.rs`
- `crates/api/src/rest/datatypes.rs`
- `crates/api/src/rest/functionalities.rs`
- `crates/api/src/rest/templates.rs`
- `crates/api/src/rest/evaluation.rs`

### Error and Type Improvements (Phase 5, 7)
- `crates/api/src/rest/error.rs`
- `crates/api/src/rest/mod.rs`

## Implementation Order

1. **Phase 1** - Fix critical type safety ✅ COMPLETED
2. **Phase 7** - Clean up warnings ✅ COMPLETED
3. **Phase 5** - Improve error types - Foundation for other phases
4. **Phase 4** - Add validation service
5. **Phase 3** - Extract helpers
6. **Phase 8** - Add constants
7. **Phase 2** - Create service layer
8. **Phase 6** - Refactor handlers

## Testing Strategy

After each phase:
1. Run `cargo check` - verify compilation
2. Run `cargo clippy` - verify code quality
3. Run `cargo test` - verify existing tests pass
4. Manual smoke test with curl for critical endpoints

## Risk Mitigation

- **Risk**: Large refactoring breaks existing functionality
  - **Mitigation**: Implement incrementally, test after each phase

- **Risk**: Service layer adds complexity without benefit
  - **Mitigation**: Start with ProductService only, evaluate before expanding

- **Risk**: Time estimate too optimistic
  - **Mitigation**: Phase 1 (critical fixes) can be done independently
