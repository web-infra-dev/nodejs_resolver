#! /bin/bash

set -e

FIXTURE='ant-design'

if [ -d "$FIXTURE" ]; then 
  echo "Directory $FIXTURE already exists."
else
  echo "Cloning $FIXTURE"
  git clone https://github.com/ant-design/ant-design/
fi

# insall deps
cd $FIXTURE && npm install && cd -
npm install

# generator benchmark case
npm run gen:rs
# npm run gen:esbuild
# npm run gen:enhanced

# run

RS_OUTPUT="rs_bench.txt"
# ESBUILD_OUTPUT="esbuild_bench.txt"
# ENHANCED_OUTPUT="enhanced_bench.txt"

# node ../esbuildResolve.js | tee $ESBUILD_OUTPUT
# node ../enhanceResolve.js | tee $ENHANCED_OUTPUT
cargo +nightly bench --package nodejs-resolver --test bench --all-features -- bench_test::ant_design_bench | tee $RS_OUTPUT
