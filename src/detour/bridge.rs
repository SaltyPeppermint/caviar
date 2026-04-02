use crate::argparse::Params;
use crate::structs::{ExpressionStruct, ResultStructure};
use crate::trs::{rules, ConstantFold, Math};

use std::time::{Duration, Instant};

use colored::Colorize;
use egg::{AstDepth, EGraph, Extractor, Pattern, RecExpr, Searcher, StopReason};

// const OFFSET: usize = 3; // AUTOTUNE THIS
const UNREACHABLE_COST: u128 = 10_000_000; // AUTOTUNE THIS

/// Runs Caviar with NPP on the expressions passed as vector using the different params passed.
#[allow(dead_code)]
pub fn prove_expression_detour(
    exprs_vect: &[ExpressionStruct],
    ruleset_class: i8,
    params: &Params,
    offset: usize,
    report: bool,
) -> Vec<ResultStructure> {
    // Initialize the results vector.
    let mut results = Vec::new();

    // For each expression try to prove it using Caviar with NPP then push the results into the results vector.
    for expression in exprs_vect {
        println!("Starting Expression: {}", expression.index);
        let mut res = detour_prove(
            expression.index,
            &expression.expression,
            ruleset_class,
            params,
            report,
            offset,
        );
        res.add_halide(expression.halide_data.clone());
        results.push(res);
    }
    results
}

// NOT USING THE ILC CHECK SINCE IT IS INCOMPATIBLE?
fn detour_prove(
    index: i32,
    start_expression: &str,
    ruleset_class: i8,
    params: &Params,
    print_report: bool,
    offset: usize,
) -> ResultStructure {
    // Parse the input expression and the goals
    let start: RecExpr<Math> = start_expression.parse().unwrap();
    // Set up the goals we will check for.
    let goals: [Pattern<Math>; 2] = ["0".parse().unwrap(), "1".parse().unwrap()];
    let best_expr;

    let mut egraph = EGraph::default();
    let root = egraph.add_expr(&start);

    if print_report {
        println!(
            "\n====================================\nProving Expression:\n {start_expression}\n"
        );
    }

    let myhook = move |eg: &EGraph<Math, ConstantFold>| {
        let id = eg.find(root);
        // Check if the end expression matches any representation of the root eclass.
        for (goal_index, goal) in goals.iter().enumerate() {
            if goal.search_eclass(eg, id).is_some() {
                return Err(goals[goal_index].to_string());
            }
        }
        Ok(())
    };

    let start = Instant::now();

    let report = super::copy_paste::detour_run(
        &[root],
        &rules(ruleset_class),
        &mut egraph,
        &mut [Box::new(myhook)],
        Duration::from_secs_f64(params.time),
        params.nodes,
        |_| 1,
        offset as u128,
        UNREACHABLE_COST,
    );
    // 'outer: loop {
    //     i += 1;
    //     detour_step(
    //         i,
    //         &[root],
    //         &rules(ruleset_class),
    //         &mut egraph,
    //         stop,
    //         params.nodes,
    //         offset,
    //     );
    //     if egraph.total_size() > params.nodes {
    //         report.stop_reason = StopReason::NodeLimit(egraph.total_size());
    //         break 'outer;
    //     }
    //     if Instant::now() > stop {
    //         report.stop_reason = StopReason::TimeLimit(start.elapsed().as_secs_f64());
    //         break 'outer;
    //     }

    //     let id = egraph.find(root);
    //     // Check if the end expression matches any representation of the root eclass.
    //     for (goal_index, goal) in goals.iter().enumerate() {
    //         if goal.search_eclass(&egraph, id).is_some() {
    //             found = true;
    //             proved_goal_index = goal_index;
    //             report.stop_reason = StopReason::Other("Goal reached!".to_string());
    //             break 'outer;
    //         }
    //     }

    //     egraph.rebuild();
    // }
    // report.memo_size = egraph.total_size();
    // report.egraph_nodes = egraph.total_number_of_nodes();
    // report.egraph_classes = egraph.number_of_classes();
    // report.iterations = i;
    // report.total_time = start.elapsed().as_secs_f64();

    if let StopReason::Other(found_string) = &report.stop_reason {
        if print_report {
            println!(
                "{}\n{:?}",
                "Proved goal:".bright_green().bold(),
                found_string
            );
        }
        best_expr = Some(found_string.to_owned());
    } else {
        // If we couldn't prove the goal, we extract the best expression.
        let extractor = Extractor::new(&egraph, AstDepth);
        let now = Instant::now();
        let (_, best_exprr) = extractor.find_best(root);
        let extraction_time = now.elapsed().as_secs_f32();
        best_expr = Some(best_exprr.to_string());

        if print_report {
            println!("{}\n", "Could not prove any goal:".bright_red().bold(),);
            println!(
                "Best Expr: {}",
                best_exprr.to_string().bright_green().bold()
            );
            println!(
                "{} {}",
                "Extracting Best Expression took:".bright_red(),
                extraction_time.to_string().bright_green()
            );
        }
    }

    let total_time = start.elapsed().as_secs_f64();
    if print_report {
        println!("Report (search, apply and rebuild time are fake):\n{report}");
    }

    ResultStructure::new(
        index,
        start_expression.to_string(),
        "1/0".to_string(),
        matches!(report.stop_reason, StopReason::Other(_)),
        best_expr.unwrap_or_default(),
        i64::from(ruleset_class),
        report.iterations,
        egraph.total_number_of_nodes(),
        report.rebuilds,
        total_time,
        fmt_stop_reason(&report.stop_reason),
        None,
    )
}

fn fmt_stop_reason(stop_reason: &StopReason) -> String {
    let stop_reason = match stop_reason {
        StopReason::Saturated => "Saturation".to_string(),
        StopReason::IterationLimit(iter) => format!("Iterations: {iter}"),
        StopReason::NodeLimit(nodes) => format!("Node Limit: {nodes}"),
        StopReason::TimeLimit(time) => format!("Time Limit : {time}"),
        StopReason::Other(reason) => reason.to_owned(),
    };
    stop_reason
}
