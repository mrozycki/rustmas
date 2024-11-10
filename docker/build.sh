#!/bin/bash
set -e

docker build --target rustmas-backend --tag localhost:rustmas-backend -f ./docker/backend.dockerfile .
docker build --target rustmas-frontend --tag localhost:rustmas-frontend -f ./docker/frontend.dockerfile .
