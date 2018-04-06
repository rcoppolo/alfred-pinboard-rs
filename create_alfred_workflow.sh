#!/bin/bash

set -x
alfred_pinboard_rs=$(pwd)
workflow_dir="$HOME/Dropbox/Alfred/Alfred.alfredpreferences/workflows/user.workflow.665EAB20-5141-463D-8C5A-90093EEAA756"
res_dir="$alfred_pinboard_rs/res/workflow"

echo "Building new release..."
cd "$alfred_pinboard_rs" || exit
cargo build --release > build.log 2>&1

echo "Copying resoursces from Alfred's workflow dir..."
cp "$workflow_dir"/* "$res_dir"

echo "Copying executable to workflow's folder..."
strip target/release/alfred-pinboard-rs
cp target/release/alfred-pinboard-rs "$res_dir"

echo "Updating version in info.plist"
version=$(git describe --tags --abbrev=0)
defaults write "$res_dir"/info.plist version "$version"
plutil -convert xml1 "$res_dir"/info.plist

echo "Creating the workflow bundle..."
cd "$res_dir" || exit
rm -f AlfredPinboardRust.alfredworkflow

zip -r AlfredPinboardRust.alfredworkflow ./*

echo "Moving bundle to executable folder..."
mv AlfredPinboardRust.alfredworkflow "$alfred_pinboard_rs"
rm alfred-pinboard-rs
