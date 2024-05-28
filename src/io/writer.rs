use csv::Writer;

use crate::structs::{PaperResult, ResultStructure};

///Writes the results (a vector of `ResultStructure`) into a CSV file.
#[allow(dead_code)]
pub fn write_results(path: &str, results: &Vec<ResultStructure>) -> anyhow::Result<()> {
    let mut wtr = Writer::from_path(path)?;

    for result in results {
        wtr.serialize(result)?;
    }
    wtr.flush()?;
    Ok(())
}

///Writes the paper results (a vector of `PaperResult`) into a CSV file.
#[allow(dead_code)]
pub fn write_results_paper(path: &str, results: &Vec<PaperResult>) -> anyhow::Result<()> {
    let mut wtr = Writer::from_path(path)?;

    for result in results {
        wtr.serialize(result)?;
    }

    wtr.flush()?;

    Ok(())
}
