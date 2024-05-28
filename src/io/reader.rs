use std::fs::File;

use crate::structs::ExpressionStruct;
use crate::structs::HalideData;
use crate::structs::Rule;

use csv;

/// Reads expressions from a csv file into an `ExpressionStruct` Vector.
#[allow(dead_code)]
pub fn read_expressions(file_path: &str) -> anyhow::Result<Vec<ExpressionStruct>> {
    // Declare the vector and the reader
    let mut expressions_vect = Vec::new();
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    // Read each record and extract then cast the values.
    for result in rdr.records() {
        // get the String of the value
        let record = result?;
        let index: i32 = record
            .get(0)
            .unwrap()
            .parse::<i32>()
            .expect("No index means csv is broken.");
        let expression = &record.get(1).expect("No expression means csv is broken.");
        // Check if Halide's resluts are included then add them if they are
        let halide_result =
            if let (Some(halide_result), Some(halide_time)) = (record.get(2), record.get(3)) {
                Some(HalideData {
                    halide_result: halide_result.into(),
                    halide_time: halide_time.parse()?,
                })
            } else {
                None
            };

        // Push the new ExpressionStruct initialized with the values extracted into the vector.
        expressions_vect.push(ExpressionStruct::new(
            index,
            (*expression).to_string(),
            halide_result,
        ));
    }
    Ok(expressions_vect)
}

/// Reads the expressions in the format specified for the work done for the paper variant.
#[allow(dead_code)]
pub fn read_expressions_paper(file_path: &str) -> anyhow::Result<Vec<(String, String)>> {
    let mut expressions_vect = Vec::new();
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    for result in rdr.records() {
        let record = result?;
        let infix = record[0].to_string();
        let prefix = record[1].to_string();
        expressions_vect.push((infix, prefix));
    }
    Ok(expressions_vect)
}

/// Reads the rules from a CSV file then pareses them into a Rule Vector.
#[allow(dead_code)]
pub fn read_rules(file_path: &str) -> anyhow::Result<Vec<Rule>> {
    let mut rules_vect: Vec<Rule> = Vec::new();
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        let index: i32 = record[0].parse::<i32>().unwrap();
        let lhs = record[2].to_string();
        let rhs = record[3].to_string();
        let condition = record[4].to_string();
        rules_vect.push(Rule::new(index, lhs, rhs, Some(condition)));
    }
    Ok(rules_vect)
}

// Gets the nth argument from the command line.
// pub fn get_nth_arg(n: usize) -> Result<OsString, Box<dyn Error>> {
//     match env::args_os().nth(n) {
//         None => Err(From::from("expected 1 argument, but got none")),
//         Some(file_path) => Ok(file_path),
//     }
// }

// Gets the params passed to the runner from the command line
// #[allow(clippy::unnecessary_wraps)]
// pub fn get_runner_params(start: usize) -> Result<(usize, usize, f64), Box<dyn Error>> {
//     // Get the number of iterations from the command line else initialize it to a default value
//     let iter = match env::args_os().nth(start) {
//         None => 30,
//         Some(i) => i.into_string().unwrap().parse::<usize>().unwrap(),
//     };

//     // Get the number of nodes from the command line else initialize it to a default value
//     let nodes = match env::args_os().nth(start + 1) {
//         None => 10000,
//         Some(i) => i.into_string().unwrap().parse::<usize>().unwrap(),
//     };

//     // Get the timelimit from the command line else initialize it to a default value
//     let time = match env::args_os().nth(start + 2) {
//         None => 3.0,
//         Some(i) => i.into_string().unwrap().parse::<f64>().unwrap(),
//     };

//     Ok((iter, nodes, time))
// }

// Reads the start and end expressions from the exprs file in the tmp folder (used for quick testing)
// pub fn get_start_end() -> Result<(String, String), Box<dyn Error>> {
//     let mut file = File::open("./tmp/exprs.txt")?;
//     let mut s = String::new();
//     file.read_to_string(&mut s)?;
//     let v: Vec<&str> = s.split('\n').collect();
//     Ok((v[0].to_string(), v[1].to_string()))
// }
