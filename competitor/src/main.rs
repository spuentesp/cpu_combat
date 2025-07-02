// competitor/src/main.rs

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Importamos todo lo necesario de nuestro crate `core`
use core_challenges::{
    P2PMessage, Challenge, Solver,
    solvers::{ProofOfWorkSolver},
};
use core_challenges::solvers::proof_of_work::{PoWInput};

// -- Bincode 2.0 Imports --
use bincode::{config::standard, decode_from_slice, encode_to_vec};

type SolverRegistry = HashMap<String, Arc<dyn Solver + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // -- Registro de Solvers --
    let mut solvers = SolverRegistry::new();
    solvers.insert("pow".to_string(), Arc::new(ProofOfWorkSolver));
    let solvers = Arc::new(solvers);

    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str);

    match mode {
        Some("listen") => {
            println!("[Modo Oyente] Esperando un desafío...");
            listen(solvers).await?;
        }
        Some("challenge") => {
            println!("[Modo Retador] Iniciando un desafío...");
            challenge(solvers).await?;
        }
        _ => {
            eprintln!("Error: modo no especificado.");
            eprintln!("Uso: cargo run -p competitor -- [listen|challenge]");
        }
    }

    Ok(())
}

async fn listen(solvers: Arc<SolverRegistry>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (socket, addr) = listener.accept().await?;
    println!("¡Conexión recibida de {}!", addr);
    handle_duel(socket, solvers, "listen").await?;
    Ok(())
}

async fn challenge(solvers: Arc<SolverRegistry>) -> Result<(), Box<dyn Error>> {
    let socket = TcpStream::connect("127.0.0.1:8080").await?;
    println!("¡Conectado al oponente!");
    handle_duel(socket, solvers, "challenge").await?;
    Ok(())
}

/// Lógica principal del duelo
async fn handle_duel(mut socket: TcpStream, solvers: Arc<SolverRegistry>, mode: &str) -> Result<(), Box<dyn Error>> {
    let mut difficulty = 4; // Dificultad inicial

    if mode == "challenge" {
        // El retador envía el primer desafío
        println!("\n--- TURNO 1 (Retador) ---");
        let challenge = create_pow_challenge("reto-inicial".to_string(), difficulty)?;
        send_message(&mut socket, &P2PMessage::Challenge(challenge)).await?;
    }

    loop {
        match read_message(&mut socket).await? {
            P2PMessage::Challenge(challenge) => {
                println!("\n--- Desafío Recibido: '{}' con dificultad {} ---", challenge.name, difficulty);
                let solver = solvers.get(&challenge.name).ok_or("Solver no encontrado")?;
                
                let solution = solver.solve(&challenge.payload)?;
                println!("Solución enviada. Preparando próximo desafío...");
                
                // Aumentamos la dificultad para el siguiente
                difficulty += 1;
                let next_challenge = create_pow_challenge(format!("reto-{}", difficulty), difficulty)?;

                send_message(&mut socket, &P2PMessage::Reply(solution)).await?;
                send_message(&mut socket, &P2PMessage::Challenge(next_challenge)).await?;
            }
            P2PMessage::Reply(solution) => {
                println!("Respuesta del oponente recibida.");
            }
            P2PMessage::YouWin(reason) => {
                println!("\n¡VICTORIA! Motivo: {}", reason);
                break;
            }
        }
    }
    Ok(())
}

fn create_pow_challenge(data: String, difficulty: u32) -> Result<Challenge, Box<dyn Error>> {
    let pow_input = PoWInput { data, difficulty };
    let config = standard();
    let payload = encode_to_vec(&pow_input, config)?;
    Ok(Challenge { name: "pow".to_string(), payload })
}

// --- Funciones de Red con Bincode 2.0 ---

async fn send_message(stream: &mut TcpStream, msg: &P2PMessage) -> Result<(), Box<dyn Error>> {
    let config = standard();
    let payload = encode_to_vec(msg, config)?;
    let len = payload.len() as u32;
    stream.write_u32(len).await?;
    stream.write_all(&payload).await?;
    Ok(())
}

async fn read_message(stream: &mut TcpStream) -> Result<P2PMessage, Box<dyn Error>> {
    let len = stream.read_u32().await?;
    let mut payload = vec![0; len as usize];
    stream.read_exact(&mut payload).await?;
    let config = standard();
    let (msg, _) = decode_from_slice(&payload, config)?;
    Ok(msg)
}