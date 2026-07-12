use smart_home::device::SmartDevice;
use smart_home::error::SmartHouseError;
use smart_home::house::SmartHouse;
use smart_home::report::Report;
use smart_home::room;
use smart_home::room::Room;
use smart_home::socket::SmartSocket;
use smart_home::thermometer::SmartThermometer;

fn print_report(item: &impl Report) {
    println!("{}", item.report());
}

fn main() {
    let mut house = SmartHouse::new("Мой дом");

    house.add_room(
        "Гостиная",
        room! {
            "Термометр гостиной" => SmartThermometer::new(21.5),
            "Розетка гостиной" => SmartSocket::new(150.0),
        },
    );
    house.add_room(
        "Кухня",
        room! {
            "Термометр кухни" => SmartThermometer::new(24.0),
            "Розетка кухни" => SmartSocket::new(200.0),
        },
    );

    for (room, device) in [("Гостиная", "Розетка гостиной"), ("Кухня", "Розетка кухни")]
    {
        if let Ok(SmartDevice::Socket(socket)) = house.device_mut(room, device) {
            socket.turn_on();
        }
    }

    println!("Первичный отчет:");
    print_report(&house);

    println!();
    println!("Отчет об отдельной комнате «Гостиная»:");
    let living_room = house.room("Гостиная").expect("комната была добавлена");
    print_report(living_room);

    println!();
    println!("Отчет об отдельном устройстве «Розетка гостиной»:");
    let socket = house
        .device("Гостиная", "Розетка гостиной")
        .expect("устройство было добавлено");
    print_report(socket);

    demo_dynamic_rooms(&mut house);
    demo_dynamic_devices(&mut house);
    demo_errors(&house);
}

/// Динамическое добавление и удаление комнаты.
fn demo_dynamic_rooms(house: &mut SmartHouse) {
    println!();
    println!("=== Динамическое управление комнатами ===");

    house.add_room("Спальня", room![]);
    println!("Добавили «Спальня». Комнаты: {:?}", room_names(house));

    let removed = house.remove_room("Кухня");
    println!(
        "Удалили «Кухня» ({} устройств). Комнаты: {:?}",
        removed.map_or(0, |room| room.device_names().count()),
        room_names(house)
    );

    println!(
        "Повторное удаление «Кухня» вернуло: {:?}",
        house.remove_room("Кухня").map(|_: Room| "комната")
    );
}

/// Динамическое добавление и удаление устройства.
fn demo_dynamic_devices(house: &mut SmartHouse) {
    println!();
    println!("=== Динамическое управление устройствами ===");

    let bedroom = house.room_mut("Спальня").expect("комната была добавлена");
    bedroom.add_device("Термометр спальни", SmartThermometer::new(19.0));
    bedroom.add_device("Ночник", SmartSocket::new(15.0));
    println!("Добавили два устройства в «Спальня»:");
    print_report(house.room("Спальня").expect("комната была добавлена"));

    let bedroom = house.room_mut("Спальня").expect("комната была добавлена");
    let removed = bedroom.remove_device("Ночник");
    println!(
        "Удалили «Ночник» (был он там: {}). Осталось:",
        removed.is_some()
    );
    print_report(house.room("Спальня").expect("комната была добавлена"));

    println!();
    println!("Отчет о доме после всех изменений:");
    print_report(house);
}

/// Обработка ошибок сквозного доступа к устройству.
fn demo_errors(house: &SmartHouse) {
    println!();
    println!("=== Обработка ошибок ===");

    for (room, device) in [("Кухня", "Розетка кухни"), ("Спальня", "Ночник")]
    {
        match house.device(room, device) {
            Ok(device) => println!("Найдено: {}", device.report()),
            Err(e @ SmartHouseError::RoomNotFound { .. }) => {
                println!("Ошибка поиска комнаты: {e}");
            }
            Err(e @ SmartHouseError::DeviceNotFound { .. }) => {
                println!("Ошибка поиска устройства: {e}");
            }
        }
    }

    let boxed: Box<dyn std::error::Error> = Box::new(
        house
            .device("Подвал", "Розетка")
            .expect_err("комнаты «Подвал» нет"),
    );
    println!("Как Box<dyn Error>: {boxed}");
}

fn room_names(house: &SmartHouse) -> Vec<&str> {
    house.room_names().collect()
}
