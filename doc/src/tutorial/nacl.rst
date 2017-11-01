Hello Sodium Chloride
=====================

In this example, we will simulate a Sodium Chloride crystal, using molecular
dynamics to propagate the position throughout time. Sodium Chloride add a
challenge in simulation because each atom carry a charge.  These charges
interact with a Coulomb potential which goes to zero as :math:`1 / r`. The
problem is that the cutoff scheme used for pair potential in most molecular
simulations can not be used for anything that goes to zero slower than :math:`1
/ r^3`. So we need to use alternate methods to compute the potential for the
charges-charges interactions.

For this simulation, you will need the following files:

* the initial configuration ``nacl.xyz```
* the input file ``nacl.toml``

You can download both files :download:`here <../data/nacl.tar.gz>`. Again you
can run the simulation which should complete in a minute with:

::

    lumol nacl.toml

This will perform a molecular dynamics simulation of a NaCl crystal, using
electrostatic interactions between the atomic charges.

The input file commented
------------------------

We start with the input version again:

.. code::

    [input]
    version = 1

Then we load the system from the ``nacl.xyz`` file and define the unit cell.

.. code::

    [[systems]]
    file = "nacl.xyz"
    cell = 22.5608

Here we define some global values for the interactions: setting
``systems.potentials.global.cutoff`` will use the given cutoff for all the pair
interactions. The ``systems.potentials.charges`` section defined the atomic
charges in the system.

.. code::

    [systems.potentials.global]
    cutoff = "8 A"

    [systems.potentials.charges]
    Na = 1.0
    Cl = -1.0

We need to define the pair interactions for all the pair combinations in the
system, *i.e.* (Na, Na); (Cl, Cl); and (Na, Cl).

.. code::

    [[systems.potentials.pairs]]
    atoms = ["Na", "Cl"]
    lj = {sigma = "3.5545 A", epsilon = "0.04425 kcal/mol"}

    [[systems.potentials.pairs]]
    atoms = ["Na", "Na"]
    lj = {sigma = "2.497 A", epsilon = "0.07826 kcal/mol"}

    [[systems.potentials.pairs]]
    atoms = ["Cl", "Cl"]
    lj = {sigma = "4.612 A", epsilon = "0.02502 kcal/mol"}

Because our system have charges, we need to use an electrostatic potential
solver. Here we are going for the ``Wolf`` solver, with a cutoff of 8 A.

.. code::

    [systems.potentials.coulomb]
    wolf = {cutoff = "8 A"}

We can now define the simulation and the outputs for this simulation.

.. code::

    [[simulations]]
    nsteps = 5000
    outputs = [
        {type = "Trajectory", file = "trajectory.xyz", frequency = 10}
    ]

We are using here a molecular dynamics simulation of the NaCl crystal, and a
timestep of 1 fs for integration.

.. code::

    [simulations.propagator]
    type = "MolecularDynamics"
    timestep = "1 fs"

In the next example, we will see how to run a :doc:`water`.
