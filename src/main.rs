#![warn(clippy::all, clippy::pedantic)]

mod argparse;
mod dataset;
mod io;
mod rules;
mod structs;
mod trs;

use argparse::{CliArgs, Operation, Params, ProveParams, ProveStrategy};
use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

use json::parse;

//use crate::io::reader::read_expressions_paper;
//use crate::io::writer::write_results_paper;
use crate::io::reader::read_expressions;
//use crate::io::writer::write_results;
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
    //Initialize the results vector.
    let mut results = Vec::new();

    //For each expression try to prove it then push the results into the results vector.
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
    //Initialize the results vector.
    let mut results = Vec::new();
    //For each expression try to prove it using Caviar with Pulses then push the results into the results vector.
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
    //Initialize the results vector.
    let mut results = Vec::new();

    //For each expression try to prove it using Caviar with NPP then push the results into the results vector.
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
    //Initialize the results vector.
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

///Runs Caviar with Pulses and NPP on the expressions passed as vector using the different params passed.
fn prove_expressions_pulses_npp(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    threshold: f64,
    params: &Params,
    use_iteration_check: bool,
    report: bool,
) -> Vec<ResultStructure> {
    //Initialize the results vector.
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

/// Runs Caviar using hierarchical clusters of rules to prove the expressions passed as vector using the different params passed.
#[allow(clippy::cast_precision_loss)]
fn prove_clusters(
    path: &str,
    exprs_vect: &[ExpressionStruct],
    params: &Params,
    count: usize,
    use_iteration_check: bool,
    report: bool,
) {
    //Read the clusters from the files generated using Python.
    let mut file = File::open(path).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let classes = parse(&s).unwrap();

    //Initialization
    let mut results_structs = Vec::new();
    let mut results_proving_class = Vec::new();
    let mut results_exec_time = Vec::new();
    let start_t = Instant::now();
    let mut average;
    let mut prove_result: (ResultStructure, i64, Duration);
    let mut i;

    //For each expression try to prove it using the clusters generated one after the other.
    for expression in exprs_vect {
        if report {
            println!("Starting Expression: {}", expression.index);
        }
        i = 0;
        average = 0.0;
        loop {
            prove_result = trs::prove_expression_with_file_classes(
                &classes,
                params,
                expression.index,
                &expression.expression.clone(),
                use_iteration_check,
                report,
            )
            .unwrap();
            if report {
                println!("Iter: {} | time: {}", i, prove_result.0.total_time);
            }
            average += prove_result.0.total_time;
            i += 1;
            if i == count || !prove_result.0.result {
                break;
            }
        }
        prove_result.0.total_time = average / (i as f64);
        results_structs.push(prove_result.0);
        results_proving_class.push(prove_result.1);
        results_exec_time.push(prove_result.2);
        if report {
            println!("Average time: {}", average / (i as f64));
        }
    }
    let duration = start_t.elapsed().as_secs();
    let exec_time: f64 = results_exec_time.iter().map(|i| i.as_secs() as f64).sum();
    if report {
        println!("Execution time : |{duration}| |{exec_time}|");
    }

    //Write the results into the results csv file.
    writer::write_results(
        &format!(
            "results/k_{}_class_analysis_results_params_{}_{}_{}_exec_{}.csv",
            classes[0].len(),
            params.iter,
            params.nodes,
            params.time,
            duration
        ),
        &results_structs,
    )
    .unwrap();
}

/// Runs Simple Caviar to simplify the expressions passed as vector using the different params passed.
fn simplify_expressions(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    params: &Params,
    report: bool,
) -> Vec<ResultStructure> {
    //Initialize the results vector.
    let mut results = Vec::new();

    //For each expression try to prove it then push the results into the results vector.
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

fn prove(params: &Params, prove_params: &ProveParams, strategy: ProveStrategy) {
    match strategy {
        ProveStrategy::Simple {} => {
            let expression_vect = read_expressions(&params.expressions_file).unwrap();
            let results = prove_expressions(
                &expression_vect,
                -1,
                &params,
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
                threshold,
                &params,
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
                &params,
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
                threshold,
                &params,
                prove_params.use_iteration_check,
                prove_params.report,
            );
            writer::write_results(&format!("tmp/results_beh_npp_{threshold}.csv"), &results)
                .unwrap();
        }
        ProveStrategy::Clusters {
            classes_file,
            iterations_count,
        } => {
            let expression_vect = read_expressions(&params.expressions_file).unwrap();
            prove_clusters(
                &classes_file,
                &expression_vect,
                &params,
                iterations_count,
                prove_params.use_iteration_check,
                prove_params.report,
            );
        }
    }
}

#[allow(clippy::too_many_lines)]
fn main() {
    let args = CliArgs::parse();
    // let params = (args.iter, args.nodes, args.time);
    match args.operation {
        Operation::Dataset {
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

        Operation::Simplify { report } => {
            let expression_vect = read_expressions(&args.params.expressions_file).unwrap();
            let results = simplify_expressions(&expression_vect, -1, &args.params, report);
            writer::write_results("tmp/results_simplify.csv", &results).unwrap();
        }
        Operation::Prove {
            strategy,
            prove_params,
        } => prove(&args.params, &prove_params, strategy),
    }
    // } else {
    //     //Quick executions with default parameters
    //     let params = get_runner_params(1).unwrap();
    //     let (start, end) = get_start_end().unwrap();
    //     println!("Simplifying expression:\n {start}\n to {end}");
    //     //Example of NPP execution with default parameters
    //     println!("{:?}", simplify(-1, &start, -1, params, true));
    // }
}
