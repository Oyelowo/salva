use crate::boundary::{Boundary, BoundaryHandle};
use crate::fluid::Fluid;
use crate::geometry::{ContactManager, ParticlesContacts};
use crate::math::Vector;
use crate::solver::PBFSolver;
use crate::TimestepManager;
use na::RealField;

#[cfg(feature = "nphysics")]
use {
    crate::coupling::ColliderCouplingManager,
    nphysics::object::{BodySet, ColliderSet},
};

pub struct LiquidWorld<N: RealField> {
    particle_radius: N,
    h: N,
    fluids: Vec<Fluid<N>>,
    boundaries: Vec<Boundary<N>>,
    solver: PBFSolver<N>,
    contact_manager: ContactManager<N>,
    timestep_manager: TimestepManager<N>,
}

impl<N: RealField> LiquidWorld<N> {
    pub fn new(particle_radius: N, smoothing_factor: N) -> Self {
        Self {
            particle_radius,
            h: particle_radius * smoothing_factor * na::convert(2.0),
            fluids: Vec::new(),
            boundaries: Vec::new(),
            solver: PBFSolver::new(),
            contact_manager: ContactManager::new(),
            timestep_manager: TimestepManager::new(),
        }
    }

    pub fn step(&mut self, dt: N, gravity: &Vector<N>) {
        let step_start_time = instant::now();
        let mut remaining_time = dt;

        self.solver.step(
            dt,
            &self.timestep_manager,
            &mut self.contact_manager,
            gravity,
            self.h,
            self.particle_radius,
            &mut self.fluids,
            &self.boundaries,
        );

        println!("Total step time: {}ms", instant::now() - step_start_time);
    }

    #[cfg(feature = "nphysics")]
    pub fn step_with_coupling<Bodies, Colliders>(
        &mut self,
        dt: N,
        gravity: &Vector<N>,
        coupling: &mut ColliderCouplingManager<N, Colliders::Handle>,
        bodies: &mut Bodies,
        colliders: &mut Colliders,
    ) where
        Bodies: BodySet<N>,
        Colliders: ColliderSet<N, Bodies::Handle>,
    {
        coupling.update_boundaries(&mut self.boundaries, &self.fluids, colliders);
        self.step(dt, gravity);
        coupling.transmit_forces(&mut self.boundaries, &self.fluids, bodies, colliders);
    }

    pub fn add_fluid(&mut self, fluid: Fluid<N>) {
        self.fluids.push(fluid)
    }
    pub fn add_boundary(&mut self, boundary: Boundary<N>) -> BoundaryHandle {
        let handle = self.boundaries.len();
        self.boundaries.push(boundary);
        handle
    }

    pub fn fluids(&self) -> &[Fluid<N>] {
        &self.fluids
    }
    pub fn boundaries(&self) -> &[Boundary<N>] {
        &self.boundaries
    }

    pub fn h(&self) -> N {
        self.h
    }
    pub fn particle_radius(&self) -> N {
        self.particle_radius
    }
}