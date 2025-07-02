// core-challenges/src/lib.rs

use serde::{Serialize, Deserialize};
use bincode::{Decode, Encode};
use std::fmt; // Necesitamos fmt para implementar los traits de error

pub mod solvers;

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub enum SolverError {
    InvalidInput(String),
    ComputationFailed(String),
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::InvalidInput(s) => write!(f, "Invalid Input: {}", s),
            SolverError::ComputationFailed(s) => write!(f, "Computation Failed: {}", s),
        }
    }
}

impl std::error::Error for SolverError {}


#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct Challenge {
    pub name: String,
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct Solution {
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub enum P2PMessage {
    Challenge(Challenge),
    Reply(Solution),
    YouWin(String),
}

pub trait Solver {
    fn solve(&self, payload: &[u8]) -> Result<Solution, SolverError>;
}