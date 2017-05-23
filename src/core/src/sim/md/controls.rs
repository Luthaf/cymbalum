// Lumol, an extensible molecular simulation engine
// Copyright (C) Lumol's contributors — BSD license

//! While running a simulation, we often want to have control over some
//! simulation parameters: the temperature, the pressure, etc. This is the goal
//! of the control algorithms, all implementing of the `Control` trait.
use types::{Matrix3, Vector3D, Zero};
use sys::System;
use sys::veloc;
use sim::Alternator;

/// Trait for controlling some parameters in a system during a simulation.
pub trait Control {
    /// Function called once at the beginning of the simulation, which allow
    /// for some setup of the control algorithm if needed.
    fn setup(&mut self, _: &System) {}

    /// Do your job, control algorithm!
    fn control(&mut self, system: &mut System);

    /// Function called once at the end of the simulation.
    fn finish(&mut self, _: &System) {}
}

/// Trait for controls usable as thermostats
pub trait Thermostat: Control {}

/******************************************************************************/
/// Velocity rescaling thermostat.
///
/// This algorithm controls the temperature by rescaling all the velocities when
/// the temperature differs exceedingly from the desired temperature. A
/// tolerance parameter prevent this algorithm from running too often: if
/// tolerance is 10K and the target temperature is 300K, the algorithm will only
/// run if the instant temperature is below 290K or above 310K.
pub struct RescaleThermostat {
    /// Target temperature
    temperature: f64,
    /// Tolerance in temperature
    tol: f64,
}

impl RescaleThermostat {
    /// Create a new `RescaleThermostat` acting at temperature `temperature`, with a
    /// tolerance of `5% * temperature`.
    pub fn new(temperature: f64) -> RescaleThermostat {
        assert!(temperature >= 0.0, "The temperature must be positive in thermostats.");
        let tol = 0.05 * temperature;
        RescaleThermostat::with_tolerance(temperature, tol)
    }

    /// Create a new `RescaleThermostat` acting at temperature `T`, with a
    /// tolerance of `tol`. For rescaling all the steps, use `tol = 0`.
    pub fn with_tolerance(temperature: f64, tol: f64) -> RescaleThermostat {
        RescaleThermostat{temperature: temperature, tol: tol}
    }
}

impl Control for RescaleThermostat {
    fn control(&mut self, system: &mut System) {
        let instant_temperature = system.temperature();
        if f64::abs(instant_temperature - self.temperature) > self.tol {
            veloc::scale(system, self.temperature);
        }
    }
}

impl Thermostat for RescaleThermostat {}

/******************************************************************************/
/// Berendsen thermostat.
///
/// The Berendsen thermostat sets the simulation temperature by exponentially
/// relaxing to a desired temperature. A more complete description of this
/// algorithm can be found in the original article [1].
///
/// [1] H.J.C. Berendsen, et al. J. Chem Phys 81, 3684 (1984); doi: 10.1063/1.448118
pub struct BerendsenThermostat {
    /// Target temperature
    temperature: f64,
    /// Timestep of the thermostat, expressed as a multiplicative factor of the
    /// integrator timestep.
    tau: f64,
}

impl BerendsenThermostat {
    /// Create a new `BerendsenThermostat` acting at temperature `T`, with a
    /// timestep of `tau` times the integrator timestep.
    pub fn new(temperature: f64, tau: f64) -> BerendsenThermostat {
        assert!(temperature >= 0.0, "The temperature must be positive in thermostats.");
        assert!(tau >= 0.0, "The timestep must be positive in berendsen thermostat.");
        BerendsenThermostat{temperature: temperature, tau: tau}
    }
}

impl Control for BerendsenThermostat {
    fn control(&mut self, system: &mut System) {
        let instant_temperature = system.temperature();
        let factor = f64::sqrt(1.0 + 1.0 / self.tau * (self.temperature / instant_temperature - 1.0));
        for particle in system {
            particle.velocity *= factor;
        }
    }
}
impl Thermostat for BerendsenThermostat {}

/******************************************************************************/

impl<T> Control for Alternator<T> where T: Control {

    fn control(&mut self, system: &mut System) {
        if self.can_run() {
            self.as_mut().control(system)
        }
    }
}

/// Remove global translation from the system
pub struct RemoveTranslation;

impl RemoveTranslation {
    /// Create a new `RemoveTranslation` control.
    pub fn new() -> RemoveTranslation {
        RemoveTranslation
    }
}

impl Control for RemoveTranslation {
    fn control(&mut self, system: &mut System) {
        let total_mass = system.iter().fold(0.0, |total_mass, particle| total_mass + particle.mass);

        let total_velocity = system.iter().fold(
            Vector3D::zero(),
            |total_velocity, particle| total_velocity + particle.velocity * particle.mass / total_mass
        );

        for particle in system {
            particle.velocity -= total_velocity;
        }
    }
}

/******************************************************************************/
/// Remove global rotation from the system
pub struct RemoveRotation;

impl RemoveRotation {
    /// Create a new `RemoveRotation` control.
    pub fn new() -> RemoveRotation {
        RemoveRotation
    }
}

