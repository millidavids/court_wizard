#!/bin/bash

echo "Starting local web server..."
echo "Open http://localhost:8000 in your browser"
echo "Press Ctrl+C to stop the server"
echo ""

cd web && python3 -m http.server 8000
