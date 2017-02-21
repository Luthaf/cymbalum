// Lumol, an extensible molecular simulation engine
// Copyright (C) 2015-2016 Lumol's contributors — BSD license

//! Testing energy of a Lennard-Jones fluid using data from
//! http://www.nist.gov/mml/csd/informatics_research/lj_refcalcs.cfm
extern crate lumol;
extern crate lumol_input as input;

use lumol::sys::{System, UnitCell};
use lumol::sys::Trajectory;

use input::Input;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

pub fn get_system(name: &str) -> System {
    let path = Path::new(file!()).parent().unwrap().join("data")
                                 .join(name);
    return Input::new(path).unwrap().read_system().unwrap();
}

pub trait RoundAt {
    /// Round a float at a given decimal place
    fn round_at(&self, decimal: i32) -> f64;
}

impl RoundAt for f64 {
    fn round_at(&self, decimal: i32) -> f64 {
        let factor = f64::powi(10.0, decimal);
        (self * factor).round() / factor
    }
}

mod cutoff_3_lrc {
    use super::*;
    use lumol::sys::System;
    use lumol::energy::{PairInteraction, LennardJones};


    use std::path::Path;

    enum PairKind {
        None,
        Tail
    }

    fn set_interaction(system: &mut System, kind: PairKind) {
        let mut lj = PairInteraction::new(Box::new(LennardJones{
            epsilon: 1.0, sigma: 1.0
        }), 3.0);

        match kind {
            PairKind::None => {}
            PairKind::Tail => lj.enable_tail_corrections()
        }

        system.interactions_mut().add_pair("X", "X", lj);
    }

    #[test]
    fn nist1() {
        let path = "nist1.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(1), -4351.5);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(2), -568.67);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(2), -198.49);
    }

    #[test]
    fn nist2() {
        let path = "nist2.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(2), -690.00);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(2), -568.46);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(3), -24.230);
    }

    #[test]
    fn nist3() {
        let path = "nist3.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(1), -1146.7);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(1), -1164.9);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(3), -49.622);
    }

    #[test]
    fn nist4() {
        let path = "nist4.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(3), -16.790);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(3), -46.249);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(5), -0.54517);
    }
}

mod cutoff_4_lrc {
    use super::*;
    use lumol::sys::System;
    use lumol::energy::{PairInteraction, LennardJones};

    enum PairKind {
        None,
        Tail
    }

    fn set_interaction(system: &mut System, kind: PairKind) {
        let mut lj = PairInteraction::new(Box::new(LennardJones{
            epsilon: 1.0, sigma: 1.0
        }), 4.0);

        match kind {
            PairKind::None => {}
            PairKind::Tail => lj.enable_tail_corrections()
        }

        system.interactions_mut().add_pair("X", "X", lj);
    }

    #[test]
    fn nist1() {
        let path = "nist1.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(1), -4467.5);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(1), -1263.9);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(3), -83.769);
    }

    #[test]
    fn nist2() {
        let path = "nist2.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(2), -704.60);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(2), -655.99);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(3), -10.226);
    }

    #[test]
    fn nist3() {
        let path = "nist3.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(1), -1175.4);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(1), -1337.1);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(3), -20.942);
    }

    #[test]
    fn nist4() {
        let path = "nist4.toml";
        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::None);

        let energy = system.potential_energy();
        assert_eq!(energy.round_at(3), -17.060);

        let virial = system.virial().trace();
        assert_eq!(virial.round_at(3), -47.869);

        let mut system = get_system(path);
        set_interaction(&mut system, PairKind::Tail);

        let tail = system.potential_energy() - energy;
        assert_eq!(tail.round_at(5), -0.23008);
    }
}
