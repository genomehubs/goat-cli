#!/usr/bin/env bash

# remove old docs
rm -rf ./docs
# don't want dependencies
cargo doc --no-deps --document-private-items
# magic..?
echo "<meta http-equiv=\"refresh\" content=\"0; url=goat_cli\">" > target/doc/index.html
# copy to docs
cp -r target/doc ./docs