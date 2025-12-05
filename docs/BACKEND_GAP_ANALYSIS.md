# Product-FARM Backend Gap Analysis

## Executive Summary

This document compares the **legacy Kotlin/Java implementation** with the **current Rust backend** to identify missing functionality, data model gaps, and areas requiring implementation.

**Overall Assessment:** The Rust backend has a solid foundation for the **rule engine core** but is missing approximately **60-70%** of the full product management functionality from the legacy system.

---

## 1. DATA MODEL COMPARISON

### 1.1 Core Entities

| Entity | Legacy | Rust | Gap |
|--------|--------|------|-----|
| **Product** | ✅ Full (id, name, status, templateType, effectiveFrom, expiryAt, parentProductId, description, version) | ⚠️ Partial (missing name field) | Missing: `name` field |
| **AbstractAttribute** | ✅ Full (abstractPath, componentType, componentId, datatype, enumeration, constraintRule, immutable, displayNames[], tags[], relatedAttributes[]) | ⚠️ Partial | Missing: `constraintRule` relationship, full `relatedAttributes` handling |
| **Attribute** | ✅ Full (path, abstractPath, type, value, rule, displayNames[]) | ⚠️ Partial | Missing: `JUST_DEFINITION` type handling |
| **Rule** | ✅ Full (id, type, inputAttributes[], outputAttributes[], displayExpression, displayExpressionVersion, compiledExpression, description) | ⚠️ Partial | Missing: `displayExpressionVersion` |
| **ProductFunctionality** | ✅ Full (id, name, productId, immutable, status, requiredAttributes[]) | ❌ Missing | **NOT IMPLEMENTED** |
| **ProductApproval** | ✅ Full (productId, approvedBy, discontinuedProductId, changeDescription) | ❌ Missing | **NOT IMPLEMENTED** |
| **Datatype** | ✅ Full (name, type, description) | ✅ Full | OK |
| **ProductTemplateEnumeration** | ✅ Full (id, name, productTemplateType, values[], description) | ⚠️ Partial (as EnumDefinition) | Missing: `id` field, productTemplateType handling |

### 1.2 Relationship Tables

| Relationship | Legacy | Rust | Gap |
|--------------|--------|------|-----|
| **AttributeDisplayName** | ✅ (productId, displayName, abstractPath/path, displayNameFormat, order) | ❌ Missing | **NOT IMPLEMENTED** - Critical for UI |
| **AbstractAttributeTag** | ✅ (abstractPath, tag, productId, order) | ⚠️ Partial (tags as Vec) | Missing: ordering, separate table |
| **AbstractAttributeRelatedAttribute** | ✅ (abstractPath, referenceAbstractPath, relationship, order) | ❌ Missing | **NOT IMPLEMENTED** |
| **FunctionalityRequiredAttribute** | ✅ (functionalityId, abstractPath, description, order) | ❌ Missing | **NOT IMPLEMENTED** |
| **RuleInputAttribute** | ✅ (ruleId, path, order) | ⚠️ Partial (as Vec) | Missing: ordering as separate entity |
| **RuleOutputAttribute** | ✅ (ruleId, path, order) | ⚠️ Partial (as Vec) | Missing: ordering as separate entity |

### 1.3 Enums Comparison

| Enum | Legacy Values | Rust Values | Match |
|------|--------------|-------------|-------|
| **ProductStatus** | DRAFT, PENDING_APPROVAL, ACTIVE, DISCONTINUED | Draft, PendingApproval, Active, Discontinued | ✅ Match (case differs) |
| **ProductFunctionalityStatus** | DRAFT, PENDING_APPROVAL, ACTIVE | Draft, PendingApproval, Active | ✅ Match |
| **AttributeValueType** | FIXED_VALUE, RULE_DRIVEN, JUST_DEFINITION | Static, Dynamic | ❌ **MISMATCH** - Missing JUST_DEFINITION |
| **DatatypeType** | OBJECT, ARRAY, INT, NUMBER, BOOLEAN, STRING | String, Int, Float, Decimal, Bool, Datetime, Enum, Array, Object, AttributeReference, Identifier | ⚠️ Extended in Rust |
| **DisplayNameFormat** | SYSTEM, ORIGINAL, HUMAN | ❌ Missing | **NOT IMPLEMENTED** |
| **ProductTemplateType** | INSURANCE (extensible) | Dynamic string | ✅ OK (more flexible) |
| **AttributeRelationshipType** | enumeration, key-enumeration, value-enumeration | ❌ Missing | **NOT IMPLEMENTED** |

