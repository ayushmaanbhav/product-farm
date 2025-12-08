# Frontend-Backend Integration Plan

## Executive Summary

This plan details the work required to wire up the Product-FARM React frontend to the Rust backend REST API. The frontend currently operates with mock data (`VITE_USE_MOCK=true`). The integration requires:

1. **Backend work**: Implement missing endpoints, fix response structures
2. **Frontend work**: Align API client with actual backend URLs/types
3. **Both**: Ensure type compatibility between TypeScript and Rust

---

## Current State Analysis

### Frontend Stack
- **Framework**: React 19.2.0 + TypeScript 5.9.3 + Vite 7.2.4
- **State**: Zustand 5.0.9 (ProductStore, SimulationStore, UIStore)
- **Visualization**: @xyflow/react 12.9.3 (DAG canvas)
- **API Client**: `/frontend/src/services/api.ts` with mock fallback

### Backend Stack
- **Framework**: Axum (async Rust web framework)
- **Storage**: In-memory EntityStore with RwLock
- **Rule Engine**: JSON Logic evaluator with DAG execution

---

## Gap Analysis

### Critical: Rule Evaluation Not Implemented

**Impact**: HIGH - Core functionality broken

The frontend simulation panel calls `/api/evaluate` which returns `501 Not Implemented`.

```
Backend: evaluation.rs:51
  Err(ApiError::not_implemented("Rule evaluation is not yet implemented..."))
```

### API URL Mismatches

| Frontend Calls | Backend Expects | Status |
|----------------|-----------------|--------|
| `POST /api/abstract-attributes` | `POST /api/products/{product_id}/abstract-attributes` | **MISMATCH** |
| `POST /api/attributes` | `POST /api/products/{product_id}/attributes` | **MISMATCH** |
| `POST /api/rules` | `POST /api/products/{product_id}/rules` | **MISMATCH** |
| `POST /api/functionalities` | `POST /api/products/{product_id}/functionalities` | **MISMATCH** |

### Missing Backend Endpoints

| Endpoint | Frontend Usage | Priority |
|----------|---------------|----------|
| `POST /api/evaluate` | SimulationPanel, BatchEvaluator | **P0** |
| `POST /api/batch-evaluate` | BatchEvaluator | **P0** |
| `POST /api/evaluate-functionality` | FunctionalityPanel | P1 |
| `POST /api/products/{id}/impact-analysis` | RuleCanvas impact highlighting | P1 |
| `GET /api/products/{id}/abstract-attributes/by-component/{type}` | AttributeExplorer grouping | P2 |
| `GET /api/products/{id}/abstract-attributes/by-tag/{tag}` | Tag filtering | P2 |
| `GET /api/products/{id}/functionalities/{name}/rules` | Functionality rule list | P2 |
| `POST /api/products/{id}/functionalities/{name}/submit` | Functionality approval | P2 |
| `POST /api/products/{id}/functionalities/{name}/approve` | Functionality approval | P2 |
| `POST /api/products/from-template` | ProductCreationWizard | P3 |
| `GET /api/product-templates/{id}` | Template details | P3 |

### Response Type Mismatches

| Issue | Frontend Expects | Backend Returns |
|-------|------------------|-----------------|
| Paginated products | `{ items: [], nextPageToken, totalCount }` | `{ products: [], total }` |
| Paginated attributes | `{ items: [] }` | `{ abstractAttributes: [], total }` |
| Timestamps | Unix seconds (number) | Unix milliseconds (i64) |
| Clone response | `{ newProductId, pathMapping, ... }` | `{ product: {...}, ... }` |

---

## Implementation Plan

### Phase 1: Backend - Implement Rule Evaluation (P0)

**Files to modify:**
- `/backend/crates/api/src/rest/evaluation.rs`

**Work:**
1. Implement `evaluate()` endpoint using `RuleExecutor` from rule-engine crate
2. Implement `batch_evaluate()` for multiple input sets
3. Wire up the JSON Logic evaluator with proper context building

