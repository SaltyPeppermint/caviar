#!/bin/bash
for i in 0.01 0.05 0.1 0.25 0.5 0.75 1; do
     ./target/release/caviar --expressions-file ./data/prefix/expressions_egg.csv -i 10000000 -n 10000000 --time 3 prove pulse-npp --threshold $i
done
