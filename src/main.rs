#![warn(clippy::all, clippy::pedantic)]

mod argparse;
mod dataset;
mod io;
mod rules;
mod structs;
mod trs;

use argparse::{CliArgs, Mode, Params, ProveParams, ProveStrategy};
use clap::Parser;

use crate::io::reader::read_expressions;
use crate::io::writer;
use crate::structs::{ExpressionStruct, PaperResult, ResultStructure};

/// Runs Simple Caviar to prove the expressions passed as vector using the different params passed.
#[allow(dead_code)]
fn prove_expressions(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    params: &Params,
    use_iteration_check: bool,
    report: bool,
) -> Vec<ResultStructure> {
    // Initialize the results vector.
    let mut results = Vec::new();

    // For each expression try to prove it then push the results into the results vector.
    for expression in exprs_vect {
        println!("Starting Expression: {}", expression.index);
        let mut res = trs::prove(
            expression.index,
            &expression.expression,
            ruleset_class,
            params,
            use_iteration_check,
            report,
        );
        res.add_halide(expression.halide_data.clone());
        results.push(res);
    }
    results
}

/// Runs Caviar with Pulses on the expressions passed as vector using the different params passed.
fn prove_expressions_pulses(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    threshold: f64,
    params: &Params,
    use_iteration_check: bool,
    report: bool,
) -> Vec<ResultStructure> {
    // Initialize the results vector.
    let mut results = Vec::new();
    // For each expression try to prove it using Caviar with Pulses then push the results into the results vector.
    for expression in exprs_vect {
        println!("Starting Expression: {}", expression.index);
        let mut res = trs::prove_pulses(
            expression.index,
            &expression.expression,
            ruleset_class,
            threshold,
            params,
            use_iteration_check,
            report,
        );
        res.add_halide(expression.halide_data.clone());
        results.push(res);
    }
    results
}

/// Runs Caviar with NPP on the expressions passed as vector using the different params passed.
#[allow(dead_code)]
fn prove_expressions_npp(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    params: &Params,
    use_iteration_check: bool,
    report: bool,
) -> Vec<ResultStructure> {
    // Initialize the results vector.
    let mut results = Vec::new();

    // For each expression try to prove it using Caviar with NPP then push the results into the results vector.
    for expression in exprs_vect {
        println!("Starting Expression: {}", expression.index);
        let mut res = trs::prove_npp(
            expression.index,
            &expression.expression,
            ruleset_class,
            params,
            use_iteration_check,
            report,
        );
        res.add_halide(expression.halide_data.clone());
        results.push(res);
    }
    results
}

/// Runs  Caviar with Pulses and NPP on the expressions passed as vector using the different params passed.
#[allow(dead_code)]
fn prove_expressions_pulses_npp_paper(
    exprs_vect: &[(String, String)],
    ruleset_class: i8,
    threshold: f64,
    params: &Params,
    use_iteration_check: bool,
    report: bool,
) -> Vec<PaperResult> {
    // Initialize the results vector.
    let mut results = Vec::new();
    // For each expression try to prove it using Caviar with Pulses and NPP then push the results into the results vector.
    for expression in exprs_vect {
        println!("Starting Expression: {}", expression.0);
        let res = trs::prove_pulses_npp(
            -1,
            &expression.1,
            ruleset_class,
            threshold,
            params,
            use_iteration_check,
            report,
        );
        // res.add_halide(expression.halide_result, expression.halide_time);
        results.push(PaperResult::new(
            expression.0.clone(),
            expression.1.clone(),
            i8::from(res.result),
        ));
    }
    results
}

/// Runs Caviar with Pulses and NPP on the expressions passed as vector using the different params passed.
fn prove_expressions_pulses_npp(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    threshold: f64,
    params: &Params,
    use_iteration_check: bool,
    report: bool,
) -> Vec<ResultStructure> {
    // Initialize the results vector.
    let mut results = Vec::new();
    // For each expression try to prove it using Caviar with Pulses and NPP then push the results into the results vector.
    for expression in exprs_vect {
        println!("Starting Expression: {}", expression.index);
        results.push(trs::prove_pulses_npp(
            expression.index,
            &expression.expression,
            ruleset_class,
            threshold,
            params,
            use_iteration_check,
            report,
        ));
    }
    results
}

/// Runs Simple Caviar to simplify the expressions passed as vector using the different params passed.
fn simplify_expressions(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    params: &Params,
    report: bool,
) -> Vec<ResultStructure> {
    // Initialize the results vector.
    let mut results = Vec::new();

    // For each expression try to prove it then push the results into the results vector.
    for expression in exprs_vect {
        println!("Starting Expression: {}", expression.index);
        let mut res = trs::simplify(
            expression.index,
            &expression.expression,
            ruleset_class,
            params,
            report,
        );
        res.add_halide(expression.halide_data.clone());
        results.push(res);
    }
    results
}

fn prove(params: &Params, prove_params: &ProveParams, strategy: &ProveStrategy) {
    match strategy {
        ProveStrategy::Simple {} => {
            let expression_vect = read_expressions(&params.expressions_file).unwrap();
            let results = prove_expressions(
                &expression_vect,
                -1,
                params,
                prove_params.use_iteration_check,
                prove_params.report,
            );
            writer::write_results("tmp/results_prove.csv", &results).unwrap();
        }
        ProveStrategy::Pulse { threshold } => {
            let expression_vect = read_expressions(&params.expressions_file).unwrap();
            let results = prove_expressions_pulses(
                &expression_vect,
                -1,
                *threshold,
                params,
                prove_params.use_iteration_check,
                prove_params.report,
            );
            writer::write_results(&format!("tmp/results_beh_{threshold}.csv"), &results).unwrap();
        }
        ProveStrategy::Npp => {
            let expression_vect = read_expressions(&params.expressions_file).unwrap();
            let results = prove_expressions_npp(
                &expression_vect,
                -1,
                params,
                prove_params.use_iteration_check,
                prove_params.report,
            );
            writer::write_results("tmp/results_fast.csv", &results).unwrap();
        }
        ProveStrategy::PulseNpp { threshold } => {
            let expression_vect = read_expressions(&params.expressions_file).unwrap();
            let results = prove_expressions_pulses_npp(
                &expression_vect,
                -1,
                *threshold,
                params,
                prove_params.use_iteration_check,
                prove_params.report,
            );
            writer::write_results(&format!("tmp/results_beh_npp_{threshold}.csv"), &results)
                .unwrap();
        }
    }
}

#[allow(clippy::too_many_lines)]
fn main() {
    let args = CliArgs::parse();
    match args.mode {
        Mode::Dataset {
            reorder_count,
            batch_size,
            continoue_from_expr: continue_from_expr,
            cores,
        } => {
            rayon::ThreadPoolBuilder::new()
                .num_threads(cores)
                .build_global()
                .unwrap();
            dataset::generation_execution(
                &args.params,
                reorder_count,
                batch_size,
                continue_from_expr,
            );
        }
        Mode::Simplify { report } => {
            let expression_vect = read_expressions(&args.params.expressions_file).unwrap();
            let results = simplify_expressions(&expression_vect, -1, &args.params, report);
            writer::write_results("tmp/results_simplify.csv", &results).unwrap();
        }
        Mode::Prove {
            strategy,
            prove_params,
        } => prove(&args.params, &prove_params, &strategy),
    }
}