```rust
// Pseudocode for evaluate endpoint
async fn evaluate(store, req) -> Result<EvaluateResponse> {
    let rules = store.get_rules_for_product(&req.product_id);
    let executor = RuleExecutor::new();
    let context = ExecutionContext::from_input_data(req.input_data);
    let result = executor.execute(&rules, &mut context)?;
    Ok(EvaluateResponse::from(result))
}
```

**Estimated scope**: ~200 lines

---

### Phase 2: Frontend - Fix API URL Structure

**Files to modify:**
- `/frontend/src/services/api.ts`

**Work:**
1. Fix `abstractAttributeApi.create()` to include productId in URL
2. Fix `attributeApi.create()` to include productId in URL
3. Fix `ruleApi.create()` to include productId in URL
4. Fix `functionalityApi.create()` to include productId in URL
5. Update all response type expectations to match backend

**Changes:**

```typescript
// Before
async create(data: Partial<AbstractAttribute>): Promise<AbstractAttribute> {
  return fetchJson('/api/abstract-attributes', { method: 'POST', body: JSON.stringify(data) });
}

// After
async create(productId: string, data: CreateAbstractAttributeRequest): Promise<AbstractAttribute> {
  return fetchJson(`/api/products/${productId}/abstract-attributes`, {
    method: 'POST',
    body: JSON.stringify(data)
  });
}
```

**Estimated scope**: ~150 lines of changes

---

### Phase 3: Backend - Fix Response Structures

**Files to modify:**
- `/backend/crates/api/src/rest/types.rs`
- `/backend/crates/api/src/rest/products.rs`
- `/backend/crates/api/src/rest/attributes.rs`
- `/backend/crates/api/src/rest/rules.rs`

**Work:**
1. Standardize all list endpoints to return `PaginatedResponse<T>` with `items` field
2. Fix timestamp serialization (ensure consistent format)
3. Update `CloneProductResponse` to match frontend expectations

**Example fix:**
```rust
// Before
pub struct ListProductsResponse {
    pub products: Vec<ProductResponse>,
    pub total: usize,
}

// After - use generic PaginatedResponse
pub type ListProductsResponse = PaginatedResponse<ProductResponse>;
// Where PaginatedResponse has { items, nextPageToken, totalCount }
```

**Estimated scope**: ~100 lines

---

### Phase 4: Backend - Add Missing Filter Endpoints

**Files to modify:**
- `/backend/crates/api/src/rest/attributes.rs`
- `/backend/crates/api/src/rest/functionalities.rs`

**Work:**
1. Add `GET /api/products/{id}/abstract-attributes/by-component/{type}`
2. Add `GET /api/products/{id}/abstract-attributes/by-tag/{tag}`
3. Add `GET /api/products/{id}/attributes/by-tag/{tag}`

**Example:**
```rust
async fn get_abstract_attrs_by_component(
    State(store): State<SharedStore>,
    Path((product_id, component_type)): Path<(String, String)>,
) -> ApiResult<Json<ListAbstractAttributesResponse>> {
    let store = store.read().await;
    let attrs = store.get_abstract_attrs_by_component(&product_id, &component_type, None);
    Ok(Json(ListAbstractAttributesResponse {
        items: attrs.into_iter().map(|a| a.into()).collect(),
        next_page_token: String::new(),
        total_count: attrs.len() as i32,
    }))
}
```

**Estimated scope**: ~150 lines

---

### Phase 5: Backend - Add Impact Analysis Endpoint

**Files to modify:**
- `/backend/crates/api/src/rest/evaluation.rs` (or new `impact.rs`)
- `/backend/crates/api/src/rest/types.rs`
- `/backend/crates/api/src/rest/mod.rs`

**Work:**
1. Create `ImpactAnalysisResponse` type
2. Implement graph traversal for upstream/downstream dependencies
3. Check for immutable attribute impacts

**Endpoint:**
```
POST /api/products/{product_id}/impact-analysis
Body: { "targetPath": "product:abstract-path:component:attr" }
Response: {
  "directDependencies": [...],
  "transitiveDependencies": [...],
  "affectedRules": [...],
  "affectedFunctionalities": [...],
  "hasImmutableDependents": bool,
  "immutablePaths": [...]
}
```

