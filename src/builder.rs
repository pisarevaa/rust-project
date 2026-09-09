//! Билдер умного дома: комнаты и устройства описываются одной цепочкой вызовов.
//!
//! Состояние билдера закодировано в типе, поэтому добавить устройство
//! до первой комнаты не получится — такой код не скомпилируется:
//!
//! ```compile_fail
//! use smart_home::builder::SmartHouseBuilder;
//! use smart_home::socket::SmartSocket;
//!
//! let house = SmartHouseBuilder::new("Дом")
//!     .add_device("Розетка", SmartSocket::mock(60.0))
//!     .build();
//! ```

use crate::device::SmartDevice;
use crate::house::SmartHouse;
use crate::room::Room;
use std::marker::PhantomData;

/// Состояние билдера: ни одной комнаты еще не открыто.
#[derive(Debug)]
pub struct NoRooms;

/// Состояние билдера: есть открытая комната, в нее можно класть устройства.
#[derive(Debug)]
pub struct InRoom;

/// Пошагово собирает [`SmartHouse`].
///
/// Параметр `S` — состояние сборки: [`NoRooms`] или [`InRoom`].
/// Метод [`add_device`](SmartHouseBuilder::add_device) существует только
/// в состоянии [`InRoom`], поэтому порядок вызовов проверяет компилятор.
#[derive(Debug)]
pub struct SmartHouseBuilder<S> {
    name: String,
    rooms: Vec<(String, Room)>,
    state: PhantomData<S>,
}

impl SmartHouseBuilder<NoRooms> {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rooms: Vec::new(),
            state: PhantomData,
        }
    }
}

impl<S> SmartHouseBuilder<S> {
    #[must_use]
    pub fn add_room(mut self, name: impl Into<String>) -> SmartHouseBuilder<InRoom> {
        let name = name.into();
        let known = self
            .rooms
            .iter()
            .position(|(existing, _)| *existing == name);
        let room = match known {
            Some(position) => self.rooms.remove(position).1,
            None => Room::new(),
        };
        self.rooms.push((name, room));

        SmartHouseBuilder {
            name: self.name,
            rooms: self.rooms,
            state: PhantomData,
        }
    }

    #[must_use]
    pub fn build(self) -> SmartHouse {
        let mut house = SmartHouse::new(self.name);
        for (name, room) in self.rooms {
            house.add_room(name, room);
        }
        house
    }
}

impl SmartHouseBuilder<InRoom> {
    #[must_use]
    pub fn add_device(mut self, name: impl Into<String>, device: impl Into<SmartDevice>) -> Self {
        if let Some((_, room)) = self.rooms.last_mut() {
            room.add_device(name, device);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::Report;
    use crate::socket::SmartSocket;
    use crate::thermometer::SmartThermometer;

    #[test]
    fn builder_without_rooms_makes_an_empty_house() {
        let house = SmartHouseBuilder::new("Дом").build();
        assert_eq!(house.name(), "Дом");
        assert_eq!(house.room_names().count(), 0);
    }

    #[test]
    fn builder_creates_an_empty_room() {
        let house = SmartHouseBuilder::new("Дом").add_room("Кладовка").build();
        assert_eq!(house.room_names().collect::<Vec<_>>(), ["Кладовка"]);
    }

    #[test]
    fn devices_land_in_the_last_opened_room() {
        let house = SmartHouseBuilder::new("Дом")
            .add_room("Кухня")
            .add_device("Розетка кухни", SmartSocket::mock(60.0))
            .add_room("Спальня")
            .add_device("Термометр спальни", SmartThermometer::mock(19.0))
            .build();

        assert_eq!(
            house
                .room("Кухня")
                .expect("комната была добавлена")
                .device_names()
                .collect::<Vec<_>>(),
            ["Розетка кухни"]
        );
        assert_eq!(
            house
                .room("Спальня")
                .expect("комната была добавлена")
                .device_names()
                .collect::<Vec<_>>(),
            ["Термометр спальни"]
        );
    }

    #[test]
    fn several_devices_fit_into_one_room() {
        let house = SmartHouseBuilder::new("Дом")
            .add_room("Гостиная")
            .add_device("Розетка", SmartSocket::mock(150.0))
            .add_device("Термометр", SmartThermometer::mock(21.5))
            .build();

        assert_eq!(
            house
                .room("Гостиная")
                .expect("комната была добавлена")
                .device_names()
                .collect::<Vec<_>>(),
            ["Розетка", "Термометр"]
        );
    }

    #[test]
    fn reopening_a_room_keeps_its_devices() {
        let house = SmartHouseBuilder::new("Дом")
            .add_room("Кухня")
            .add_device("Розетка", SmartSocket::mock(60.0))
            .add_room("Спальня")
            .add_device("Ночник", SmartSocket::mock(15.0))
            .add_room("Кухня")
            .add_device("Термометр", SmartThermometer::mock(24.0))
            .build();

        assert_eq!(house.room_names().collect::<Vec<_>>(), ["Кухня", "Спальня"]);
        assert_eq!(
            house
                .room("Кухня")
                .expect("комната была добавлена")
                .device_names()
                .collect::<Vec<_>>(),
            ["Розетка", "Термометр"]
        );
    }

    #[test]
    fn house_builder_shortcut_returns_the_same_builder() {
        let house = SmartHouse::builder("Дом")
            .add_room("Кухня")
            .add_device("Розетка", SmartSocket::mock(60.0))
            .build();

        assert_eq!(
            house.report(),
            concat!(
                "=== Отчет о доме «Дом» ===\n",
                "Комната «Кухня»:\n",
                "  Розетка: розетка выключена, мощность 0.0 Вт"
            )
        );
    }
}
