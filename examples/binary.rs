// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

//! Monte Carlo simulation of a binary mixture of H20 and CO2.
extern crate lumol;
extern crate lumol_input as input;

use lumol::sim::Simulation;
use lumol::sim::mc::{MonteCarlo, Rotate, Translate};
use lumol::sys::{Molecule, Particle, TrajectoryBuilder};
use lumol::sys::read_molecule;
use lumol::units;

use input::InteractionsInput;

fn main() -> Result<(), Box<std::error::Error>> {
    let mut system = TrajectoryBuilder::new().open("data/binary.pdb")?
                                             .read()?;
    let input = InteractionsInput::new("data/binary.toml")?;
    input.read(&mut system)?;

    // We can read files to get molecule hash
    let co2 = read_molecule("data/CO2.pdb")?.hash();

    // Or define a new molecule by hand
    let mut molecule = Molecule::new(Particle::new("H"));
    molecule.add_particle_bonded_to(0, Particle::new("O"));
    molecule.add_particle_bonded_to(1, Particle::new("H"));
    let h2o = molecule.hash();

    let mut mc = MonteCarlo::new(units::from(500.0, "K")?);

    // Use the molecular types of CO2 and H2O to specify different probabilities
    mc.add(Box::new(Translate::new(units::from(0.5, "A")?, co2)), 1.0);
    mc.add(Box::new(Rotate::new(units::from(10.0, "deg")?, co2)), 1.0);

    mc.add(Box::new(Translate::new(units::from(10.0, "A")?, h2o)), 2.0);
    mc.add(Box::new(Rotate::new(units::from(20.0, "deg")?, h2o)), 2.0);

    let mut simulation = Simulation::new(Box::new(mc));
    simulation.run(&mut system, 200_000_000);

    Ok(())
}
