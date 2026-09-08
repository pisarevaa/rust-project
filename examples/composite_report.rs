//! Компоновщик отчетов: разные объекты умного дома собираются в один отчет.

use smart_home::house::SmartHouse;
use smart_home::report::Reporter;
use smart_home::room;
use smart_home::socket::SmartSocket;
use smart_home::thermometer::SmartThermometer;

fn main() {
    let socket = SmartSocket::mock(150.0);
    if let Err(e) = socket.turn_on() {
        eprintln!("не удалось включить розетку: {e}");
    }
    let thermometer = SmartThermometer::mock(21.5);

    let room = room! {
        "Розетка кухни" => SmartSocket::mock(200.0),
        "Термометр кухни" => SmartThermometer::mock(24.0),
    };

    let house = SmartHouse::builder("Дом с компоновщиком")
        .add_room("Спальня")
        .add_device("Ночник", SmartSocket::mock(15.0))
        .add_device("Термометр спальни", SmartThermometer::mock(19.0))
        .build();

    println!("Отчет обо всех объектах сразу:");
    // Типы объектов разные, но известны компилятору: трейт-объектов здесь нет.
    Reporter::new()
        .add(&socket)
        .add(&thermometer)
        .add(&room)
        .add(&house)
        .report();

    println!();
    println!("Тот же компоновщик, но только об отдельных устройствах:");
    Reporter::new().add(&socket).add(&thermometer).report();

    println!();
    println!("Пустой компоновщик печатает пустую строку:");
    Reporter::new().report();
}
