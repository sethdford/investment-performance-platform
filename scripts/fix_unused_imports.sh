#!/bin/bash

# Fix unused imports in example files
cargo fix --example api_example --allow-dirty
cargo fix --example blas_test --allow-dirty
cargo fix --example blas_test_standalone --allow-dirty
cargo fix --example graphql_example --allow-dirty
cargo fix --example graphql_server --allow-dirty
cargo fix --example graphql_server_simple --allow-dirty

# Fix unused imports in integration tests
cargo fix --test integration_tests --allow-dirty

# Fix unused imports in the library
cargo fix --lib --allow-dirty

# Fix unused imports in the binary
cargo fix --bin bootstrap --allow-dirty

echo "Fixed unused imports in example files, integration tests, library, and binary." 