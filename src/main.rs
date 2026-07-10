use smart_home::device::SmartDevice;
use smart_home::house::SmartHouse;
use smart_home::room::Room;
use smart_home::socket::SmartSocket;
use smart_home::thermometer::SmartThermometer;

fn main() {
    let mut house = SmartHouse::new(
        "Мой дом".to_string(),
        vec![
            Room::new(
                "Гостиная".to_string(),
                vec![
                    SmartDevice::Thermometer(SmartThermometer::new(
                        "Термометр гостиной".to_string(),
                        21.5,
                    )),
                    SmartDevice::Socket({
                        let mut s = SmartSocket::new("Розетка гостиной".to_string(), 150.0);
                        s.turn_on();
                        s
                    }),
                ],
            ),
            Room::new(
                "Кухня".to_string(),
                vec![
                    SmartDevice::Thermometer(SmartThermometer::new(
                        "Термометр кухни".to_string(),
                        24.0,
                    )),
                    SmartDevice::Socket({
                        let mut s = SmartSocket::new("Розетка кухни".to_string(), 200.0);
                        s.turn_on();
                        s
                    }),
                ],
            ),
        ],
    );

    println!("Первичный отчет:");
    house.report();

    // Выключаем розетку в гостиной (комната 0, устройство 1).
    if let SmartDevice::Socket(socket) = house.room_mut(0).device_mut(1) {
        socket.turn_off();
    }

    println!();
    println!("Отчет после выключения розетки в гостиной:");
    house.report();
}