---

## 2. API ENDPOINT COMPARISON

### 2.1 Product APIs

| Endpoint | Legacy | Rust | Gap |
|----------|--------|------|-----|
| `PUT /product` | ✅ CreateProductRequest | ⚠️ gRPC only | Missing: REST endpoint |
| `GET /product/{id}` | ✅ GetProductResponse | ⚠️ gRPC only | Missing: REST endpoint |
| `PUT /product/{id}/clone` | ✅ CloneProductRequest | ❌ Missing | **NOT IMPLEMENTED** |
| `POST /product/{id}/submit` | ✅ SubmitProductResponse | ❌ Missing | **NOT IMPLEMENTED** |
| `POST /product/{id}/approve` | ✅ ProductApprovalRequest/Response | ❌ Missing | **NOT IMPLEMENTED** |

### 2.2 Attribute APIs

| Endpoint | Legacy | Rust | Gap |
|----------|--------|------|-----|
| `PUT /product/{id}/attribute` | ✅ CreateAttributeRequest | ❌ Missing | **NOT IMPLEMENTED** |
| `GET /product/{id}/attribute/{displayName}` | ✅ GetAttributeResponse | ❌ Missing | **NOT IMPLEMENTED** |
| `GET /product/{id}/functionality/{name}/attribute` | ✅ GetFunctionalityAttributeListResponse | ❌ Missing | **NOT IMPLEMENTED** |
| `GET /product/{id}/attributeByTag/{tag}` | ✅ GetAttributeListByTagResponse | ❌ Missing | **NOT IMPLEMENTED** |

### 2.3 AbstractAttribute APIs

| Endpoint | Legacy | Rust | Gap |
|----------|--------|------|-----|
| `PUT /product/{id}/abstractAttribute` | ✅ CreateAbstractAttributeRequest | ❌ Missing | **NOT IMPLEMENTED** |
| `GET /product/{id}/abstractAttribute/{displayName}` | ✅ GetAbstractAttributeResponse | ❌ Missing | **NOT IMPLEMENTED** |

### 2.4 ProductFunctionality APIs

| Endpoint | Legacy | Rust | Gap |
|----------|--------|------|-----|
| `PUT /product/{id}/functionality` | ✅ CreateProductFunctionalityRequest | ❌ Missing | **NOT IMPLEMENTED** |
| `GET /product/{id}/functionality/{name}` | ✅ GetProductFunctionalityResponse | ❌ Missing | **NOT IMPLEMENTED** |
| `POST /product/{id}/functionality/{name}/submit` | ✅ | ❌ Missing | **NOT IMPLEMENTED** |
| `POST /product/{id}/functionality/{name}/approve` | ✅ | ❌ Missing | **NOT IMPLEMENTED** |

### 2.5 Datatype APIs

| Endpoint | Legacy | Rust | Gap |
|----------|--------|------|-----|
| `PUT /datatype` | ✅ DatatypeDto | ❌ Missing | **NOT IMPLEMENTED** |
| `GET /datatype/{name}` | ✅ DatatypeDto | ❌ Missing | **NOT IMPLEMENTED** |

### 2.6 ProductTemplate APIs

| Endpoint | Legacy | Rust | Gap |
|----------|--------|------|-----|
| `PUT /productTemplate/{type}/enum` | ✅ ProductTemplateEnumerationDto | ❌ Missing | **NOT IMPLEMENTED** |
| `GET /productTemplate/{type}/enum/{name}` | ✅ ProductTemplateEnumerationDto | ❌ Missing | **NOT IMPLEMENTED** |

---

## 3. VALIDATION COMPARISON

### 3.1 Regex Patterns

