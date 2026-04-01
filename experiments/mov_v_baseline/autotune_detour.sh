#!/bin/bash
set -euo pipefail

END=10

for i in $(seq $END); do
    ./target/release/caviar --expressions-file ./data/prefix/evaluation.csv -i 10000000 -n 10000000 -t 3 prove detour --offset $i &
done
wait