#!/bin/bash

# Cortex Neural Vault Build Script
# Compiles the Rust Core Engine and copies the library to the Go Orchestrator directory

echo "==> Building CortexCore Engine (Rust) in Release mode..."
cd "$(dirname "$0")/core" || exit
cargo build --release
if [ $? -ne 0 ]; then
    echo "ERROR: Rust build failed!"
    exit 1
fi

echo "==> Rust Core Engine built successfully."

echo "==> Copying dynamic library to orchestrator..."
cd ..

# Detect OS and copy the appropriate library
if [[ "$OSTYPE" == "darwin"* ]]; then
    # Mac OSX
    cp core/target/release/libcortex_core.dylib orchestrator/
    echo "Copied libcortex_core.dylib to orchestrator/."
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    cp core/target/release/libcortex_core.so orchestrator/
    echo "Copied libcortex_core.so to orchestrator/."
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    # Windows
    cp core/target/release/cortex_core.dll orchestrator/
    echo "Copied cortex_core.dll to orchestrator/."
else
    echo "OS Type $OSTYPE not recognized, trying to copy .so and .dylib..."
    cp core/target/release/libcortex_core.so orchestrator/ 2>/dev/null
    cp core/target/release/libcortex_core.dylib orchestrator/ 2>/dev/null
fi

echo "==> Done! You can now run the Go Orchestrator in the 'orchestrator' directory."
