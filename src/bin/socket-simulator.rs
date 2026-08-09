/// Имитатор умной розетки.

use smart_home::simulator::socket::SocketSimulator;
use std::process::ExitCode;

const DEFAULT_ADDRESS: &str = "127.0.0.1:8181";
const POWER: f64 = 150.0;

fn main() -> ExitCode {
    let address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_ADDRESS.to_string());

    let mut simulator = match SocketSimulator::bind(&address, POWER) {
        Ok(simulator) => simulator,
        Err(e) => {
            eprintln!("не удалось занять адрес «{address}»: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "Имитатор розетки слушает {} (мощность {POWER:.1} Вт). Ctrl+C для остановки.",
        simulator.local_addr()
    );
    simulator.run();
}
