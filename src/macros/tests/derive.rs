#[macro_use]
extern crate lumol_macros;

#[derive(Debug, StructOfArray)]
struct Particle {
    name: String,
    mass: f64
}

impl Particle {
    pub fn new(name: String, mass: f64) -> Self {
        Particle {
            name: name,
            mass: mass,
        }
    }
}

#[test]
fn push() {
    let mut particles = ParticleVec::new();
    particles.push(Particle::new(String::from("Na"), 56.0));

    assert_eq!(particles.name[0], "Na");
    assert_eq!(particles.mass[0], 56.0);
}

#[test]
fn len() {
    let mut particles = ParticleVec::new();
    assert_eq!(particles.len(), 0);

    particles.push(Particle::new(String::from("Na"), 56.0));
    assert_eq!(particles.len(), 1);
}
