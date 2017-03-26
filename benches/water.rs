// Lumol, an extensible molecular simulation engine
// Copyright (C) 2015-2016 Lumol's contributors — BSD license

#[macro_use]
extern crate bencher;
extern crate rand;
extern crate lumol;
extern crate lumol_input;

use bencher::Bencher;
use rand::Rng;

use lumol::energy::{Ewald, Wolf, PairRestriction, CoulombicPotential, GlobalPotential};
use lumol::sys::EnergyCache;
use lumol::types::Vector3D;

mod utils;

fn get_ewald() -> Ewald {
    let mut ewald = Ewald::new(8.0, 7);
    ewald.set_restriction(PairRestriction::InterMolecular);
    ewald
}

fn get_wolf() -> Wolf {
    let mut wolf = Wolf::new(9.0);
    wolf.set_restriction(PairRestriction::InterMolecular);
    wolf
}

fn energy_ewald(bencher: &mut Bencher) {
    let system = utils::get_system("water");
    let mut ewald = get_ewald();

    bencher.iter(||{
        let _ = ewald.energy(&system);
    })
}

fn forces_ewald(bencher: &mut Bencher) {
    let system = utils::get_system("water");
    let mut ewald = get_ewald();

    bencher.iter(||{
        let _ = ewald.forces(&system);
    })
}

fn virial_ewald(bencher: &mut Bencher) {
    let system = utils::get_system("water");
    let mut ewald = get_ewald();

    bencher.iter(||{
        let _ = ewald.virial(&system);
    })
}

fn energy_wolf(bencher: &mut Bencher) {
    let system = utils::get_system("water");
    let mut wolf = get_wolf();

    bencher.iter(||{
        let _ = wolf.energy(&system);
    })
}

fn forces_wolf(bencher: &mut Bencher) {
    let system = utils::get_system("water");
    let mut wolf = get_wolf();

    bencher.iter(||{
        let _ = wolf.forces(&system);
    })
}

fn virial_wolf(bencher: &mut Bencher) {
    let system = utils::get_system("water");
    let mut wolf = get_wolf();

    bencher.iter(||{
        let _ = wolf.virial(&system);
    })
}

fn cache_move_particles_wolf(bencher: &mut Bencher) {
    let mut system = utils::get_system("water");
    system.interactions_mut().set_coulomb(Box::new(get_wolf()));

    let mut cache = EnergyCache::new();
    cache.init(&system);

    let mut rng = utils::get_rng(454548784);

    let molecule = rng.choose(system.molecules()).unwrap();
    let mut delta = vec![];
    for i in molecule {
        let position = system[i].position;
        delta.push(position + Vector3D::new(rng.gen(), rng.gen(), rng.gen()));
    }

    bencher.iter(||{
        cache.move_particles_cost(&system, molecule.iter().collect(), &delta)
    })
}

fn cache_move_particles_ewald(bencher: &mut Bencher) {
    let mut system = utils::get_system("water");
    system.interactions_mut().set_coulomb(Box::new(get_ewald()));

    let mut cache = EnergyCache::new();
    cache.init(&system);

    let mut rng = utils::get_rng(9886565);

    let molecule = rng.choose(system.molecules()).unwrap();
    let mut delta = vec![];
    for i in molecule {
        let position = system[i].position;
        delta.push(position + Vector3D::new(rng.gen(), rng.gen(), rng.gen()));
    }

    bencher.iter(||{
        cache.move_particles_cost(&system, molecule.iter().collect(), &delta)
    })
}

fn cache_move_all_rigid_molecules_wolf(bencher: &mut Bencher) {
    let mut system = utils::get_system("water");
    system.interactions_mut().set_coulomb(Box::new(get_wolf()));

    let mut cache = EnergyCache::new();
    cache.init(&system);

    let mut rng = utils::get_rng(3);
    for molecule in system.molecules().to_owned() {
        let delta = Vector3D::new(rng.gen(), rng.gen(), rng.gen());
        for i in molecule {
            system[i].position += delta;
        }
    }

    bencher.iter(||{
        cache.move_all_rigid_molecules_cost(&system)
    })
}

fn cache_move_all_rigid_molecules_ewald(bencher: &mut Bencher) {
    let mut system = utils::get_system("water");
    system.interactions_mut().set_coulomb(Box::new(get_ewald()));

    let mut cache = EnergyCache::new();
    cache.init(&system);

    let mut rng = utils::get_rng(2121);
    for molecule in system.molecules().to_owned() {
        let delta = Vector3D::new(rng.gen(), rng.gen(), rng.gen());
        for i in molecule {
            system[i].position += delta;
        }
    }

    bencher.iter(||{
        cache.move_all_rigid_molecules_cost(&system)
    })
}

benchmark_group!(ewald, energy_ewald, forces_ewald, virial_ewald);
benchmark_group!(wolf, energy_wolf, forces_wolf, virial_wolf);
benchmark_group!(monte_carlo_cache,
    cache_move_particles_wolf, cache_move_particles_ewald,
    cache_move_all_rigid_molecules_wolf, cache_move_all_rigid_molecules_ewald
);

benchmark_main!(ewald, wolf, monte_carlo_cache);
