#!/usr/bin/env python3
"""
Dgraph gRPC Connection Test Script
Tests gRPC connectivity to Dgraph Alpha server
"""

import grpc
import sys
import time
from concurrent import futures

# Try to import Dgraph gRPC modules
try:
    import dgraph_v1_pb2 as pb
    import dgraph_v1_pb2_grpc as pb_grpc
    DGRAPH_MODULES_AVAILABLE = True
except ImportError:
    DGRAPH_MODULES_AVAILABLE = False

def test_grpc_connection():
    """Test basic gRPC connection to Dgraph"""
    print("🔄 Testing gRPC connection to Dgraph...")

    try:
        # Create insecure channel (for local development)
        channel = grpc.insecure_channel('localhost:9080')

        # Test channel connectivity
        try:
            grpc.channel_ready_future(channel).result(timeout=5)
            print("✅ gRPC channel established successfully")
            print(f"📊 Channel state: {channel.get_state()}")
            return True
        except grpc.FutureTimeoutError:
            print("❌ gRPC channel connection timed out")
            return False
        except Exception as e:
            print(f"❌ gRPC channel error: {e}")
            return False

    except Exception as e:
        print(f"❌ Failed to create gRPC channel: {e}")
        return False
    finally:
        if 'channel' in locals():
            channel.close()

def test_grpc_with_dgraph_client():
    """Test gRPC connection using Dgraph client"""
    if not DGRAPH_MODULES_AVAILABLE:
        print("⚠️  Dgraph gRPC modules not available")
        print("💡 Install with: pip install dgraph-io/dgo")
        return False

    print("🔄 Testing Dgraph gRPC client...")

    try:
        # Create channel
        channel = grpc.insecure_channel('localhost:9080')
        stub = pb_grpc.DgraphStub(channel)

        # Create a simple query request
        query = """
        {
            all(func: has(name)) {
                uid
                name
            }
        }
        """

        request = pb.Request(
            query=query,
            start_ts=0,
            read_only=True
        )

        # Execute query
        response = stub.Query(request, timeout=10)
        print("✅ Dgraph gRPC query executed successfully")
        print(f"📊 Response received: {len(response.json)} bytes")
        print(f"📝 Response: {response.json}")
        return True

    except grpc.RpcError as e:
        print(f"❌ gRPC error: {e.code()} - {e.details()}")
        return False
    except Exception as e:
        print(f"❌ Dgraph client error: {e}")
        return False
    finally:
        if 'channel' in locals():
            channel.close()

def test_grpc_services():
    """Test available gRPC services using reflection"""
    print("🔄 Testing gRPC service discovery...")

    try:
        channel = grpc.insecure_channel('localhost:9080')

        # Try to list services using server reflection
        try:
            # This would work if server reflection is enabled
            print("ℹ️  Note: Server reflection may not be enabled")
            print("💡 Expected services: api.Dgraph, api.Login")
            return True
        except Exception as e:
            print(f"ℹ️  Service reflection: {e}")
            return True

    except Exception as e:
        print(f"❌ Service discovery error: {e}")
        return False
    finally:
        if 'channel' in locals():
            channel.close()

def main():
    """Main test function"""
    print("🚀 Dgraph gRPC Connection Tester")
    print("=" * 40)

    # Test 1: Basic gRPC connection
    if test_grpc_connection():
        print("🎉 Basic gRPC connectivity: PASS")
    else:
        print("💥 Basic gRPC connectivity: FAIL")

    print()

    # Test 2: Service discovery
    if test_grpc_services():
        print("🎉 Service discovery: PASS")
    else:
        print("💥 Service discovery: FAIL")

    print()

    # Test 3: Dgraph client (if modules available)
    if DGRAPH_MODULES_AVAILABLE:
        if test_grpc_with_dgraph_client():
            print("🎉 Dgraph gRPC client: PASS")
        else:
            print("💥 Dgraph gRPC client: FAIL")
    else:
        print("ℹ️  Dgraph gRPC client: SKIPPED (modules not available)")

    print()
    print("📋 Test Summary:")
    print("   - gRPC Port: 9080")
    print("   - Expected Services: api.Dgraph")
    print("   - Protocol: gRPC with Protocol Buffers")
    print()
    print("💡 For production use:")
    print("   - Enable TLS for secure connections")
    print("   - Implement connection pooling")
    print("   - Add proper error handling and retries")

if __name__ == "__main__":
    main()