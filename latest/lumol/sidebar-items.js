initSidebarItems({"enum":[["BondPath","Shortest bond path between two particles in a system"],["CellShape","The shape of a cell determine how we will be able to compute the periodic boundaries condition."],["DegreesOfFreedom","The number of degrees of freedom simulated in a given system"],["OpenMode","Possible modes when opening a `Trajectory`."],["PairRestriction","Possible restrictions on the pair interactions."]],"fn":[["get_atomic_mass","Get the mass of the element with the given atomic `name`"],["read_molecule","Read a the first molecule from the file at `path`. If no bond information exists in the file, bonds are guessed."]],"mod":[["consts","Useful physical constants, expressed in the internal unit system."],["energy","Interaction potentials for energy and forces computations"],["input","This module provide a way to build a Lumol simulation using input files."],["sim","Type and algorithms for simulations"],["sys","Representations of a simulated system"],["types","This module provides complexe numbers; 3D vectors and matrix; and multidimensional arrays for use in all other modules."],["units","This module allow to convert from and to the internal unit system."]],"static":[["VERSION","The full version of the crate, containing git state if available"]],"struct":[["Angle","An `Angle` formed by the particles at indexes `i`, `j` and `k`"],["Array2","Two dimensional tensors, based on ndarray."],["Array3","Three dimensional tensors, based on ndarray"],["Bond","A `Bond` between the particles at indexes `i` and `j`"],["BondDistances","The `BondDistances` bitflag encode the topological distance between two particles in the molecule, i.e. the number of bonds between the particles. Two particles can have multiple bond path lionking them (in the case of cyclic molecules), which is why a bit flag is used instead of a single distance value."],["Bonding","The basic building block for a topology. A `Bonding` contains data about the connectivity (bonds, angles, dihedrals) between particles in a single molecule."],["BornMayerHuggins","Born-Mayer-Huggins potential."],["Buckingham","Buckingham potential."],["Complex","Complex number, with double precision real and imaginary parts."],["Composition","The system composition contains the number of particles of each kind in the system, as well as the number of molecules of each molecule type."],["Configuration","The `Configuration` contains the physical data of the system:"],["CosineHarmonic","Cosine harmonic potential."],["Dihedral","A `Dihedral` angle formed by the particles at indexes `i`, `j`, `k` and `m`"],["EnergyCache","This is a cache for energy computation."],["EnergyEvaluator","An helper struct to evaluate energy components of a system."],["Ewald","Ewald summation for coulombic interactions."],["Gaussian","Gaussian potential."],["Harmonic","Harmonic potential."],["Interactions","The `Interaction` type hold all data about the potentials in the system."],["LennardJones","Lennard-Jones potential."],["Matrix3","A 3x3 square matrix type."],["Mie","Mie potential."],["Molecule","A Molecule associate some particles bonded together."],["MoleculeHash","A molecule hash allow to identify a molecule from its atoms and bonds, and to know wether two molecules are the same without checking each atom and bond."],["MoleculeIter","An iterator over all the molecules in a `Configuration`"],["MoleculeIterMut","A mutable iterator over all the molecules in a `Configuration`"],["MoleculeRef","An analog to [`&Molecule`] using particles stored elsewhere (in a system or an `Molecule`)."],["MoleculeRefMut","An analog to [`&mut Molecule`] using particles stored elsewhere (in a system or an `Molecule`)."],["Morse","Morse potential"],["NullPotential","No-op potential."],["PairInteraction","A non-bonded interaction between two particle."],["Particle","The Particle type hold basic data about a particle in the system. It is self contained, so that it will be easy to send data between parallels processes."],["ParticleKind","A particle kind. Particles with the same name will have the same kind. This is used for faster potential lookup."],["ParticlePtr","An analog of a pointer to `Particle` with struct of array layout."],["ParticlePtrMut","An analog of a mutable pointer to `Particle` with struct of array layout."],["ParticleRef","A reference to a `Particle` with struct of array layout."],["ParticleRefMut","A mutable reference to a `Particle` with struct of array layout."],["ParticleSlice","A slice of `Particle` inside a `ParticleVec` ."],["ParticleSliceMut","A mutable slice of `Particle` inside a `ParticleVec` ."],["ParticleVec","An analog to `Vec<Particle>\n` with Struct of Array (SoA) layout"],["Permutation","The `Permutation` struct contains the old and new particle index in a `Configuration` after the particles where moved due to a new bond being added"],["RestrictionInfo","Restriction information attached to a pair of `Particles` in a `System`."],["SharedEwald","Thread-sade wrapper around Ewald implementing `CoulombicPotential`."],["System","The `System` type hold all the data about a simulated system."],["TableComputation","Computation of a potential using tabulated values."],["Torsion","Torsion potential."],["Trajectory","A Trajectory is a file containing one or more successive simulation steps."],["TrajectoryBuilder","A `Trajectory` builder, to set some options before opening a trajectory."],["TrajectoryError","Error type for Chemfiles."],["UnitCell","An `UnitCell` defines the system physical boundaries."],["Vector3D","A 3-dimensional vector type"],["Wolf","Wolf summation for coulombic interactions."]],"trait":[["AnglePotential","Marker trait for potentials that can be used for molecular angles."],["BondPotential","Marker trait for potentials that can be used for molecular bonds."],["Computation","Alternative energy and forces computation."],["CoulombicPotential","Electrostatic potential solver."],["DihedralPotential","Marker trait for potentials that can be used for molecular dihedral angles."],["GlobalCache","Energetic cache for global potentials."],["GlobalPotential","A potential acting on the whole System at once."],["PairPotential","Marker trait for potentials that can be used for non-bonded two body interactions."],["Potential","A potential for force and energy computations."]]});