| Pattern | Legacy | Rust | Gap |
|---------|--------|------|-----|
| PRODUCT_ID_REGEX | `[a-zA-Z]([_][a-zA-Z0-9]|[a-zA-Z0-9]){0,50}` | ❌ Missing | **NOT IMPLEMENTED** |
| COMPONENT_TYPE_REGEX | `[a-z]([-][a-z]|[a-z]){0,50}` | ❌ Missing | **NOT IMPLEMENTED** |
| COMPONENT_ID_REGEX | `[a-z]([-][a-z0-9]|[a-z0-9]){0,50}` | ❌ Missing | **NOT IMPLEMENTED** |
| ATTRIBUTE_NAME_REGEX | `[a-z](...){0,100}` | ❌ Missing | **NOT IMPLEMENTED** |
| PATH_REGEX | Complex pattern | ❌ Missing | **NOT IMPLEMENTED** |
| ABSTRACT_PATH_REGEX | Complex pattern | ❌ Missing | **NOT IMPLEMENTED** |
| DISPLAY_NAME_REGEX | `[a-z](...){0,200}` | ❌ Missing | **NOT IMPLEMENTED** |
| TAG_REGEX | `[a-z]([-][a-z]|[a-z]){0,50}` | ❌ Missing | **NOT IMPLEMENTED** |
| DATATYPE_REGEX | `[a-z]([-][a-z]|[a-z]){0,50}` | ❌ Missing | **NOT IMPLEMENTED** |
| DESCRIPTION_REGEX | `[a-zA-Z0-9,...]{0,200}` | ❌ Missing | **NOT IMPLEMENTED** |
| UUID_REGEX | `[a-f0-9]{32}` | ✅ Using uuid crate | OK |

### 3.2 Business Validations

| Validation | Legacy | Rust | Gap |
|------------|--------|------|-----|
| **Product Status Transitions** | ✅ Full state machine | ⚠️ Partial | Missing: full workflow enforcement |
| **Attribute Type Constraints** | ✅ JUST_DEFINITION/FIXED_VALUE/RULE_DRIVEN rules | ❌ Missing | **NOT IMPLEMENTED** |
| **Immutability Enforcement** | ✅ Product/Functionality/Attribute level | ❌ Missing | **NOT IMPLEMENTED** |
| **DAG Cycle Detection** | ✅ Full | ✅ Full | OK |
| **Duplicate Output Detection** | ✅ Full | ✅ Full | OK |
| **Type Consistency** | ✅ Datatype validation | ⚠️ Partial | Runtime only, no schema validation |
| **Enumeration Value Validation** | ✅ Full | ❌ Missing | **NOT IMPLEMENTED** |
| **effectiveFrom/expiryAt** | ✅ Full date validation | ❌ Missing | **NOT IMPLEMENTED** |
| **Constraint Expression Validation** | ✅ Full | ❌ Missing | **NOT IMPLEMENTED** |

---

## 4. SERVICE LAYER COMPARISON

### 4.1 Services

| Service | Legacy | Rust | Gap |
|---------|--------|------|-----|
| **ProductService** | ✅ Full (create, get, submit, approve, clone) | ⚠️ Partial (CRUD only) | Missing: submit, approve, clone |
| **AttributeService** | ✅ Full (create, get, getByFunctionality, getByTag, clone) | ❌ Missing | **NOT IMPLEMENTED** |
| **AbstractAttributeService** | ✅ Full (create, get, clone) | ❌ Missing | **NOT IMPLEMENTED** |
| **DatatypeService** | ✅ Full (create, get) | ❌ Missing | **NOT IMPLEMENTED** |
| **ProductFunctionalityService** | ✅ Full (create, get, submit, approve) | ❌ Missing | **NOT IMPLEMENTED** |
| **ProductApprovalService** | ✅ Full | ❌ Missing | **NOT IMPLEMENTED** |
| **ProductTemplateService** | ✅ Full (enum CRUD) | ❌ Missing | **NOT IMPLEMENTED** |
| **CloneProductService** | ✅ Full (deep copy) | ❌ Missing | **NOT IMPLEMENTED** |

### 4.2 Transformers

