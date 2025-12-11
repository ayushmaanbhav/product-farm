---
layout: default
title: API Reference
---

# API Reference

Product-FARM provides both REST and gRPC APIs for integration.

## Base URLs

| Protocol | Port | Base URL |
|----------|------|----------|
| REST | 8081 | `http://localhost:8081/api` |
| gRPC | 50051 | `localhost:50051` |

---

## REST API

### Products

#### List Products

```http
GET /api/products
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `status` | string | Filter by status (DRAFT, PENDING_APPROVAL, ACTIVE, DISCONTINUED) |
| `template_type` | string | Filter by template type |

**Response:**
```json
{
  "products": [
    {
      "id": "insurance-v1",
      "name": "Insurance Premium Calculator",
      "status": "ACTIVE",
      "template_type": "INSURANCE",
      "description": "Calculate insurance premiums",
      "effective_from": "2024-01-01T00:00:00Z",
      "expiry_at": null,
      "version": 1
    }
  ]
}
```

#### Create Product

```http
POST /api/products
Content-Type: application/json
```

**Request Body:**
```json
{
  "name": "my-product",
  "template_type": "INSURANCE",
  "description": "Product description",
  "effective_from": "2024-01-01T00:00:00Z",
  "expiry_at": "2025-12-31T23:59:59Z"
}
```

**Response:** `201 Created`
```json
{
  "id": "my-product",
  "name": "my-product",
  "status": "DRAFT",
  "template_type": "INSURANCE",
  "description": "Product description",
  "effective_from": "2024-01-01T00:00:00Z",
  "expiry_at": "2025-12-31T23:59:59Z",
  "version": 1
}
```

#### Get Product

```http
GET /api/products/{product_id}
```

**Response:** `200 OK`
```json
{
  "id": "insurance-v1",
  "name": "Insurance Premium Calculator",
  "status": "ACTIVE",
  "template_type": "INSURANCE",
  "description": "Calculate insurance premiums",
  "effective_from": "2024-01-01T00:00:00Z",
  "expiry_at": null,
  "version": 1
}
```

#### Update Product

```http
PUT /api/products/{product_id}
Content-Type: application/json
```

**Request Body:**
```json
{
  "name": "updated-name",
  "description": "Updated description"
}
```

**Note:** Only DRAFT products can be updated.

#### Delete Product

```http
DELETE /api/products/{product_id}
```

**Response:** `204 No Content`

**Note:** Only DRAFT and DISCONTINUED products can be deleted.

#### Clone Product

```http
POST /api/products/{product_id}/clone
Content-Type: application/json
```

**Request Body:**
```json
{
  "new_product_id": "insurance-v2",
  "new_name": "Insurance Premium Calculator v2"
}
```

**Response:** `201 Created` - Returns the new cloned product in DRAFT status.

#### Product Lifecycle Operations

```http
# Submit for approval
POST /api/products/{product_id}/submit

# Approve product (changes status to ACTIVE)
POST /api/products/{product_id}/approve

# Reject product (returns to DRAFT)
POST /api/products/{product_id}/reject
Content-Type: application/json
{"reason": "Missing required attributes"}

