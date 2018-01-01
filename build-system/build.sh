#!/usr/bin/env bash
set -e

echo "==> Building build container."
CONTAINER_ID=$(docker build -q .)
echo "==> Building."
docker run --rm -v "$(cd .. && pwd)":/project $CONTAINER_ID bash -c "cd build-system && make"
echo "==> Running in QEMU."
qemu-system-x86_64 -cdrom experiment-x86_64.iso 