| Transformer | Legacy | Rust | Gap |
|-------------|--------|------|-----|
| CreateProductTransformer | ✅ | ❌ | Missing DTO layer |
| GetProductTransformer | ✅ | ❌ | Missing DTO layer |
| CreateAttributeTransformer | ✅ | ❌ | Missing DTO layer |
| GetAttributeTransformer | ✅ | ❌ | Missing DTO layer |
| CreateRuleTransformer | ✅ | ❌ | Missing DTO layer |
| GetRuleTransformer | ✅ | ❌ | Missing DTO layer |
| ... (10+ more) | ✅ | ❌ | Missing DTO layer |

---

## 5. NAMING CONVENTION DIFFERENCES

### 5.1 Field Naming

| Concept | Legacy (Kotlin) | Rust | Issue |
|---------|----------------|------|-------|
| Attribute type | `FIXED_VALUE`, `RULE_DRIVEN`, `JUST_DEFINITION` | `Static`, `Dynamic` | **SEMANTIC MISMATCH** - Missing JUST_DEFINITION |
| Product status | `PENDING_APPROVAL` | `PendingApproval` | Case difference (minor) |
| Path separator | `:` (COMPONENT_SEPARATOR) | `.` | **INCONSISTENT** |
| Abstract path prefix | `abstract-path` | None | **Missing concept** |
| Rule expression | `compiledExpression` | `expression` | Naming difference |
| Display expression version | `displayExpressionVersion` | Missing | **NOT IMPLEMENTED** |

### 5.2 Path Format

**Legacy Format:**
```
Product Path: {productId}:{componentType}:{componentId}:{attributeName}
Abstract Path: {productId}:abstract-path:{componentType}[:{componentId}]:{attributeName}

Example:
- health-plan-2024:coverage:basic:deductible
- health-plan-2024:abstract-path:coverage:deductible
```

**Rust Format:**
```
Concrete Path: {component_type}.{component_id}.{path.name}
Abstract Path: {component_type}(.{component_id}).{path.name}

Example:
- coverage.basic.deductible
- coverage.deductible
```

**Issue:** Path format is incompatible. Rust omits `productId` from path, uses `.` instead of `:`.

---

## 6. DATABASE SCHEMA COMPARISON

### 6.1 Legacy PostgreSQL Tables

| Table | Columns | Rust Equivalent | Gap |
|-------|---------|-----------------|-----|
| `product` | 10 columns | DGraph Product node | Missing: `name` |
| `abstract_attribute` | 12 columns + index | DGraph AbstractAttribute | Missing: full index |
| `attribute` | 8 columns + index | DGraph Attribute | OK |
| `rule` | 8 columns | DGraph Rule | OK |
| `datatype` | 3 columns | DGraph DataType | OK |
| `product_functionality` | 7 columns + unique index | ❌ Missing | **NOT IMPLEMENTED** |
| `product_functionality_required_attribute` | 5 columns | ❌ Missing | **NOT IMPLEMENTED** |
| `product_approval` | 5 columns | ❌ Missing | **NOT IMPLEMENTED** |
| `product_template_enumeration` | 6 columns + unique index | ❌ Missing | **NOT IMPLEMENTED** |
| `product_display_name` | 7 columns + 2 unique indexes + constraint | ❌ Missing | **NOT IMPLEMENTED** |
| `abstract_attribute_tag` | 5 columns + index | Embedded in node | Different approach |
| `abstract_attribute_related_attribute` | 4 columns | ❌ Missing | **NOT IMPLEMENTED** |
| `rule_input_attribute` | 4 columns | Embedded in node | Different approach |
| `rule_output_attribute` | 4 columns | Embedded in node | Different approach |

---

## 7. RULE ENGINE COMPARISON

### 7.1 Capabilities

| Feature | Legacy (Kotlin) | Rust | Status |
|---------|----------------|------|--------|
| JSON Logic Parsing | ✅ | ✅ | OK |
| JSON Logic Evaluation | ✅ | ✅ | OK |
| DAG Construction | ✅ | ✅ | OK |
| Topological Sort | ✅ | ✅ | OK |
| Cycle Detection | ✅ | ✅ | OK |
| Caching | ✅ (CacheEnabledRuleEngine) | ✅ (CachedExpression) | OK |
| Tiered Compilation | ❌ | ✅ (AST → Bytecode) | **RUST BETTER** |
| Bytecode Persistence | ❌ | ✅ | **RUST BETTER** |
| Query by Tag | ✅ (QueryType.ATTRIBUTE_TAG) | ❌ | Missing |
| Query by Rule Type | ✅ (QueryType.RULE_TYPE) | ❌ | Missing |
| Streaming Evaluation | ❌ | ✅ (gRPC streaming) | **RUST BETTER** |

