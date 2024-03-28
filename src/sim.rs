use rand::prelude::*;
use rand_distr::{Normal, Distribution};

pub const STAR_COUNT: u32 = 300;
pub const TIMESTEP: f64 = 1e13; // In seconds
pub const G: f64 = 6.674e-11; // In seconds

/// Contains information about the simulation domain
///
/// the position (0, 0) is in the center of the screen and simulation, the simulation extends to
/// \[-width / 2, width / 2\] and \[-height / 2, height / 2\]
pub struct SimulationDomain2D {
    width: f64,
    height: f64,
    grid_spacing: f64,
    target_screen_width: u32,
    target_screen_height: u32,
}

/// Represents a celestial object
#[derive(Clone)]
pub struct Star2D {
    /// The x position in meters
    pub x: f64,
    /// The y position in meters
    pub y: f64,
    /// The x velocities in meters per second
    pub x_vel: f64,
    /// The y velocities in meters per second
    pub y_vel: f64,
    /// The mass of the star in 1e27 kg
    pub x_old: f64,
    /// The y position in meters
    pub y_old: f64,
    /// The x velocities in meters per second
    pub x_vel_old: f64,
    /// The y velocities in meters per second
    pub y_vel_old: f64,
    /// The mass of the star in 1e27 kg
    pub mass: f64,
    /// The brightness of the star, expressed relative to the sun's luminosity. 0 for
    /// a black hole
    pub luminosity: f64,
    /// The temp of the star in kelvin, not used for black holes
    pub temperature: u32,
}

impl Star2D {
    pub fn new_random_star_2d(sim_domain: &SimulationDomain2D) -> Self {
        let normal = Normal::new(8e29, 5e28).unwrap();
        let normalx = Normal::new(0.0, sim_domain.width / 8.0).unwrap();
        let normaly = Normal::new(0.0, sim_domain.height / 8.0).unwrap();
        Star2D {
            // x: ((rand::thread_rng().gen::<f64>() - 0.5) * sim_domain.width),
            // y: ((rand::thread_rng().gen::<f64>() - 0.5) * sim_domain.height),
            x: normalx.sample(&mut rand::thread_rng()).clamp(-sim_domain.width/2.0, sim_domain.width/2.0),
            y: normaly.sample(&mut rand::thread_rng()).clamp(-sim_domain.height/2.0, sim_domain.height/2.0),
            x_vel: ((rand::thread_rng().gen::<f64>() - 0.5) * 2e5),
            y_vel: ((rand::thread_rng().gen::<f64>() - 0.5) * 2e5),
            x_old: 0.0,
            y_old: 0.0,
            x_vel_old: 0.0,
            y_vel_old: 0.0,
            mass: normal.sample(&mut rand::thread_rng()),
            luminosity: 1.0,
            temperature: 5000,
        }
    }
}

fn gravitational_force_components(m1: f64, m2: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> (f64, f64) {
    // Calculate the distance between the two points in 2D space
    let dx = x2 - x1;
    let dy = y2 - y1;
    let r_squared = dx.powi(2) + dy.powi(2);
    let r = r_squared.sqrt();

    // Calculate the magnitude of the gravitational force
    let f = G * (m1 * m2) / r_squared;

    // Calculate the components of the gravitational force
    let fx = f * dx / r;
    let fy = f * dy / r;

    (fx, fy)
}

/// Converts positions in simulation space to positions in screen space
///
/// Screen coordinates go from -1 to 1 regardless of actual pixel dimensions so screen size is
/// required for accurate scaling
pub fn point2d_to_screen_coords(pos: (f64, f64), screen_size: (u32, u32), sim_domain: &SimulationDomain2D) -> Option<(f64, f64)> {
    let (x, y) = pos;
    let (screen_width, screen_height) = screen_size;

    let x_norm = 2.0*x / sim_domain.width;
    let y_norm = 2.0*y / sim_domain.height;
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
    stars1: Vec<Star2D>,
    stars2: Vec<Star2D>,
    stars3: Vec<Star2D>,
    pub time: f64,
    stars_array_in_use: u8,
}

impl Simulation2D {
    /// Currently returns a vec of all the stars for debugging
    pub fn gen_simulation_2d() -> Self {
        let sim_domain = SimulationDomain2D {
            width: 1e14, // 50 times the milky way diameter galaxy across
            height: 1e14, // 50 times the milky way diameter galaxy high
            grid_spacing: 1e20, // The width and height of grid cells for cell based optimisation
            target_screen_height: 1000, // Pixels
            target_screen_width: 1000, // Pixels
        };
        let mut stars: Vec<Star2D> = vec![];
        stars.push(Star2D { x: 0.0, y: 0.0, x_vel: 0.0, y_vel: 0.0, mass: 1e31, luminosity: 0.0, temperature: 0 });
        for _ in 1..STAR_COUNT {
            stars.push(Star2D::new_random_star_2d(&sim_domain))
        }
        return Simulation2D {
            sim_domain,
            stars1: stars,
            stars2: vec![],
            stars3: vec![],
            stars_array_in_use: 0,
            time: 0.0,
        }
    }
    pub fn step(&mut self) {
        let mut old_stars = &self.stars3;
        let mut curr_stars = &self.stars1;
        let mut next_stars = &mut self.stars2;
        match self.stars_array_in_use {
            1 => {
                old_stars = &self.stars1;
                curr_stars = &self.stars2;
                next_stars = &mut self.stars3;
            }
            2 => {
                old_stars = &self.stars2;
                curr_stars = &self.stars3;
                next_stars = &mut self.stars1;
            }
            _ => {
                old_stars = &self.stars3;
                curr_stars = &self.stars1;
                next_stars = &mut self.stars2;
            }
        }
        for (i, star) in curr_stars.iter().enumerate() {
            let mut force_x: f64 = 0.0;
            let mut force_y: f64 = 0.0;
            for curr_star in curr_stars {
                if star.x == curr_star.x && star.y == curr_star.y {
                    continue;
                }
                let (force_x_part, force_y_part) = gravitational_force_components(star.mass, curr_star.mass, star.x, star.y, curr_star.x, curr_star.y);
                force_x += force_x_part;
                force_y += force_y_part;
            }
            next_stars[i].x = star.x + ((star.x_vel + force_x / star.mass) / 2.0) * TIMESTEP;
            next_stars[i].x_vel += force_x / star.mass;
            next_stars[i].y = star.y + ((star.y_vel + force_y / star.mass) / 2.0) * TIMESTEP;
            next_stars[i].y_vel += force_y / star.mass;
        }
        self.stars_array_in_use = (self.stars_array_in_use + 1) % 3;
    }
    pub fn get_stars(&self) -> &Vec<Star2D> {
        match self.stars_array_in_use {
            1 => {
                &self.stars2
            }
            2 => {
                &self.stars3
            }
            _ => {
                &self.stars1
            }
        }
    }
}
