//! Билдер умного дома: весь дом описывается одной цепочкой вызовов.

use smart_home::house::SmartHouse;
use smart_home::report::Report;
use smart_home::socket::SmartSocket;
use smart_home::thermometer::SmartThermometer;

fn main() {
    let house = SmartHouse::builder("Дом на билдере")
        .add_room("Гостиная")
        .add_device("Розетка гостиной", SmartSocket::mock(150.0))
        .add_device("Термометр гостиной", SmartThermometer::mock(21.5))
        .add_room("Кухня")
        .add_device("Розетка кухни", SmartSocket::mock(200.0))
        .add_device("Термометр кухни", SmartThermometer::mock(24.0))
        // Комнату можно открыть повторно — устройства просто добавятся к уже собранным.
        .add_room("Гостиная")
        .add_device("Торшер", SmartSocket::mock(40.0))
        .build();

    println!("{}", house.report());

    println!();
    println!("Комнаты: {:?}", house.room_names().collect::<Vec<_>>());

    println!();
    println!("Билдер до первой комнаты не дает добавить устройство:");
    println!("    SmartHouse::builder(\"Дом\").add_device(...) // не компилируется");
    println!("Метод add_device есть только у SmartHouseBuilder<InRoom>,");
    println!("а builder() возвращает SmartHouseBuilder<NoRooms>.");
}
