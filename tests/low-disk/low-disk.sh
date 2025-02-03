#!/usr/bin/env bash
# Run Qdrant container with limited disk amount
# and verify that it doesn't crash when disk
# is running low during points insertion.

# possible values are search|indexing
declare TEST=${1:-"search"}

set -xeuo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

declare DOCKER_IMAGE_NAME=qdrant-recovery

docker buildx build --build-arg=PROFILE=ci --load ../../ --tag=$DOCKER_IMAGE_NAME

declare OOD_CONTAINER_NAME=qdrant-ood-$TEST

docker rm -f "${OOD_CONTAINER_NAME}" || true

declare container && container=$(
    docker run -d \
      --mount type=tmpfs,target=/qdrant/storage,tmpfs-size=10240000 \
      -p 127.0.0.1:6333:6333 \
      -p 127.0.0.1:6334:6334 \
      --name ${OOD_CONTAINER_NAME} \
      $DOCKER_IMAGE_NAME
)

function cleanup {
    docker logs $container -n 20 || true
    docker stop $container || true
}

trap cleanup EXIT

# Wait (up to ~30 seconds) for the service to start
declare retry=0
while [[ $(curl -sS localhost:6333 -w ''%{http_code}'' -o /dev/null) != 200 ]]; do
    if ((retry++ < 30)); then
      sleep 1
    else
        echo "Service failed to start in ~30 seconds" >&2
        exit 7
    fi
done

# check that low disk is handled OK during points insertion
# this also does search after each insertion
python3 create_and_search_items.py "$TEST" low-disk 2000 6333

sleep 5

# Check that there's an OOD log message in service logs.
declare OUT_OF_DISK_MSG='No space left on device:'

if (! docker logs "$container" 2>&1 | grep "$OUT_OF_DISK_MSG") ; then
    echo "'$OUT_OF_DISK_MSG' log message not found in $container container logs" >&2
    exit 9
fi

printf '%s: OK\n\n' "${TEST}"

echo "Success"