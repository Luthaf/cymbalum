// Lumol, an extensible molecular simulation engine
// Copyright (C) 2015-2016 Lumol's contributors — BSD license

//! Testing molecular dynamics of methane
extern crate lumol;
extern crate lumol_input as input;

use lumol::Logger;
use lumol::sys::{System, UnitCell, Trajectory};
use lumol::sys::veloc::{BoltzmannVelocities, InitVelocities};
use lumol::sim::{Simulation, MolecularDynamics};
use lumol::units;

use input::InteractionsInput;

use std::path::Path;
use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;

fn setup_system() -> System {
    let data_dir = Path::new(file!()).parent().unwrap();
    let configuration = data_dir.join("data").join("methane.xyz");
    let mut system = Trajectory::open(configuration)
                                .and_then(|mut traj| traj.read_guess_bonds())
                                .unwrap();
    system.set_cell(UnitCell::cubic(20.0));

    let interactions = data_dir.join("data").join("methane.toml");
    let input = InteractionsInput::new(interactions).unwrap();
    input.read(&mut system).unwrap();

    let mut velocities = BoltzmannVelocities::new(units::from(300.0, "K").unwrap());
    velocities.init(&mut system);

    return system;
}

#[test]
fn bonds_detection() {
    START.call_once(|| {Logger::stdout();});
    let system = setup_system();
    assert_eq!(system.molecules().len(), 150);

    for molecule in system.molecules() {
        assert_eq!(molecule.bonds().len(), 4);
        assert_eq!(molecule.angles().len(), 6);
        assert_eq!(molecule.dihedrals().len(), 0);
    }
}

#[test]
fn constant_energy() {
    START.call_once(|| {Logger::stdout();});
    let mut system = setup_system();

    let mut simulation = Simulation::new(Box::new(
        MolecularDynamics::new(units::from(1.0, "fs").unwrap())
    ));

    let e_initial = system.total_energy();
    simulation.run(&mut system, 500);
    let e_final = system.total_energy();
    assert!(f64::abs((e_initial - e_final)/e_final) < 1e-2);
}