# Discontinue product
POST /api/products/{product_id}/discontinue
```

---

### Rules

#### List Rules

```http
GET /api/products/{product_id}/rules
```

**Response:**
```json
{
  "rules": [
    {
      "id": "rule-001",
      "product_id": "insurance-v1",
      "rule_type": "CALCULATION",
      "display_expression": "base_premium = coverage × 0.02",
      "compiled_expression": "{\"*\": [{\"var\": \"coverage\"}, 0.02]}",
      "input_attributes": [
        {"attribute_path": "coverage", "order_index": 0}
      ],
      "output_attributes": [
        {"attribute_path": "base_premium", "order_index": 0}
      ],
      "order_index": 0,
      "enabled": true
    }
  ]
}
```

#### Create Rule

```http
POST /api/products/{product_id}/rules
Content-Type: application/json
```

**Request Body:**
```json
{
  "rule_type": "CALCULATION",
  "display_expression": "base_premium = coverage × 0.02",
  "expression": {
    "*": [{"var": "coverage"}, 0.02]
  },
  "input_attributes": ["coverage"],
  "output_attributes": ["base_premium"],
  "order_index": 0,
  "enabled": true
}
```

#### Get Rule

```http
GET /api/products/{product_id}/rules/{rule_id}
```

#### Update Rule

```http
PUT /api/products/{product_id}/rules/{rule_id}
Content-Type: application/json
```

#### Delete Rule

```http
DELETE /api/products/{product_id}/rules/{rule_id}
```

---

### Evaluation

#### Evaluate Rules

```http
POST /api/products/{product_id}/evaluate
Content-Type: application/json
```

**Request Body:**
```json
{
  "inputs": {
    "coverage": 250000,
    "customer_age": 65,
    "smoker": false
  }
}
```

**Response:**
```json
{
  "outputs": {
    "base_premium": 5000,
    "age_factor": 1.2,
    "final_premium": 6000
  },
  "rule_results": [
    {
      "rule_id": "rule-001",
      "outputs": {"base_premium": 5000},
      "execution_time_ns": 850
    },
    {
      "rule_id": "rule-002",
      "outputs": {"age_factor": 1.2},
      "execution_time_ns": 420
    },
    {
      "rule_id": "rule-003",
      "outputs": {"final_premium": 6000},
      "execution_time_ns": 380
    }
  ],
  "total_execution_time_ns": 2150,
  "execution_levels": 2
}
```

#### Batch Evaluate

```http
POST /api/products/{product_id}/batch-evaluate
Content-Type: application/json
```

**Request Body:**
```json
{
  "inputs": [
    {
      "id": "customer-001",
      "data": {
        "coverage": 250000,
        "customer_age": 35
      }
    },
    {
      "id": "customer-002",
      "data": {
        "coverage": 500000,
        "customer_age": 65
      }
    }
  ]
}
```

**Response:**
```json
{
  "results": [
    {
      "input_id": "customer-001",
      "outputs": {"final_premium": 5000},
      "success": true
    },
    {
      "input_id": "customer-002",
      "outputs": {"final_premium": 12000},
      "success": true
    }
  ],
  "total_execution_time_ns": 4500
}
```

#### Validate Rules

```http
POST /api/products/{product_id}/validate-rules
```

**Response:**
```json
{
  "valid": true,
  "errors": [],
  "warnings": [
    {
      "rule_id": "rule-004",
      "message": "Rule is disabled"
    }
  ]
}
```

#### Get Execution Plan

```http
GET /api/products/{product_id}/execution-plan
```

**Response:**
```json
{
  "levels": [
    {
      "level": 0,
      "rules": ["rule-001", "rule-002"]
    },
    {
      "level": 1,
      "rules": ["rule-003"]
    }
  ],
  "dependencies": [
    {"from": "rule-003", "to": "rule-001"},
    {"from": "rule-003", "to": "rule-002"}
  ],
  "dot_graph": "digraph { ... }",
  "mermaid_graph": "graph TD\n  ...",
  "ascii_graph": "..."
}
```

---

### Abstract Attributes

#### List Abstract Attributes

```http
GET /api/products/{product_id}/abstract-attributes
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `component_type` | string | Filter by component type |
| `tag` | string | Filter by tag |

#### Create Abstract Attribute

```http
POST /api/products/{product_id}/abstract-attributes
Content-Type: application/json
```

**Request Body:**
```json
{
  "component_type": "CUSTOMER",
  "component_id": "main",
  "attribute_name": "age",
  "datatype_id": "int",
  "display_names": [
    {"name": "customer_age", "format": "SYSTEM"},
    {"name": "Customer Age", "format": "HUMAN"}
  ],
  "tags": [
    {"name": "input", "order_index": 0},
    {"name": "demographics", "order_index": 1}
  ],
  "immutable": false
}
```

#### Get by Tag

```http
GET /api/products/{product_id}/abstract-attributes/by-tag/{tag_name}
```

---

### Datatypes

#### List Datatypes

```http
GET /api/datatypes
```

**Response:**
```json
{
  "datatypes": [
    {
      "id": "int",
      "primitive_type": "INT",
      "description": "Integer number",
      "constraints": null
    },
    {
      "id": "percentage",
      "primitive_type": "DECIMAL",
      "description": "Percentage value 0-100",
      "constraints": {
        "min": 0,
        "max": 100,
        "precision": 2
      }
    }
  ]
}
```

