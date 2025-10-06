#!/bin/bash

echo "Testing Prometheus and Loki endpoints..."
echo ""

# Test Prometheus
echo "1. Testing Prometheus at http://localhost:9090"
prometheus_status=$(curl -s 'http://localhost:9090/api/v1/query?query=up' | jq -r '.status' 2>/dev/null)
if [ "$prometheus_status" = "success" ]; then
    echo "   ✅ Prometheus is working"
    
    # Check for metrics
    echo "   Checking available metrics..."
    metrics=$(curl -s 'http://localhost:9090/api/v1/label/__name__/values' | jq -r '.data[]' 2>/dev/null | head -5)
    echo "   Sample metrics found:"
    echo "$metrics" | sed 's/^/      - /'
else
    echo "   ❌ Prometheus is not responding correctly"
fi

echo ""

# Test Loki
echo "2. Testing Loki at http://localhost:3100"
loki_ready=$(curl -s 'http://localhost:3100/ready' 2>/dev/null)
if [ -n "$loki_ready" ]; then
    echo "   ✅ Loki is responding"
    echo "   Status: $loki_ready"
    
    # Try to get some logs using range query
    echo "   Trying to fetch logs..."
    logs=$(curl -s -X GET 'http://localhost:3100/loki/api/v1/query_range' -G --data-urlencode 'query={app=~".+"}' --data-urlencode 'limit=1' 2>/dev/null | jq -r '.status' 2>/dev/null)
    if [ "$logs" = "success" ]; then
        echo "   ✅ Loki API is working"
        # Get sample log
        sample=$(curl -s -X GET 'http://localhost:3100/loki/api/v1/query_range' -G --data-urlencode 'query={app=~".+"}' --data-urlencode 'limit=1' 2>/dev/null | jq -r '.data.result[0].values[0][1]' 2>/dev/null | head -c 100)
        if [ -n "$sample" ]; then
            echo "   Sample log: ${sample}..."
        fi
    else
        echo "   ⚠️  Loki API may not be fully ready or has no logs"
    fi
else
    echo "   ❌ Loki is not responding"
fi

echo ""
echo "Dashboard will use fallback data if metrics are not available."
echo "To run the dashboard: cargo run"