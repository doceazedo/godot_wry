#!/bin/bash
start=`date +%s`

targets=(
  "aarch64-apple-darwin"
  "x86_64-pc-windows-gnu"
  "aarch64-unknown-linux-gnu"
)

printf "Setting up...\n"
rm -rf dist/* 2> /dev/null
mkdir -p "dist"

printf "Copying project files...\n"
cp -a godot/addons "dist/addons"

for target in "${targets[@]}"; do
  printf "Building binaries for $target...\n"
  (cd rust && cargo build --target=$target --release)

  printf "Copying binaries...\n"
  mkdir -p "dist/addons/godot_wry/bin/$target"
  find "rust/target/$target/release" \( -name "*.dylib" -o -name "*.dll" -o -name "*.so" \) -exec cp {} "dist/addons/godot_wry/bin/$target" \;

  printf "Built target $target successfully!\n"
done

if [[ $* == *--zip* ]]; then
  printf "Zipping files...\n"
  (cd dist && zip -r "$(date '+%Y-%m-%d').zip" .)
fi

end=`date +%s`
time_elapsed=$((end-start))

printf "✅ Done! (${time_elapsed}s)\n"