#### Create Datatype

```http
POST /api/datatypes
Content-Type: application/json
```

**Request Body:**
```json
{
  "id": "currency",
  "primitive_type": "DECIMAL",
  "description": "Currency amount with 2 decimal places",
  "constraints": {
    "min": 0,
    "precision": 2,
    "scale": 2
  }
}
```

#### Validate Value

```http
POST /api/datatypes/{datatype_id}/validate
Content-Type: application/json
```

**Request Body:**
```json
{
  "value": 150.50
}
```

**Response:**
```json
{
  "valid": true,
  "errors": []
}
```

---

### Enumerations

#### List Enumerations

```http
GET /api/enumerations
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `template_type` | string | Filter by template type |

#### Create Enumeration

```http
POST /api/enumerations
Content-Type: application/json
```

**Request Body:**
```json
{
  "name": "CoverageType",
  "template_type": "INSURANCE",
  "description": "Types of insurance coverage",
  "values": [
    {"value": "BASIC", "order_index": 0},
    {"value": "STANDARD", "order_index": 1},
    {"value": "COMPREHENSIVE", "order_index": 2}
  ]
}
```

#### Add Enumeration Value

```http
POST /api/enumerations/{enum_name}/values
Content-Type: application/json
```

**Request Body:**
```json
{
  "value": "PREMIUM",
  "order_index": 3
}
```

---

### Functionalities

#### List Functionalities

```http
GET /api/products/{product_id}/functionalities
```

#### Create Functionality

```http
POST /api/products/{product_id}/functionalities
Content-Type: application/json
```

**Request Body:**
```json
{
  "name": "PREMIUM_CALCULATION",
  "description": "Calculate insurance premiums",
  "required_abstract_attributes": [
    "coverage",
    "customer_age",
    "final_premium"
  ]
}
```

#### Evaluate Functionality

```http
POST /api/functionalities/{functionality_id}/evaluate
Content-Type: application/json
```

**Request Body:**
```json
{
  "inputs": {
    "coverage": 250000,
    "customer_age": 65
  }
}
```

---

## gRPC API

### Service Definitions

```protobuf
syntax = "proto3";
package product_farm;

// Main evaluation service
service ProductFarmService {
  // Evaluate rules for a product
  rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);

  // Evaluate multiple inputs in batch
  rpc BatchEvaluate(BatchEvaluateRequest) returns (BatchEvaluateResponse);

  // Stream evaluation (for real-time data)
  rpc EvaluateStream(stream EvaluateRequest) returns (stream EvaluateResponse);

  // Validate rules without executing
  rpc ValidateRules(ValidateRulesRequest) returns (ValidateRulesResponse);

  // Get execution plan and DAG visualization
  rpc GetExecutionPlan(GetExecutionPlanRequest) returns (ExecutionPlanResponse);

  // Health check
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}

// Product management service
service ProductService {
  rpc CreateProduct(CreateProductRequest) returns (Product);
  rpc GetProduct(GetProductRequest) returns (Product);
  rpc UpdateProduct(UpdateProductRequest) returns (Product);
  rpc DeleteProduct(DeleteProductRequest) returns (Empty);
  rpc ListProducts(ListProductsRequest) returns (ListProductsResponse);
  rpc CloneProduct(CloneProductRequest) returns (Product);
  rpc SubmitProduct(SubmitProductRequest) returns (Product);
  rpc ApproveProduct(ApproveProductRequest) returns (Product);
  rpc RejectProduct(RejectProductRequest) returns (Product);
}

// Rule management service
service RuleService {
  rpc CreateRule(CreateRuleRequest) returns (Rule);
  rpc GetRule(GetRuleRequest) returns (Rule);
  rpc UpdateRule(UpdateRuleRequest) returns (Rule);
  rpc DeleteRule(DeleteRuleRequest) returns (Empty);
  rpc ListRules(ListRulesRequest) returns (ListRulesResponse);
}
```

### Message Types

```protobuf
message EvaluateRequest {
  string product_id = 1;
  map<string, Value> input_data = 2;
}

