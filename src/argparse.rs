use clap::Args;
use clap::Parser;
use clap::Subcommand;

/// Parser for the cli options
// Example args:
// cargo run --release dataset `results/expressions_egg.csv` 1000000 10000000 5 5 3 0 4
// cargo run --release `prove_exprs_passes` `data/prefix/expressions_egg.csv` 10000000 10000000 3 $i
// cargo run --release `prove_exprs_fast` `data/prefix/expressions_egg.csv` 10000000 10000000 3
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Name of the person to greet
    #[command(subcommand)]
    pub mode: Mode,

    #[command(flatten)]
    pub params: Params,
}

// Parameters shared by all subcommands
#[derive(Args, Debug)]
pub struct Params {
    #[arg(short, long)]
    pub expressions_file: String,

    #[arg(short, long, default_value_t = 30)]
    pub iter: usize,

    #[arg(short, long, default_value_t = 10000)]
    pub nodes: usize,

    #[arg(short, long, default_value_t = 3.0)]
    pub time: f64,
}
/// Parameters shared by all prove subcommands
#[derive(Args, Debug)]
pub struct ProveParams {
    #[arg(short, long, default_value_t = true)]
    pub use_iteration_check: bool,

    #[arg(short, long, default_value_t = true)]
    pub report: bool,
}

/// Subcommands for the specific modes
#[derive(Subcommand, Debug)]
pub enum ProveStrategy {
    /// Prove expressions using Caviar with/without ILC
    Simple,
    /// Prove expressions using Caviar with pulses.
    Pulse {
        #[arg(short, long)]
        threshold: f64,
    },
    /// Prove expressions using Caviar with NPP.
    Npp,
    /// Prove expressions using Caviar with Pulses and NPP and with pulses.
    PulseNpp {
        #[arg(short, long)]
        threshold: f64,
    },
    /// Prove expressions using Caviar with clusters of rules and with pulses.
    Clusters {
        #[arg(short, long)]
        classes_file: String,
        #[arg(short, long)]
        iterations_count: usize,
    },
}

/// Subcommands for the specific modes
#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Generates a dataset for minimum rulesets needed for each expression from the expressions file passed as argument
    Dataset {
        #[arg(short, long)]
        reorder_count: usize,
        #[arg(short, long)]
        batch_size: usize,
        #[arg(short, long)]
        continoue_from_expr: usize,
        #[arg(short, long)]
        cores: usize,
    },
    /// Prove expressions
    Prove {
        #[command(subcommand)]
        strategy: ProveStrategy,
        #[command(flatten)]
        prove_params: ProveParams,
    },
    /// Simplify Expression
    Simplify {
        #[arg(short, long, default_value_t = true)]
        report: bool,
    },
}
