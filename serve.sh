#!/bin/bash

# Check for --release flag to serve from docs/ instead of web/
SERVE_DIR="web"
if [ "$1" = "--release" ]; then
    SERVE_DIR="docs"
    echo "Starting local web server (serving RELEASE build from docs/)..."
else
    echo "Starting local web server (serving DEBUG build from web/)..."
fi

echo "Open http://localhost:8000 in your browser"
echo "Press Ctrl+C to stop the server"
echo ""

cd $SERVE_DIR && python3 -m http.server 8000
