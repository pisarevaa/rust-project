/// Умный дом, собранный из устройств, которые работают с имитаторами.

use smart_home::device::SmartDevice;
use smart_home::house::SmartHouse;
use smart_home::report::Report;
use smart_home::room;
use smart_home::socket::SmartSocket;
use smart_home::thermometer::SmartThermometer;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

const SOCKET_ADDRESS: &str = "127.0.0.1:8181";
const THERMOMETER_ADDRESS: &str = "127.0.0.1:8182";
const OFFLINE_SOCKET_ADDRESS: &str = "127.0.0.1:8199";
const WARMUP: Duration = Duration::from_millis(1200);

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let socket_address = args.next().unwrap_or_else(|| SOCKET_ADDRESS.to_string());
    let thermometer_address = args
        .next()
        .unwrap_or_else(|| THERMOMETER_ADDRESS.to_string());

    let thermometer = match SmartThermometer::bind(&thermometer_address) {
        Ok(thermometer) => thermometer,
        Err(e) => {
            eprintln!("не удалось занять адрес «{thermometer_address}»: {e}");
            eprintln!("вероятно, он уже занят другим экземпляром примера");
            return ExitCode::FAILURE;
        }
    };

    let mut house = SmartHouse::new("Дом с имитаторами");
    house.add_room(
        "Гостиная",
        room! {
            "Розетка гостиной" => SmartSocket::reconnecting(&socket_address),
            "Термометр гостиной" => thermometer,
        },
    );
    house.add_room(
        "Кладовка",
        room! {
            "Розетка кладовки" => SmartSocket::reconnecting(OFFLINE_SOCKET_ADDRESS),
            "Термометр кладовки" => SmartThermometer::bind("127.0.0.1:0")
                .expect("свободный порт всегда находится"),
        },
    );

    println!("Имитатор розетки: {socket_address}");
    println!("Термометр слушает: {thermometer_address}");
    println!(
        "Ждем первый пакет с температурой ({} мс)...",
        WARMUP.as_millis()
    );
    thread::sleep(WARMUP);

    println!();
    println!("{}", house.report());

    control_socket(&house);

    println!();
    println!("Отчет после управления розеткой:");
    println!("{}", house.report());

    ExitCode::SUCCESS
}

fn control_socket(house: &SmartHouse) {
    println!();
    println!("=== Управление розеткой ===");

    let device = match house.device("Гостиная", "Розетка гостиной") {
        Ok(device) => device,
        Err(e) => {
            eprintln!("не удалось найти устройство: {e}");
            return;
        }
    };

    let SmartDevice::Socket(socket) = device else {
        eprintln!("ожидалась розетка");
        return;
    };

    match socket.turn_on() {
        Ok(()) => println!("Розетка гостиной включена."),
        Err(e) => println!("Не удалось включить розетку гостиной: {e}"),
    }

    match socket.current_power() {
        Ok(power) => println!("Текущая мощность: {power:.1} Вт"),
        Err(e) => println!("Не удалось узнать мощность: {e}"),
    }
}
