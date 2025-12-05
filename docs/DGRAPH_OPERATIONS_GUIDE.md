# Dgraph Operations Guide for AI Agents

## 📋 Dgraph Version Information

**Version**: Dgraph v25.0.0
**Codename**: dgraph
**SHA-256**: 091ce46ba667c1d88b7f5a3fab6600aec7d9da1ea0a1f97b46e9d5bf4d4a69a2
**Binary Location**: `/home/vscode/product-farm/dgraph`

## 🚀 Starting Dgraph

### Start Dgraph Zero (Management Server)
```bash
./dgraph zero --my=localhost:5080 --replicas=1 --raft="idx=1"
```

**Parameters**:
- `--my=localhost:5080`: Sets the Zero server address
- `--replicas=1`: Configures 1 replica for the cluster
- `--raft="idx=1"`: Sets Raft ID to 1

### Start Dgraph Alpha (Database Server)
```bash
./dgraph alpha --my=localhost:7080 --zero=localhost:5080
```

**Parameters**:
- `--my=localhost:7080`: Sets the Alpha server address
- `--zero=localhost:5080`: Connects to Zero server at this address

## 🔌 Port Configuration

### Dgraph Zero Ports
- **Raft/Internal Communication**: 5080

### Dgraph Alpha Ports
- **Internal Communication**: 7080
- **HTTP API**: 8080
- **gRPC API**: 9080

## 📊 Process Management

### Check Running Processes
```bash
ps aux | grep dgraph
```

### Kill Dgraph Zero
```bash
pkill -f "dgraph zero"
```

### Kill Dgraph Alpha
```bash
pkill -f "dgraph alpha"
```

### Kill All Dgraph Processes
```bash
pkill -f dgraph
```

## 🔗 Connection Information

### Health Check Endpoint
```bash
curl -s localhost:8080/health
```

**Expected Response**:
```json
[{"instance":"alpha","address":"localhost:7080","status":"healthy","group":"1","version":"v25.0.0","uptime":47,"lastEcho":1764753376,"ongoing":["opRollup"],"ee_features":["backup_restore","cdc"],"max_assigned":2}]
```

### API Endpoints

#### HTTP/JSON Endpoints

##### Query Endpoint
```bash
curl -X POST localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "{ your_query_here }"}'
```

##### Mutation Endpoint
```bash
curl -X POST localhost:8080/mutate \
  -H "Content-Type: application/json" \
  -d '{"set": [{"name": "Alice", "age": 30}]}'
```

##### GraphQL Endpoint
```bash
curl -X POST localhost:8080/admin \
  -H "Content-Type: application/json" \
  -d '{"query": "query { your_graphql_query }"}'
```

#### gRPC Endpoints

Dgraph provides gRPC connectivity on port **9080** for high-performance client connections.

##### gRPC Connection Details
- **Port**: 9080
- **Protocol**: gRPC with Protocol Buffers
- **Service**: `api.Dgraph` (main service)
- **Methods**: `Query`, `Mutate`, `Alter`, `Commit`, `CheckVersion`

##### gRPC Client Example (Go)
```go
package main

import (
	"context"
	"log"
	"time"

	"github.com/dgraph-io/dgo/v230"
	"github.com/dgraph-io/dgo/v230/protos/api"
	"google.golang.org/grpc"
)

func main() {
	// Create gRPC connection
	conn, err := grpc.Dial("localhost:9080", grpc.WithInsecure())
	if err != nil {
		log.Fatal(err)
	}
	defer conn.Close()

	// Create Dgraph client
	dgraphClient := dgo.NewDgraphClient(api.NewDgraphClient(conn))

	// Perform query
	query := `{
		all(func: has(name)) {
			uid
			name
			age
		}
	}`

	resp, err := dgraphClient.NewTxn().Query(context.Background(), query)
	if err != nil {
		log.Fatal(err)
	}

	log.Printf("Response: %s", resp.Json)
}
```

