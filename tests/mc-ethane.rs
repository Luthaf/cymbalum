// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license
use lumol::input::Input;
use lumol::units;

use std::path::Path;
use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;

mod utils;

#[test]
fn constant_pressure() {
    START.call_once(::env_logger::init);
    let path = Path::new(file!()).parent()
                                 .unwrap()
                                 .join("data")
                                 .join("mc-ethane")
                                 .join("npt.toml");
    let mut config = Input::new(path).unwrap().read().unwrap();

    let collecter = utils::Collecter::starting_at((config.nsteps - 50_000) as u64);
    let pressures = collecter.pressures();

    config.simulation.add_output(Box::new(collecter));
    config.simulation.run(&mut config.system, config.nsteps);

    let pressure = utils::mean(pressures.clone());
    let expected = units::from(200.0, "bar").unwrap();
    let tolerance = units::from(200.0, "bar").unwrap();
    assert!(f64::abs(pressure - expected) < tolerance);
}
