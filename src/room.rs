use crate::device::SmartDevice;
use crate::report::Report;
use std::collections::BTreeMap;
use std::fmt;

pub trait DeviceSubscriber {
    fn on_device_added(&mut self, name: &str, device: &SmartDevice);
}

impl<F: FnMut(&str, &SmartDevice)> DeviceSubscriber for F {
    fn on_device_added(&mut self, name: &str, device: &SmartDevice) {
        self(name, device);
    }
}

#[derive(Default)]
pub struct Room {
    devices: BTreeMap<String, SmartDevice>,
    subscribers: Vec<Box<dyn DeviceSubscriber>>,
}

impl Room {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&mut self, subscriber: impl DeviceSubscriber + 'static) {
        self.subscribers.push(Box::new(subscriber));
    }

    pub fn unsubscribe_all(&mut self) -> usize {
        let count = self.subscribers.len();
        self.subscribers.clear();
        count
    }

    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    pub fn add_device(
        &mut self,
        name: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        let name = name.into();
        let displaced = self.devices.insert(name.clone(), device.into());

        if let Some(added) = self.devices.get(&name) {
            for subscriber in &mut self.subscribers {
                subscriber.on_device_added(&name, added);
            }
        }

        displaced
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

impl fmt::Debug for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Room")
            .field("devices", &self.devices)
            .field("subscribers", &self.subscribers.len())
            .finish()
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
    use std::cell::RefCell;
    use std::rc::Rc;

    fn sample_room() -> Room {
        let mut room = Room::new();
        room.add_device("Термометр", SmartThermometer::mock(22.0));
        room.add_device("Розетка", SmartSocket::mock(60.0));
        room
    }

    #[derive(Default)]
    struct Journal {
        seen: Rc<RefCell<Vec<String>>>,
    }

    impl DeviceSubscriber for Journal {
        fn on_device_added(&mut self, name: &str, _device: &SmartDevice) {
            self.seen.borrow_mut().push(name.to_string());
        }
    }

    #[test]
    fn add_device_returns_none_for_a_fresh_name() {
        let mut room = Room::new();
        assert!(room
            .add_device("Розетка", SmartSocket::mock(60.0))
            .is_none());
    }

    #[test]
    fn add_device_returns_the_displaced_device() {
        let mut room = sample_room();
        let displaced = room.add_device("Розетка", SmartSocket::mock(90.0));
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
        s.turn_on().expect("имитация отвечает");
        assert!(s.is_on().expect("имитация отвечает"));
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
            "Термометр" => SmartThermometer::mock(22.0),
            "Розетка" => SmartSocket::mock(60.0),
        };
        assert_eq!(
            room.device_names().collect::<Vec<_>>(),
            ["Розетка", "Термометр"]
        );
    }

    #[test]
    fn macro_builds_room_without_trailing_comma() {
        let room = room! {
            "Термометр" => SmartThermometer::mock(22.0),
            "Розетка" => SmartSocket::mock(60.0)
        };
        assert_eq!(room.device_names().count(), 2);
    }

    #[test]
    fn macro_builds_empty_room() {
        let room = room![];
        assert_eq!(room.device_names().count(), 0);
    }

    #[test]
    fn closure_subscriber_is_notified_about_a_new_device() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut room = Room::new();
        {
            let seen = Rc::clone(&seen);
            room.subscribe(move |name: &str, _device: &SmartDevice| {
                seen.borrow_mut().push(name.to_string());
            });
        }

        room.add_device("Розетка", SmartSocket::mock(60.0));

        assert_eq!(seen.borrow().as_slice(), ["Розетка"]);
    }

    #[test]
    fn object_subscriber_is_notified_about_a_new_device() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut room = Room::new();
        room.subscribe(Journal {
            seen: Rc::clone(&seen),
        });

        room.add_device("Термометр", SmartThermometer::mock(22.0));

        assert_eq!(seen.borrow().as_slice(), ["Термометр"]);
    }

    #[test]
    fn subscriber_sees_the_device_that_was_added() {
        let reports = Rc::new(RefCell::new(Vec::new()));
        let mut room = Room::new();
        {
            let reports = Rc::clone(&reports);
            room.subscribe(move |_name: &str, device: &SmartDevice| {
                reports.borrow_mut().push(device.report());
            });
        }

        room.add_device("Термометр", SmartThermometer::mock(22.0));

        assert_eq!(
            reports.borrow().as_slice(),
            ["термометр показывает 22.0 °C"]
        );
    }

    #[test]
    fn all_subscribers_are_notified_in_subscription_order() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut room = Room::new();
        for tag in ["первый", "второй"] {
            let seen = Rc::clone(&seen);
            room.subscribe(move |name: &str, _device: &SmartDevice| {
                seen.borrow_mut().push(format!("{tag}: {name}"));
            });
        }

        room.add_device("Розетка", SmartSocket::mock(60.0));

        assert_eq!(
            seen.borrow().as_slice(),
            ["первый: Розетка", "второй: Розетка"]
        );
    }

    #[test]
    fn subscribers_are_notified_about_a_replacement_too() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut room = Room::new();
        {
            let seen = Rc::clone(&seen);
            room.subscribe(move |name: &str, _device: &SmartDevice| {
                seen.borrow_mut().push(name.to_string());
            });
        }

        room.add_device("Розетка", SmartSocket::mock(60.0));
        room.add_device("Розетка", SmartSocket::mock(90.0));

        assert_eq!(seen.borrow().as_slice(), ["Розетка", "Розетка"]);
    }

    #[test]
    fn removing_a_device_does_not_notify_subscribers() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut room = sample_room();
        {
            let seen = Rc::clone(&seen);
            room.subscribe(move |name: &str, _device: &SmartDevice| {
                seen.borrow_mut().push(name.to_string());
            });
        }

        room.remove_device("Розетка");

        assert!(seen.borrow().is_empty());
    }

    #[test]
    fn unsubscribe_all_stops_notifications() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut room = Room::new();
        {
            let seen = Rc::clone(&seen);
            room.subscribe(move |name: &str, _device: &SmartDevice| {
                seen.borrow_mut().push(name.to_string());
            });
        }
        assert_eq!(room.subscriber_count(), 1);

        assert_eq!(room.unsubscribe_all(), 1);
        room.add_device("Розетка", SmartSocket::mock(60.0));

        assert_eq!(room.subscriber_count(), 0);
        assert!(seen.borrow().is_empty());
    }

    #[test]
    fn debug_shows_the_number_of_subscribers() {
        let mut room = Room::new();
        room.subscribe(|_name: &str, _device: &SmartDevice| {});
        assert!(format!("{room:?}").contains("subscribers: 1"));
    }
}
