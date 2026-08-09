/// Имитатор умного термометра.

use smart_home::config::SimulatorConfig;
use smart_home::simulator::thermometer::ThermometerSimulator;
use std::process::ExitCode;

const DEFAULT_CONFIG: &str = "thermometer-simulator.conf";

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG.to_string());

    let config = match SimulatorConfig::load(&path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("не удалось прочитать «{path}»: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut simulator = match ThermometerSimulator::new(config.clone()) {
        Ok(simulator) => simulator,
        Err(e) => {
            eprintln!(
                "не удалось подготовить отправку на «{}»: {e}",
                config.address
            );
            return ExitCode::FAILURE;
        }
    };

    println!(
        "Имитатор термометра шлет показания на {} каждые {} мс. Ctrl+C для остановки.",
        simulator.target(),
        config.period.as_millis()
    );
    simulator.run();
}
