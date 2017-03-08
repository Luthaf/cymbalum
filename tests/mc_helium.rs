// Lumol, an extensible molecular simulation engine
// Copyright (C) 2015-2016 Lumol's contributors — BSD license

//! Testing physical properties of a Lennard-Jones gaz of Helium using
//! Monte-Carlo simulation
extern crate lumol;
extern crate lumol_input as input;

use lumol::Logger;
use lumol::sys::{System, Trajectory, UnitCell};
use lumol::energy::{LennardJones, PairInteraction};
use lumol::sim::Simulation;
use lumol::sim::mc::{MonteCarlo, Translate};
use lumol::units;
use lumol::consts::K_BOLTZMANN;

use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;

use std::path::Path;

fn get_system() -> System {
    let data_dir = Path::new(file!()).parent().unwrap();
    let configuration = data_dir.join("data").join("helium.xyz");
    let mut system = Trajectory::open(configuration)
                                .and_then(|mut traj| traj.read())
                                .unwrap();
    system.set_cell(UnitCell::cubic(10.0));

    let lj = Box::new(LennardJones{
        sigma: units::from(2.0, "A").unwrap(),
        epsilon: units::from(0.2, "kJ/mol").unwrap()
    });
    system.interactions_mut().add_pair("He", "He", PairInteraction::new(lj, 5.0));
    return system;
}

#[test]
fn perfect_gaz() {
    START.call_once(|| {Logger::stdout();});
    let mut system = get_system();

    let temperature = units::from(300.0, "K").unwrap();
    let mut mc = MonteCarlo::new(temperature);
    mc.add(Box::new(Translate::new(units::from(3.0, "A").unwrap())), 1.0);
    let mut simulation = Simulation::new(Box::new(mc));

    // dilating the system!
    for particle in system.iter_mut() {
        particle.position = 36.0 * particle.position;
    }
    system.set_cell(UnitCell::cubic(60.0));

    simulation.run(&mut system, 5000);
    let pressure = system.pressure();
    let volume = system.volume();

    let pv = pressure * volume;
    let nkt = system.size() as f64 * K_BOLTZMANN * temperature;
    assert!(f64::abs(pv - nkt) / pv < 2e-2);
}
