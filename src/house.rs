use crate::device::SmartDevice;
use crate::error::SmartHouseError;
use crate::report::Report;
use crate::room::Room;
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Debug)]
pub struct SmartHouse {
    name: String,
    rooms: BTreeMap<String, Room>,
}

impl SmartHouse {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rooms: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_room(&mut self, name: impl Into<String>, room: Room) -> Option<Room> {
        self.rooms.insert(name.into(), room)
    }

    pub fn remove_room(&mut self, name: &str) -> Option<Room> {
        self.rooms.remove(name)
    }

    #[must_use]
    pub fn room(&self, name: &str) -> Option<&Room> {
        self.rooms.get(name)
    }

    pub fn room_mut(&mut self, name: &str) -> Option<&mut Room> {
        self.rooms.get_mut(name)
    }

    pub fn room_names(&self) -> impl Iterator<Item = &str> {
        self.rooms.keys().map(String::as_str)
    }

    /// Возвращает устройство по имени комнаты и имени устройства.
    ///
    /// # Errors
    /// [`SmartHouseError::RoomNotFound`], если комнаты нет;
    /// [`SmartHouseError::DeviceNotFound`], если комната есть, а устройства в ней нет.
    pub fn device(&self, room: &str, device: &str) -> Result<&SmartDevice, SmartHouseError> {
        self.room(room)
            .ok_or_else(|| SmartHouseError::RoomNotFound {
                room: room.to_string(),
            })?
            .device(device)
            .ok_or_else(|| SmartHouseError::DeviceNotFound {
                room: room.to_string(),
                device: device.to_string(),
            })
    }

    /// Возвращает изменяемую ссылку на устройство.
    ///
    /// # Errors
    /// Те же, что у [`SmartHouse::device`].
    pub fn device_mut(
        &mut self,
        room: &str,
        device: &str,
    ) -> Result<&mut SmartDevice, SmartHouseError> {
        self.rooms
            .get_mut(room)
            .ok_or_else(|| SmartHouseError::RoomNotFound {
                room: room.to_string(),
            })?
            .device_mut(device)
            .ok_or_else(|| SmartHouseError::DeviceNotFound {
                room: room.to_string(),
                device: device.to_string(),
            })
    }
}

impl Report for SmartHouse {
    fn report(&self) -> String {
        let mut report = format!("=== Отчет о доме «{}» ===", self.name);
        for (name, room) in &self.rooms {
            let _ = write!(report, "\nКомната «{name}»:");
            let devices = room.report();
            if !devices.is_empty() {
                let _ = write!(report, "\n{devices}");
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room;
    use crate::socket::SmartSocket;
    use crate::thermometer::SmartThermometer;

    fn sample_house() -> SmartHouse {
        let mut house = SmartHouse::new("Дом");
        house.add_room(
            "Кухня",
            room! {
                "Розетка" => SmartSocket::new(60.0),
            },
        );
        house
    }

    #[test]
    fn add_room_returns_none_for_a_fresh_name() {
        let mut house = SmartHouse::new("Дом");
        assert!(house.add_room("Кухня", Room::new()).is_none());
    }

    #[test]
    fn add_room_returns_the_displaced_room() {
        let mut house = sample_house();
        let displaced = house.add_room("Кухня", Room::new());
        assert!(displaced.is_some());
    }

    #[test]
    fn remove_room_returns_the_removed_room() {
        let mut house = sample_house();
        let removed = house.remove_room("Кухня").expect("комната была добавлена");
        assert_eq!(removed.device_names().count(), 1);
        assert!(house.room("Кухня").is_none());
    }

    #[test]
    fn remove_room_returns_none_for_unknown_name() {
        let mut house = sample_house();
        assert!(house.remove_room("Спальня").is_none());
    }

    #[test]
    fn room_returns_reference_by_name() {
        let house = sample_house();
        let kitchen = house.room("Кухня").expect("комната была добавлена");
        assert_eq!(kitchen.device_names().collect::<Vec<_>>(), ["Розетка"]);
    }

    #[test]
    fn room_returns_none_for_unknown_name_instead_of_panicking() {
        let house = sample_house();
        assert!(house.room("Спальня").is_none());
    }

    #[test]
    fn room_mut_allows_adding_a_device() {
        let mut house = sample_house();
        let kitchen = house.room_mut("Кухня").expect("комната была добавлена");
        kitchen.add_device("Термометр", SmartThermometer::new(24.0));
        assert_eq!(house.room("Кухня").unwrap().device_names().count(), 2);
    }

    #[test]
    fn room_names_are_sorted() {
        let mut house = sample_house();
        house.add_room("Гостиная", Room::new());
        assert_eq!(
            house.room_names().collect::<Vec<_>>(),
            ["Гостиная", "Кухня"]
        );
    }

    #[test]
    fn device_returns_reference_to_the_device() {
        let house = sample_house();
        let device = house.device("Кухня", "Розетка").expect("устройство есть");
        assert!(matches!(device, SmartDevice::Socket(_)));
    }

    #[test]
    fn device_reports_missing_room() {
        let house = sample_house();
        assert_eq!(
            house.device("Спальня", "Розетка").err(),
            Some(SmartHouseError::RoomNotFound {
                room: "Спальня".to_string()
            })
        );
    }

    #[test]
    fn device_reports_missing_device_in_an_existing_room() {
        let house = sample_house();
        assert_eq!(
            house.device("Кухня", "Лампа").err(),
            Some(SmartHouseError::DeviceNotFound {
                room: "Кухня".to_string(),
                device: "Лампа".to_string(),
            })
        );
    }

    #[test]
    fn device_mut_allows_switching_a_socket_on() {
        let mut house = sample_house();
        let device = house
            .device_mut("Кухня", "Розетка")
            .expect("устройство есть");
        let SmartDevice::Socket(socket) = device else {
            panic!("ожидалась розетка");
        };
        socket.turn_on();
        assert!(socket.is_on());
    }

    #[test]
    fn device_mut_reports_missing_room() {
        let mut house = sample_house();
        assert_eq!(
            house.device_mut("Спальня", "Розетка").err(),
            Some(SmartHouseError::RoomNotFound {
                room: "Спальня".to_string()
            })
        );
    }

    #[test]
    fn report_nests_rooms_and_devices() {
        let mut house = sample_house();
        house.add_room(
            "Гостиная",
            room! {
                "Термометр" => SmartThermometer::new(21.5),
            },
        );
        assert_eq!(
            house.report(),
            concat!(
                "=== Отчет о доме «Дом» ===\n",
                "Комната «Гостиная»:\n",
                "  Термометр: термометр показывает 21.5 °C\n",
                "Комната «Кухня»:\n",
                "  Розетка: розетка выключена, мощность 0.0 Вт"
            )
        );
    }

    #[test]
    fn report_of_an_empty_room_has_no_blank_line() {
        let mut house = SmartHouse::new("Дом");
        house.add_room("Кладовка", Room::new());
        assert_eq!(
            house.report(),
            "=== Отчет о доме «Дом» ===\nКомната «Кладовка»:"
        );
    }

    #[test]
    fn report_of_an_empty_house_is_a_header_only() {
        let house = SmartHouse::new("Дом");
        assert_eq!(house.report(), "=== Отчет о доме «Дом» ===");
    }
}
