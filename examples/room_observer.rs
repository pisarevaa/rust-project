//! Наблюдатели комнаты: подписчик узнает о каждом новом устройстве.

use smart_home::device::SmartDevice;
use smart_home::report::Report;
use smart_home::room::{DeviceSubscriber, Room};
use smart_home::socket::SmartSocket;
use smart_home::thermometer::SmartThermometer;
use std::cell::RefCell;
use std::rc::Rc;

/// Подписчик-объект: ведет журнал добавленных устройств.
#[derive(Default)]
struct Journal {
    entries: Rc<RefCell<Vec<String>>>,
}

impl DeviceSubscriber for Journal {
    fn on_device_added(&mut self, name: &str, device: &SmartDevice) {
        self.entries
            .borrow_mut()
            .push(format!("{name} — {}", device.report()));
    }
}

fn main() {
    let entries = Rc::new(RefCell::new(Vec::new()));
    let mut room = Room::new();

    // Подписчик-объект.
    room.subscribe(Journal {
        entries: Rc::clone(&entries),
    });

    // Подписчик-замыкание.
    let mut power = 0.0_f64;
    room.subscribe(move |name: &str, device: &SmartDevice| {
        if let SmartDevice::Socket(socket) = device {
            match socket.current_power() {
                Ok(value) => {
                    power += value;
                    println!("Замыкание: «{name}» добавлена, суммарная мощность {power:.1} Вт");
                }
                Err(e) => println!("Замыкание: «{name}» недоступна: {e}"),
            }
        } else {
            println!("Замыкание: «{name}» — не розетка, мощность не меняется");
        }
    });

    println!("Подписчиков: {}", room.subscriber_count());
    println!();

    let socket = SmartSocket::mock(150.0);
    if let Err(e) = socket.turn_on() {
        eprintln!("не удалось включить розетку: {e}");
    }
    room.add_device("Розетка гостиной", socket);
    room.add_device("Термометр гостиной", SmartThermometer::mock(21.5));

    let torch = SmartSocket::mock(40.0);
    if let Err(e) = torch.turn_on() {
        eprintln!("не удалось включить торшер: {e}");
    }
    room.add_device("Торшер", torch);

    println!();
    println!("Журнал подписчика-объекта:");
    for entry in entries.borrow().iter() {
        println!("  {entry}");
    }

    println!();
    println!(
        "Отписываем всех ({} шт.) и добавляем еще одно устройство:",
        room.unsubscribe_all()
    );
    room.add_device("Ночник", SmartSocket::mock(15.0));
    println!("Журнал не изменился: {} записей", entries.borrow().len());

    println!();
    println!("Итоговая комната:");
    println!("{}", room.report());
}
