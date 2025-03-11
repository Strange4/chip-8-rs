#!/bin/bash

if ! [ -x "$(command -v wasm-pack)" ]; then
    echo "You don't have wasm pack installed. Please run 'cargo install wasm-pack'"
    exit 1
fi

if [ -z $1 ]; then
    wasm-pack build --target web --dev --out-dir web/wasm
elif [ $1 == "--release" ]; then
    wasm-pack build --target web --release --out-dir web/wasm
elif [ $1 == "--profiling" ]; then
    wasm-pack build --target web --profiling --out-dir web/wasm
fi

echo "Copying the roms..."
cp -r roms web