**Estimated scope**: ~200 lines

---

### Phase 6: Backend - Add Functionality Workflow Endpoints

**Files to modify:**
- `/backend/crates/api/src/rest/functionalities.rs`

**Work:**
1. Add `POST /api/products/{id}/functionalities/{name}/submit`
2. Add `POST /api/products/{id}/functionalities/{name}/approve`
3. Add approval state tracking

**Estimated scope**: ~80 lines

---

### Phase 7: Frontend - Update Store Actions

**Files to modify:**
- `/frontend/src/store/index.ts`

**Work:**
1. Update ProductStore actions to use corrected API client
2. Add proper error handling for API failures
3. Remove mock data fallback paths (or make configurable)

**Estimated scope**: ~100 lines

---

### Phase 8: Integration Testing

**Work:**
1. Create integration test script
2. Test CRUD operations for all entities
3. Test rule evaluation with sample data
4. Test DAG visualization with real data
5. Test simulation panel end-to-end

---

## Type Alignment Reference

### Frontend → Backend Type Mapping

| Frontend Type | Backend Type | Notes |
|--------------|--------------|-------|
| `Product` | `ProductResponse` | Match |
| `AbstractAttribute` | `AbstractAttributeResponse` | Check field names |
| `Attribute` | `AttributeResponse` | Check field names |
| `Rule` | `RuleResponse` | Check `compiledExpression` vs `expressionJson` |
| `ProductFunctionality` | `FunctionalityResponse` | Match |
| `DataType` | `DatatypeResponse` | Match |
| `TemplateEnumeration` | `EnumerationResponse` | Match |
| `AttributeValue` | `AttributeValueJson` | Tagged union structure |
| `EvaluateRequest` | `EvaluateRequest` | Need to implement |
| `EvaluateResponse` | `EvaluateResponse` | Need to implement |

### Key Type Differences to Fix

```typescript
// Frontend AttributeValue (tagged union)
type AttributeValue =
  | { type: 'null' }
  | { type: 'bool'; value: boolean }
  | { type: 'int'; value: number }
  // ...

// Backend AttributeValueJson (serde tagged enum)
#[serde(tag = "type", content = "value")]
pub enum AttributeValueJson {
    Null,
    Bool(bool),
    Int(i64),
    // ...
}
```

These should serialize compatibly, but verify with tests.

---

## Priority Order

| Priority | Phase | Description | Effort |
|----------|-------|-------------|--------|
| P0 | Phase 1 | Implement Rule Evaluation | 2-3 days |
| P0 | Phase 2 | Fix Frontend API URLs | 1 day |
| P1 | Phase 3 | Fix Response Structures | 1 day |
| P1 | Phase 4 | Add Filter Endpoints | 1 day |
| P1 | Phase 5 | Add Impact Analysis | 1-2 days |
| P2 | Phase 6 | Functionality Workflows | 0.5 day |
| P2 | Phase 7 | Store Updates | 0.5 day |
| P3 | Phase 8 | Integration Testing | 1 day |

**Total estimated effort**: 8-10 days

---

## Risk Mitigation

1. **Type mismatches causing runtime errors**
   - Add runtime validation on frontend
   - Add comprehensive logging
   - Create shared type definitions (consider OpenAPI spec)

2. **Breaking existing mock functionality**
   - Keep mock mode available via env var
   - Add feature flag for gradual rollout

3. **Performance issues with large rule sets**
   - Add pagination to all list endpoints
   - Implement caching in evaluation endpoint

---

## Success Criteria

1. Frontend can connect to backend with `VITE_USE_MOCK=false`
2. All CRUD operations work for all entity types
3. Rule simulation panel evaluates rules via backend
4. DAG visualization shows real dependency data
5. Product lifecycle workflows (submit/approve/reject) work
6. No console errors or TypeScript type errors
7. All existing frontend features work with real data
