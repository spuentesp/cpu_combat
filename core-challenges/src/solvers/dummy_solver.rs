// core/src/solvers/dummy_solver.rs

use crate::{Solution, Solver, SolverError}; // Importa las definiciones del nivel superior

/// Un solucionador de prueba que no hace ningún cálculo real.
pub struct DummySolver;

impl Solver for DummySolver {
    fn solve(&self, payload: &[u8]) -> Result<Solution, SolverError> {
        println!(
            "DummySolver: 'resolviendo' un desafío con payload de {} bytes.",
            payload.len()
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("DummySolver: ¡Desafío 'resuelto'!");

        Ok(Solution {
            payload: "solucion_fija_123".as_bytes().to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_solver_solve() {
        let solver = DummySolver;
        let payload = vec![1, 2, 3];
        let result = solver.solve(&payload).unwrap();
        assert_eq!(result.payload, "solucion_fija_123".as_bytes());
    }
}