##### gRPC Client Example (Python)
```python
import grpc
import dgraph_v1_pb2 as pb
import dgraph_v1_pb2_grpc as pb_grpc

def main():
    # Create gRPC channel
    channel = grpc.insecure_channel('localhost:9080')
    stub = pb_grpc.DgraphStub(channel)

    # Create query
    query = """
    {
        all(func: has(name)) {
            uid
            name
            age
        }
    }
    """

    # Execute query
    request = pb.Request(
        query=query,
        start_ts=0,
        read_only=True
    )

    response = stub.Query(request)
    print("Response:", response.json)

if __name__ == '__main__':
    main()
```

### Protobuf Definitions

Dgraph uses Protocol Buffers for its gRPC API. The main protobuf definitions include:

#### Core Dgraph Protobuf Services
- **Dgraph Service**: Main database operations
- **Login Service**: Authentication operations
- **Backup Service**: Backup/restore operations

#### Key Message Types
- **Request**: Query/mutation requests
- **Response**: Query/mutation responses
- **Payload**: Data payloads
- **NQuad**: RDF-style data representation

### Product-FARM Specific gRPC

This project includes its own gRPC implementation for the Product-FARM system:

#### Protobuf File Location
```bash
engine-rs/crates/api/proto/product_farm.proto
```

#### Product-FARM Services
- **ProductFarmService**: Rule evaluation and management
- **ProductService**: Product CRUD operations
- **RuleService**: Rule CRUD operations

#### Example gRPC Client (Rust)
See the complete example in:
```bash
engine-rs/crates/api/examples/grpc_client.rs
```

This client demonstrates:
- Health checks
- Product creation
- Rule management
- Execution plan visualization
- Rule evaluation
- Data querying

## 📊 Process Management

### Check Running Processes
```bash
ps aux | grep dgraph
```

### Kill Dgraph Zero
```bash
pkill -f "dgraph zero"
```

### Kill Dgraph Alpha
```bash
pkill -f "dgraph alpha"
```

### Kill All Dgraph Processes
```bash
pkill -f dgraph
```

## 🧪 Sample Operations

### Create Test Data
```bash
curl -X POST localhost:8080/mutate \
  -H "Content-Type: application/json" \
  -d '{"set": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}'
```

### Query Test Data
```bash
curl -X POST localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "{ me(func: uid(0x1)) { uid name age } }"}'
```

## 📝 Process Identification

### Dgraph Zero Process Characteristics
- **Command**: Contains `dgraph zero`
- **Port**: Listens on 5080
- **Role**: Cluster management, Raft consensus

### Dgraph Alpha Process Characteristics
- **Command**: Contains `dgraph alpha`
- **Ports**: Listens on 7080 (internal), 8080 (HTTP), 9080 (gRPC)
- **Role**: Data storage, query processing

## 🔄 Startup Sequence

1. **Start Zero first**: `./dgraph zero --my=localhost:5080 --replicas=1 --raft="idx=1"`
2. **Wait 5-10 seconds** for Zero to initialize
3. **Start Alpha**: `./dgraph alpha --my=localhost:7080 --zero=localhost:5080`
4. **Verify health**: `curl -s localhost:8080/health`

## ⚠️ Troubleshooting

### Zero Not Starting
- Check port 5080 is available: `netstat -tuln | grep 5080`
- Verify no other Dgraph instances running: `pkill -f dgraph`

### Alpha Not Connecting to Zero
- Verify Zero is running: `curl -s localhost:5080/health`
- Check network connectivity: `telnet localhost 5080`

### Query Failures
- Ensure data was properly mutated
- Verify query syntax matches Dgraph query language
- Check for proper Content-Type headers

## 📚 Additional Commands

### Version Check
```bash
./dgraph version
```

### Help Information
```bash
./dgraph --help
./dgraph zero --help
./dgraph alpha --help
```

## 🎯 AI Agent Notes

**For AI Agents**: This setup uses Dgraph in standalone mode with one Zero and one Alpha instance. The configuration is optimized for local development and testing. For production environments, consider:
- Multiple replicas for fault tolerance
- Proper data persistence configuration
- Security settings (TLS, ACLs)
- Resource allocation tuning
## 🎯 AI Agent Notes

**For AI Agents**: This setup uses Dgraph in standalone mode with one Zero and one Alpha instance. The configuration is optimized for local development and testing.

### Connection Options Summary

