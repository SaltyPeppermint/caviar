#!/bin/bash
#
#SBATCH --job-name=proving_expressions_10s
#SBATCH --output=output_10s.txt
#
#SBATCH --ntasks=1

srun ../../../target/release/egg_halides_trs prove_exprs ../../../data/prefix/expressions_egg.csv 100000000 100000000 10
