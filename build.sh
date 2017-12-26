echo "==> Buliding build container."
CONTAINER_ID=$(docker build -q .)
echo "==> Building."
docker run --rm -v "$(pwd)":/project $CONTAINER_ID make
