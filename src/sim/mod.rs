use nalgebra::{Vector2, vector};
pub mod state;
use self::state::{StateDerivative, State};

type SimVector = Vector2<f64>;

const G: f64 = 1.0;

pub struct Simulation {
    pub masses: Vec<f64>,
    pub initial_state: State,
    pub time: f64,
    pub dt: f64,
}

impl Simulation {
    pub fn new_3body(dt: f64) -> Simulation {
        let positions = vec![
            vector![-0.3092050, 0.0],
            vector![0.1546025, -0.09875616],
            vector![0.1546025, 0.09875616]
        ];

        let velocities = vec![
            vector![0.0, -0.50436399],
            vector![-1.18437049, 0.25218199],
            vector![1.18437049, 0.25218199]
        ];

        let initial_state = State {positions, velocities};
        let masses = vec![1.0/3.0, 1.0/3.0, 1.0/3.0];

        Simulation {
            initial_state,
            masses,
            time: 0.0,
            dt,
        }
    }
}

fn get_state_derivative(state: State, masses: &Vec<f64>) -> StateDerivative {
    let mut state_derivative = StateDerivative {
        velocities: state.velocities,
        accelerations: vec![SimVector::zeros(); state.positions.len()],
    };

    for i in 0..(state.positions.len() - 1) {
        for j in (i + 1)..state.positions.len() {
            let m1 = masses[i];
            let m2 = masses[j];
            let r1 = &state.positions[i];
            let r2 = &state.positions[j];
            let r_vec = r2 - r1;
            let r = r_vec.norm();
            let force = G * m1 * m2 * r_vec / r.powi(3);
            let a1 = force * (1.0/m1);
            let a2 = -force * (1.0/m2);

            state_derivative.accelerations[i] += a1;
            state_derivative.accelerations[j] += a2;
        }
    }

    state_derivative
}

fn rk4(state: State, masses: &Vec<f64>, dt: f64, evaluate: fn(State, &Vec<f64>) -> StateDerivative) -> State {
    let k1 = evaluate(state.clone(), &masses);
    let k2 = evaluate(state.clone() + (k1.clone() * 0.5 * dt).to_state(), &masses);
    let k3 = evaluate(state.clone() + (k2.clone() * 0.5 * dt).to_state(), &masses);
    let k4 = evaluate(state.clone() + (k3.clone() * dt).to_state(), &masses);

    let x_prime = (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (1.0/6.0);
    state + (x_prime * dt).to_state()
}

pub fn step(state: State, masses: &Vec<f64>, dt: f64) -> State {
    let old_state = state.clone();
    let new_state = rk4(old_state, masses, dt, get_state_derivative);
    new_state
}
