# cpu_combat

⚔️ CPU Combat ⚔️
Un proyecto para aprender Rust a través de la creación de una aplicación de "peleas de computadoras", donde los programas compiten para resolver problemas intensivos en CPU.

## ¿Qué es?
CPU Combat es una aplicación cliente-servidor y peer-to-peer diseñada como un ejercicio práctico para explorar conceptos de bajo nivel en Rust. En lugar de una interfaz gráfica, los "combatientes" son aplicaciones de consola que se conectan entre sí a través de la red para competir.

El objetivo principal es aprender sobre:

- Networking Asíncrono con Tokio.
- Serialización Binaria de alto rendimiento con Bincode 2.0.
- Manejo de Concurrencia y estado compartido.
- Computación de Alto Rendimiento y optimización de algoritmos.
- Estructura de Proyectos en Rust con workspaces y módulos.

## ¿Cómo Funciona?
Actualmente, el proyecto implementa un modo de duelo 1-vs-1 (Peer-to-Peer).

1.  **Conexión Directa**: Dos instancias del programa `competitor` se conectan directamente. Una actúa como "oyente" (listen) y la otra como "retador" (challenge).
2.  **El Desafío**: El reto implementado es una Prueba de Trabajo (Proof-of-Work), donde un competidor debe encontrar un nonce que, al ser hasheado (SHA-256) junto con unos datos, produce un resultado que empieza con un número específico de ceros.
3.  **Combate por Turnos**:
    *   El retador inicia enviando el primer desafío con una dificultad base.
    *   El oyente recibe el desafío, lo resuelve, y responde con la solución y un nuevo desafío de dificultad incrementada.
    *   El ciclo se repite, y cada turno se vuelve computacionalmente más costoso.
4.  **Victoria/Derrota**: Un competidor pierde si su programa no logra encontrar una solución dentro del límite de intentos programado y devuelve un error.

## Cómo Ejecutarlo
Para probar el combate 1-vs-1, necesitas tener el toolchain de Rust instalado.

**Clona el Repositorio (si estuviera en uno):**
```bash
git clone <url-del-repositorio>
cd cpu_combat
```

Abre dos terminales en la raíz del proyecto.

**En la Primera Terminal (El Oyente):**
Ejecuta el siguiente comando para poner una instancia en modo de espera.
```bash
cargo run -p competitor -- listen
```

**En la Segunda Terminal (El Retador):**
Ejecuta este comando para que la segunda instancia inicie el combate.
```bash
cargo run -p competitor -- challenge
```
Verás en ambas consolas cómo los programas intercambian desafíos, los resuelven y la dificultad aumenta progresivamente.

## 🗺️ Roadmap Futuro
Este proyecto está en desarrollo. Las próximas características planeadas son:

- **Implementar el Árbitro**: Crear el programa `arbiter` que actuará como un nodo central.
- **Modo "Free For All"**: Permitir que múltiples competidores se conecten al Árbitro para competir simultáneamente en una misma prueba.
- **Scoreboard Global**: El Árbitro mantendrá una tabla de clasificación en tiempo real.
- **Más Desafíos**: Añadir nuevos tipos de `Solvers`, como:
    - Cálculo de números primos en rangos grandes.
    - Resolución de problemas de grafos (ej. Problema del viajante).
- **Despliegue con Docker**: Crear un `docker-compose.yml` para desplegar fácilmente el Árbitro y múltiples competidores en contenedores.