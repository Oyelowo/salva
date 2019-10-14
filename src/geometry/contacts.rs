use crate::boundary::Boundary;
use crate::fluid::Fluid;
use crate::geometry::HGrid;
use crate::math::Vector;
use na::RealField;
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Contact<N: RealField> {
    pub i: usize,
    pub i_model: usize,
    pub j: usize,
    pub j_model: usize,
    pub weight: N,
    pub gradient: Vector<N>,
}

#[derive(Clone, Debug)]
pub struct ParticlesContacts<N: RealField> {
    contacts: Vec<Contact<N>>,
    contact_ranges: Vec<Range<usize>>,
}

impl<N: RealField> ParticlesContacts<N> {
    pub fn new() -> Self {
        Self {
            contacts: Vec::new(),
            contact_ranges: Vec::new(),
        }
    }

    pub fn particle_contacts(&self, i: usize) -> &[Contact<N>] {
        &self.contacts[self.contact_ranges[i].clone()]
    }

    pub fn particle_contacts_mut(&mut self, i: usize) -> &mut [Contact<N>] {
        &mut self.contacts[self.contact_ranges[i].clone()]
    }

    pub fn contacts(&self) -> &[Contact<N>] {
        &self.contacts[..]
    }

    pub fn contacts_mut(&mut self) -> &mut [Contact<N>] {
        &mut self.contacts[..]
    }
}

pub fn compute_contacts<N: RealField>(
    h: N,
    fluids: &[Fluid<N>],
    boundaries: &[Boundary<N>],
    fluid_delta_pos: Option<&[Vec<Vector<N>>]>,
    fluid_fluid_contacts: &mut Vec<ParticlesContacts<N>>,
    fluid_boundary_contacts: &mut Vec<ParticlesContacts<N>>,
    boundary_boundary_contacts: &mut Vec<ParticlesContacts<N>>,
)
{
    fluid_fluid_contacts.resize(fluids.len(), ParticlesContacts::new());
    fluid_boundary_contacts.resize(fluids.len(), ParticlesContacts::new());
    boundary_boundary_contacts.resize(boundaries.len(), ParticlesContacts::new());

    for (fluid, contacts) in fluids.iter().zip(fluid_fluid_contacts.iter_mut()) {
        contacts.contact_ranges.resize(fluid.num_particles(), 0..0)
    }

    for (fluid, contacts) in fluids.iter().zip(fluid_boundary_contacts.iter_mut()) {
        contacts.contact_ranges.resize(fluid.num_particles(), 0..0)
    }

    for (boundary, contacts) in boundaries.iter().zip(boundary_boundary_contacts.iter_mut()) {
        contacts
            .contact_ranges
            .resize(boundary.num_particles(), 0..0)
    }

    let mut grid = HGrid::new(h);

    for (fluid_id, fluid) in fluids.iter().enumerate() {
        if let Some(deltas) = fluid_delta_pos {
            let fluid_deltas = &deltas[fluid_id];

            for (particle_id, point) in fluid.positions.iter().enumerate() {
                grid.insert(
                    &(point + fluid_deltas[particle_id]),
                    (fluid_id, particle_id, false),
                );
            }
        } else {
            for (particle_id, point) in fluid.positions.iter().enumerate() {
                grid.insert(&point, (fluid_id, particle_id, false));
            }
        }
    }

    for (boundary_id, boundary) in boundaries.iter().enumerate() {
        for (particle_id, point) in boundary.positions.iter().enumerate() {
            grid.insert(&point, (boundary_id, particle_id, true));
        }
    }

    for (cell, curr_particles) in grid.cells() {
        let neighbors: Vec<_> = grid.neighbor_cells(cell, h).collect();

        for (fluid_i, particle_i, is_boundary_i) in curr_particles {
            if *is_boundary_i {
                let bb_contacts = &mut boundary_boundary_contacts[*fluid_i];
                let bb_start = bb_contacts.contacts.len();
                bb_contacts.contact_ranges[*particle_i] = bb_start..bb_start;

                for (_, nbh_particles) in &neighbors {
                    for (fluid_j, particle_j, is_boundary_j) in *nbh_particles {
                        // NOTE: we are not interested by boundary-fluid contacts.
                        // Those will already be detected as fluid-boundary contacts instead.
                        if *is_boundary_j {
                            let mut pi = &boundaries[*fluid_i].positions[*particle_i];
                            let mut pj = &boundaries[*fluid_j].positions[*particle_j];

                            if na::distance_squared(pi, pj) <= h * h {
                                let contact = Contact {
                                    i_model: *fluid_i,
                                    j_model: *fluid_j,
                                    i: *particle_i,
                                    j: *particle_j,
                                    weight: N::zero(),
                                    gradient: Vector::zeros(),
                                };

                                bb_contacts.contacts.push(contact);
                                bb_contacts.contact_ranges[*particle_i].end += 1;
                            }
                        }
                    }
                }
            } else {
                let ff_contacts = &mut fluid_fluid_contacts[*fluid_i];
                let fb_contacts = &mut fluid_boundary_contacts[*fluid_i];
                let ff_start = ff_contacts.contacts.len();
                let fb_start = fb_contacts.contacts.len();

                ff_contacts.contact_ranges[*particle_i] = ff_start..ff_start;
                fb_contacts.contact_ranges[*particle_i] = fb_start..fb_start;

                for (_, nbh_particles) in &neighbors {
                    for (fluid_j, particle_j, is_boundary_j) in *nbh_particles {
                        let mut pi = fluids[*fluid_i].positions[*particle_i];
                        let mut pj = if *is_boundary_j {
                            boundaries[*fluid_j].positions[*particle_j]
                        } else {
                            fluids[*fluid_j].positions[*particle_j]
                        };

                        if let Some(deltas) = fluid_delta_pos {
                            pi += deltas[*fluid_i][*particle_i];

                            if !is_boundary_j {
                                pj += deltas[*fluid_j][*particle_j];
                            }
                        }

                        if na::distance_squared(&pi, &pj) <= h * h {
                            let contact = Contact {
                                i_model: *fluid_i,
                                j_model: *fluid_j,
                                i: *particle_i,
                                j: *particle_j,
                                weight: N::zero(),
                                gradient: Vector::zeros(),
                            };

                            if *is_boundary_j {
                                fb_contacts.contacts.push(contact);
                                fb_contacts.contact_ranges[*particle_i].end += 1;
                            } else {
                                ff_contacts.contacts.push(contact);
                                ff_contacts.contact_ranges[*particle_i].end += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}