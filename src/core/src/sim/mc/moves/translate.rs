// Lumol, an extensible molecular simulation engine
// Copyright (C) 2015-2016 G. Fraux — BSD license

use rand::distributions::{Sample, Range};
use rand::Rng;

use std::usize;
use std::f64;

use super::MCMove;
use super::select_molecule;

use types::Vector3D;
use sys::{System, EnergyCache};

/// Monte Carlo move for translating a molecule
pub struct Translate {
    /// Type of molecule to translate. `None` means all molecules.
    moltype: Option<u64>,
    /// Index of the molecule to translate
    molid: usize,
    /// New positions of the atom in the translated molecule
    newpos: Vec<Vector3D>,
    /// Maximum displacement value
    dr: f64,
    /// The maximum value must not exceed this value
    dr_max: f64,
    /// Translation range for random number generation
    range: Range<f64>,
}

impl Translate {
    /// Create a new `Translate` move, with maximum displacement of `dr`.
    /// Translating all the molecules in the system.
    pub fn new(dr: f64) -> Translate {
        Translate::create(dr, None)
    }

    /// Create a new `Translate` move, with maximum displacement of `dr`.
    /// Translating only molecules with `moltype` type.
    pub fn with_moltype(dr: f64, moltype: u64) -> Translate {
        Translate::create(dr, Some(moltype))
    }

    /// Factorizing the constructors
    fn create(dr: f64, moltype: Option<u64>) -> Translate {
        assert!(dr > 0.0, "dr must be positive in Translate move");
        let dr = dr / f64::sqrt(3.0);
        Translate {
            moltype: moltype,
            molid: usize::MAX,
            newpos: Vec::new(),
            dr: dr,
            dr_max: 0.0,
            range: Range::new(-dr, dr),
        }
    }
}

impl Default for Translate {
    fn default() -> Translate {
        Translate::new(1.0)
    }
}

impl MCMove for Translate {
    fn describe(&self) -> &str {
        "molecular translation"
    }

    fn setup(&mut self, system: &System) {
        // Limit the displacement range to the maximum cutoff
        self.dr_max = system.interactions()
                            .all_pairs()
                            .iter()
                            .map(|i| i.cutoff())
                            .fold(f64::NAN, f64::max);
        if self.dr > self.dr_max {
            self.dr = self.dr_max
        }
    }

    fn prepare(&mut self, system: &mut System, rng: &mut Box<Rng>) -> bool {
        if let Some(id) = select_molecule(system, self.moltype, rng) {
            self.molid = id;
        } else {
            warn!("Can not translate molecule: no molecule of this type in the system.");
            return false;
        }

        // Create random displacement vector.
        let delta = Vector3D::new(
            self.range.sample(rng),
            self.range.sample(rng),
            self.range.sample(rng)
        );

        self.newpos.clear();
        // Create new position.
        // Note that this may move a particles' center-
        // of-mass (com) out of the cell.
        // If the move is accepted, we have to wrap the
        // com such that it lies inside the cell.
        for i in system.molecule(self.molid) {
            self.newpos.push(system[i].position + delta);
        }
        return true;
    }

    fn cost(&self, system: &System, beta: f64, cache: &mut EnergyCache) -> f64 {
        let idxes = system.molecule(self.molid).iter().collect::<Vec<_>>();
        let cost = cache.move_particles_cost(system, idxes, &self.newpos);
        return cost*beta;
    }

    fn apply(&mut self, system: &mut System) {
        // Update positions.
        for (i, pi) in system.molecule(self.molid).iter().enumerate() {
            system[pi].position = self.newpos[i];
        }
        // Move molecule such that its center-of-mass
        // is inside the simulation cell.
        // Note that particles of the molecule may still be
        // outside the cell, but that is not important.
        system.wrap_molecule(self.molid)
    }

    fn restore(&mut self, _: &mut System) {
        // Nothing to do.
    }

    fn update_amplitude(&mut self, scaling_factor: Option<f64>) {
        if let Some(s) = scaling_factor {
            if (self.dr * s) < self.dr_max {
                self.dr *= s;
                self.range = Range::new(-self.dr, self.dr);
            } else {
                warn_once!(
                    "Tried to increase the maximum amplitude for translations to more than the set maximum."
                );
            }
        };
    }
}
