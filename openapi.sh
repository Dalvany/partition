#!/usr/bin/env bash

OPENAPI_IMAGE=openapitools/openapi-generator-cli:latest

docker pull "$OPENAPI_IMAGE"

# Generate code
docker run \
  --rm \
  -u "$UID:$GID" \
  -v "${PWD}:/local" \
  "$OPENAPI_IMAGE" generate \
    -i /local/docs/openapi.yml \
    -g rust-server \
    -o /local/libraries/server-lib \
    --additional-properties=packageName=server-lib
