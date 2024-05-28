#!/bin/bash
./target/release/caviar prove_exprs_fast_passes ./data/prefix/evaluation.csv 10000000 10000000 3 0.1
./target/release/caviar prove_exprs_fast_passes ./data/prefix/evaluation.csv 10000000 10000000 3 0.25
./target/release/caviar prove_exprs ./data/prefix/evaluation.csv 10000000 10000000 3
