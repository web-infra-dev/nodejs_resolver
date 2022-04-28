
#!/bin/sh
set -e
command=$1

# make sure you install grcov in local.
rm -rf ./coverage
RUSTFLAGS="-Zinstrument-coverage" cargo build --verbose
RUSTFLAGS="-Zinstrument-coverage" LLVM_PROFILE_FILE="test-%p-%m.profraw" cargo test --verbose
grcov . -s . --binary-path ./target/debug/ --branch --ignore-not-existing  -o ./coverage
find . -type f -name '*.profraw' -delete