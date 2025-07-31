#!/usr/bin/env bash

set -e

cargo vendor > vendor.toml

curl -sSL https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.27.0.zip -o vendor/swagger-ui.zip

pushd vendor/trustify-ui/res
npm ci --ignore-scripts
popd
