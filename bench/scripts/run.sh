#!/bin/bash

set -e

MANAGER=$1
FIXTURE='ant-design'

if [ -d "$FIXTURE" ]; then
    echo "Directory $FIXTURE already exists."
else
    echo "Cloning $FIXTURE (only cloned master branch and without commmites)"
    git clone https://github.com/bvanjoi/ant-design.git -b master --single-branch --depth 1
fi

# insall deps
cd $FIXTURE

if [[ "$MANAGER" == "npm" ]]; then
    npm install --force
    elif [[ "$MANAGER" == "yarn" ]]; then
    npm install -g yarn
    yarn
    elif [[ "$MANAGER" == "pnpm" ]]; then
    npm install -g pnpm
    pnpm i
else
    echo "Unexpected package manager"
    echo $MANAGER
    echo "$MANAGER"
    exit 1
fi

cd -

# generator benchmark case
npm install
npm run gen:rs
# npm run gen:esbuild
# npm run gen:enhanced

# run

RS_OUTPUT="rs_bench.txt"
# ESBUILD_OUTPUT="esbuild_bench.txt"
# ENHANCED_OUTPUT="enhanced_bench.txt"

# node ../esbuildResolve.js | tee $ESBUILD_OUTPUT
# node ../enhanceResolve.js | tee $ENHANCED_OUTPUT
cargo bench --package nodejs-resolver --test bench --all-features -- bench_test::ant_design_bench | tee $RS_OUTPUT
