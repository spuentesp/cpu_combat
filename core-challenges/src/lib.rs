
// 1. Declara el nuevo módulo 'solvers'
pub mod solvers;

use serde::{Serialize, Deserialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
pub enum SolverError {
    InvalidInput(String),
    ComputationFailed(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Challenge {
    pub name: String,
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Solution {
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum P2PMessage {
    Challenge(Challenge),
    Reply(Solution),
    YouWin(String),
}

pub trait Solver {
    fn solve(&self, payload: &[u8]) -> Result<Solution, SolverError>;
}