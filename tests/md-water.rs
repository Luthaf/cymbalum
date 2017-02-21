// Lumol, an extensible molecular simulation engine
// Copyright (C) 2015-2016 Lumol's contributors — BSD license

//! Testing physical properties of f-SPC water
extern crate lumol;
extern crate lumol_input as input;

use lumol::Logger;

use input::Input;

use std::path::Path;
use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;


#[test]
fn constant_energy_ewald() {
    START.call_once(|| {Logger::stdout();});
    let path = Path::new(file!()).parent().unwrap().join("data")
                                 .join("md-water-ewald.toml");

    let mut config = Input::new(path).unwrap().read().unwrap();

    let e_initial = config.system.total_energy();
    config.simulation.run(&mut config.system, config.nsteps);
    let e_final = config.system.total_energy();

    // TODO: use a better thresold when updating Ewald to work with triclinic
    // cells.
    assert!(f64::abs((e_initial - e_final)/e_final) < 1e-1);
}

#[test]
fn constant_energy_wolf() {
    START.call_once(|| {Logger::stdout();});
    let path = Path::new(file!()).parent().unwrap().join("data")
                                 .join("md-water-ewald.toml");

    let mut config = Input::new(path).unwrap().read().unwrap();

    let e_initial = config.system.total_energy();
    config.simulation.run(&mut config.system, config.nsteps);
    let e_final = config.system.total_energy();
    assert!(f64::abs((e_initial - e_final)/e_final) < 3e-2);
}
