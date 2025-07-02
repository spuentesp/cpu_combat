// core/src/solvers/proof_of_work.rs

use crate::{Solution, Solver, SolverError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use bincode::{config::standard, decode_from_slice, encode_to_vec, Decode, Encode};

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct PoWInput {
    pub data: String,
    pub difficulty: u32,
}

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
pub struct PoWOutput {
    pub nonce: u64,
}

pub struct ProofOfWorkSolver;

impl Solver for ProofOfWorkSolver {
    fn solve(&self, payload: &[u8]) -> Result<Solution, SolverError> {
        let config = standard();
        let (input, _): (PoWInput, usize) = decode_from_slice(payload, config)
            .map_err(|e| SolverError::InvalidInput(e.to_string()))?;

        let prefix = "0".repeat(input.difficulty as usize);
        let mut base_hasher = Sha256::new();
        base_hasher.update(input.data.as_bytes());

        // --- CORRECCIÓN 2: Especificar el tipo del nonce (u64) ---
        for nonce in 0..10_000_000u64 {
            let mut hasher = base_hasher.clone();
            hasher.update(&nonce.to_le_bytes());
            
            let hash = hasher.finalize();
            let hash_hex = hex::encode(hash);

            if hash_hex.starts_with(&prefix) {
                let output = PoWOutput { nonce };
                
                let config = standard();
                let solution_payload = encode_to_vec(&output, config)
                    .map_err(|e| SolverError::ComputationFailed(e.to_string()))?;
                
                return Ok(Solution { payload: solution_payload });
            }
        }

        Err(SolverError::ComputationFailed("Nonce no encontrado dentro del límite de intentos.".to_string()))
    }
}

// ... (Las pruebas no necesitan cambios si ya estaban usando la API 2.0) ...
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_solver_finds_solution() {
        let solver = ProofOfWorkSolver;
        let input = PoWInput {
            data: "Hola, mundo".to_string(),
            difficulty: 4,
        };
        
        let config = standard();
        let payload = encode_to_vec(&input, config).unwrap();

        let result = solver.solve(&payload);
        assert!(result.is_ok());
        
        let solution_payload = result.unwrap().payload;
        let (output, _): (PoWOutput, usize) = decode_from_slice(&solution_payload, config).unwrap();
        
        let attempt = format!("{}-{}", input.data, output.nonce);
        let hash = Sha256::digest(attempt.as_bytes());
        let hash_hex = hex::encode(hash);

        assert!(hash_hex.starts_with("0000"));
    }
}