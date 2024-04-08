#[macro_use]
extern crate glium;
use std::{thread::{sleep, self}, time::Duration, sync::{Arc, Mutex}};
mod render;
use render::renderer;
mod sim;
use sim::{Simulation, step};

const LOOPRATE: u64 = 120;
const FRAMERATE: u64 = 120;
const TIMESTEP: f64 = 1e-3;

fn main() {
    let sim = Simulation::new_3body(TIMESTEP);
    let shared_state = Arc::new(Mutex::new(sim.initial_state));
    let render_state = Arc::clone(&shared_state);

    thread::spawn(move || {
        loop {
            {
                let mut state = shared_state.lock().unwrap();
                *state = step(state.clone(), &sim.masses, sim.dt);
            }
            sleep(Duration::from_nanos(1000000000/LOOPRATE));
        }
    });

    renderer(render_state);
}
