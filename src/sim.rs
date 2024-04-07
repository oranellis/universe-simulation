use std::vec;
use rand::prelude::*;
use rand_distr::{Normal, Distribution};
use nalgebra::{Vector2, vector};

pub const STAR_COUNT: u64 = 10;
pub const TIMESTEP: f64 = 1e7; // In seconds
pub const FRAMESKIPS: u32 = 1; // The number of timesteps to perform per frame
pub const G: f64 = 6.674e-11; // In seconds

type SimVector = Vector2<f64>;

/// Contains information about the simulation domain
///
/// the position (0, 0) is in the center of the screen and simulation, the simulation extends to
/// \[-width / 2, width / 2\] and \[-height / 2, height / 2\]
pub struct SimulationDomain2D {
    width: f64,
    height: f64,
    target_screen_width: u32,
    target_screen_height: u32,
}

/// Represents a celestial object
#[derive(Clone)]
pub struct Star2D {
    /// The unique id of the star
    pub id: u64,
    /// The position in meters
    pub position: SimVector,
    /// The velocity in meters per second
    pub velocity: SimVector,
    /// The mass in kg
    pub mass: f64,
    /// The brightness of the star, expressed relative to the sun's luminosity. 0 for
    /// a black hole
    pub luminosity: f64,
    /// The temp of the star in kelvin, not used for black holes
    pub temperature: u32,
}

impl Star2D {
    pub fn new_random_star_2d(sim_domain: &SimulationDomain2D, id: u64) -> Self {
        let normal = Normal::new(8e29, 5e28).unwrap();
        let normalx = Normal::new(0.0, sim_domain.width / 8.0).unwrap();
        let normaly = Normal::new(0.0, sim_domain.height / 8.0).unwrap();
        let x = normalx.sample(&mut rand::thread_rng()).clamp(-sim_domain.width/2.0, sim_domain.width/2.0);
        let y = normaly.sample(&mut rand::thread_rng()).clamp(-sim_domain.height/2.0, sim_domain.height/2.0);
        let x_vel = (rand::thread_rng().gen::<f64>() - 0.5) * 0.0;
        let y_vel = (rand::thread_rng().gen::<f64>() - 0.5) * 0.0;
        Star2D {
            id,
            position: vector![x, y],
            velocity: vector![x_vel, y_vel],
            mass: normal.sample(&mut rand::thread_rng()),
            luminosity: 1.0,
            temperature: 5000,
        }
    }
}

// Generic function to calculate gravitational force components between two points
// It supports both Vector2 and Vector3 from nalgebra
fn gravitational_force_components(m1: f64, m2: f64, pos1: SimVector, pos2: SimVector) -> SimVector {
    // Calculate the displacement vector between the two points
    let displacement: SimVector = pos2 - pos1;
    let r_squared = displacement.norm_squared();

    // Calculate the magnitude of the gravitational force
    let f = G * (m1 * m2) / r_squared;

    // Normalize the displacement vector and scale it by the magnitude of the force
    let force_vector = displacement.normalize() * f;

    force_vector
}

/// Converts positions in simulation space to positions in screen space
///
/// Screen coordinates go from -1 to 1 regardless of actual pixel dimensions so screen size is
/// required for accurate scaling
pub fn point2d_to_screen_coords(pos: SimVector, screen_size: (u32, u32), sim_domain: &SimulationDomain2D) -> Option<(f64, f64)> {
    let (screen_width, screen_height) = screen_size;

    let x_norm = 2.0*pos.x / sim_domain.width;
    let y_norm = 2.0*pos.y / sim_domain.height;
    let x_transform = f64::from(sim_domain.target_screen_width) / f64::from(screen_width);
    let y_transform = f64::from(sim_domain.target_screen_height) / f64::from(screen_height);
    let screen_x = x_norm * x_transform;
    let screen_y = y_norm * y_transform;

    if screen_x > 1.0 || screen_x < -1.0 || screen_y > 1.0 || screen_y < -1.0 {
        return None
    }

    return Some((screen_x, screen_y))
}

/// Structure representing a 2D simulation
pub struct Simulation2D {
    pub sim_domain: SimulationDomain2D,
    stars: Vec<Star2D>,
    pub time: f64,
}

impl Simulation2D {
    /// Currently returns a vec of all the stars for debugging
    pub fn gen_simulation_2d() -> Self {
        let sim_domain = SimulationDomain2D {
            width: 1e14, // 50 times the milky way diameter galaxy across
            height: 1e14, // 50 times the milky way diameter galaxy high
            target_screen_height: 1000, // Pixels
            target_screen_width: 1000, // Pixels
        };
        let mut stars: Vec<Star2D> = vec![];
        for id in 1..STAR_COUNT {
            stars.push(Star2D::new_random_star_2d(&sim_domain, id));
        }
        return Simulation2D {
            sim_domain,
            stars,
            time: 0.0,
        }
    }

    /// Steps the simulation
    pub fn step_rk4(&mut self) {
        let old_stars = self.stars.clone();

        for star in &mut self.stars {
            // Initial velocity and acceleration
            let initial_velocity = star.velocity;
            let initial_acceleration = total_gravitational_acceleration(star, &old_stars);

            // k1: Initial velocity and acceleration
            let k1_v = initial_velocity;
            let k1_a = initial_acceleration;

            // Creating an intermediate star for k2 using struct update syntax
            let star_k2 = Star2D {
                position: star.position + k1_v * (TIMESTEP * 0.5),
                velocity: star.velocity + k1_a * (TIMESTEP * 0.5),
                ..*star
            };
            let k2_v = star_k2.velocity;
            let k2_a = total_gravitational_acceleration(&star_k2, &old_stars);

            // Intermediate star for k3
            let star_k3 = Star2D {
                position: star.position + k2_v * (TIMESTEP * 0.5),
                velocity: star.velocity + k2_a * (TIMESTEP * 0.5),
                ..*star
            };
            let k3_v = star_k3.velocity;
            let k3_a = total_gravitational_acceleration(&star_k3, &old_stars);

            // Intermediate star for k4
            let star_k4 = Star2D {
                position: star.position + k3_v * TIMESTEP,
                velocity: star.velocity + k3_a * TIMESTEP,
                ..*star
            };
            let k4_v = star_k4.velocity;
            let k4_a = total_gravitational_acceleration(&star_k4, &old_stars);

            // Combining the results to compute the final position and velocity
            star.position += (k1_v + 2.0*k2_v + 2.0*k3_v + k4_v) * (TIMESTEP / 6.0);
            star.velocity += (k1_a + 2.0*k2_a + 2.0*k3_a + k4_a) * (TIMESTEP / 6.0);
        }
    }

    pub fn step(&mut self) {
        let old_stars = self.stars.clone();
        for star in &mut self.stars {
            let acceleration = total_gravitational_acceleration(star, &old_stars);
            star.position = star.position + star.velocity*TIMESTEP + 0.5*acceleration*TIMESTEP*TIMESTEP;
            star.velocity += acceleration * TIMESTEP;
        }
    }

    /// Returns a reference to the vector of stars
    pub fn get_stars(&self) -> &Vec<Star2D> {
        &self.stars
    }
}

fn total_gravitational_acceleration(star: &Star2D, old_stars: &Vec<Star2D>) -> SimVector {
    let mut force: SimVector = vector![0.0, 0.0];
    for old_star in old_stars {
        if star.id == old_star.id {
            continue;
        }
        let force_part = gravitational_force_components(star.mass, old_star.mass, star.position, old_star.position);
        force += force_part;
    }
    force / star.mass
}
