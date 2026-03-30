use crate::argparse::Params;
use crate::structs::{ExpressionStruct, ResultStructure};
use crate::trs::{rules, Math};

use std::time::{Duration, Instant};

use colored::Colorize;
use egg::{
    Analysis, AstDepth, AstSize, CostFunction, EGraph, ENodeOrVar, Extractor, Id, Language,
    Pattern, PatternAst, RecExpr, Rewrite, Runner, Searcher, StopReason, Subst,
};

// const OFFSET: usize = 3; // AUTOTUNE THIS

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
    let mut found = false;
    let mut proved_goal_index = 0;
    let best_expr;

    let mut egraph = EGraph::default();
    let root = egraph.add_expr(&start);

    if print_report {
        println!(
            "\n====================================\nProving Expression:\n {start_expression}\n"
        );
    }

    let start = Instant::now();
    let stop = start + Duration::from_secs_f64(params.time);
    let mut report = Runner::<Math, ()>::new(()).run([]).report(); // fake report
    let mut i = 0;

    'outer: loop {
        i += 1;
        crate::detour::detour_step(
            i,
            &[root],
            &rules(ruleset_class),
            &mut egraph,
            stop,
            params.nodes,
            offset,
        );
        if egraph.total_size() > params.nodes {
            report.stop_reason = StopReason::NodeLimit(egraph.total_size());
            break 'outer;
        }
        if Instant::now() > stop {
            report.stop_reason = StopReason::TimeLimit(start.elapsed().as_secs_f64());
            break 'outer;
        }

        let id = egraph.find(root);
        // Check if the end expression matches any representation of the root eclass.
        for (goal_index, goal) in goals.iter().enumerate() {
            if goal.search_eclass(&egraph, id).is_some() {
                found = true;
                proved_goal_index = goal_index;
                report.stop_reason = StopReason::Other("Goal reached!".to_string());
                break 'outer;
            }
        }

        egraph.rebuild();
    }
    report.memo_size = egraph.total_size();
    report.egraph_nodes = egraph.total_number_of_nodes();
    report.egraph_classes = egraph.number_of_classes();
    report.iterations = i;
    report.total_time = start.elapsed().as_secs_f64();

    if found {
        if print_report {
            println!(
                "{}\n{:?}",
                "Proved goal:".bright_green().bold(),
                goals[proved_goal_index].to_string()
            );
        }
        best_expr = Some(goals[proved_goal_index].to_string());
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
        found,
        best_expr.unwrap_or_default(),
        i64::from(ruleset_class),
        i,
        egraph.total_number_of_nodes(),
        i,
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

pub fn detour_step<L: Language, N: Analysis<L> + Default>(
    i: usize,
    roots: &[Id],
    rws: &[Rewrite<L, N>],
    eg: &mut EGraph<L, N>,
    stop: Instant,
    node_limit: usize,
    offset: usize,
) {
    if i.is_multiple_of(2) {
        pat_detour_eqsat_step(roots, rws, eg, stop, node_limit, offset);
    } else {
        let egr = std::mem::take(eg);
        let mut runner = Runner::<L, N, ()>::new(N::default())
            .with_egraph(egr)
            .with_iter_limit(1)
            .with_node_limit(node_limit)
            .with_time_limit(stop - Instant::now())
            .run(rws);
        *eg = std::mem::take(&mut runner.egraph);
    }
}
fn pat_detour_eqsat_step<L: Language, N: Analysis<L>>(
    roots: &[Id],
    rws: &[Rewrite<L, N>],
    eg: &mut EGraph<L, N>,
    stop: Instant,
    node_limit: usize,
    offset: usize,
) {
    let ex = Extractor::new(eg, AstSize);
    let ctxt_cost = compute_ctxt_costs(roots, eg, &ex);

    #[expect(clippy::type_complexity)]
    let mut matches: BTreeMap<
        /*detour cost*/ usize,
        Vec<(
            /*rw id*/ usize,
            Id,
            Subst,
            /*ctxt_cost*/ usize,
            /*pat_cost*/ usize,
        )>,
    > = BTreeMap::default();
    for (rw_i, rw) in rws.iter().enumerate() {
        let lhs_pat = rw.searcher.get_pattern_ast().unwrap();

        for m in rw.searcher.search(eg) {
            let lhs = m.eclass;
            for subst in m.substs {
                let pat_cost = pat_cost(lhs_pat, &subst, &ex);
                // We don't subtract the root cost here, it's a constant offset, so why would we.
                let cx_cost = *ctxt_cost.get(&lhs).unwrap_or(&100_000_000); // this is the cost you get from not being able to reach any root.
                let detour_cost = cx_cost + pat_cost;
                matches.entry(detour_cost).or_default();
                matches
                    .get_mut(&detour_cost)
                    .unwrap()
                    .push((rw_i, lhs, subst, cx_cost, pat_cost));
                if Instant::now() > stop {
                    return;
                }
            }
        }
    }

    let eg_data = |eg: &EGraph<_, _>| (eg.number_of_classes(), eg.total_size());

    let og_data = eg_data(eg);
    let mut found_cost = None;

    'outer: for (full_cost, new_apps) in matches {
        if let Some(found) = found_cost {
            if full_cost > found + offset {
                break;
            }
        }
        for (rw_i, lhs, subst, _cx_cost, _pat_cost) in &new_apps {
            let rw = &rws[*rw_i];
            let pat_ast = rw.searcher.get_pattern_ast();
            rw.applier.apply_one(eg, *lhs, subst, pat_ast, rw.name);
            if eg_data(eg) != og_data {
                found_cost = Some(full_cost);
            }
            if Instant::now() > stop {
                break 'outer;
            }
            if eg.total_size() > node_limit {
                break 'outer;
            }
        }
    }

    eg.rebuild();
}