### 7.2 Performance

| Metric | Legacy | Rust | Winner |
|--------|--------|------|--------|
| Simple rule | ~500ns-5µs (interpreted) | ~300ns (bytecode) | **Rust** |
| Complex chain | Unknown | ~10-100µs | Rust has benchmarks |
| Cold start | ~seconds (JVM) | <1s | **Rust** |
| Memory | ~100MB+ (JVM) | <100MB | **Rust** |

---

## 8. CRITICAL GAPS SUMMARY

### 8.1 Must Fix (Breaking)

1. **AttributeValueType.JUST_DEFINITION** - Attributes that are definition-only (no value, no rule) cannot be represented
2. **Path Format Incompatibility** - `:` vs `.` separator, missing productId in paths
3. **DisplayName System** - Multi-format display names not implemented
4. **Product Lifecycle APIs** - Submit/Approve workflow missing
5. **ProductFunctionality** - Entire entity and workflow missing

### 8.2 Should Fix (Functional)

1. **Validation Regex Patterns** - No input validation
2. **Immutability Enforcement** - No protection for approved products
3. **Enumeration Management** - ProductTemplateEnumeration APIs missing
4. **Related Attributes** - AbstractAttributeRelatedAttribute missing
5. **Clone Operations** - Product/attribute cloning missing
6. **REST API** - Only gRPC available, no HTTP/REST

### 8.3 Nice to Have

1. **Query by Tag/Functionality** - Advanced querying
2. **Display Expression Versioning** - Version tracking for display formats
3. **Full Transformer Layer** - DTO separation
4. **Optimistic Locking** - Version field enforcement

---

## 9. RECOMMENDED IMPLEMENTATION ORDER

### Phase 1: Data Model Alignment (Critical)
1. Add `name` field to Product
2. Implement `AttributeValueType.JustDefinition`
3. Fix path format to match legacy (use `:` separator, include productId)
4. Add `displayExpressionVersion` to Rule
5. Implement DisplayName with formats (SYSTEM, ORIGINAL, HUMAN)

### Phase 2: Core Entities (High Priority)
1. Implement ProductFunctionality entity
2. Implement ProductApproval entity
3. Implement ProductTemplateEnumeration entity
4. Implement FunctionalityRequiredAttribute relationship
5. Implement AbstractAttributeRelatedAttribute relationship

### Phase 3: API Layer (High Priority)
1. Add REST API alongside gRPC
2. Implement Product lifecycle APIs (submit, approve)
3. Implement Functionality lifecycle APIs
4. Implement Attribute CRUD APIs
5. Implement AbstractAttribute CRUD APIs
6. Implement Datatype APIs
7. Implement Enumeration APIs

### Phase 4: Validation Layer (Medium Priority)
1. Add regex validation for all ID/path patterns
2. Implement status transition validation
3. Implement immutability enforcement
4. Implement enumeration value validation
5. Implement date range validation

### Phase 5: Advanced Features (Lower Priority)
1. Clone operations
2. Query by tag/functionality
3. Full transformer/DTO layer
4. Optimistic locking enforcement

---

## 10. CONCLUSION

The current Rust backend provides an excellent **high-performance rule execution engine** with features that exceed the legacy system (tiered compilation, bytecode persistence, streaming). However, it is missing significant **product management** functionality:

- **~70% of API endpoints** are not implemented
- **3 core entities** (ProductFunctionality, ProductApproval, ProductTemplateEnumeration) are missing
- **Critical validations** and **lifecycle workflows** are not implemented
- **Path format** is incompatible with legacy

The frontend UI built earlier is extremely premature because the backend APIs it would need don't exist yet.

**Recommendation:** Focus on completing the backend data model and API layer before building any UI.
