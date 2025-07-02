// competitor/src/main.rs

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use std::io::Write;
use sha2::Digest;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use core_challenges::{
    Challenge, P2PMessage, Solver,
    solvers::ProofOfWorkSolver,
};
use core_challenges::solvers::proof_of_work::{PoWInput, PoWOutput};
use bincode::{config::standard, decode_from_slice, encode_to_vec};

use sysinfo::{System};
use indicatif::{ProgressBar, ProgressStyle};

type SolverRegistry = HashMap<String, Arc<dyn Solver + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();

    // Revisa si el flag --stats está presente
    if args.contains(&"--stats".to_string()) {
        tokio::spawn(spawn_stats_reporter());
    }

    // Busca el modo ("listen" o "challenge") ignorando el nombre del programa y los flags
    let mode = args.iter()
        .skip(1) // Ignora el primer argumento (el path del ejecutable)
        .find(|arg| !arg.starts_with("--")); // Encuentra el primer argumento que no sea un flag

    let mut solvers = SolverRegistry::new();
    solvers.insert("pow".to_string(), Arc::new(ProofOfWorkSolver));
    let solvers = Arc::new(solvers);

    match mode.map(String::as_str) {
        Some("listen") => {
            println!("[Modo Oyente] Esperando un desafío...");
            listen(solvers).await?;
        }
        Some("challenge") => {
            println!("[Modo Retador] Iniciando un desafío...");
            challenge(solvers).await?;
        }
        _ => {
            // Solo muestra el error si no se está ejecutando únicamente con --stats
            if !args.contains(&"--stats".to_string()) || args.len() == 1 {
                 eprintln!("Error: modo no especificado. Uso: cargo run -p competitor -- [listen|challenge] [--stats]");
            } else if args.len() > 1 && mode.is_none() {
                // Si solo se pasó --stats, simplemente esperamos para que el reportero funcione
                tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
            }
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

async fn handle_duel(mut socket: TcpStream, solvers: Arc<SolverRegistry>, mode: &str) -> Result<(), Box<dyn Error>> {
    let mut difficulty = 4; // Dificultad inicial estándar
    let solver_name = "pow";
    let solver = solvers.get(solver_name).ok_or("Solver no encontrado")?;

    let mut last_challenge_sent: Option<Challenge> = None;

    if mode == "challenge" {
        println!("\n--- TURNO 1 (Retador) ---");
        // No se aumenta la dificultad inicial, se usa la estándar.
        let challenge = solver.create_challenge(difficulty)?;
        
        println!("Enviando desafío inicial con dificultad {}...", difficulty);
        send_message(&mut socket, &P2PMessage::Challenge(challenge.clone())).await?;
        last_challenge_sent = Some(challenge);
    }

    loop {
        match read_message(&mut socket).await? {
            P2PMessage::Challenge(challenge) => {
                println!("\n--- Desafío Recibido: '{}' ---", challenge.name);
                let current_solver = solvers.get(&challenge.name).ok_or("Solver no encontrado")?;
                
                let challenge_payload = challenge.payload.clone();
                let solver_clone = current_solver.clone();

                let pb = ProgressBar::new_spinner();
                pb.enable_steady_tick(Duration::from_millis(120));
                pb.set_style(
                    ProgressStyle::with_template("{spinner:.green} {msg}")
                        .unwrap()
                        .tick_strings(&["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"]),
                );
                pb.set_message("Resolviendo desafío (en paralelo)...");

                let solution = tokio::task::spawn_blocking(move || {
                    solver_clone.solve(&challenge_payload)
                }).await??;

                // Lógica para mostrar los resultados del desafío resuelto
                let config = standard();
                let (pow_input, _): (PoWInput, usize) = decode_from_slice(&challenge.payload, config)?;
                let (pow_output, _): (PoWOutput, usize) = decode_from_slice(&solution.payload, config)?;
                
                let mut final_hasher = sha2::Sha256::new();
                final_hasher.update(pow_input.data.as_bytes());
                final_hasher.update(&pow_output.nonce.to_le_bytes());
                let hash_hex = hex::encode(final_hasher.finalize());

                pb.finish_with_message(format!(
                    "✅ ¡Solución Encontrada! Nonce: {}, Hash: {}...",
                    pow_output.nonce,
                    &hash_hex[..8]
                ));

                // La dificultad del siguiente desafío se basa en la del que acabamos de resolver
                let next_difficulty = pow_input.difficulty + 1;
                let next_challenge = current_solver.create_challenge(next_difficulty)?;

                send_message(&mut socket, &P2PMessage::Reply(solution)).await?;
                send_message(&mut socket, &P2PMessage::Challenge(next_challenge.clone())).await?;
                last_challenge_sent = Some(next_challenge);
            }
            P2PMessage::Reply(solution) => {
                println!("Respuesta del oponente recibida. Verificando...");
                
                if let Some(challenge_to_review) = last_challenge_sent.as_ref() {
                    match solver.review_solution(&challenge_to_review.payload, &solution) {
                        Ok(()) => {
                            println!("¡Solución del oponente es VÁLIDA! Esperando próximo desafío...");
                        }
                        Err(e) => {
                            let reason = format!("Solución inválida: {:?}", e);
                            println!("¡VICTORIA! Oponente envió una solución incorrecta. Motivo: {}", reason);
                            send_message(&mut socket, &P2PMessage::YouWin(reason)).await?;
                            break;
                        }
                    }
                } else {
                    return Err("Se recibió una respuesta sin haber enviado un desafío.".into());
                }
            }
            P2PMessage::YouWin(reason) => {
                println!("\n¡DERROTA! Motivo: {}", reason);
                break;
            }
        }
    }
    Ok(())
}
async fn spawn_stats_reporter() {
    let mut sys = System::new_all(); // new_all() carga todo inicialmente

    println!("-- Reportero de Estadísticas Activado --");
    loop {
        // La forma simple y correcta de refrescar los datos necesarios
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let mem_usage = sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0;
        let mut output = format!("\r\x1B[K📊 Memoria: {:.2}% ", mem_usage);

        // Iteramos sobre cada CPU para obtener su uso individual
        for (i, cpu) in sys.cpus().iter().enumerate() {
            output.push_str(&format!("| CPU-{}: {:.1}% ", i, cpu.cpu_usage()));
        }

        print!("{}", output);
        std::io::stdout().flush().unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

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