// === ctxt cost ===

fn compute_ctxt_costs<L: Language, N: Analysis<L>>(
    roots: &[Id],
    eg: &EGraph<L, N>,
    ex: &Extractor<AstSize, L, N>,
) -> HashMap<Id, usize> {
    let mut ctxt_cost = HashMap::new();

    let mut queue: MinPrioQueue<usize, Id> = MinPrioQueue::new();

    // initial
    for root in roots {
        queue.push(0, *root);
    }

    while let Some((cst, i)) = queue.pop() {
        if ctxt_cost.contains_key(&i) {
            continue;
        }
        ctxt_cost.insert(i, cst);
        for e in &eg[i].nodes {
            let e_cost = AstSize.cost(e, |k| ex.find_best_cost(k));
            for &c in e.children() {
                // optimization: don't push junk to the queue.
                // NOTE: if we remembered what's the best thing we already pushed to the queue for some class,
                // we could do more efficient pruning.
                if ctxt_cost.contains_key(&c) {
                    continue;
                }

                let c_cost = ex.find_best_cost(c);
                let ncst = e_cost + cst - c_cost;
                queue.push(ncst, c);
            }
        }
    }

    ctxt_cost
}

fn pat_cost<L: Language, N: Analysis<L>>(
    pat: &PatternAst<L>,
    subst: &Subst,
    ex: &Extractor<AstSize, L, N>,
) -> usize {
    let mut vec: Vec<usize> = Vec::new();
    for i in 0..pat.as_ref().len() {
        let cost = match &pat[i.into()] {
            ENodeOrVar::ENode(n) => AstSize.cost(n, |x| vec[usize::from(x)]),
            ENodeOrVar::Var(v) => ex.find_best_cost(subst[*v]),
        };
        vec.push(cost);
    }
    vec.last().copied().unwrap()
}

// // === misc ===

// fn lookup_pat<L: Language, N: Analysis<L>>(
//     pat: &PatternAst<L>,
//     eg: &EGraph<L, N>,
//     subst: &Subst,
// ) -> Option<Id> {
//     let mut vec = Vec::new();
//     for i in 0..pat.as_ref().len() {
//         match &pat[i.into()] {
//             ENodeOrVar::ENode(n) => {
//                 let mut n = n.clone().map_children(|k| vec[usize::from(k)]);
//                 let k = eg.lookup(&mut n)?;
//                 vec.push(k);
//             }
//             ENodeOrVar::Var(v) => vec.push(subst[*v]),
//         }
//     }
//     vec.last().copied()
// }

// === minqueue ===

use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};

struct MinPrioQueue<U, T>(BinaryHeap<WithOrdRev<U, T>>);

impl<U: Ord, T: Eq> MinPrioQueue<U, T> {
    pub fn new() -> Self {
        MinPrioQueue(BinaryHeap::default())
    }

    pub fn push(&mut self, u: U, t: T) {
        self.0.push(WithOrdRev(u, t));
    }

    pub fn pop(&mut self) -> Option<(U, T)> {
        self.0.pop().map(|WithOrdRev(u, t)| (u, t))
    }
}

// Takes the `Ord` from U, but reverses it.
#[derive(PartialEq, Eq, Debug)]
struct WithOrdRev<U, T>(pub U, pub T);

#[expect(clippy::non_canonical_partial_ord_impl)]
impl<U: Ord, T: Eq> PartialOrd for WithOrdRev<U, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // It's the other way around, because we want a min-heap!
        other.0.partial_cmp(&self.0)
    }
}
impl<U: Ord, T: Eq> Ord for WithOrdRev<U, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