message EvaluateResponse {
  map<string, Value> outputs = 1;
  repeated RuleResult rule_results = 2;
  uint64 total_execution_time_ns = 3;
  uint32 execution_levels = 4;
}

message Value {
  oneof value {
    bool bool_value = 1;
    int64 int_value = 2;
    double float_value = 3;
    string decimal_value = 4;
    string string_value = 5;
    ArrayValue array_value = 6;
    ObjectValue object_value = 7;
  }
}

message RuleResult {
  string rule_id = 1;
  map<string, Value> outputs = 2;
  uint64 execution_time_ns = 3;
  bool success = 4;
  string error = 5;
}
```

### Using grpcurl

```bash
# Health check
grpcurl -plaintext localhost:50051 product_farm.ProductFarmService/HealthCheck

# Create product
grpcurl -plaintext -d '{
  "name": "my-product",
  "template_type": "INSURANCE",
  "description": "Test product"
}' localhost:50051 product_farm.ProductService/CreateProduct

# Evaluate
grpcurl -plaintext -d '{
  "product_id": "my-product",
  "input_data": {
    "coverage": {"decimal_value": "250000"},
    "customer_age": {"int_value": 65}
  }
}' localhost:50051 product_farm.ProductFarmService/Evaluate

# Get execution plan
grpcurl -plaintext -d '{
  "product_id": "my-product"
}' localhost:50051 product_farm.ProductFarmService/GetExecutionPlan
```

---

## Error Responses

### HTTP Status Codes

| Status | Description |
|--------|-------------|
| `200 OK` | Request successful |
| `201 Created` | Resource created |
| `204 No Content` | Resource deleted |
| `400 Bad Request` | Invalid request body or parameters |
| `404 Not Found` | Resource not found |
| `409 Conflict` | Resource conflict (e.g., duplicate ID) |
| `422 Unprocessable Entity` | Validation error |
| `500 Internal Server Error` | Server error |

### Error Response Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Rule expression is invalid",
    "details": [
      {
        "field": "expression",
        "message": "Unknown operator 'unknown_op'"
      }
    ]
  }
}
```

### Common Error Codes

| Code | Description |
|------|-------------|
| `PRODUCT_NOT_FOUND` | Product with given ID does not exist |
| `RULE_NOT_FOUND` | Rule with given ID does not exist |
| `INVALID_STATUS_TRANSITION` | Cannot transition product to requested status |
| `PRODUCT_IMMUTABLE` | Cannot modify ACTIVE product |
| `CYCLIC_DEPENDENCY` | Rules form a cycle |
| `VALIDATION_ERROR` | Input validation failed |
| `EXPRESSION_ERROR` | JSON Logic expression error |

---

## Rate Limiting

Currently, no rate limiting is implemented. For production deployments, consider adding rate limiting at the load balancer level.

---

## Authentication

Authentication is not currently implemented. For production use, implement OAuth2/OIDC or API key authentication.

---

## SDK Examples

### Rust Client

```rust
use product_farm_client::ProductFarmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ProductFarmClient::connect("http://localhost:50051").await?;

    let result = client.evaluate(
        "my-product",
        serde_json::json!({
            "coverage": 250000,
            "customer_age": 65
        })
    ).await?;

    println!("Premium: {}", result.outputs["final_premium"]);
    Ok(())
}
```

### Python Client

```python
import grpc
import product_farm_pb2 as pb
import product_farm_pb2_grpc as pb_grpc

channel = grpc.insecure_channel('localhost:50051')
stub = pb_grpc.ProductFarmServiceStub(channel)

request = pb.EvaluateRequest(
    product_id="my-product",
    input_data={
        "coverage": pb.Value(decimal_value="250000"),
        "customer_age": pb.Value(int_value=65)
    }
)

response = stub.Evaluate(request)
print(f"Premium: {response.outputs['final_premium'].decimal_value}")
```

### JavaScript/TypeScript Client

```typescript
const response = await fetch('http://localhost:8081/api/products/my-product/evaluate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    inputs: {
      coverage: 250000,
      customer_age: 65
    }
  })
});

const result = await response.json();
console.log('Premium:', result.outputs.final_premium);
```
