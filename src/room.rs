use crate::device::SmartDevice;
use crate::report::Report;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Room {
    devices: BTreeMap<String, SmartDevice>,
}

impl Room {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_device(
        &mut self,
        name: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        self.devices.insert(name.into(), device.into())
    }

    pub fn remove_device(&mut self, name: &str) -> Option<SmartDevice> {
        self.devices.remove(name)
    }

    #[must_use]
    pub fn device(&self, name: &str) -> Option<&SmartDevice> {
        self.devices.get(name)
    }

    pub fn device_mut(&mut self, name: &str) -> Option<&mut SmartDevice> {
        self.devices.get_mut(name)
    }

    pub fn device_names(&self) -> impl Iterator<Item = &str> {
        self.devices.keys().map(String::as_str)
    }
}

impl Report for Room {
    fn report(&self) -> String {
        self.devices
            .iter()
            .map(|(name, device)| format!("  {name}: {}", device.report()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[macro_export]
macro_rules! room {
    ($($name:expr => $device:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut room = $crate::room::Room::new();
        $( room.add_device($name, $device); )*
        room
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::SmartSocket;
    use crate::thermometer::SmartThermometer;

    fn sample_room() -> Room {
        let mut room = Room::new();
        room.add_device("Термометр", SmartThermometer::new(22.0));
        room.add_device("Розетка", SmartSocket::new(60.0));
        room
    }

    #[test]
    fn add_device_returns_none_for_a_fresh_name() {
        let mut room = Room::new();
        assert!(room.add_device("Розетка", SmartSocket::new(60.0)).is_none());
    }

    #[test]
    fn add_device_returns_the_displaced_device() {
        let mut room = sample_room();
        let displaced = room.add_device("Розетка", SmartSocket::new(90.0));
        assert!(matches!(displaced, Some(SmartDevice::Socket(_))));
    }

    #[test]
    fn remove_device_returns_the_removed_device() {
        let mut room = sample_room();
        let removed = room.remove_device("Розетка");
        assert!(matches!(removed, Some(SmartDevice::Socket(_))));
        assert!(room.device("Розетка").is_none());
    }

    #[test]
    fn remove_device_returns_none_for_unknown_name() {
        let mut room = sample_room();
        assert!(room.remove_device("Лампа").is_none());
    }

    #[test]
    fn device_returns_reference_by_name() {
        let room = sample_room();
        assert!(matches!(
            room.device("Термометр"),
            Some(SmartDevice::Thermometer(_))
        ));
    }

    #[test]
    fn device_returns_none_for_unknown_name_instead_of_panicking() {
        let room = sample_room();
        assert!(room.device("Лампа").is_none());
    }

    #[test]
    fn device_mut_allows_mutation() {
        let mut room = sample_room();
        let Some(SmartDevice::Socket(s)) = room.device_mut("Розетка") else {
            panic!("ожидалась розетка");
        };
        s.turn_on();
        assert!(s.is_on());
    }

    #[test]
    fn device_names_are_sorted() {
        let room = sample_room();
        assert_eq!(
            room.device_names().collect::<Vec<_>>(),
            ["Розетка", "Термометр"]
        );
    }

    #[test]
    fn report_lists_indented_devices_sorted_by_name() {
        let room = sample_room();
        assert_eq!(
            room.report(),
            "  Розетка: розетка выключена, мощность 0.0 Вт\n  Термометр: термометр показывает 22.0 °C"
        );
    }

    #[test]
    fn report_of_empty_room_is_empty() {
        assert_eq!(Room::new().report(), "");
    }

    #[test]
    fn macro_builds_room_with_trailing_comma() {
        let room = room! {
            "Термометр" => SmartThermometer::new(22.0),
            "Розетка" => SmartSocket::new(60.0),
        };
        assert_eq!(
            room.device_names().collect::<Vec<_>>(),
            ["Розетка", "Термометр"]
        );
    }

    #[test]
    fn macro_builds_room_without_trailing_comma() {
        let room = room! {
            "Термометр" => SmartThermometer::new(22.0),
            "Розетка" => SmartSocket::new(60.0)
        };
        assert_eq!(room.device_names().count(), 2);
    }

    #[test]
    fn macro_builds_empty_room() {
        let room = room![];
        assert_eq!(room.device_names().count(), 0);
    }
}