impl Control for RemoveRotation {
    fn control(&mut self, system: &mut System) {
        // Center-of-mass
        let com = system.center_of_mass();

        // Angular momentum
        let moment = system.iter().fold(
            Vector3D::zero(),
            |moment, particle| {
                let delta = particle.position - com;
                moment + particle.mass * (delta ^ particle.velocity)
            }
        );

        let mut inertia = system.iter().fold(
            Matrix3::zero(),
            |inertia, particle| {
                let delta = particle.position - com;
                inertia - particle.mass * delta.tensorial(&delta)
            }
        );

        let trace = inertia.trace();
        inertia[(0, 0)] += trace;
        inertia[(1, 1)] += trace;
        inertia[(2, 2)] += trace;

        // The angular velocity omega is defined by `L = I w` with L the angular
        // momentum, and I the inertia matrix.
        let angular = inertia.inverse() * moment;
        for particle in system {
            let delta = particle.position - com;
            particle.velocity -= delta ^ angular;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sys::*;
    use sys::veloc::{BoltzmannVelocities, InitVelocities};
    use types::*;
    use sim::Alternator;

    fn testing_system() -> System {
        let mut system = System::with_cell(UnitCell::cubic(20.0));;

        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    let mut p = Particle::new("Cl");
                    p.position = Vector3D::new(
                        i as f64 * 2.0, j as f64 * 2.0, k as f64 * 2.0
                    );
                    system.add_particle(p);
                }
            }
        }

        let mut velocities = BoltzmannVelocities::new(300.0);
        velocities.init(&mut system);
        return system;
    }

    #[test]
    fn rescale_thermostat() {
        let mut system = testing_system();
        let temperature = system.temperature();
        assert_ulps_eq!(temperature, 300.0, epsilon=1e-12);

        let mut thermostat = RescaleThermostat::with_tolerance(250.0, 100.0);
        thermostat.control(&mut system);
        let temperature = system.temperature();
        assert_ulps_eq!(temperature, 300.0, epsilon=1e-12);

        let mut thermostat = RescaleThermostat::with_tolerance(250.0, 10.0);
        thermostat.control(&mut system);
        let temperature = system.temperature();
        assert_ulps_eq!(temperature, 250.0, epsilon=1e-12);
    }

    #[test]
    fn berendsen_thermostat() {
        let mut system = testing_system();
        let temperature = system.temperature();
        assert_ulps_eq!(temperature, 300.0, epsilon=1e-9);

        let mut thermostat = BerendsenThermostat::new(250.0, 100.0);
        for _ in 0..3000 {
            thermostat.control(&mut system);
        }
        let temperature = system.temperature();
        assert_ulps_eq!(temperature, 250.0, epsilon=1e-9);
    }

    #[test]
    #[should_panic]
    fn negative_temperature_rescale() {
        let _ = RescaleThermostat::new(-56.0);
    }

    #[test]
    #[should_panic]
    fn negative_temperature_berendsen() {
        let _ = BerendsenThermostat::new(-56.0, 1000.0);
    }

    #[test]
    fn remove_translation() {
        let mut system = System::with_cell(UnitCell::cubic(20.0));
        system.add_particle(Particle::new("Ag"));
        system.add_particle(Particle::new("Ag"));
        system[0].position = Vector3D::zero();
        system[1].position = Vector3D::new(1.0, 1.0, 1.0);
        system[0].velocity = Vector3D::new(1.0, 2.0, 0.0);
        system[1].velocity = Vector3D::new(1.0, 0.0, 0.0);

        let mut control = Alternator::new(4, RemoveTranslation::new());

        // The three first controls do nothing
        let vel_0 = system[0].velocity;
        let vel_1 = system[1].velocity;
        for _ in 0..3 {
            control.control(&mut system);
            assert_ulps_eq!(system[0].velocity, vel_0);
            assert_ulps_eq!(system[1].velocity, vel_1);
        }

        // The fourth one removes global translation
        control.control(&mut system);
        assert_ulps_eq!(system[0].velocity, Vector3D::new(0.0, 1.0, 0.0));
        assert_ulps_eq!(system[1].velocity, Vector3D::new(0.0, -1.0, 0.0));
    }

    #[test]
    fn remove_rotation() {
        let mut system = System::with_cell(UnitCell::cubic(20.0));
        system.add_particle(Particle::new("Ag"));
        system.add_particle(Particle::new("Ag"));
        system[0].position = Vector3D::zero();
        system[1].position = Vector3D::new(1.0, 0.0, 0.0);
        system[0].velocity = Vector3D::new(0.0, 1.0, 0.0);
        system[1].velocity = Vector3D::new(0.0, -1.0, 2.0);

        let mut control = Alternator::new(4, RemoveRotation::new());

        // The three first controls do nothing
        let vel_0 = system[0].velocity;
        let vel_1 = system[1].velocity;
        for _ in 0..3 {
            control.control(&mut system);
            assert_ulps_eq!(system[0].velocity, vel_0);
            assert_ulps_eq!(system[1].velocity, vel_1);
        }

        // The fourth one removes global rotation
        control.control(&mut system);
        assert_ulps_eq!(system[0].velocity, Vector3D::new(0.0, 0.0, 1.0));
        assert_ulps_eq!(system[1].velocity, Vector3D::new(0.0, 0.0, 1.0));
    }
}
