// competitor/src/main.rs

use std::error::Error;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Decide el modo basado en los argumentos:
    // `cargo run -p competitor -- listen` para ser el oyente.
    // `cargo run -p competitor -- challenge` para ser el retador.
    let mode = args.get(1).map(String::as_str);

    match mode {
        Some("listen") => {
            println!("[Modo Oyente] Esperando un desafío...");
            listen().await?;
        }
        Some("challenge") => {
            println!("[Modo Retador] Iniciando un desafío...");
            challenge().await?;
        }
        _ => {
            eprintln!("Error: modo no especificado.");
            eprintln!("Uso: cargo run -p competitor -- [listen|challenge]");
        }
    }

    Ok(())
}

/// Escucha conexiones entrantes de otros competidores.
async fn listen() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (socket, addr) = listener.accept().await?;
    
    println!("¡Conexión recibida de {}!", addr);
    
    // Aquí irá la lógica para manejar la pelea.
    handle_duel(socket).await?;

    Ok(())
}

/// Se conecta a un competidor que está escuchando.
async fn challenge() -> Result<(), Box<dyn Error>> {
    let socket = TcpStream::connect("127.0.0.1:8080").await?;
    
    println!("¡Conectado al oponente!");

    // Aquí irá la lógica para manejar la pelea.
    handle_duel(socket).await?;

    Ok(())
}

/// Lógica principal del duelo (aún por implementar).
async fn handle_duel(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("Manejando el duelo... (lógica por implementar)");
    // TODO: Enviar y recibir mensajes P2PMessage usando bincode.
    Ok(())
}