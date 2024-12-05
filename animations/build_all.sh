#!/bin/bash
set -e

mkdir -p ../target/animations

for d in `find . -type d -depth 1`; do
    cd $d
    crabwrap
    mv *.crab ../../target/animations/
    cd ..
done
