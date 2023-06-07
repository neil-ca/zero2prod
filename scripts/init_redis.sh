#!/bin/bash
set -x
set -eo pipefail

RUNINNG_CONTAINER=$(docker ps --filter 'name=redis' --format '{{.ID}}')
if [[ -n $RUNINNG_CONTAINER ]]; then
    echo >&2 "there is a redis container running:"
    echo >&2 "  ${RUNINNG_CONTAINER}"
    exit 1
fi


docker run \
    -p "6379:6379" \
    -d \
    --name "redis_$(date '+%s')" \
    redis
>&2 echo "Redis is ready"
