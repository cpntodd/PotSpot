#!/bin/sh
# Deploy built static files to the mounted output directory
set -e
echo "[web] Deploying frontend..."
cp -a /app/build/* /output/
echo "[web] Done. $(find /output -type f 2>/dev/null | wc -l) files deployed."
