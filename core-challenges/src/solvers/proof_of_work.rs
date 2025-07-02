// core-challenges/src/solvers/proof_of_work.rs

use crate::{Challenge, Solution, Solver, SolverError};
use bincode::{config::standard, decode_from_slice, encode_to_vec, Decode, Encode};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Encode, Decode, Clone)]
pub struct PoWInput {
    pub data: String,
    pub difficulty: u32,
    pub start_nonce: u64,
    pub max_attempts: u64,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct PoWOutput {
    pub nonce: u64,
}

// --- LÓGICA DE VERIFICACIÓN CENTRALIZADA ---

/// Función privada que verifica si un nonce es válido para un input dado.
fn verify_nonce(input: &PoWInput, nonce: u64) -> bool {
    let prefix = "0".repeat(input.difficulty as usize);
    let mut hasher = Sha256::new();
    hasher.update(input.data.as_bytes());
    hasher.update(&nonce.to_le_bytes());
    let hash_hex = hex::encode(hasher.finalize());
    hash_hex.starts_with(&prefix)
}


pub struct ProofOfWorkSolver;

impl Solver for ProofOfWorkSolver {
    fn solve(&self, payload: &[u8]) -> Result<Solution, SolverError> {
        let config = standard();
        let (input, _): (PoWInput, usize) = decode_from_slice(payload, config)
            .map_err(|e| SolverError::InvalidInput(e.to_string()))?;

        let search_range = input.start_nonce..(input.start_nonce + input.max_attempts);

        let found_nonce = search_range
            .into_par_iter()
            .find_any(|&nonce| verify_nonce(&input, nonce));

        match found_nonce {
            Some(nonce) => {
                let output = PoWOutput { nonce };
                let solution_payload = encode_to_vec(&output, config)
                    .map_err(|e| SolverError::ComputationFailed(e.to_string()))?;
                Ok(Solution { payload: solution_payload })
            }
            None => Err(SolverError::ComputationFailed(
                "Nonce no encontrado dentro del rango especificado.".to_string(),
            )),
        }
    }

    fn review_solution(&self, challenge_payload: &[u8], solution: &Solution) -> Result<(), SolverError> {
        let config = standard();
        let (input, _): (PoWInput, usize) = decode_from_slice(challenge_payload, config)
            .map_err(|e| SolverError::InvalidInput(format!("No se pudo decodificar el payload del desafío: {}", e)))?;
        let (output, _): (PoWOutput, usize) = decode_from_slice(&solution.payload, config)
            .map_err(|e| SolverError::InvalidInput(format!("No se pudo decodificar el payload de la solución: {}", e)))?;

        // La revisión ahora solo llama a la función de ayuda.
        if verify_nonce(&input, output.nonce) {
            Ok(())
        } else {
            Err(SolverError::ComputationFailed(
                "La solución proporcionada es incorrecta.".to_string(),
            ))
        }
    }

    fn create_challenge(&self, difficulty: u32) -> Result<Challenge, SolverError> {
        let input = PoWInput {
            data: format!("reto-pow-{}", difficulty),
            difficulty,
            start_nonce: 0,
            max_attempts: 500_000_000,
        };
        let config = standard();
        let payload = encode_to_vec(&input, config)
            .map_err(|e| SolverError::InvalidInput(e.to_string()))?;
        Ok(Challenge {
            name: "pow".to_string(),
            payload,
        })
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_finds_valid_solution() {
        let solver = ProofOfWorkSolver;
        let challenge = solver.create_challenge(4).unwrap();
        let solution = solver.solve(&challenge.payload).unwrap();
        
        // La prueba más robusta es usar el propio `review_solution` para verificar.
        let review = solver.review_solution(&challenge.payload, &solution);
        assert!(review.is_ok());
    }

    #[test]
    fn test_solve_returns_error_if_not_in_range() {
        let solver = ProofOfWorkSolver;
        let config = standard();
        let impossible_input = PoWInput {
            data: "Test de rango".to_string(),
            difficulty: 5,
            start_nonce: 0,
            max_attempts: 10,
        };
        let payload = encode_to_vec(&impossible_input, config).unwrap();
        let result = solver.solve(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_review_rejects_invalid_solution() {
        let solver = ProofOfWorkSolver;
        let challenge = solver.create_challenge(3).unwrap();
        let config = standard();
        let fake_output = PoWOutput { nonce: 12345 }; // Un nonce incorrecto
        let fake_payload = encode_to_vec(&fake_output, config).unwrap();
        let fake_solution = Solution { payload: fake_payload };
        let review_result = solver.review_solution(&challenge.payload, &fake_solution);
        assert!(review_result.is_err());
    }
}