#!/bin/bash
echo "Starting server..."
cargo run --bin vectorizer-server -- --host 127.0.0.1 --port 15001 --project ../gov &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"
echo "To stop the server, run: kill $SERVER_PID"

