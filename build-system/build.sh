#!/usr/bin/env bash
set -e

echo "==> Building build container."
CONTAINER_ID=$(docker build -q .)
echo "==> Building."
cd ..
mkdir -p build
mkdir -p build/cache
mkdir -p build/cache/cargo
docker run -t --rm -v "$(pwd)":/project -v "$(pwd)"/build/cache/xargo:/root/.xargo $CONTAINER_ID bash -c "cd build-system && make"
echo "==> Running in QEMU."
cd build-system
qemu-system-x86_64 -cdrom ../build/experiment-x86_64.iso
# qemu-system-x86_64 -d int --no-reboot -cdrom ../build/experiment-x86_64.iso