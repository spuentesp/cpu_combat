// core-challenges/src/solvers/dummy_solver.rs

use crate::{Challenge, Solution, Solver, SolverError};

/// Un solucionador de prueba que no hace ningún cálculo real
/// y devuelve valores fijos.
pub struct DummySolver;

impl Solver for DummySolver {
    /// Ignora la entrada y siempre devuelve una solución predefinida.
    fn solve(&self, _payload: &[u8]) -> Result<Solution, SolverError> {
        Ok(Solution {
            payload: "solucion_fija_123".as_bytes().to_vec(),
        })
    }

    /// Implementación faltante: Crea un desafío dummy.
    fn create_challenge(&self, _difficulty: u32) -> Result<Challenge, SolverError> {
        Ok(Challenge {
            name: "dummy".to_string(),
            payload: vec![], // Un payload vacío para la prueba
        })
    }

    /// Implementación faltante: Siempre aprueba la solución.
    fn review_solution(&self, _challenge_payload: &[u8], _solution: &Solution) -> Result<(), SolverError> {
        // El dummy solver siempre considera cualquier solución como válida.
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_solve() {
        let solver = DummySolver;
        let result = solver.solve(&[]).unwrap();
        assert_eq!(result.payload, b"solucion_fija_123");
    }

    #[test]
    fn test_dummy_create() {
        let solver = DummySolver;
        let challenge = solver.create_challenge(1).unwrap();
        assert_eq!(challenge.name, "dummy");
    }

    #[test]
    fn test_dummy_review() {
        let solver = DummySolver;
        let solution = Solution { payload: vec![] };
        let result = solver.review_solution(&[], &solution);
        assert!(result.is_ok());
    }
}