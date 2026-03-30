#!/bin/bash
./target/release/caviar --expressions-file ./data/prefix/expressions_egg.csv -i 10000000 -n 10000000 --time 3 prove pulse --threshold 0.1
./target/release/caviar --expressions-file ./data/prefix/expressions_egg.csv -i 10000000 -n 10000000 --time 3 prove pulse-npp --threshold 0.1
./target/release/caviar --expressions-file ./data/prefix/expressions_egg.csv -i 10000000 -n 10000000 --time 3 prove npp
./target/release/caviar --expressions-file ./data/prefix/expressions_egg.csv -i 10000000 -n 10000000 --time 3 prove simple
