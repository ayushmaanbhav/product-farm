# Dgraph Local Setup Guide

## Summary
Dgraph has been successfully installed and is running locally on your system.

## Installation Details
- **Version**: Dgraph v25.0.0
- **Download URL**: https://github.com/dgraph-io/dgraph/releases/download/v25.0.0/dgraph-linux-amd64.tar.gz
- **Binary Location**: `/home/vscode/product-farm/dgraph`

## Running Services

### Dgraph Zero (Management Server)
- **Port**: 5080
- **Command**: `./dgraph zero --my=localhost:5080 --replicas=1 --raft="idx=1"`
- **Status**: Running ✅

### Dgraph Alpha (Database Server)
- **Port**: 7080 (internal), 8080 (HTTP), 9080 (gRPC)
- **Command**: `./dgraph alpha --my=localhost:7080 --zero=localhost:5080`
- **Status**: Running ✅

## API Endpoints

### Health Check
```bash
curl -s localhost:8080/health
```

### Query Endpoint
```bash
curl -X POST localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "{ your_query_here }"}'
```

### Mutation Endpoint
```bash
curl -X POST localhost:8080/mutate \
  -H "Content-Type: application/json" \
  -d '{"set": [{"name": "Alice", "age": 30}]}'
```

## Test Data
Two test entities have been created:
- Entity 1 (UID: 0x1)
- Entity 2 (UID: 0x2)

## Verification
The setup has been verified with:
- Health check endpoint ✅
- Data mutation ✅
- Data query ✅

## Stopping Dgraph
To stop the running Dgraph instances:
```bash
pkill -f "dgraph zero"
pkill -f "dgraph alpha"
```

## Starting Dgraph Again
1. Start Zero server: `./dgraph zero --my=localhost:5080 --replicas=1 --raft="idx=1"`
2. Start Alpha server: `./dgraph alpha --my=localhost:7080 --zero=localhost:5080`

## Notes
- Dgraph is running in standalone mode with one Zero and one Alpha instance
- Data is stored in the default `p` directory for postings
- The setup is ready for local development and testing