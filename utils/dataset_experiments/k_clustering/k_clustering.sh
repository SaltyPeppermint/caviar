#!/bin/bash
K=$1
python "`dirname "$0"`/k_clustering.py" $1
../../../target/release/egg_halides_trs test_classes ../../../results/expressions_egg.csv 1000000 100000000 3 "./results/k_"$K"_classes.json"
