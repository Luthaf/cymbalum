// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license
use std::path::PathBuf;
use toml::value::Table;

use lumol::out::*;

use super::Input;
use FromToml;
use error::{Error, Result};
use extract;

impl Input {
    /// Get the the simulation outputs.
    pub(crate) fn read_outputs(&self) -> Result<Vec<(Box<Output>, u64)>> {
        let config = try!(self.simulation_table());
        if let Some(outputs) = config.get("outputs") {
            let outputs = try!(
                outputs.as_array()
                       .ok_or(Error::from("'outputs' must be an array of tables in simulation"))
            );

            let mut result = Vec::new();
            for output in outputs {
                let output = try!(
                    output.as_table()
                          .ok_or(Error::from("'outputs' must be an array of tables in simulation"))
                );

                let frequency = match output.get("frequency") {
                    Some(frequency) => {
                        try!(frequency.as_integer().ok_or(
                            Error::from("'frequency' must be an integer in output")
                        )) as u64
                    }
                    None => 1u64,
                };

                let typ = try!(extract::typ(output, "output"));
                let output: Box<Output> = match &*typ.to_lowercase() {
                    "trajectory" => Box::new(try!(TrajectoryOutput::from_toml(output))),
                    "properties" => Box::new(try!(PropertiesOutput::from_toml(output))),
                    "energy" => Box::new(try!(EnergyOutput::from_toml(output))),
                    "stress" => Box::new(try!(StressOutput::from_toml(output))),
                    "forces" => Box::new(try!(ForcesOutput::from_toml(output))),
                    "cell" => Box::new(try!(CellOutput::from_toml(output))),
                    "custom" => Box::new(try!(CustomOutput::from_toml(output))),
                    other => return Err(Error::from(format!("Unknown output type '{}'", other))),
                };

                result.push((output, frequency));
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
}

fn get_file(config: &Table) -> Result<&str> {
    let file = try!(config.get("file").ok_or(Error::from("Missing 'file' key in output")));

    file.as_str().ok_or(Error::from("'file' must be a string in output"))
}

impl FromToml for TrajectoryOutput {
    fn from_toml(config: &Table) -> Result<TrajectoryOutput> {
        let path = try!(get_file(config));
        let output = try!(TrajectoryOutput::new(path));
        Ok(output)
    }
}

impl FromToml for CellOutput {
    fn from_toml(config: &Table) -> Result<CellOutput> {
        let path = try!(get_file(config));
        let output = try_io!(CellOutput::new(path), PathBuf::from(path));
        Ok(output)
    }
}

impl FromToml for EnergyOutput {
    fn from_toml(config: &Table) -> Result<EnergyOutput> {
        let path = try!(get_file(config));
        let output = try_io!(EnergyOutput::new(path), PathBuf::from(path));
        Ok(output)
    }
}

impl FromToml for PropertiesOutput {
    fn from_toml(config: &Table) -> Result<PropertiesOutput> {
        let path = try!(get_file(config));
        let output = try_io!(PropertiesOutput::new(path), PathBuf::from(path));
        Ok(output)
    }
}

impl FromToml for StressOutput {
    fn from_toml(config: &Table) -> Result<StressOutput> {
        let path = try!(get_file(config));
        let output = try_io!(StressOutput::new(path), PathBuf::from(path));
        Ok(output)
    }
}

impl FromToml for ForcesOutput {
    fn from_toml(config: &Table) -> Result<ForcesOutput> {
        let path = try!(get_file(config));
        let output = try_io!(ForcesOutput::new(path), PathBuf::from(path));
        Ok(output)
    }
}

impl FromToml for CustomOutput {
    fn from_toml(config: &Table) -> Result<CustomOutput> {
        let path = try!(get_file(config));
        let template = try!(extract::str("template", config, "custom output"));
        let output = try_io!(CustomOutput::new(path, template), PathBuf::from(path));
        Ok(output)
    }
}
