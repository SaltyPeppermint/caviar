use std::{env, ffi::OsString, fs::File, io::Read, time::Instant};

use io::reader::get_nth_arg;
use json::{parse, JsonValue};
use serde::de::Expected;

use crate::io::reader::{get_runner_params, get_start_end, read_expressions};
use crate::io::writer::write_results;
use crate::structs::{ExpressionStruct, ResultStructure};
use crate::trs::{filtered_rules, prove_expr, prove_expression_with_file_classes};

mod trs;

mod dataset;
mod io;
mod rules;
mod structs;

#[allow(dead_code)]
fn prove_expressions(
    exprs_vect: &Vec<ExpressionStruct>,
    ruleset_class: i8,
    params: (usize, usize, u64),
    use_iteration_check: bool,
    report: bool,
) -> Vec<ResultStructure> {
    let mut results = Vec::new();
    for expression in exprs_vect.iter() {
        results.push(prove_expr(
            expression,
            ruleset_class,
            params,
            use_iteration_check,
            report,
        ));
    }
    results
}

fn test_classes(
    path: OsString,
    exprs_vect: &Vec<ExpressionStruct>,
    params: (usize, usize, u64),
    use_iteration_check: bool,
    report: bool,
) -> () {
    let mut file = File::open(path).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let classes = parse(&s).unwrap();
    let mut results_structs = Vec::new();
    let mut results_proving_class = Vec::new();
    let mut results_exec_time = Vec::new();
    let start_t = Instant::now();

    for expression in exprs_vect.iter() {
        println!("Starting Expression: {}", expression.index);
        let (strct, class, exec_time) = prove_expression_with_file_classes(
            &classes,
            params,
            expression.index,
            &expression.expression.clone(),
            use_iteration_check,
            report,
        )
        .unwrap();
        results_structs.push(strct);
        results_proving_class.push(class);
        results_exec_time.push(exec_time);
    }
    let duration = start_t.elapsed().as_secs();
    let exec_time: f64 = results_exec_time.iter().map(|i| i.as_secs() as f64).sum();
    println!("Execution time : |{}| |{}|", duration, exec_time);
    write_results("results/class_analysis_results.csv", &results_structs).unwrap();
}

fn main() {
    let _args: Vec<String> = env::args().collect();
    // let expressions = vec![
    //     ("( <= ( - v0 11 ) ( + ( * ( / ( - v0 v1 ) 12 ) 12 ) v1 ) )","1"),
    //     ("( <= ( + ( / ( - v0 v1 ) 8 ) 32 ) ( max ( / ( + ( - v0 v1 ) 257 ) 8 ) 0 ) )","1"),
    //     ("( <= (/ a 2) (a))", "1"),
    //     ("( <= ( min ( + ( * ( + v0 v1 ) 161 ) ( + ( min v2 v3 ) v4 ) ) v5 ) ( + ( * ( + v0 v1 ) 161 ) ( + v2 v4 ) ) )","1"),
    //     ("( == (+ a b) (+ b a) )","1"),
    //     ("( == (min a b) (a))","1"),
    // ];
    // generate_dataset(expressions,(30, 10000, 5), 2, 2);
    // generate_dataset_par(&expressions,(30, 10000, 5), 2, 10);
    // println!("Printing rules ...");
    // let arr = filteredRules(&get_first_arg().unwrap(), 1).unwrap();
    // for rule in arr{
    //     println!("{}", rule.name());
    // }
    // println!("End.");

    if _args.len() > 4 {
        let operation = get_nth_arg(1).unwrap();
        let expressions_file = get_nth_arg(2).unwrap();
        let params = get_runner_params(3).unwrap();
        let expression_vect = read_expressions(&expressions_file).unwrap();
        match operation.to_str().unwrap() {
            "prove_exprs" => {
                // let mut expression_str_vct = Vec::new();
                // for expressionStruct in expression_vect.iter() {
                //     expression_str_vct.push( expressionStruct.expression.clone());
                // }
                // generate_dataset_0_1_par(&expression_str_vct, -1,params,true, 10);
                prove_expressions(&expression_vect, -1, params, true, true);
            }
            "test_classes" => {
                let classes_file = get_nth_arg(6).unwrap();
                test_classes(classes_file, &expression_vect, params, true, false);
            }
            _ => {}
        }
    } else {
        // let file_path = get_first_arg().unwrap();
        // let params = get_runner_params(2).unwrap();
        // let (start, end) = get_start_end().unwrap();
        // println!("Simplifying expression:\n {}\n to {}", start, end);
        // let result: ResultStructure =
        //     trs::prove_expression_with_file_classes(params, 1, &start, &end, &file_path, true)
        //         .unwrap();
        // println!("Total time = {}", result.total_time);
    }
}