| Protocol | Port | Service | Use Case |
|----------|------|---------|----------|
| HTTP/JSON | 8080 | Alpha HTTP API | RESTful queries, mutations, admin |
| gRPC | 9080 | Alpha gRPC API | High-performance client connections |
| Internal | 7080 | Alpha Internal | Cluster communication |
| Raft | 5080 | Zero Management | Cluster coordination |

### gRPC vs HTTP Comparison

**Use gRPC when**:
- High performance is required
- Low latency connections needed
- Streaming operations required
- Type-safe client generation available
- Long-lived connections beneficial

**Use HTTP when**:
- Simple debugging/testing needed
- Browser-based access required
- Quick curl commands sufficient
- No protobuf compilation needed

### Protocol Buffers Information

**Dgraph Protobuf Location**:
- Built into Dgraph binary (standard Dgraph protobufs)
- Custom protobufs: `engine-rs/crates/api/proto/product_farm.proto`

**Key Protobuf Services**:
- `api.Dgraph` - Main database operations
- `api.Login` - Authentication
- `api.Backup` - Backup/restore

### For Production Environments

Consider these additional configurations:
- **Multiple replicas** for fault tolerance
- **Proper data persistence** configuration
- **Security settings** (TLS, ACLs)
- **Resource allocation** tuning
- **gRPC load balancing** for client connections
- **Connection pooling** for high-volume clients

### gRPC Client Generation

To generate gRPC clients from protobufs:

```bash
# For Dgraph standard protobufs (if available)
protoc --go_out=. --go-grpc_out=. dgraph.proto

# For Product-FARM protobufs
cd engine-rs/crates/api
protoc --rust_out=. --rust-grpc_out=. proto/product_farm.proto
```

### Connection Testing

**Test gRPC Connection**:
```bash
# Using grpcurl (install with: go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest)
grpcurl -plaintext localhost:9080 list
```

**Test HTTP Connection**:
```bash
curl -s localhost:8080/health
```

**Test Data Access**:
```bash
# HTTP
curl -X POST localhost:8080/query -H "Content-Type: application/json" -d '{"query": "{ all(func: has(name)) { uid name } }"}'

# gRPC (using grpcurl)
grpcurl -plaintext -d '{"query": "{ all(func: has(name)) { uid name } }"}' localhost:9080 api.Dgraph/Query
```

### gRPC Performance Considerations

**Connection Management**:
- Reuse gRPC connections for multiple requests
- Implement connection pooling for high-volume scenarios
- Use keep-alive settings for long-running applications

**Error Handling**:
- Implement proper retry logic for transient failures
- Handle gRPC status codes appropriately
- Implement circuit breakers for production systems

**Security**:
- For production: Use TLS instead of plaintext
- Implement proper authentication
- Consider mTLS for internal service communication

### Debugging gRPC Connections

**Common Issues**:
- **Connection refused**: Verify Dgraph Alpha is running on port 9080
- **Protocol errors**: Ensure proper protobuf message formatting
- **Timeout issues**: Check gRPC client timeout settings
- **SSL errors**: Use `-plaintext` flag for local development

**Debugging Tools**:
```bash
# List available gRPC services
grpcurl -plaintext localhost:9080 list

# Describe a specific service
grpcurl -plaintext localhost:9080 describe api.Dgraph

# Test with verbose output
grpcurl -v -plaintext localhost:9080 list
```

### gRPC vs HTTP Performance Comparison

| Metric | gRPC | HTTP/JSON |
|--------|------|-----------|
| Latency | ⚡ Low | 🐢 Higher |
| Throughput | 🚀 High | 🏎️ Medium |
| Payload Size | 📦 Small | 📦📦 Larger |
| Connection Overhead | 🔗 Low | 🔗🔗 Higher |
| Streaming Support | ✅ Yes | ❌ No |
| Browser Support | ❌ No | ✅ Yes |
| Development Speed | 🐢 Slower (protobuf) | 🏃 Faster |

### Best Practices for AI Agents

1. **Use gRPC for programmatic access** when performance matters
2. **Use HTTP for debugging** and quick testing
3. **Cache connections** for repeated operations
4. **Handle errors gracefully** with proper retry logic
5. **Monitor connection health** periodically
6. **Use appropriate timeouts** based on operation type
7. **Implement circuit breakers** for production systems