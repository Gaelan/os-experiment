#!/usr/bin/env bash
echo "==> Building build container."
CONTAINER_ID=$(docker build -q .)
echo "==> Building."
docker run --rm -v "$(cd ../packages && pwd)":/project $CONTAINER_ID cd build-system && make
