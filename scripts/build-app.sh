#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app_dir="$project_dir/target/release/Capture Viewer.app"
contents_dir="$app_dir/Contents"

cd "$project_dir"
cargo build --release

mkdir -p "$contents_dir/MacOS"
cp "$project_dir/Info.plist" "$contents_dir/Info.plist"
cp "$project_dir/target/release/capture-viewer" "$contents_dir/MacOS/capture-viewer"
codesign --force --sign - "$app_dir"

echo "$app_dir"
