# Dgraph gRPC Testing

This directory contains test scripts and examples for testing gRPC connectivity to Dgraph.

## 📋 Available Tests

### Python Test
```bash
python3 test_grpc_connection.py
```

**Features**:
- Basic gRPC connection testing
- Dgraph client testing (if modules available)
- Service discovery
- Error handling

### Go Test
```bash
go run test_grpc_connection.go
```

**Features**:
- gRPC connection testing
- Connection state verification
- Basic error handling

## 🔧 Requirements

### Python Requirements
```bash
pip install grpcio grpcio-tools
# For Dgraph client (optional)
pip install dgraph-io/dgo
```

### Go Requirements
```bash
go get google.golang.org/grpc
```

## 🚀 Running Tests

### Test gRPC Connection
```bash
cd grpc-testing
python3 test_grpc_connection.py
```

### Test with Go
```bash
cd grpc-testing
go run test_grpc_connection.go
```

## 📊 Expected Results

**Successful Connection**:
```
✅ gRPC channel established successfully
📊 Channel state: READY
🎉 gRPC connectivity test: PASS
```

**Connection Issues**:
```
❌ gRPC channel connection timed out
💥 Basic gRPC connectivity: FAIL
```

## ⚠️ Troubleshooting

### Common Issues

**Connection Refused**:
- Verify Dgraph Alpha is running: `ps aux | grep "dgraph alpha"`
- Check port 9080 is listening: `netstat -tuln | grep 9080`
- Verify no firewall blocking: `sudo ufw status`

**Module Import Errors**:
- Install required Python packages: `pip install grpcio`
- For Dgraph client: `pip install dgraph-io/dgo`

**Timeout Issues**:
- Check Dgraph server logs for errors
- Verify network connectivity: `telnet localhost 9080`
- Increase timeout values in test scripts

## 📚 Dgraph gRPC Information

### gRPC Port
- **Port**: 9080
- **Protocol**: gRPC with Protocol Buffers
- **Service**: `api.Dgraph`

### Key gRPC Methods
- `Query` - Execute GraphQL queries
- `Mutate` - Perform data mutations
- `Alter` - Schema modifications
- `Commit` - Transaction management
- `CheckVersion` - Version compatibility

### Protobuf Services
- **Dgraph Service**: Main database operations
- **Login Service**: Authentication
- **Backup Service**: Backup/restore operations

## 🔗 Related Documentation

- **Main Guide**: `../docs/DGRAPH_OPERATIONS_GUIDE.md`
- **Dgraph Official Docs**: https://dgraph.io/docs
- **gRPC Documentation**: https://grpc.io/docs

## 🎯 Test Coverage

| Test Type | Coverage | Status |
|-----------|----------|--------|
| Basic Connection | ✅ gRPC channel establishment | ✅ Implemented |
| Service Discovery | ✅ Service listing | ✅ Implemented |
| Dgraph Client | ✅ Query execution | ✅ Implemented |
| Error Handling | ✅ Timeout handling | ✅ Implemented |
| Performance | ❌ Load testing | ❌ Not implemented |

## 💡 Usage Tips

1. **Start Dgraph first**: Ensure both Zero and Alpha are running
2. **Check ports**: Verify 9080 is accessible
3. **Use insecure for local**: `-plaintext` flag for local development
4. **Enable TLS for production**: Use proper certificates
5. **Monitor connections**: Check for leaks in long-running apps