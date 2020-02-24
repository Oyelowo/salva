#[cfg(feature = "parallel")]
use rayon::prelude::*;

use na::{self, RealField};

use crate::geometry::ParticlesContacts;

use crate::math::Vector;
use crate::object::Fluid;
use crate::solver::NonPressureForce;

// See http://www.astro.lu.se/~david/teaching/SPH/notes/annurev.aa.30.090192.pdf
#[derive(Clone)]
pub struct ArtificialViscosity<N: RealField> {
    pub alpha: N,
    pub beta: N,
    pub speed_of_sound: N,
    pub viscosity_coefficient: N,
}

impl<N: RealField> ArtificialViscosity<N> {
    pub fn new(viscosity_coefficient: N) -> Self {
        Self {
            alpha: N::one(),
            beta: na::convert(0.0),
            speed_of_sound: na::convert(10.0),
            viscosity_coefficient,
        }
    }
}

impl<N: RealField> NonPressureForce<N> for ArtificialViscosity<N> {
    fn solve(
        &mut self,
        dt: N,
        inv_dt: N,
        kernel_radius: N,
        fluid_fluid_contacts: &ParticlesContacts<N>,
        fluid: &mut Fluid<N>,
        densities: &[N],
    ) {
        let viscosity_coefficient = self.viscosity_coefficient;
        let speed_of_sound = self.speed_of_sound;
        let alpha = self.alpha;
        let beta = self.beta;
        let density0 = fluid.density0;
        let volumes = &fluid.volumes;
        let positions = &fluid.positions;
        let velocities = &fluid.velocities;
        let _0_5: N = na::convert(0.5);

        par_iter_mut!(fluid.accelerations)
            .enumerate()
            .for_each(|(i, acceleration)| {
                let mut added_acc = Vector::zeros();

                for c in fluid_fluid_contacts
                    .particle_contacts(i)
                    .read()
                    .unwrap()
                    .iter()
                {
                    if c.i_model == c.j_model {
                        let r_ij = positions[c.i] - positions[c.j];
                        let v_ij = velocities[c.i] - velocities[c.j];
                        let vr = r_ij.dot(&v_ij);

                        if vr < N::zero() {
                            let density_average = (densities[c.i] + densities[c.j]) * _0_5;
                            let eta2 = kernel_radius * kernel_radius * na::convert(0.01);
                            let mu_ij = kernel_radius * vr / (r_ij.norm_squared() + eta2);

                            added_acc += c.gradient
                                * ((speed_of_sound * alpha * mu_ij - beta * mu_ij * mu_ij)
                                    * (volumes[c.j] * density0 / density_average));
                        }
                    }
                }

                *acceleration += added_acc * viscosity_coefficient;
            })
    }

    fn apply_permutation(&mut self, _: &[usize]) {}
}
