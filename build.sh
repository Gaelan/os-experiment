echo "==> Buliding build container."
NAME=$(docker build -q .)
echo "==> Building."
docker run --rm -v $(pwd):/project $NAME make