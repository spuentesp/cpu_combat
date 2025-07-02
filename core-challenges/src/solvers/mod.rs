// core/src/solvers/mod.rs

pub mod dummy_solver;
pub mod proof_of_work;

pub use dummy_solver::DummySolver;
pub use proof_of_work::ProofOfWorkSolver;

