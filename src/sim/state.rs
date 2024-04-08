use std::ops::{Add, Mul};
use crate::sim::SimVector;

/// Represents a celestial object
#[derive(Debug, Clone)]
pub(crate) struct State {
    pub positions: Vec<SimVector>,
    pub velocities: Vec<SimVector>,
}

#[derive(Debug, Clone)]
pub struct StateDerivative {
    pub velocities: Vec<SimVector>,
    pub accelerations: Vec<SimVector>
}

impl Add for State {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut positions: Vec<SimVector> = vec![];
        let mut velocities: Vec<SimVector> = vec![];
        for (i, position) in self.positions.iter().enumerate() {
            let rhs_position = rhs.positions.get(i).unwrap();
            positions.push(position + rhs_position);
        }
        for (i, velocity) in self.velocities.iter().enumerate() {
            let rhs_velocity = rhs.velocities.get(i).unwrap();
            velocities.push(velocity + rhs_velocity);
        }
        Self {positions, velocities}
    }
}

impl Add for StateDerivative {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut velocities: Vec<SimVector> = vec![];
        let mut accelerations: Vec<SimVector> = vec![];
        for (i, velocity) in self.velocities.iter().enumerate() {
            let rhs_velocity = rhs.velocities.get(i).unwrap();
            velocities.push(velocity + rhs_velocity);
        }
        for (i, acceleration) in self.accelerations.iter().enumerate() {
            let rhs_acceleration = rhs.accelerations.get(i).unwrap();
            accelerations.push(acceleration + rhs_acceleration);
        }
        Self {velocities, accelerations}
    }
}

impl Mul<f64> for State {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        let mut positions: Vec<SimVector> = vec![];
        let mut velocities: Vec<SimVector> = vec![];
        for position in self.positions {
            positions.push(position * rhs);
        }
        for velocity in self.velocities {
            velocities.push(velocity * rhs);
        }
        Self {positions, velocities}
    }
}

impl Mul<f64> for StateDerivative {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        let mut velocities: Vec<SimVector> = vec![];
        let mut accelerations: Vec<SimVector> = vec![];
        for velocity in self.velocities {
            velocities.push(velocity * rhs);
        }
        for acceleration in self.accelerations {
            accelerations.push(acceleration * rhs);
        }
        Self {velocities, accelerations}
    }
}

impl StateDerivative {
    pub fn to_state(self) -> State {
        State {
            positions: self.velocities,
            velocities: self.accelerations,
        }

    }
}
